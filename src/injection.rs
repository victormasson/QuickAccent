//! Accent character injection.
//!
//! Linux: characters reachable in the active layout are typed directly by the
//! grab thread through the uinput virtual keyboard (see `grab.rs`); this
//! module only handles the fallback for characters the layout can't type in
//! one keystroke — clipboard set + virtual Ctrl+V — on a worker thread so the
//! slow wl-copy/wl-paste process spawns never block the grab callback.
//!
//! macOS: enigo-based backspace + text replacement (unchanged behavior).

#[cfg(target_os = "linux")]
mod imp {
    use crate::virtual_kb::{self, KeyEvt, KEY_LEFTCTRL};
    use crate::xkb_map;
    use std::sync::mpsc::{self, SyncSender};
    use std::sync::OnceLock;
    use std::thread;
    use std::time::Duration;

    struct Paste {
        ch: String,
        /// evdev codes of physically held Shift keys to neutralize around Ctrl+V
        held_shifts: Vec<u16>,
    }

    static TX: OnceLock<SyncSender<Paste>> = OnceLock::new();

    /// WAYLAND_DISPLAY is removed from our own environment (the overlay runs
    /// through XWayland to stay focus-proof) but wl-copy/wl-paste still need
    /// it — stash it here and pass it to the spawned processes.
    static WAYLAND_DISPLAY: OnceLock<std::ffi::OsString> = OnceLock::new();

    pub fn set_wayland_display(value: std::ffi::OsString) {
        let _ = WAYLAND_DISPLAY.set(value);
    }

    fn with_wayland_env(cmd: &mut std::process::Command) {
        if let Some(wl) = WAYLAND_DISPLAY.get() {
            cmd.env("WAYLAND_DISPLAY", wl);
        }
    }

    pub fn start() {
        let (tx, rx) = mpsc::sync_channel(32);
        if TX.set(tx).is_err() {
            return;
        }
        thread::spawn(move || {
            while let Ok(paste) = rx.recv() {
                if let Err(e) = paste_via_clipboard(&paste) {
                    eprintln!("[QuickAccent] inject failed: {e}");
                }
            }
        });
    }

    /// Clipboard-paste fallback — emergency path only, when neither the
    /// keyboard layout nor the portal can type the character directly.
    pub fn inject_char_fallback(ch: String, held_shifts: Vec<u16>) {
        eprintln!(
            "[QuickAccent] WARNING: typing {ch:?} via clipboard fallback. \
             For direct typing, accept the one-time input-sharing authorization \
             (restart QuickAccent to be prompted again)."
        );
        if let Some(tx) = TX.get() {
            let _ = tx.send(Paste { ch, held_shifts });
        }
    }

    fn paste_via_clipboard(paste: &Paste) -> Result<(), String> {
        let prev = clipboard_set(&paste.ch)?;
        let result = press_ctrl_v(&paste.held_shifts);
        // Let the focused app read the clipboard before restoring it.
        thread::sleep(Duration::from_millis(150));
        clipboard_restore(prev);
        result.map(|()| eprintln!("[QuickAccent] injected via clipboard"))
    }

    fn press_ctrl_v(held_shifts: &[u16]) -> Result<(), String> {
        // 'v' is not on the same key in every layout (Dvorak…).
        let v = xkb_map::combo_for_char('v').map(|c| c.code).unwrap_or(47);
        let mut seq = Vec::new();
        for &s in held_shifts {
            seq.push(KeyEvt::Release(s));
        }
        seq.push(KeyEvt::Press(KEY_LEFTCTRL));
        seq.push(KeyEvt::Press(v));
        seq.push(KeyEvt::Release(v));
        seq.push(KeyEvt::Release(KEY_LEFTCTRL));
        for &s in held_shifts {
            seq.push(KeyEvt::Press(s));
        }
        virtual_kb::emit(&seq)
    }

    fn clipboard_set(ch: &str) -> Result<Option<String>, String> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let copy = clipboard_bin("wl-copy")?;
        let paste = clipboard_bin("wl-paste").ok();

