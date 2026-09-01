//! Direct Unicode character injection via the XDG RemoteDesktop portal
//! (`NotifyKeyboardKeysym`) — the Wayland equivalent of PowerAccent's
//! `SendInput(KEYEVENTF_UNICODE)` on Windows. The compositor types the
//! character itself (mutter/KWin map arbitrary keysyms onto reserved
//! keycodes), so this works for characters the keyboard layout cannot
//! produce, in every app, with no clipboard involved.
//!
//! Authorization: the portal asks ONCE; we request
//! `PersistMode::ExplicitlyRevoked` and store the `restore_token`, so every
//! later start is silent (like a macOS Accessibility grant) until the user
//! revokes it in Settings.

use ashpd::desktop::remote_desktop::{
    DeviceType, KeyState, NotifyKeyboardKeysymOptions, RemoteDesktop, SelectDevicesOptions,
    StartOptions,
};
use ashpd::desktop::{CreateSessionOptions, PersistMode, Session};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::{RecvTimeoutError, SyncSender};
use std::sync::OnceLock;
use std::time::Duration;
use xkbcommon::xkb;

const STATE_STARTING: u8 = 0;
const STATE_READY: u8 = 1;
const STATE_FAILED: u8 = 2;

static STATE: AtomicU8 = AtomicU8::new(STATE_FAILED);
static TX: OnceLock<tokio::sync::mpsc::UnboundedSender<Req>> = OnceLock::new();

struct Req {
    text: String,
    resp: SyncSender<Result<(), String>>,
}

pub enum InjectError {
    /// Portal unavailable/denied or the call itself failed — nothing was
    /// typed, a fallback is safe.
    Unavailable(String),
    /// No reply in time — the character may still land, so the caller must
    /// NOT inject through another path (double input).
    Timeout,
}

/// Spawn the portal thread and open the session. Idempotent. The one-time
/// authorization dialog (first run only) appears here, not mid-typing.
pub fn start() {
    static STARTED: std::sync::Once = std::sync::Once::new();
    STARTED.call_once(|| {
        STATE.store(STATE_STARTING, Ordering::SeqCst);
        std::thread::spawn(|| {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("[QuickAccent] portal runtime failed: {e}");
                    STATE.store(STATE_FAILED, Ordering::SeqCst);
                    return;
                }
            };
            rt.block_on(run());
        });
    });
}

/// Whether any portal backend implements RemoteDesktop. xdg-desktop-portal
/// only exports the interface when an implementation exists — on Hyprland
/// (xdg-desktop-portal-hyprland) it does not, so this tier is a dead end
/// there and must not be offered as a remedy (issue #9).
pub fn available() -> bool {
    let Ok(conn) = zbus::blocking::Connection::session() else {
        return false;
    };
    conn.call_method(
        Some("org.freedesktop.portal.Desktop"),
        "/org/freedesktop/portal/desktop",
        Some("org.freedesktop.DBus.Properties"),
        "Get",
        &("org.freedesktop.portal.RemoteDesktop", "version"),
    )
    .is_ok()
}

fn is_gnome() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|d| d.to_ascii_uppercase().contains("GNOME"))
        .unwrap_or(false)
}

/// Type `text` directly (keysym press/release per char). Blocking, called
/// from the grab thread — while it blocks, physical events queue in the
/// kernel, which keeps injected chars ordered before the user's next
/// keystroke.
pub fn inject_text_sync(text: &str) -> Result<(), InjectError> {
    match STATE.load(Ordering::SeqCst) {
        STATE_READY => {}
        STATE_STARTING => {
            return Err(InjectError::Unavailable(
                "portal session not ready yet".into(),
            ))
        }
        _ => return Err(InjectError::Unavailable("portal session unavailable".into())),
    }
    let Some(tx) = TX.get() else {
        return Err(InjectError::Unavailable("portal thread not running".into()));
    };
    let (rtx, rrx) = std::sync::mpsc::sync_channel(1);
    if tx
        .send(Req {
            text: text.to_string(),
            resp: rtx,
        })
        .is_err()
    {
        return Err(InjectError::Unavailable("portal thread gone".into()));
    }
    match rrx.recv_timeout(Duration::from_millis(500)) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(InjectError::Unavailable(e)),
        Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => {
            Err(InjectError::Timeout)
        }
    }
}

