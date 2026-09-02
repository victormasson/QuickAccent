mod app;
mod config;
mod grab;
mod injection;
mod mappings;
mod state_machine;
#[cfg(target_os = "linux")]
mod hyprland;
#[cfg(target_os = "linux")]
mod portal_keysym;
#[cfg(target_os = "linux")]
mod shell_ext;
#[cfg(target_os = "linux")]
mod virtual_kb;
#[cfg(target_os = "linux")]
mod xkb_custom;
#[cfg(target_os = "linux")]
mod xkb_map;
#[cfg(target_os = "macos")]
mod macos;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use notify::{recommended_watcher, RecursiveMode, Watcher};

const HELP: &str = "\
QuickAccent \u{2014} hold a letter, press Space, pick an accent character.

Usage: quickaccent [OPTIONS]

Options:
  -h, --help       Print this help
  -V, --version    Print version

Runs as a background daemon: no window is shown until the picker opens.
Config: ~/.config/quickaccent/config.toml (hot-reloaded)
Logs:   journalctl --user -u quickaccent -f
";

fn main() -> iced::Result {
    env_logger::init();

    // Answer CLI flags before touching the config, display or input devices.
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("quickaccent {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{HELP}");
        return Ok(());
    }

    // Diagnostic mode: quickaccent --portal-probe [char]
    #[cfg(target_os = "linux")]
    {
        let args: Vec<String> = std::env::args().collect();
        if let Some(i) = args.iter().position(|a| a == "--portal-probe") {
            let test_char = args.get(i + 1).and_then(|a| {
                let mut it = a.chars();
                it.next().filter(|_| it.next().is_none())
            });
            portal_keysym::probe(test_char);
            return Ok(());
        }
    }

    if !acquire_single_instance_lock() {
        return Ok(());
    }

    let config = config::load_config();
    mappings::init(&config.languages);

    start_config_watcher();

    injection::start();

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let grab_rx = Arc::new(Mutex::new(Some(rx)));

    #[cfg(target_os = "linux")]
    linux_setup();

    grab::run_grab_thread(tx, &config);

    #[cfg(target_os = "macos")]
    crate::macos::setup_status_item();

    let grab_rx_clone = grab_rx.clone();
    iced::daemon("QuickAccent", app::App::update, app::App::view)
        .subscription(app::App::subscription)
        .theme(app::App::theme)
        .run_with(move || app::App::new(grab_rx_clone.clone()))
}

/// Linux startup sequence with load-bearing ordering: make every accent
/// directly typeable, then hide the Wayland socket BEFORE iced starts so the
/// overlay runs through XWayland — a native Wayland toplevel always takes
/// keyboard focus on GNOME and would swallow the injected characters
/// (override_redirect windows are focus-proof, but only exist on X11).
#[cfg(target_os = "linux")]
fn linux_setup() {
    setup_direct_typing();
    // Lets the overlay open on the monitor being typed on (GNOME).
    shell_ext::ensure_installed();
    if hyprland::is_running() {
        // Omarchy reloads Hyprland's config on every theme change, which
        // drops runtime-set options — put ours back each time.
        hyprland::watch_config_reloads(setup_direct_typing);
    }
    if let Some(wl) = std::env::var_os("WAYLAND_DISPLAY") {
        injection::set_wayland_display(wl.clone());
        std::env::remove_var("WAYLAND_DISPLAY");
    }
}

/// Make every configured accent character directly typeable: characters the
/// base layout lacks are added to the keymap via the custom xkb option
/// (GNOME); any still-uncovered chars get the portal keysym tier.
#[cfg(target_os = "linux")]
fn setup_direct_typing() {
    let all = mappings::all_variant_chars();
    let base_missing = xkb_map::chars_missing_from_base(&all);
    if !base_missing.is_empty() {
        eprintln!(
            "[QuickAccent] {} accent characters are not in the base keyboard layout \
             (e.g. {:?})",
            base_missing.len(),
            base_missing.iter().take(5).collect::<String>()
        );
        xkb_custom::ensure_installed(&base_missing);
    }
    if xkb_custom::is_active() {
        // The custom option provides the level-3 switch on F24; the plain
        // layout may have no real AltGr at all (us).
        virtual_kb::set_level3_code(xkb_custom::LEVEL3_CODE);
    }
    // Build the combo map with the option applied and see what's left.
    xkb_map::warm_combos();
    let still_missing: Vec<char> = all
        .into_iter()
        .filter(|&c| xkb_map::combo_for_char(c).is_none())
        .collect();
    if !still_missing.is_empty() {
        let sample: String = still_missing.iter().take(5).collect();
        if portal_keysym::available() {
            eprintln!(
                "[QuickAccent] {} characters still not typeable via the keymap \
                 (e.g. {sample:?}) — starting portal injection",
                still_missing.len()
            );
            portal_keysym::start();
        } else {
            // No RemoteDesktop backend (Hyprland's portal has none): don't send
            // the user to a dead end — tell them what actually works here.
            let hint = if hyprland::is_running() {
                xkb_custom::hyprland_enable_hint()
            } else {
                format!(
                    "add the xkb option '{}' to your compositor's keyboard options",
                    xkb_custom::OPTION_NAME
                )
            };
            eprintln!(
                "[QuickAccent] {} characters still not typeable via the keymap \
                 (e.g. {sample:?}) and this desktop has no RemoteDesktop portal \
                 backend — they will use the clipboard fallback (Ctrl+V; terminals \
                 treat it literally). To type them directly, {hint}",
                still_missing.len()
            );
        }
    }
}

