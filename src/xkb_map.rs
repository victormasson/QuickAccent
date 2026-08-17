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

fn compile_keymap(layout: &str, variant: &str, options: Option<String>) -> Option<xkb::Keymap> {
    let ctx = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
    xkb::Keymap::new_from_names(
        &ctx,
        "",
        "",
        layout,
        variant,
        options,
        xkb::KEYMAP_COMPILE_NO_FLAGS,
    )
}

/// The xkb options the session applies: env override, else the desktop's
/// setting (includes our `quickaccent:accents` once installed).
fn active_options() -> Option<String> {
    if let Ok(opts) = std::env::var("XKB_DEFAULT_OPTIONS") {
        return Some(opts);
    }
    crate::xkb_custom::gsettings_get_options().map(|v| v.join(","))
}

fn compile_active_keymap() -> Option<xkb::Keymap> {
    let (layout, variant) = active_layout_variant();

    if variant.is_empty() {
        eprintln!("[QuickAccent] XKB layout: {layout}");
    } else {
        eprintln!("[QuickAccent] XKB layout: {layout} ({variant})");
    }

    compile_keymap(&layout, &variant, active_options())
}

/// Which of `chars` the layout can NOT type before our custom option is
/// applied (drives generation of the option, so it must exclude it).
pub fn chars_missing_from_base(chars: &[char]) -> Vec<char> {
    let (layout, variant) = active_layout_variant();
    let options = active_options().map(|opts| {
        opts.split(',')
            .filter(|o| *o != crate::xkb_custom::OPTION_NAME)
            .collect::<Vec<_>>()
            .join(",")
    });
    let Some(keymap) = compile_keymap(&layout, &variant, options) else {
        return chars.to_vec();
    };
    let combos = build_combos_from(&keymap);
    chars
        .iter()
        .copied()
        .filter(|c| !combos.contains_key(c))
        .collect()
}

/// Resolve the layout the session actually types with. Env overrides first,
/// then the desktop's own setting (GNOME manages layouts in gsettings and can
/// disagree with system-wide localectl), then localectl.
fn active_layout_variant() -> (String, String) {
    if let Ok(layout) = std::env::var("XKB_DEFAULT_LAYOUT").or_else(|_| std::env::var("XKBLAYOUT"))
    {
        return (layout, std::env::var("XKB_DEFAULT_VARIANT").unwrap_or_default());
    }
    detect_gnome_layout()
        .or_else(|| detect_layout().map(|l| (l, String::new())))
        .unwrap_or_else(|| ("us".into(), String::new()))
}

/// First xkb source from GNOME's input-sources settings, e.g. "fr+oss" →
/// ("fr", "oss"). mru-sources lists most-recently-used first; falls back to
/// the configured sources list.
fn detect_gnome_layout() -> Option<(String, String)> {
    ["mru-sources", "sources"]
        .iter()
        .filter_map(|key| crate::xkb_custom::gsettings_get(key))
        .find_map(|text| parse_gnome_source(&text))
}

/// Extract the first ('xkb', '<layout>[+variant]') tuple from a gsettings
/// input-sources value.
fn parse_gnome_source(text: &str) -> Option<(String, String)> {
    let rest = text.split("('xkb', '").nth(1)?;
    let value = rest.split('\'').next()?;
    if value.is_empty() {
        return None;
    }
    let (layout, variant) = match value.split_once('+') {
        Some((l, v)) => (l, v),
        None => (value, ""),
    };
    Some((layout.to_string(), variant.to_string()))
}

