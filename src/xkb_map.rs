//! Physical evdev key → logical letter via the active XKB layout (AZERTY, etc.).

use crate::mappings::MappingKey;
use std::cell::OnceCell;
use std::collections::HashMap;
use xkbcommon::xkb;

thread_local! {
    static MAP: OnceCell<HashMap<MappingKey, MappingKey>> = const { OnceCell::new() };
}

pub fn logical_letter(physical: MappingKey) -> MappingKey {
    MAP.with(|cell| {
        cell.get_or_init(build_map)
            .get(&physical)
            .copied()
            .unwrap_or(physical)
    })
}

pub fn letter_from_name(name: &str) -> Option<MappingKey> {
    let mut it = name.chars();
    let ch = it.next()?;
    (it.next().is_none()).then_some(ch).and_then(char_to_key)
}

fn char_to_key(ch: char) -> Option<MappingKey> {
    Some(match ch.to_ascii_lowercase() {
        'a' => MappingKey::A,
        'b' => MappingKey::B,
        'c' => MappingKey::C,
        'd' => MappingKey::D,
        'e' => MappingKey::E,
        'f' => MappingKey::F,
        'g' => MappingKey::G,
        'h' => MappingKey::H,
        'i' => MappingKey::I,
        'j' => MappingKey::J,
        'k' => MappingKey::K,
        'l' => MappingKey::L,
        'm' => MappingKey::M,
        'n' => MappingKey::N,
        'o' => MappingKey::O,
        'p' => MappingKey::P,
        'q' => MappingKey::Q,
        'r' => MappingKey::R,
        's' => MappingKey::S,
        't' => MappingKey::T,
        'u' => MappingKey::U,
        'v' => MappingKey::V,
        'w' => MappingKey::W,
        'x' => MappingKey::X,
        'y' => MappingKey::Y,
        'z' => MappingKey::Z,
        _ => return None,
    })
}

fn build_map() -> HashMap<MappingKey, MappingKey> {
    let layout = std::env::var("XKB_DEFAULT_LAYOUT")
        .or_else(|_| std::env::var("XKBLAYOUT"))
        .unwrap_or_else(|_| detect_layout().unwrap_or_else(|| "us".into()));
    let variant = std::env::var("XKB_DEFAULT_VARIANT").unwrap_or_default();

    if variant.is_empty() {
        eprintln!("[QuickAccent] XKB layout: {layout}");
    } else {
        eprintln!("[QuickAccent] XKB layout: {layout} ({variant})");
    }

    let mut map: HashMap<MappingKey, MappingKey> =
        LETTERS.into_iter().map(|k| (k, k)).collect();

    let ctx = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
    let Some(keymap) = xkb::Keymap::new_from_names(
        &ctx,
        "",
        "",
        &layout,
        &variant,
        std::env::var("XKB_DEFAULT_OPTIONS").ok(),
        xkb::KEYMAP_COMPILE_NO_FLAGS,
    ) else {
        eprintln!("[QuickAccent] XKB keymap failed; using physical keys");
        return map;
    };
    let state = xkb::State::new(&keymap);

    for physical in LETTERS {
        let code = xkb::Keycode::new(u32::from(evdev_code(physical)) + 8);
        if let Some(logical) = state.key_get_utf8(code).chars().next().and_then(char_to_key) {
            map.insert(physical, logical);
        }
    }
    map
}

fn detect_layout() -> Option<String> {
    let out = std::process::Command::new("localectl").arg("status").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for prefix in ["X11 Layout:", "VC Keymap:"] {
        if let Some(line) = text.lines().find(|l| l.trim_start().starts_with(prefix)) {
            let v = line.split_once(':')?.1.trim().split(',').next()?.trim();
            if !v.is_empty() && v != "n/a" {
                return Some(v.into());
            }
        }
    }
    None
}

// linux/input-event-codes.h KEY_* for letters
fn evdev_code(k: MappingKey) -> u16 {
    match k {
        MappingKey::A => 30,
        MappingKey::B => 48,
        MappingKey::C => 46,
        MappingKey::D => 32,
        MappingKey::E => 18,
        MappingKey::F => 33,
        MappingKey::G => 34,
        MappingKey::H => 35,
        MappingKey::I => 23,
        MappingKey::J => 36,
        MappingKey::K => 37,
        MappingKey::L => 38,
        MappingKey::M => 50,
        MappingKey::N => 49,
        MappingKey::O => 24,
        MappingKey::P => 25,
        MappingKey::Q => 16,
        MappingKey::R => 19,
        MappingKey::S => 31,
        MappingKey::T => 20,
        MappingKey::U => 22,
        MappingKey::V => 47,
        MappingKey::W => 17,
        MappingKey::X => 45,
        MappingKey::Y => 21,
        MappingKey::Z => 44,
    }
}

const LETTERS: [MappingKey; 26] = [
    MappingKey::A, MappingKey::B, MappingKey::C, MappingKey::D, MappingKey::E,
    MappingKey::F, MappingKey::G, MappingKey::H, MappingKey::I, MappingKey::J,
    MappingKey::K, MappingKey::L, MappingKey::M, MappingKey::N, MappingKey::O,
    MappingKey::P, MappingKey::Q, MappingKey::R, MappingKey::S, MappingKey::T,
    MappingKey::U, MappingKey::V, MappingKey::W, MappingKey::X, MappingKey::Y,
    MappingKey::Z,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letter_from_name_single_ascii() {
        assert_eq!(letter_from_name("a"), Some(MappingKey::A));
        assert_eq!(letter_from_name("E"), Some(MappingKey::E));
        assert_eq!(letter_from_name("z"), Some(MappingKey::Z));
    }

    #[test]
    fn letter_from_name_rejects_multi_or_empty() {
        assert_eq!(letter_from_name(""), None);
        assert_eq!(letter_from_name("ab"), None);
        assert_eq!(letter_from_name("1"), None);
        assert_eq!(letter_from_name("é"), None);
    }

    #[test]
    fn char_to_key_case_insensitive() {
        assert_eq!(char_to_key('Q'), Some(MappingKey::Q));
        assert_eq!(char_to_key('q'), Some(MappingKey::Q));
        assert_eq!(char_to_key('@'), None);
    }

    #[test]
    fn us_layout_identity_for_physical_qwerty_letters() {
        // Force US layout for this process/thread map build.
        std::env::set_var("XKB_DEFAULT_LAYOUT", "us");
        std::env::set_var("XKB_DEFAULT_VARIANT", "");
        // logical_letter uses thread_local OnceCell — first call wins per thread.
        // Run in a fresh thread so env is picked up.
        let handle = std::thread::spawn(|| {
            assert_eq!(logical_letter(MappingKey::Q), MappingKey::Q);
            assert_eq!(logical_letter(MappingKey::A), MappingKey::A);
            assert_eq!(logical_letter(MappingKey::E), MappingKey::E);
        });
        handle.join().expect("xkb thread");
    }
}
