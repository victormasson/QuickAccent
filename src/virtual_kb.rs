//! Kernel-level key injection via a uinput virtual keyboard (Linux only).
//!
//! All replayed/injected keystrokes go through this single device so their
//! ordering is preserved and no display-server permission (portal) is needed.
//! The device is excluded from the evdev grab by name (see
//! `rdev::grab_skip_device_named`), so injected events go straight to the
//! compositor without looping back through our own grab callback.

use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, EventType, InputEvent, KeyCode};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Mutex, OnceLock};

pub use crate::state_machine::{KeyEvt, KEY_LEFTCTRL, KEY_RIGHTALT};

pub const DEVICE_NAME: &str = "QuickAccent Virtual Keyboard";

static DEVICE: OnceLock<Mutex<VirtualDevice>> = OnceLock::new();

/// Key pressed to reach xkb levels 3/4: the real AltGr by default, KEY_F24
/// when the custom xkb option provides the level-3 switch (set once at
/// startup by `setup_direct_typing`).
static LEVEL3_CODE: AtomicU16 = AtomicU16::new(KEY_RIGHTALT);

pub fn set_level3_code(code: u16) {
    LEVEL3_CODE.store(code, Ordering::Relaxed);
}

/// Create the virtual keyboard. Must be called before the grab starts.
pub fn init() -> Result<(), String> {
    let mut keys = AttributeSet::<KeyCode>::new();
    // KEY_ESC(1) ..= KEY_MICMUTE(248): every key a physical keyboard can send.
    for code in 1..=248u16 {
        keys.insert(KeyCode::new(code));
    }
    let dev = VirtualDevice::builder()
        .map_err(|e| format!("open /dev/uinput: {e}"))?
        .name(DEVICE_NAME)
        .with_keys(&keys)
        .map_err(|e| format!("set key capabilities: {e}"))?
        .build()
        .map_err(|e| format!("create virtual device: {e}"))?;
    DEVICE
        .set(Mutex::new(dev))
        .map_err(|_| "virtual keyboard already initialized".to_string())
}

/// Emit key events, each in its own SYN_REPORT frame so modifier state is
/// applied strictly in order by the compositor. Never emits autorepeat
/// (value 2) — clients synthesize repeats themselves.
pub fn emit(events: &[KeyEvt]) -> Result<(), String> {
    let Some(dev) = DEVICE.get() else {
        return Err("virtual keyboard not initialized".into());
    };
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
pub fn emit_combo(
    code: u16,
    want_shift: bool,
    want_altgr: bool,
    held_shifts: &[u16],
    held_altgr: bool,
) -> Result<(), String> {
    emit(&combo_sequence(
        code,
        want_shift,
        want_altgr,
        LEVEL3_CODE.load(Ordering::Relaxed),
        held_shifts,
        held_altgr,
    ))
}

fn combo_sequence(
    code: u16,
    want_shift: bool,
    want_altgr: bool,
    altgr_code: u16,
    held_shifts: &[u16],
    held_altgr: bool,
) -> Vec<KeyEvt> {
    use crate::state_machine::KEY_LEFTSHIFT;
    // Wrappers are collected as (before, after) pairs; the epilogue runs in
    // reverse so modifiers unwind in the opposite order they were applied.
    let mut pre = Vec::new();
    let mut post = Vec::new();
    if want_shift {
        if held_shifts.is_empty() {
            pre.push(KeyEvt::Press(KEY_LEFTSHIFT));
            post.push(KeyEvt::Release(KEY_LEFTSHIFT));
        }
    } else {
        for &s in held_shifts {
            pre.push(KeyEvt::Release(s));
            post.push(KeyEvt::Press(s));
        }
    }
    // Skip the tap only when the wanted level-3 key is the real AltGr and
    // the user is already physically holding it.
    let tap_altgr = want_altgr && !(held_altgr && altgr_code == KEY_RIGHTALT);
    if tap_altgr {
        pre.push(KeyEvt::Press(altgr_code));
        post.push(KeyEvt::Release(altgr_code));
    } else if !want_altgr && held_altgr {
        // The physically held AltGr is always the real right-alt key.
        pre.push(KeyEvt::Release(KEY_RIGHTALT));
        post.push(KeyEvt::Press(KEY_RIGHTALT));
    }
    post.reverse();
    let mut seq = pre;
    seq.push(KeyEvt::Press(code));
    seq.push(KeyEvt::Release(code));
    seq.extend(post);
    seq
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_machine::KEY_LEFTSHIFT;
    use KeyEvt::{Press, Release};

    #[test]
    fn plain_combo_is_a_bare_tap() {
        assert_eq!(
            combo_sequence(18, false, false, KEY_RIGHTALT, &[], false),
            vec![Press(18), Release(18)]
        );
    }

    #[test]
    fn shift_combo_wraps_when_shift_not_held() {
        assert_eq!(
            combo_sequence(18, true, false, KEY_RIGHTALT, &[], false),
            vec![
                Press(KEY_LEFTSHIFT),
                Press(18),
                Release(18),
                Release(KEY_LEFTSHIFT)
            ]
        );
    }

    #[test]
    fn held_shift_is_neutralized_and_restored() {
        assert_eq!(
            combo_sequence(18, false, false, KEY_RIGHTALT, &[42, 54], false),
            vec![
                Release(42),
                Release(54),
                Press(18),
                Release(18),
                Press(54),
                Press(42)
            ]
        );
    }

    #[test]
    fn altgr_level_uses_configured_level3_key() {
        // Custom-option case: F24 (194) is the level-3 switch.
        assert_eq!(
            combo_sequence(183, false, true, 194, &[], false),
            vec![Press(194), Press(183), Release(183), Release(194)]
        );
        // Real AltGr already physically held: no tap needed.
        assert_eq!(
            combo_sequence(3, false, true, KEY_RIGHTALT, &[], true),
            vec![Press(3), Release(3)]
        );
    }
}