async fn run() {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Req>();
    if TX.set(tx).is_err() {
        return;
    }

    match connect().await {
        Ok((proxy, session)) => {
            STATE.store(STATE_READY, Ordering::SeqCst);
            eprintln!(
                "[QuickAccent] direct Unicode injection ready (portal authorization remembered)"
            );
            while let Some(req) = rx.recv().await {
                let result = send_text(&proxy, &session, &req.text)
                    .await
                    .map_err(|e| e.to_string());
                if let Err(e) = &result {
                    eprintln!("[QuickAccent] portal injection failed: {e}");
                }
                let _ = req.resp.send(result);
            }
        }
        Err(e) => {
            STATE.store(STATE_FAILED, Ordering::SeqCst);
            let remedy = if is_gnome() {
                "To enable direct typing, restart QuickAccent and accept the one-time\n\
                 input-sharing dialog (GNOME remembers it permanently)."
            } else {
                "This desktop has no RemoteDesktop portal backend; enabling the keymap\n\
                 option is the way to get direct typing (see the lines above)."
            };
            eprintln!(
                "[QuickAccent] portal input unavailable ({e}).\n\
                 Characters outside your keyboard layout will use the clipboard fallback\n\
                 (Ctrl+V — terminals treat it literally).\n{remedy}"
            );
            // Keep answering so callers fail fast instead of timing out.
            while let Some(req) = rx.recv().await {
                let _ = req.resp.send(Err("portal session unavailable".into()));
            }
        }
    }
}

async fn connect(
) -> Result<(RemoteDesktop, Session<RemoteDesktop>), ashpd::Error> {
    let proxy = RemoteDesktop::new().await?;
    let session = proxy.create_session(CreateSessionOptions::default()).await?;
    let token = load_token();
    proxy
        .select_devices(
            &session,
            SelectDevicesOptions::default()
                .set_devices(ashpd::enumflags2::BitFlags::from(DeviceType::Keyboard))
                .set_persist_mode(PersistMode::ExplicitlyRevoked)
                .set_restore_token(token.as_deref()),
        )
        .await?;
    let devices = proxy
        .start(&session, None, StartOptions::default())
        .await?
        .response()?;
    eprintln!(
        "[QuickAccent] portal granted devices: {:?}",
        devices.devices()
    );
    if let Some(t) = devices.restore_token() {
        save_token(t);
    }
    Ok((proxy, session))
}

/// Diagnostic for `quickaccent --portal-probe [char]`: connect, print the
/// granted device set, optionally type `test_char` after 2s into whatever is
/// focused.
pub fn probe(test_char: Option<char>) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    rt.block_on(async {
        match connect().await {
            Ok((proxy, session)) => {
                if let Some(c) = test_char {
                    eprintln!("[probe] typing {c:?} in 2s — focus a text field…");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    match send_text(&proxy, &session, &c.to_string()).await {
                        Ok(()) => eprintln!("[probe] send_text returned Ok"),
                        Err(e) => eprintln!("[probe] send_text failed: {e}"),
                    }
                    tokio::time::sleep(Duration::from_millis(300)).await;
                }
            }
            Err(e) => eprintln!("[probe] connect failed: {e}"),
        }
    });
}

async fn send_text(
    proxy: &RemoteDesktop,
    session: &Session<RemoteDesktop>,
    text: &str,
) -> Result<(), ashpd::Error> {
    for c in text.chars() {
        let keysym = char_keysym(c);
        proxy
            .notify_keyboard_keysym(
                session,
                keysym,
                KeyState::Pressed,
                NotifyKeyboardKeysymOptions::default(),
            )
            .await?;
        proxy
            .notify_keyboard_keysym(
                session,
                keysym,
                KeyState::Released,
                NotifyKeyboardKeysymOptions::default(),
            )
            .await?;
    }
    Ok(())
}

/// X keysym for a Unicode char: Latin-1 maps directly, everything else uses
/// the 0x1000000 | codepoint Unicode keysym range.
fn char_keysym(c: char) -> i32 {
    let sym = xkb::utf32_to_keysym(c as u32).raw();
    if sym != 0 {
        sym as i32
    } else {
        (0x0100_0000 | c as u32) as i32
    }
}

fn token_path() -> std::path::PathBuf {
    crate::config::config_path()
        .parent()
        .map(|p| p.join("portal-restore-token"))
        .unwrap_or_else(|| std::path::PathBuf::from("portal-restore-token"))
}

fn load_token() -> Option<String> {
    let t = std::fs::read_to_string(token_path()).ok()?;
    let t = t.trim().to_string();
    (!t.is_empty()).then_some(t)
}

fn save_token(token: &str) {
    if let Err(e) = crate::xkb_custom::write_file(&token_path(), token) {
        eprintln!("[QuickAccent] could not save portal token: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_keysym_latin1_and_unicode() {
        assert_eq!(char_keysym('é'), 0xE9); // Latin-1 keysyms = codepoint
        assert_eq!(char_keysym('A'), 0x41);
        assert_eq!(char_keysym('ẞ'), 0x0100_1E9E); // Unicode keysym range
        assert_eq!(char_keysym('œ'), 0x13BD); // oe ligature has a legacy keysym
    }
}
