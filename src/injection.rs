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

/// Start the long-lived injection thread (call once from main).
pub fn start() {
    let (tx, rx) = mpsc::sync_channel(32);
    if TX.set(tx).is_err() {
        return;
    }

    thread::spawn(move || {
        let mut enigo = match create_enigo() {
            Ok(e) => e,
            Err(e) => {
                eprintln!(
                    "[QuickAccent] injection unavailable: {e}\n\
                     GNOME Wayland: approve the Remote Desktop / input portal prompt."
                );
                while rx.recv().is_ok() {}
                return;
            }
        };

        let delay = Duration::from_millis(20);
        while let Ok(cmd) = rx.recv() {
            thread::sleep(delay);
            let result = match cmd {
                Cmd::Char(ch) => enigo
                    .key(Key::Backspace, Direction::Click)
                    .and_then(|_| {
                        thread::sleep(delay);
                        enigo.text(&ch)
                    }),
                Cmd::Space => enigo.key(Key::Space, Direction::Click),
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

fn create_enigo() -> Result<Enigo, enigo::NewConError> {
    let mut settings = Settings::default();
    // On Wayland, $DISPLAY is often XWayland — XTEST only reaches X11 clients.
    // Point x11 at a dummy display so enigo uses virtual-keyboard / libei instead.
    #[cfg(target_os = "linux")]
    if is_wayland() {
        settings.x11_display = Some("__quickaccent_no_xwayland__".into());
    }
    Enigo::new(&settings)
}

#[cfg(target_os = "linux")]
fn is_wayland() -> bool {
    std::env::var("XDG_SESSION_TYPE")
        .map(|v| v.eq_ignore_ascii_case("wayland"))
        .unwrap_or(false)
        || std::env::var_os("WAYLAND_DISPLAY").is_some()
}