/// Refuse to run twice. A second instance — typically the app-grid icon
/// clicked while the systemd service is running — would lose the evdev grab
/// race with a silent EBUSY and leave everyone confused about which copy is
/// live. When the *service* starts and finds a stray holding the lock, the
/// service wins: the stray is asked to quit (same user, same binary).
fn acquire_single_instance_lock() -> bool {
    use std::io::Write;

    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let path = dir.join("quickaccent.lock");
    let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(&path)
    else {
        return true; // can't lock → don't block startup over it
    };

    let mut attempts = 0;
    loop {
        match file.try_lock() {
            Ok(()) => {
                let _ = file.set_len(0);
                let _ = write!(&file, "{}", std::process::id());
                std::mem::forget(file); // held for the life of the process
                return true;
            }
            Err(std::fs::TryLockError::WouldBlock) => {}
            Err(_) => return true,
        }
        let holder = std::fs::read_to_string(&path).unwrap_or_default().trim().to_string();
        let running_as_service = std::env::var_os("INVOCATION_ID").is_some();
        if running_as_service && attempts < 10 {
            if attempts == 0 {
                eprintln!(
                    "[QuickAccent] another instance (PID {holder}) holds the input grab; \
                     the service takes over — asking it to quit."
                );
                terminate_own_instance(&holder, false);
            } else if attempts == 5 {
                // Still there after ~1.5s of SIGTERM: it is verified to be
                // our own stale binary, so force it.
                terminate_own_instance(&holder, true);
            }
            attempts += 1;
            std::thread::sleep(Duration::from_millis(300));
            continue;
        }
        if running_as_service {
            // Exit non-zero so Restart=on-failure keeps trying instead of
            // leaving the service silently dead.
            eprintln!("[QuickAccent] could not take over from PID {holder}; retrying via systemd");
            std::process::exit(1);
        }
        eprintln!(
            "[QuickAccent] already running (PID {holder}) — nothing to do.\n{}",
            if cfg!(target_os = "macos") {
                "Quit it from the menu bar icon first if you want to restart it."
            } else {
                "Manage it with: systemctl --user restart quickaccent"
            }
        );
        return false;
    }
}

/// Signal `pid` only if it really is a quickaccent process. Identified via
/// /proc/PID/comm — NOT /proc/PID/exe, whose link gains a " (deleted)"
/// suffix as soon as the binary on disk is replaced by an upgrade, which
/// made the old check silently refuse the exact process it existed for.
fn terminate_own_instance(pid: &str, force: bool) {
    if pid.parse::<u32>().is_err() {
        return;
    }
    let comm = std::fs::read_to_string(format!("/proc/{pid}/comm")).unwrap_or_default();
    if comm.trim() != "quickaccent" {
        return;
    }
    let mut cmd = std::process::Command::new("kill");
    if force {
        cmd.arg("-9");
    }
    let _ = cmd.arg(pid).status();
}

fn start_config_watcher() {
    let config_dir = match config::config_path().parent() {
        Some(p) => p.to_path_buf(),
        None => return,
    };
    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = match recommended_watcher(tx) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("[QuickAccent] Failed to create config watcher: {}", e);
                return;
            }
        };
        if let Err(e) = watcher.watch(&config_dir, RecursiveMode::NonRecursive) {
            eprintln!("[QuickAccent] Failed to watch config dir: {}", e);
            return;
        }
        let mut last_reload = Instant::now() - Duration::from_secs(1);
        let debounce = Duration::from_millis(400);
        for res in rx {
            match res {
                Ok(event) => {
                    let relevant = event.paths.iter().any(|p| {
                        p.file_name().map_or(false, |n| n == "config.toml")
                    });
                    if !relevant {
                        continue;
                    }
                    let now = Instant::now();
                    if now.duration_since(last_reload) < debounce {
                        continue;
                    }
                    last_reload = now;
                    if let Some(cfg) = config::read_config() {
                        mappings::reload(&cfg.languages);
                        eprintln!("[QuickAccent] Reloaded config: languages = {:?}", cfg.languages);
                        // New languages may need new keymap slots / the portal.
                        #[cfg(target_os = "linux")]
                        setup_direct_typing();
                    }
                }
                Err(e) => eprintln!("[QuickAccent] Config watcher error: {}", e),
            }
        }
    });
}
