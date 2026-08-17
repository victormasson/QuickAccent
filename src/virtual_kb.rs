//! Kernel-level key injection via a uinput virtual keyboard (Linux only).
//!
//! All replayed/injected keystrokes go through this single device so their
//! ordering is preserved and no display-server permission (portal) is needed.
//!
//! Loopback: rdev's grab hot-plugs and EVIOCGRABs this device too, so every
//! event we emit comes back through our own grab callback. `emit` registers
//! the expected (code, value) pairs *before* writing; the callback calls
//! `take_loopback` first and passes matching events through untouched.

use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, EventType, InputEvent, KeyCode};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

static DEVICE: OnceLock<Mutex<VirtualDevice>> = OnceLock::new();
static REGISTRY: Mutex<Vec<(u16, i32, Instant)>> = Mutex::new(Vec::new());

/// Registry entries older than this are dropped (lost/never-looped events
/// must not poison future matching).
const LOOPBACK_TTL: Duration = Duration::from_millis(500);

pub const KEY_LEFTCTRL: u16 = 29;
pub const KEY_LEFTSHIFT: u16 = 42;
pub const KEY_RIGHTALT: u16 = 100; // AltGr

pub use crate::state_machine::KeyEvt;

/// Create the virtual keyboard. Must be called before the grab starts so the
/// device is grabbed (and loopback-filtered) deterministically from the start.
pub fn init() -> Result<(), String> {
    let mut keys = AttributeSet::<KeyCode>::new();
    // KEY_ESC(1) ..= KEY_MICMUTE(248): every key a physical keyboard can send.
    for code in 1..=248u16 {
        keys.insert(KeyCode::new(code));
    }
    let mut dev = VirtualDevice::builder()
        .map_err(|e| format!("open /dev/uinput: {e}"))?
        .name("QuickAccent Virtual Keyboard")
        .with_keys(&keys)
        .map_err(|e| format!("set key capabilities: {e}"))?
        .build()
        .map_err(|e| format!("create virtual device: {e}"))?;
    wait_for_node_access(&mut dev);
    DEVICE
        .set(Mutex::new(dev))
        .map_err(|_| "virtual keyboard already initialized".to_string())
}

/// The grab (rdev) enumerates /dev/input right after us and opens every node.
/// Our freshly created node starts root-owned until udev applies the
/// input-group rule — wait for that, or the whole grab fails with EACCES.
fn wait_for_node_access(dev: &mut VirtualDevice) {
    let _ = std::process::Command::new("udevadm")
        .args(["settle", "--timeout=2"])
        .status();

    let node = dev.get_syspath().ok().and_then(|sys| {
        std::fs::read_dir(sys).ok()?.find_map(|e| {
            let name = e.ok()?.file_name();
            name.to_str()?
                .starts_with("event")
                .then(|| std::path::Path::new("/dev/input").join(name))
        })
    });
    let Some(node) = node else { return };
    for _ in 0..40 {
        if std::fs::OpenOptions::new().read(true).open(&node).is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    eprintln!(
        "[QuickAccent] warning: {} not readable after 2s (udev rule missing?)",
        node.display()
    );
}

/// Emit key events, each in its own SYN_REPORT frame so modifier state is
/// applied strictly in order by the compositor. Never emits autorepeat
/// (value 2) — clients synthesize repeats themselves.
pub fn emit(events: &[KeyEvt]) -> Result<(), String> {
    let Some(dev) = DEVICE.get() else {
        return Err("virtual keyboard not initialized".into());
    };
    {
        // Register expected loopback events BEFORE writing: the grab thread
        // can see them microseconds after the write syscall.
        let mut reg = REGISTRY.lock().unwrap();
        let now = Instant::now();
        for e in events {
            reg.push((e.code(), e.value(), now));
        }
    }
    let mut dev = dev.lock().unwrap();
    for e in events {
        dev.emit(&[InputEvent::new(EventType::KEY.0, e.code(), e.value())])
            .map_err(|err| format!("uinput write: {err}"))?;
    }
    Ok(())
}

/// Tap `code` with exactly the wanted Shift/AltGr state, neutralizing
/// physically held modifiers around it (release-then-restore) so the
/// compositor sees the intended level, then returns to the live state.
/// `altgr_code` is the key pressed to reach levels 3/4 — KEY_RIGHTALT on
/// layouts with a real AltGr, KEY_F24 when our custom xkb option provides
/// the level-3 switch.
pub fn emit_combo(
    code: u16,
    want_shift: bool,
    want_altgr: bool,
    altgr_code: u16,
    held_shifts: &[u16],
    held_altgr: bool,
) -> Result<(), String> {
    let mut seq = Vec::new();
    if want_shift {
        if held_shifts.is_empty() {
            seq.push(KeyEvt::Press(KEY_LEFTSHIFT));
        }
    } else {
        for &s in held_shifts {
            seq.push(KeyEvt::Release(s));
        }
    }
    // Skip the tap only when the wanted level-3 key is the real AltGr and
    // the user is already physically holding it.
    let tap_altgr = want_altgr && !(held_altgr && altgr_code == KEY_RIGHTALT);
    if tap_altgr {
        seq.push(KeyEvt::Press(altgr_code));
    } else if !want_altgr && held_altgr {
        // The physically held AltGr is always the real right-alt key.
        seq.push(KeyEvt::Release(KEY_RIGHTALT));
    }
    seq.push(KeyEvt::Press(code));
    seq.push(KeyEvt::Release(code));
    if tap_altgr {
        seq.push(KeyEvt::Release(altgr_code));
    } else if !want_altgr && held_altgr {
        seq.push(KeyEvt::Press(KEY_RIGHTALT));
    }
    if want_shift {
        if held_shifts.is_empty() {
            seq.push(KeyEvt::Release(KEY_LEFTSHIFT));
        }
    } else {
        for &s in held_shifts {
            seq.push(KeyEvt::Press(s));
        }
    }
    emit(&seq)
}

/// Returns true if (code, value) matches an event we injected; the entry is
/// consumed. Called first for every key event in the grab callback.
pub fn take_loopback(code: u16, value: i32) -> bool {
    let mut reg = REGISTRY.lock().unwrap();
    let now = Instant::now();
    reg.retain(|(_, _, t)| now.duration_since(*t) < LOOPBACK_TTL);
    if let Some(pos) = reg.iter().position(|(c, v, _)| *c == code && *v == value) {
        reg.remove(pos);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Single test for the shared global REGISTRY (parallel tests would race).
    #[test]
    fn loopback_registry_matches_consumes_and_expires() {
        // Use the registry directly (no /dev/uinput in CI).
        {
            let mut reg = REGISTRY.lock().unwrap();
            reg.clear();
            let now = Instant::now();
            reg.push((18, 1, now)); // KEY_E press
            reg.push((18, 0, now)); // KEY_E release
            reg.push((30, 1, now - Duration::from_millis(600))); // expired KEY_A
        }
        assert!(!take_loopback(30, 1)); // expired entry dropped
        assert!(take_loopback(18, 1));
        assert!(!take_loopback(18, 1)); // consumed
        assert!(take_loopback(18, 0));
        assert!(!take_loopback(44, 1)); // never registered
    }

    #[test]
    fn keyevt_codes_and_values() {
        assert_eq!(KeyEvt::Press(57).code(), 57);
        assert_eq!(KeyEvt::Press(57).value(), 1);
        assert_eq!(KeyEvt::Release(57).value(), 0);
    }
}
