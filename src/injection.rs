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
                Cmd::Char(ch) => {
                    let _ = enigo.key(Key::Backspace, Direction::Click);
                    thread::sleep(delay);
                    inject_text(&mut enigo, &ch)
                }
                Cmd::Space => enigo
                    .key(Key::Space, Direction::Click)
                    .map_err(|e| e.to_string()),
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

fn inject_text(enigo: &mut Enigo, ch: &str) -> Result<(), String> {
    if enigo.text(ch).is_ok() {
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        if paste_via_clipboard(enigo, ch).is_ok() {
            return Ok(());
        }
        if paste_via_gtk_unicode(enigo, ch).is_ok() {
            return Ok(());
        }
    }
    Err(format!("could not inject {ch:?}"))
}

#[cfg(target_os = "linux")]
fn paste_via_clipboard(enigo: &mut Enigo, ch: &str) -> Result<(), String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    if command_missing("wl-copy") {
        return Err("wl-copy not found".into());
    }

    let prev = Command::new("wl-paste")
        .arg("-n")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok());

    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(ch.as_bytes()).map_err(|e| e.to_string())?;
    }
    let _ = child.wait();

    thread::sleep(Duration::from_millis(30));
    enigo
        .key(Key::Control, Direction::Press)
        .and_then(|_| enigo.key(Key::Unicode('v'), Direction::Click))
        .and_then(|_| enigo.key(Key::Control, Direction::Release))
        .map_err(|e| e.to_string())?;

    thread::sleep(Duration::from_millis(80));
    if let Some(prev) = prev {
        let mut child = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(prev.as_bytes());
        }
        let _ = child.wait();
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn paste_via_gtk_unicode(enigo: &mut Enigo, ch: &str) -> Result<(), String> {
    let Some(c) = ch.chars().next() else {
        return Err("empty".into());
    };
    let hex = format!("{:x}", c as u32);

    enigo
        .key(Key::Control, Direction::Press)
        .and_then(|_| enigo.key(Key::Shift, Direction::Press))
        .and_then(|_| enigo.key(Key::Unicode('u'), Direction::Click))
        .and_then(|_| enigo.key(Key::Shift, Direction::Release))
        .and_then(|_| enigo.key(Key::Control, Direction::Release))
        .map_err(|e| e.to_string())?;

    thread::sleep(Duration::from_millis(20));
    for d in hex.chars() {
        enigo
            .key(Key::Unicode(d), Direction::Click)
            .map_err(|e| e.to_string())?;
    }
    enigo
        .key(Key::Return, Direction::Click)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn command_missing(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| !s.success())
        .unwrap_or(true)
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