        let prev = paste.and_then(|bin| {
            let mut cmd = Command::new(bin);
            cmd.arg("-n");
            with_wayland_env(&mut cmd);
            cmd.output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
        });

        let mut cmd = Command::new(copy);
        cmd.stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null());
        with_wayland_env(&mut cmd);
        let mut child = cmd.spawn().map_err(|e| e.to_string())?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(ch.as_bytes()).map_err(|e| e.to_string())?;
        }
        if !child.wait().map(|s| s.success()).unwrap_or(false) {
            return Err("wl-copy failed".into());
        }
        Ok(prev)
    }

    fn clipboard_restore(prev: Option<String>) {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let Some(prev) = prev else {
            return;
        };
        let Ok(copy) = clipboard_bin("wl-copy") else {
            return;
        };
        let mut cmd = Command::new(copy);
        cmd.stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null());
        with_wayland_env(&mut cmd);
        if let Ok(mut child) = cmd.spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(prev.as_bytes());
            }
            let _ = child.wait();
        }
    }

    /// Resolve a clipboard binary once and cache the result — each paste
    /// otherwise spawns three extra `--version` probes on a latency-visible
    /// path.
    fn clipboard_bin(name: &str) -> Result<String, String> {
        static CACHE: OnceLock<std::sync::Mutex<std::collections::HashMap<String, Option<String>>>> =
            OnceLock::new();
        let cache = CACHE.get_or_init(Default::default);
        if let Some(cached) = cache.lock().unwrap().get(name) {
            return cached.clone().ok_or_else(|| format!("{name} not found"));
        }
        let found = [name, &format!("/usr/bin/{name}")]
            .into_iter()
            .find(|candidate| {
                std::process::Command::new(candidate)
                    .arg("--version")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
            })
            .map(str::to_string);
        cache
            .lock()
            .unwrap()
            .insert(name.to_string(), found.clone());
        found.ok_or_else(|| format!("{name} not found"))
    }
}

#[cfg(not(target_os = "linux"))]
mod imp {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};
    use std::sync::mpsc::{self, SyncSender};
    use std::sync::OnceLock;
    use std::thread;
    use std::time::Duration;

    enum Cmd {
        Char(String),
        Space,
    }

    static TX: OnceLock<SyncSender<Cmd>> = OnceLock::new();

    pub fn start() {
        let (tx, rx) = mpsc::sync_channel(32);
        if TX.set(tx).is_err() {
            return;
        }

        thread::spawn(move || {
            let mut enigo = match Enigo::new(&Settings::default()) {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("[QuickAccent] injection unavailable: {e}");
                    while rx.recv().is_ok() {}
                    return;
                }
            };

            while let Ok(cmd) = rx.recv() {
                thread::sleep(Duration::from_millis(20));
                let result = match cmd {
                    Cmd::Char(ch) => inject_accent(&mut enigo, &ch),
                    Cmd::Space => click_key(&mut enigo, Key::Space),
                };
                if let Err(e) = result {
                    eprintln!("[QuickAccent] inject failed: {e}");
                }
            }
        });
    }

    pub fn inject_char(ch: String) {
        send(Cmd::Char(ch));
    }

    pub fn inject_space() {
        send(Cmd::Space);
    }

    fn send(cmd: Cmd) {
        if let Some(tx) = TX.get() {
            let _ = tx.send(cmd);
        }
    }

    fn inject_accent(enigo: &mut Enigo, ch: &str) -> Result<(), String> {
        click_key(enigo, Key::Backspace)?;
        thread::sleep(Duration::from_millis(20));
        enigo
            .text(ch)
            .map_err(|e| format!("could not inject {ch:?}: {e}"))
    }

    fn click_key(enigo: &mut Enigo, key: Key) -> Result<(), String> {
        enigo.key(key, Direction::Click).map_err(|e| e.to_string())
    }
}

pub use imp::*;