fn build_map() -> HashMap<MappingKey, MappingKey> {
    let mut map: HashMap<MappingKey, MappingKey> =
        LETTERS.into_iter().map(|k| (k, k)).collect();

    let Some(keymap) = compile_active_keymap() else {
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

/// A key combo producing a character in the active layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyCombo {
    /// evdev KEY_* code (no +8 XKB offset)
    pub code: u16,
    pub shift: bool,
    pub altgr: bool,
}

static COMBOS: std::sync::RwLock<Option<HashMap<char, KeyCombo>>> =
    std::sync::RwLock::new(None);

/// Find the (keycode, modifiers) combo producing `c` in the active layout —
/// including the keys our custom option adds. Returns None for chars the
/// keymap can't type in one keystroke (caller falls back).
pub fn combo_for_char(c: char) -> Option<KeyCombo> {
    if let Some(map) = COMBOS.read().unwrap().as_ref() {
        return map.get(&c).copied();
    }
    warm_combos();
    COMBOS.read().unwrap().as_ref().and_then(|m| m.get(&c).copied())
}

/// (Re)build the combo map — at startup off the hot path, and again after
/// the custom option's character set changes.
pub fn warm_combos() {
    let map = build_combos();
    *COMBOS.write().unwrap() = Some(map);
}

fn build_combos() -> HashMap<char, KeyCombo> {
    compile_active_keymap()
        .map(|km| build_combos_from(&km))
        .unwrap_or_default()
}

fn build_combos_from(keymap: &xkb::Keymap) -> HashMap<char, KeyCombo> {
    // char → (level, combo); keep the lowest (level, keycode) per char.
    let mut map: HashMap<char, (u32, KeyCombo)> = HashMap::new();
    keymap.key_for_each(|km, keycode| {
        let raw = keycode.raw();
        // XKB keycode = evdev + 8; skip anything outside keyboard range.
        if !(8..=256).contains(&raw) {
            return;
        }
        let code = (raw - 8) as u16;
        // Standard pc105 shift levels: 0 = plain, 1 = Shift, 2 = AltGr, 3 = Shift+AltGr.
        let levels = km.num_levels_for_key(keycode, 0).min(4);
        for level in 0..levels {
            for sym in km.key_get_syms_by_level(keycode, 0, level) {
                let cp = xkb::keysym_to_utf32(*sym);
                let Some(ch) = char::from_u32(cp).filter(|ch| cp != 0 && !ch.is_control())
                else {
                    continue;
                };
                let combo = KeyCombo {
                    code,
                    shift: level == 1 || level == 3,
                    altgr: level == 2 || level == 3,
                };
                match map.get(&ch) {
                    Some((l, existing)) if (*l, existing.code) <= (level, code) => {}
                    _ => {
                        map.insert(ch, (level, combo));
                    }
                }
            }
        }
    });
    map.into_iter().map(|(ch, (_, combo))| (ch, combo)).collect()
}

/// rdev grab-path `Key` → evdev KEY_* code. Mirrors rdev's Linux
/// `evdev_key_to_rdev_key` table exactly (the grab path never produces
/// `Key::Unknown`, and rdev's own `code_from_key` returns X11 codes — wrong
/// layer for uinput). None means "cannot replay through the virtual keyboard".
pub fn evdev_code_of(key: rdev::Key) -> Option<u16> {
    use rdev::Key::*;
    Some(match key {
        Escape => 1,
        Num1 => 2,
        Num2 => 3,
        Num3 => 4,
        Num4 => 5,
        Num5 => 6,
        Num6 => 7,
        Num7 => 8,
        Num8 => 9,
        Num9 => 10,
        Num0 => 11,
        Minus => 12,
        Equal => 13,
        Backspace => 14,
        Tab => 15,
        KeyQ => 16,
        KeyW => 17,
        KeyE => 18,
        KeyR => 19,
        KeyT => 20,
        KeyY => 21,
        KeyU => 22,
        KeyI => 23,
        KeyO => 24,
        KeyP => 25,
        LeftBracket => 26,
        RightBracket => 27,
        Return => 28,
        ControlLeft => 29,
        KeyA => 30,
        KeyS => 31,
        KeyD => 32,
        KeyF => 33,
        KeyG => 34,
        KeyH => 35,
        KeyJ => 36,
        KeyK => 37,
        KeyL => 38,
        SemiColon => 39,
        Quote => 40,
        BackQuote => 41,
        ShiftLeft => 42,
        BackSlash => 43,
        IntlBackslash => 43,
        KeyZ => 44,
        KeyX => 45,
        KeyC => 46,
        KeyV => 47,
        KeyB => 48,
        KeyN => 49,
        KeyM => 50,
        Comma => 51,
        Dot => 52,
        Slash => 53,
        ShiftRight => 54,
        KpMultiply => 55,
        Alt => 56,
        Space => 57,
        CapsLock => 58,
        F1 => 59,
        F2 => 60,
        F3 => 61,
        F4 => 62,
        F5 => 63,
        F6 => 64,
        F7 => 65,
        F8 => 66,
        F9 => 67,
        F10 => 68,
        NumLock => 69,
        ScrollLock => 70,
        Kp7 => 71,
        Kp8 => 72,
        Kp9 => 73,
        KpMinus => 74,
        Kp4 => 75,
        Kp5 => 76,
        Kp6 => 77,
        KpPlus => 78,
        Kp1 => 79,
        Kp2 => 80,
        Kp3 => 81,
        Kp0 => 82,
        F11 => 87,
        F12 => 88,
        KpReturn => 96,
        ControlRight => 97,
        KpDivide => 98,
        AltGr => 100,
        Home => 102,
        UpArrow => 103,
        PageUp => 104,
        LeftArrow => 105,
        RightArrow => 106,
        End => 107,
        DownArrow => 108,
        PageDown => 109,
        Insert => 110,
        Delete => 111,
        KpDelete => 111,
        Pause => 119,
        MetaLeft => 125,
        MetaRight => 126,
        PrintScreen => 210, // rdev maps EV_KEY::KEY_PRINT here
        _ => return None,
    })
}

// linux/input-event-codes.h KEY_* for letters
pub fn evdev_code(k: MappingKey) -> u16 {
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
    fn combo_lookup_french_direct_keys() {
        let keymap = compile_keymap("fr", "", None).expect("fr keymap");
        let combos = build_combos_from(&keymap);
        // AZERTY digit row: é on key 3 (KEY_2), è on key 8 (KEY_7) — unshifted.
        let e_acute = combos.get(&'é').expect("é");
        assert_eq!((e_acute.code, e_acute.shift, e_acute.altgr), (3, false, false));
        let e_grave = combos.get(&'è').expect("è");
        assert!(!e_grave.shift && !e_grave.altgr);
        let c_cedilla = combos.get(&'ç').expect("ç");
        assert!(!c_cedilla.shift && !c_cedilla.altgr);
        // Plain letters resolve too (AZERTY: 'a' on KEY_Q = 16).
        assert_eq!(combos.get(&'a').expect("a").code, 16);
        assert_eq!(combos.get(&'v').expect("v").code, 47);
        // Uppercase A is shifted.
        assert!(combos.get(&'A').expect("A").shift);
    }

    #[test]
    fn combo_lookup_us_has_no_accents() {
        let keymap = compile_keymap("us", "", None).expect("us keymap");
        let combos = build_combos_from(&keymap);
        assert!(combos.get(&'é').is_none());
        assert_eq!(combos.get(&'e').expect("e").code, 18);
        assert_eq!(combos.get(&'v').expect("v").code, 47);
    }

    #[test]
    fn parse_gnome_source_extracts_layout_and_variant() {
        assert_eq!(
            parse_gnome_source("[('xkb', 'fr+oss')]"),
            Some(("fr".into(), "oss".into()))
        );
        assert_eq!(
            parse_gnome_source("[('xkb', 'us'), ('xkb', 'fr')]"),
            Some(("us".into(), String::new()))
        );
        // IBus engines have no xkb tuple to use
        assert_eq!(parse_gnome_source("[('ibus', 'mozc-jp')]"), None);
        assert_eq!(parse_gnome_source("@a(ss) []"), None);
    }

    #[test]
    fn evdev_code_of_matches_input_event_codes() {
        assert_eq!(evdev_code_of(rdev::Key::Space), Some(57));
        assert_eq!(evdev_code_of(rdev::Key::ShiftLeft), Some(42));
        assert_eq!(evdev_code_of(rdev::Key::AltGr), Some(100));
        assert_eq!(evdev_code_of(rdev::Key::Escape), Some(1));
        assert_eq!(evdev_code_of(rdev::Key::KeyE), Some(18));
        assert_eq!(evdev_code_of(rdev::Key::LeftArrow), Some(105));
        assert_eq!(evdev_code_of(rdev::Key::Unknown(86)), None);
    }

    #[test]
    fn evdev_code_of_agrees_with_letter_table() {
        // The MappingKey table and the rdev table must give the same codes.
        assert_eq!(evdev_code_of(rdev::Key::KeyA), Some(evdev_code(MappingKey::A)));
        assert_eq!(evdev_code_of(rdev::Key::KeyQ), Some(evdev_code(MappingKey::Q)));
        assert_eq!(evdev_code_of(rdev::Key::KeyZ), Some(evdev_code(MappingKey::Z)));
        assert_eq!(evdev_code_of(rdev::Key::KeyM), Some(evdev_code(MappingKey::M)));
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
