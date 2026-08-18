//! GNOME Shell helper extension: on Wayland only the compositor knows window
//! geometry, so QuickAccent self-installs a micro-extension (see
//! dist/linux/gnome-extension/) that reports the focused window's frame rect
//! over D-Bus. Used to show the accent picker on the monitor being typed on;
//! everything degrades to the centered overlay when it's unavailable.

use std::path::PathBuf;
use std::sync::Mutex;

pub const UUID: &str = "quickaccent-focus@victormasson.github.io";
const EXTENSION_JS: &str = include_str!("../dist/linux/gnome-extension/extension.js");
const METADATA_JSON: &str = include_str!("../dist/linux/gnome-extension/metadata.json");

/// A window frame rectangle in global logical coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Install the extension files and enable the extension. Idempotent. A
/// freshly installed extension is only loaded by GNOME Shell at the next
/// login — until then `focused_window_rect` returns None and the overlay
/// stays centered.
pub fn ensure_installed() {
    let dir = extension_dir();
    let mut changed = false;
    for (name, content) in [("extension.js", EXTENSION_JS), ("metadata.json", METADATA_JSON)] {
        let path = dir.join(name);
        if std::fs::read_to_string(&path).ok().as_deref() != Some(content) {
            if let Err(e) = crate::xkb_custom::write_file(&path, content) {
                eprintln!("[QuickAccent] cannot install shell extension: {e}");
                return;
            }
            changed = true;
        }
    }

    // Pre-enable in gsettings so the extension activates at the next login
    // even though the running shell can't see a manually installed one yet.
    ensure_enabled_setting();
    // If the shell has already scanned it (any run after a re-login), this
    // also loads it live.
    let _ = std::process::Command::new("gnome-extensions")
        .args(["enable", UUID])
        .stderr(std::process::Stdio::null())
        .status();
    if changed && focused_window_rect().is_none() {
        eprintln!(
            "[QuickAccent] shell extension installed — log out/in once so the \
             accent picker can follow the monitor you type on"
        );
    }
}

fn ensure_enabled_setting() {
    let Some(current) = crate::xkb_custom::gsettings_get("org.gnome.shell", "enabled-extensions")
    else {
        return; // not GNOME
    };
    let mut list = crate::xkb_custom::parse_gvariant_string_list(&current);
    if list.iter().any(|u| u == UUID) {
        return;
    }
    list.push(UUID.to_string());
    if !crate::xkb_custom::gsettings_set_list("org.gnome.shell", "enabled-extensions", &list) {
        eprintln!("[QuickAccent] could not enable the shell extension in GNOME settings");
    }
}

fn extension_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("gnome-shell/extensions")
        .join(UUID)
}

/// Frame rect of the currently focused window, via the shell extension.
pub fn focused_window_rect() -> Option<Rect> {
    static CONN: Mutex<Option<zbus::blocking::Connection>> = Mutex::new(None);
    let mut guard = CONN.lock().unwrap();
    if guard.is_none() {
        *guard = zbus::blocking::Connection::session().ok();
    }
    let conn = guard.as_ref()?;
    let reply = conn
        .call_method(
            Some("org.gnome.Shell"),
            "/io/github/victormasson/QuickAccent/FocusedWindow",
            Some("io.github.victormasson.QuickAccent.FocusedWindow"),
            "Get",
            &(),
        )
        .ok()?;
    let rect: String = reply.body().deserialize().ok()?;
    parse_rect(&rect)
}

fn parse_rect(s: &str) -> Option<Rect> {
    let mut it = s.split_whitespace().map(|v| v.parse::<f32>().ok());
    let rect = Rect {
        x: it.next()??,
        y: it.next()??,
        width: it.next()??,
        height: it.next()??,
    };
    (rect.width > 0.0 && rect.height > 0.0).then_some(rect)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rect_roundtrip() {
        assert_eq!(
            parse_rect("100 -50 1920 1080"),
            Some(Rect {
                x: 100.0,
                y: -50.0,
                width: 1920.0,
                height: 1080.0
            })
        );
        assert_eq!(parse_rect(""), None);
        assert_eq!(parse_rect("1 2 3"), None);
        assert_eq!(parse_rect("0 0 0 0"), None); // empty rect = no focus info
    }

    #[test]
    fn embedded_metadata_matches_uuid() {
        assert!(METADATA_JSON.contains(UUID));
        assert!(EXTENSION_JS.contains("FocusedWindow"));
    }
}
