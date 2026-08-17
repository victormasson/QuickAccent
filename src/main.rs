mod app;
mod config;
mod grab;
mod injection;
mod mappings;
mod state_machine;
#[cfg(target_os = "linux")]
mod portal_keysym;
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

fn main() -> iced::Result {
    env_logger::init();

    // Diagnostic mode: quickaccent --portal-probe [char]
    #[cfg(target_os = "linux")]
    {
        let args: Vec<String> = std::env::args().collect();
        if args.iter().any(|a| a == "--portal-probe") {
            let test_char = args.last().and_then(|a| {
                let mut it = a.chars();
                match (it.next(), it.next()) {
                    (Some(c), None) if *a != "--portal-probe" => Some(c),
                    _ => None,
                }
            });
            portal_keysym::probe(test_char);
            return Ok(());
        }
    }

    let config = config::load_config();
    mappings::init(&config.languages);

    start_config_watcher();

    injection::start();

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let grab_rx = Arc::new(Mutex::new(Some(rx)));

    #[cfg(target_os = "linux")]
    setup_direct_typing();

    // The overlay must never take keyboard focus: on GNOME Wayland a new
    // toplevel always gets focused, which swallowed the injected accents.
    // Run the UI through XWayland instead — override_redirect windows are
    // focus-proof there. Keep the Wayland socket for wl-copy/wl-paste.
    #[cfg(target_os = "linux")]
    if let Some(wl) = std::env::var_os("WAYLAND_DISPLAY") {
        injection::set_wayland_display(wl.clone());
        std::env::remove_var("WAYLAND_DISPLAY");
    }

    // The grab swallows keystrokes and replays them through the virtual
    // keyboard — never grab without it, or we'd eat the user's typing.
    #[cfg(target_os = "linux")]
    match virtual_kb::init() {
        Ok(()) => grab::run_grab_thread(tx, &config),
        Err(e) => eprintln!(
            "[QuickAccent] virtual keyboard unavailable: {e}\n\
             QuickAccent needs /dev/uinput and /dev/input access.\n\
             Run ./dist/linux/install.sh, ensure you are in the 'input' group\n\
             (log out/in or reboot), and that the uinput module is loaded\n\
             (sudo modprobe uinput). Keyboard grabbing is disabled."
        ),
    }
    #[cfg(not(target_os = "linux"))]
    grab::run_grab_thread(tx, &config);

    #[cfg(target_os = "macos")]
    crate::macos::setup_status_item();

    let grab_rx_clone = grab_rx.clone();
    iced::daemon("QuickAccent", app::App::update, app::App::view)
        .subscription(app::App::subscription)
        .theme(app::App::theme)
        .run_with(move || app::App::new(grab_rx_clone.clone()))
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
    // Build the combo map with the option applied and see what's left.
    xkb_map::warm_combos();
    let still_missing: Vec<char> = all
        .into_iter()
        .filter(|&c| xkb_map::combo_for_char(c).is_none())
        .collect();
    if !still_missing.is_empty() {
        eprintln!(
            "[QuickAccent] {} characters still not typeable via the keymap \
             (e.g. {:?}) — starting portal injection",
            still_missing.len(),
            still_missing.iter().take(5).collect::<String>()
        );
        portal_keysym::start();
    }
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
