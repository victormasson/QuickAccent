use std::collections::HashMap;
use std::sync::RwLock;

#[path = "script_mappings.rs"]
mod script_mappings;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MappingKey {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    Comma, Period, Minus, Plus, Slash, Backslash, Divide, Multiply, Quote,
}

type LangData = &'static [(MappingKey, &'static [&'static str])];

/// Every language name `get_language_data` accepts, for the settings UI.
/// Keep in sync with the match below — the test enforces it one way.
pub const LANGUAGES: &[&str] = &[
    "CanadianAboriginalSyllabics", "CanadianAboriginalSyllabicsExtended",
    "CanadianAboriginalSyllabicsExtendedA", "Catalan", "Cherokee",
    "CrimeanTatar", "Croatian", "Czech", "Danish", "Dutch", "Esperanto",
    "Estonian", "Finnish", "French", "German", "Greek", "Hebrew", "Hungarian", "IPA",
    "Iceland", "Irish", "Italian", "Kurdish", "Lithuanian", "Maltese", "Maori",
    "Norwegian", "Osage", "Pinyin", "Polish", "Portuguese", "ProtoIndoEuropean", "Romanian",
    "Romanization", "ScottishGaelic", "Serbian", "Slovak", "Slovenian", "Spanish",
    "Swedish", "Turkish", "Vietnamese", "Welsh", "Yiddish",
];

/// Opt-in symbol sets (see docs/CHARACTERS.md), also valid in `languages`.
pub const SYMBOL_SETS: &[&str] = &[
    "Special", "Currency", "Typography", "Arrows", "Math", "CurrencyExtended",
];

static COMPILED_MAP: RwLock<Option<HashMap<MappingKey, Vec<String>>>> = RwLock::new(None);

/// COMPILED_MAP is process-global and some tests reload it destructively —
/// every test touching mappings must hold this guard or parallel test runs
/// are flaky.
#[cfg(test)]
pub(crate) fn test_guard() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn init(languages: &[String]) {
    let map = build_map(languages);
    *COMPILED_MAP.write().unwrap() = Some(map);
}

pub fn reload(languages: &[String]) {
    let map = build_map(languages);
    *COMPILED_MAP.write().unwrap() = Some(map);
}

/// Every character any current mapping can produce, both cases (used to
/// decide at startup whether direct-injection tiers beyond the keymap are
/// needed).
#[cfg(target_os = "linux")]
pub fn all_variant_chars() -> Vec<char> {
    let guard = COMPILED_MAP.read().unwrap();
    let Some(map) = guard.as_ref() else {
        return Vec::new();
    };
    let mut out: Vec<char> = Vec::new();
    for chars in map.values() {
        for s in chars {
            out.extend(s.chars());
            out.extend(s.chars().flat_map(|c| c.to_uppercase()));
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

pub fn get_variants(key: MappingKey, uppercase: bool) -> Vec<String> {
    let guard = COMPILED_MAP.read().unwrap();
    let map = guard.as_ref().expect("mappings not initialized");
    match map.get(&key) {
        Some(chars) if !chars.is_empty() => {
            if uppercase {
                chars.iter().map(|c| c.to_uppercase()).collect()
            } else {
                chars.clone()
            }
        }
        _ => Vec::new(),
    }
}

fn build_map(languages: &[String]) -> HashMap<MappingKey, Vec<String>> {
    let mut map: HashMap<MappingKey, Vec<String>> = HashMap::new();

    for lang_name in languages {
        if let Some(data) = get_language_data(lang_name) {
            for &(key, chars) in data {
                let entry = map.entry(key).or_default();
                for ch in chars {
                    if !entry.contains(&ch.to_string()) {
                        entry.push(ch.to_string());
                    }
                }
            }
        } else {
            eprintln!("[QuickAccent] Unknown language: {}", lang_name);
        }
    }

    map
}

fn get_language_data(name: &str) -> Option<LangData> {
    match name {
        "Cherokee" => Some(script_mappings::CHEROKEE),
        "Osage" => Some(script_mappings::OSAGE),
        "CanadianAboriginalSyllabics" => Some(script_mappings::CANADIAN),
        "CanadianAboriginalSyllabicsExtended" => Some(script_mappings::CANADIAN_EXTENDED),
        "CanadianAboriginalSyllabicsExtendedA" => Some(script_mappings::CANADIAN_EXTENDED_A),
        // PowerToys PowerAccent.Common/CharacterMappings.cs, MIT licensed.
        // Copyright (c) Microsoft Corporation. See THIRD_PARTY_NOTICES.md.
        "Special" => Some(&[
            (MappingKey::Num0, &["₀", "⁰", "°", "↉", "₎", "⁾"]),
            (MappingKey::Num1, &["₁", "¹", "½", "⅓", "¼", "⅕", "⅙", "⅐", "⅛", "⅑", "⅒"]),
            (MappingKey::Num2, &["₂", "²", "⅔", "⅖"]),
            (MappingKey::Num3, &["₃", "³", "¾", "⅗", "⅜"]),
            (MappingKey::Num4, &["₄", "⁴", "⅘"]),
            (MappingKey::Num5, &["₅", "⁵", "⅚", "⅝"]),
            (MappingKey::Num6, &["₆", "⁶"]),
            (MappingKey::Num7, &["₇", "⁷", "⅞"]),
            (MappingKey::Num8, &["₈", "⁸", "∞"]),
            (MappingKey::Num9, &["₉", "⁹", "₍", "⁽"]),
            (MappingKey::A, &["ȧ", "ǽ", "∀", "ᵃ", "ₐ"]),
            (MappingKey::B, &["ḃ", "ᵇ"]),
            (MappingKey::C, &["ċ", "°C", "©", "ℂ", "∁", "ᶜ"]),
            (MappingKey::D, &["ḍ", "ḋ", "∂", "ᵈ"]),
            (MappingKey::E, &["∈", "∃", "∄", "∉", "ĕ", "ᵉ", "ₑ"]),
            (MappingKey::F, &["ḟ", "°F", "ᶠ"]),
            (MappingKey::G, &["ģ", "ǧ", "ġ", "ĝ", "ǥ", "ᵍ"]),
            (MappingKey::H, &["ḣ", "ĥ", "ħ", "ʰ", "ₕ"]),
            (MappingKey::I, &["ⁱ", "ᵢ"]),
            (MappingKey::J, &["ĵ", "ʲ", "ⱼ"]),
            (MappingKey::K, &["ķ", "ǩ", "ᵏ", "ₖ"]),
            (MappingKey::L, &["ļ", "₺", "ˡ", "ₗ"]),
            (MappingKey::M, &["ṁ", "ᵐ", "ₘ"]),
            (MappingKey::N, &["ņ", "ṅ", "ⁿ", "ℕ", "№", "ₙ"]),
            (MappingKey::O, &["ȯ", "∅", "⌀", "ᵒ", "ₒ"]),
            (MappingKey::P, &["ṗ", "℗", "∏", "¶", "ᵖ", "ₚ", "‰", "‱"]),
            (MappingKey::Q, &["ℚ", "𐞥"]),
            (MappingKey::R, &["ṙ", "®", "ℝ", "ʳ", "ᵣ"]),
            (MappingKey::S, &["ṡ", "§", "∑", "∫", "ˢ", "ₛ"]),
            (MappingKey::T, &["ţ", "ṫ", "ŧ", "™", "ᵗ", "ₜ"]),
            (MappingKey::U, &["ŭ", "ᵘ", "ᵤ"]),
            (MappingKey::V, &["V̇", "ᵛ", "ᵥ"]),
            (MappingKey::W, &["ẇ", "ʷ"]),
            (MappingKey::X, &["ẋ", "×", "ˣ", "ₓ"]),
            (MappingKey::Y, &["ẏ", "ꝡ", "ʸ"]),
            (MappingKey::Z, &["ʒ", "ǯ", "ℤ", "ᶻ"]),
            (MappingKey::Comma, &["∙", "₋", "⁻", "–", "√", "‟", "⟪", "⟫", "‛", "⟨", "⟩", "″", "‴", "⁗"]),
            (MappingKey::Period, &["…", "⁝", "\u{0300}", "\u{0301}", "\u{0302}", "\u{0303}", "\u{0304}", "\u{0308}", "\u{030b}", "\u{030c}"]),
            (MappingKey::Minus, &["~", "‐", "‑", "‒", "–", "—", "―", "⁓", "−", "⸺", "⸻", "∓", "₋", "⁻"]),
            (MappingKey::Slash, &["÷", "√", "‽", "⸘"]),
            (MappingKey::Divide, &["÷", "√"]),
            (MappingKey::Multiply, &["×", "⋅", "ˣ", "ₓ"]),
            (MappingKey::Plus, &["≤", "≥", "≠", "≈", "≙", "⊕", "⊗", "±", "≅", "≡", "₊", "⁺", "₌", "⁼"]),
            (MappingKey::Backslash, &["`", "~"]),
        ]),
        "Currency" => Some(&[
            (MappingKey::B, &["฿", "в"]),
            (MappingKey::C, &["¢", "₡", "č"]),
            (MappingKey::D, &["₫"]),
            (MappingKey::E, &["€"]),
            (MappingKey::F, &["ƒ"]),
            (MappingKey::H, &["₴"]),
            (MappingKey::K, &["₭"]),
            (MappingKey::L, &["ł"]),
            (MappingKey::N, &["л"]),
            (MappingKey::M, &["₼"]),
            (MappingKey::P, &["£", "₽", "₱"]),
            (MappingKey::R, &["₹", "៛", "﷼"]),
            (MappingKey::S, &["$", "₪"]),
            (MappingKey::T, &["₮", "₺", "₸"]),
            (MappingKey::W, &["₩"]),
            (MappingKey::Y, &["¥"]),
        ]),
        "Typography" => Some(&[
            (MappingKey::Quote, &["‘", "’", "“", "”", "„", "‚", "«", "»", "‹", "›"]),
            (MappingKey::Period, &["•", "◦", "▪"]),
            (MappingKey::T, &["†", "‡", "※"]),
            (MappingKey::V, &["✓", "✔"]),
            (MappingKey::X, &["✗", "✘"]),
        ]),
        "Arrows" => Some(&[
            (MappingKey::Minus, &["←", "→", "↑", "↓", "↔", "↕", "↗", "↘", "↙", "↖"]),
            (MappingKey::Plus, &["⇒", "⇐", "⇔"]),
        ]),
        "Math" => Some(&[
            (MappingKey::Plus, &["≔", "≝", "≟", "≢", "∝"]),
            (MappingKey::Comma, &["≪", "≲", "⊂", "⊆"]),
            (MappingKey::Period, &["≫", "≳", "⊃", "⊇"]),
            (MappingKey::I, &["∩"]),
            (MappingKey::U, &["∪"]),
            (MappingKey::A, &["∧"]),
            (MappingKey::O, &["∨"]),
            (MappingKey::N, &["¬"]),
            (MappingKey::T, &["∴", "∵"]),
        ]),
        "CurrencyExtended" => Some(&[
            (MappingKey::A, &["؋"]),
            (MappingKey::B, &["₿"]),
            (MappingKey::C, &["₵", "¤"]),
            (MappingKey::D, &["֏"]),
            (MappingKey::G, &["₲"]),
            (MappingKey::L, &["₾"]),
            (MappingKey::N, &["₦"]),
            (MappingKey::R, &["₨"]),
        ]),
        // Phonetic Latin triggers; final forms are explicit choices, not an IME.
        "Hebrew" => Some(&[
            (MappingKey::A, &["א", "ע", "אַ", "אָ"]),
            (MappingKey::B, &["ב", "בּ"]),
            (MappingKey::C, &["צ", "ץ", "ח", "צ׳"]),
            (MappingKey::D, &["ד", "דּ"]),
            (MappingKey::E, &["אֶ", "אֵ", "ע"]),
            (MappingKey::F, &["פ", "ף", "פֿ"]),
            (MappingKey::G, &["ג", "גּ", "ג׳"]),
            (MappingKey::H, &["ה", "ח", "הּ"]),
            (MappingKey::I, &["י", "אִ"]),
            (MappingKey::J, &["ג׳", "י"]),
            (MappingKey::K, &["כ", "ך", "כּ", "ךּ", "ק"]),
            (MappingKey::L, &["ל"]),
            (MappingKey::M, &["מ", "ם"]),
            (MappingKey::N, &["נ", "ן"]),
            (MappingKey::O, &["וֹ", "אֹ"]),
            (MappingKey::P, &["פּ", "פ", "ף"]),
            (MappingKey::Q, &["ק"]),
            (MappingKey::R, &["ר"]),
            (MappingKey::S, &["ס", "ש", "שׁ", "שׂ"]),
            (MappingKey::T, &["ת", "ט", "תּ", "צ", "ץ"]),
            (MappingKey::U, &["וּ", "אֻ"]),
            (MappingKey::V, &["ו", "ב", "בֿ"]),
            (MappingKey::W, &["ו", "וו"]),
            (MappingKey::X, &["ח", "כ", "ך"]),
            (MappingKey::Y, &["י"]),
            (MappingKey::Z, &["ז", "ז׳"]),
            (MappingKey::Comma, &["׳", "״", "’", "”"]),
            (MappingKey::Minus, &["־"]),
            // Standalone combining marks attach to the preceding Hebrew letter.
            (MappingKey::Period, &["\u{05b0}", "\u{05b1}", "\u{05b2}", "\u{05b3}", "\u{05b4}", "\u{05b5}", "\u{05b6}", "\u{05b7}", "\u{05b8}", "\u{05b9}", "\u{05ba}", "\u{05bb}", "\u{05bc}", "\u{05bd}", "\u{05bf}", "\u{05c1}", "\u{05c2}", "\u{05c7}"]),
        ]),
        "Yiddish" => Some(&[
            (MappingKey::A, &["אַ", "אָ", "א", "ײַ"]),
            (MappingKey::B, &["ב", "בּ", "בֿ"]),
            (MappingKey::C, &["צ", "ץ", "טש"]),
            (MappingKey::D, &["ד", "דזש"]),
            (MappingKey::E, &["ע", "ײ"]),
            (MappingKey::F, &["פֿ", "ף"]),
            (MappingKey::G, &["ג"]),
            (MappingKey::H, &["ה", "ח"]),
            (MappingKey::I, &["י", "יִ"]),
            (MappingKey::J, &["דזש", "י"]),
            (MappingKey::K, &["ק", "כּ", "כ", "ך"]),
            (MappingKey::L, &["ל"]),
            (MappingKey::M, &["מ", "ם"]),
            (MappingKey::N, &["נ", "ן"]),
            (MappingKey::O, &["אָ", "ױ"]),
            (MappingKey::P, &["פּ", "פ", "ף"]),
            (MappingKey::Q, &["ק"]),
            (MappingKey::R, &["ר"]),
            (MappingKey::S, &["ס", "ש", "שׂ", "ת"]),
            (MappingKey::T, &["ט", "תּ", "ת", "צ", "ץ", "טש"]),
            (MappingKey::U, &["ו", "וּ"]),
            (MappingKey::V, &["װ", "בֿ"]),
            (MappingKey::W, &["װ"]),
            (MappingKey::X, &["כ", "ך", "ח"]),
            (MappingKey::Y, &["י", "ײ", "ײַ", "ױ"]),
            (MappingKey::Z, &["ז", "זש"]),
            (MappingKey::Comma, &["׳", "״", "„", "“"]),
            (MappingKey::Minus, &["־"]),
            (MappingKey::Period, &["\u{05b7}", "\u{05b8}", "\u{05b4}", "\u{05bc}", "\u{05bf}", "\u{05c2}"]),
        ]),
        "Catalan" => Some(&[
            (MappingKey::A, &["à", "á"]),
            (MappingKey::C, &["ç"]),
            (MappingKey::E, &["è", "é"]),
            (MappingKey::I, &["ì", "í", "ï"]),
            (MappingKey::N, &["ñ"]),
            (MappingKey::O, &["ò", "ó"]),
            (MappingKey::U, &["ù", "ú", "ü"]),
        ]),
        "CrimeanTatar" => Some(&[
            (MappingKey::A, &["â"]),
            (MappingKey::C, &["ç"]),
            (MappingKey::G, &["ğ"]),
            (MappingKey::I, &["ı", "İ"]),
            (MappingKey::N, &["ñ"]),
            (MappingKey::O, &["ö"]),
            (MappingKey::S, &["ş"]),
            (MappingKey::U, &["ü"]),
        ]),
        "Croatian" => Some(&[
            (MappingKey::C, &["ć", "č"]),
            (MappingKey::D, &["đ"]),
            (MappingKey::S, &["š"]),
            (MappingKey::Z, &["ž"]),
        ]),
        "Czech" => Some(&[
            (MappingKey::A, &["á"]),
            (MappingKey::C, &["č"]),
            (MappingKey::D, &["ď"]),
            (MappingKey::E, &["ě", "é"]),
            (MappingKey::I, &["í"]),
            (MappingKey::N, &["ň"]),
            (MappingKey::O, &["ó"]),
            (MappingKey::R, &["ř"]),
            (MappingKey::S, &["š"]),
            (MappingKey::T, &["ť"]),
            (MappingKey::U, &["ů", "ú"]),
            (MappingKey::Y, &["ý"]),
            (MappingKey::Z, &["ž"]),
        ]),
        "Danish" => Some(&[
            (MappingKey::A, &["å", "æ"]),
            (MappingKey::O, &["ø"]),
        ]),
        "Dutch" => Some(&[
            (MappingKey::A, &["á", "à", "ä"]),
            (MappingKey::C, &["ç"]),
            (MappingKey::E, &["é", "è", "ë", "ê"]),
            (MappingKey::I, &["í", "ï", "î"]),
            (MappingKey::N, &["ñ"]),
            (MappingKey::O, &["ó", "ö", "ô"]),
            (MappingKey::U, &["ú", "ü", "û"]),
        ]),
        "Esperanto" => Some(&[
            (MappingKey::C, &["ĉ"]),
            (MappingKey::G, &["ĝ"]),
            (MappingKey::H, &["ĥ"]),
            (MappingKey::J, &["ĵ"]),
            (MappingKey::S, &["ŝ"]),
            (MappingKey::U, &["ŭ"]),
        ]),
        "Estonian" => Some(&[
            (MappingKey::A, &["ä"]),
            (MappingKey::O, &["ö", "õ"]),
            (MappingKey::S, &["š"]),
            (MappingKey::U, &["ü"]),
            (MappingKey::Z, &["ž"]),
        ]),
        "Finnish" => Some(&[
            (MappingKey::A, &["ä", "å"]),
            (MappingKey::O, &["ö"]),
        ]),
        "French" => Some(&[
            (MappingKey::A, &["à", "â", "á", "ä", "ã", "æ"]),
            (MappingKey::C, &["ç"]),
            (MappingKey::E, &["é", "è", "ê", "ë"]),
            (MappingKey::I, &["î", "ï", "í", "ì"]),
            (MappingKey::O, &["ô", "ö", "ó", "ò", "õ", "œ"]),
            (MappingKey::U, &["û", "ù", "ü", "ú"]),
            (MappingKey::Y, &["ÿ", "ý"]),
        ]),
        "German" => Some(&[
            (MappingKey::A, &["ä"]),
            (MappingKey::O, &["ö"]),
            (MappingKey::S, &["ß"]),
            (MappingKey::U, &["ü"]),
        ]),
        "Greek" => Some(&[
            (MappingKey::A, &["α", "ά"]),
            (MappingKey::B, &["β"]),
            (MappingKey::C, &["χ"]),
            (MappingKey::D, &["δ"]),
            (MappingKey::E, &["ε", "έ", "η", "ή"]),
            (MappingKey::F, &["φ"]),
            (MappingKey::G, &["γ"]),
            (MappingKey::I, &["ι", "ί"]),
            (MappingKey::K, &["κ"]),
            (MappingKey::L, &["λ"]),
            (MappingKey::M, &["μ"]),
            (MappingKey::N, &["ν"]),
            (MappingKey::O, &["ο", "ό", "ω", "ώ"]),
            (MappingKey::P, &["π", "φ", "ψ"]),
            (MappingKey::R, &["ρ"]),
            (MappingKey::S, &["σ", "ς"]),
            (MappingKey::T, &["τ", "θ", "ϑ"]),
            (MappingKey::U, &["υ", "ύ"]),
            (MappingKey::X, &["ξ"]),
            (MappingKey::Y, &["υ"]),
            (MappingKey::Z, &["ζ"]),
        ]),
        "Hungarian" => Some(&[
            (MappingKey::A, &["á"]),
            (MappingKey::E, &["é"]),
            (MappingKey::I, &["í"]),
            (MappingKey::O, &["ó", "ő", "ö"]),
            (MappingKey::U, &["ú", "ű", "ü"]),
            (MappingKey::Y, &["ÿ", "ý"]),
        ]),
        "IPA" => Some(&[
            (MappingKey::A, &["ɐ", "ɑ", "ɑ̃", "ɒ", "æ"]),
            (MappingKey::B, &["β", "ɓ", "ʙ"]),
            (MappingKey::C, &["ç"]),
            (MappingKey::D, &["ð", "ɖ", "ɗ"]),
            (MappingKey::E, &["ə", "ɚ", "ɘ", "ɵ", "ɛ", "ɛ̃", "ɜ", "ɝ", "ɞ"]),
            (MappingKey::F, &["ɸ", "͡", "͜"]),
            (MappingKey::G, &["ɡ", "ɠ", "ɣ", "ɢ", "ʛ"]),
            (MappingKey::H, &["ɦ", "ɥ", "ʜ", "ħ", "ɧ", "ʰ"]),
            (MappingKey::I, &["ɨ", "ɪ", "ɪ̈"]),
            (MappingKey::J, &["ʝ", "ɟ", "ʄ", "ʲ"]),
            (MappingKey::L, &["ɬ", "ɫ", "ɮ", "ꞎ", "ɭ", "ʎ", "ʟ", "ɺ"]),
            (MappingKey::M, &["ɱ"]),
            (MappingKey::N, &["ɳ", "ɲ", "ŋ", "ɴ"]),
            (MappingKey::O, &["ɤ", "ɔ", "ɔ̃", "ø", "œ", "ɶ"]),
            (MappingKey::Q, &["ʔ", "ʕ", "ʡ", "ʢ"]),
            (MappingKey::R, &["ʁ", "ɹ", "ɻ", "ɾ", "ɽ", "ʀ"]),
            (MappingKey::S, &["ʃ", "ʂ", "ɕ"]),
            (MappingKey::T, &["θ", "ʈ"]),
            (MappingKey::U, &["ʉ", "ʊ"]),
            (MappingKey::V, &["ʋ", "ⱱ", "ʌ"]),
            (MappingKey::W, &["ɰ", "ɯ", "ʍ", "ʷ"]),
            (MappingKey::X, &["χ", "ˈ", "ˌ", "ː̆̚"]),
            (MappingKey::Y, &["ʏ"]),
            (MappingKey::Z, &["ʒ", "ʐ", "ʑ"]),
        ]),
        "Iceland" => Some(&[
            (MappingKey::A, &["á", "æ"]),
            (MappingKey::D, &["ð"]),
            (MappingKey::E, &["é"]),
            (MappingKey::I, &["í"]),
            (MappingKey::O, &["ó", "ö"]),
            (MappingKey::T, &["þ"]),
            (MappingKey::U, &["ú"]),
            (MappingKey::Y, &["ý"]),
        ]),
        "Irish" => Some(&[
            (MappingKey::A, &["á"]),
            (MappingKey::E, &["é"]),
            (MappingKey::I, &["í"]),
            (MappingKey::O, &["ó"]),
            (MappingKey::U, &["ú"]),
        ]),
        "Italian" => Some(&[
            (MappingKey::A, &["à"]),
            (MappingKey::E, &["è", "é", "ə"]),
            (MappingKey::I, &["ì", "í"]),
            (MappingKey::O, &["ò", "ó"]),
            (MappingKey::U, &["ù", "ú"]),
        ]),
        "Kurdish" => Some(&[
            (MappingKey::C, &["ç"]),
            (MappingKey::E, &["ê"]),
            (MappingKey::I, &["î"]),
            (MappingKey::L, &["ł"]),
            (MappingKey::N, &["ň"]),
            (MappingKey::O, &["ö", "ô"]),
            (MappingKey::R, &["ř"]),
            (MappingKey::S, &["ş"]),
            (MappingKey::U, &["û", "ü"]),
        ]),
        "Lithuanian" => Some(&[
            (MappingKey::A, &["ą"]),
            (MappingKey::C, &["č"]),
            (MappingKey::E, &["ę", "ė"]),
            (MappingKey::I, &["į"]),
            (MappingKey::S, &["š"]),
            (MappingKey::U, &["ų", "ū"]),
            (MappingKey::Z, &["ž"]),
        ]),
        "Maltese" => Some(&[
            (MappingKey::A, &["à"]),
            (MappingKey::C, &["ċ"]),
            (MappingKey::E, &["è"]),
            (MappingKey::G, &["ġ"]),
            (MappingKey::H, &["ħ"]),
            (MappingKey::I, &["ì"]),
            (MappingKey::O, &["ò"]),
            (MappingKey::U, &["ù"]),
            (MappingKey::Z, &["ż"]),
        ]),
        "Maori" => Some(&[
            (MappingKey::A, &["ā"]),
            (MappingKey::E, &["ē"]),
            (MappingKey::I, &["ī"]),
            (MappingKey::O, &["ō"]),
            (MappingKey::U, &["ū"]),
        ]),
        "Norwegian" => Some(&[
            (MappingKey::A, &["å", "æ"]),
            (MappingKey::E, &["é"]),
            (MappingKey::O, &["ø"]),
        ]),
        "Pinyin" => Some(&[
            (MappingKey::A, &["ā", "á", "ǎ", "à"]),
            (MappingKey::C, &["ĉ"]),
            (MappingKey::E, &["ē", "é", "ě", "è", "ê"]),
            (MappingKey::I, &["ī", "í", "ǐ", "ì"]),
            (MappingKey::M, &["ḿ"]),
            (MappingKey::N, &["ń", "ň", "ǹ", "ŋ"]),
            (MappingKey::O, &["ō", "ó", "ǒ", "ò"]),
            (MappingKey::S, &["ŝ"]),
            (MappingKey::U, &["ū", "ú", "ǔ", "ù", "ü", "ǖ", "ǘ", "ǚ", "ǜ"]),
            (MappingKey::V, &["ü", "ǖ", "ǘ", "ǚ", "ǜ"]),
            (MappingKey::Z, &["ẑ"]),
        ]),
        "Polish" => Some(&[
            (MappingKey::A, &["ą"]),
            (MappingKey::C, &["ć"]),
            (MappingKey::E, &["ę"]),
            (MappingKey::L, &["ł"]),
            (MappingKey::N, &["ń"]),
            (MappingKey::O, &["ó"]),
            (MappingKey::S, &["ś"]),
            (MappingKey::Z, &["ż", "ź"]),
        ]),
        "Portuguese" => Some(&[
            (MappingKey::A, &["á", "à", "â", "ã"]),
            (MappingKey::C, &["ç"]),
            (MappingKey::E, &["é", "ê"]),
            (MappingKey::I, &["í"]),
            (MappingKey::O, &["ô", "ó", "õ"]),
            (MappingKey::U, &["ú"]),
        ]),
        "ProtoIndoEuropean" => Some(&[
            (MappingKey::A, &["ā"]),
            (MappingKey::E, &["ē"]),
            (MappingKey::G, &["ǵ"]),
            (MappingKey::K, &["ḱ"]),
            (MappingKey::O, &["ō"]),
        ]),
        "Romanian" => Some(&[
            (MappingKey::A, &["ă", "â"]),
            (MappingKey::I, &["î"]),
            (MappingKey::S, &["ș"]),
            (MappingKey::T, &["ț"]),
        ]),
        "Romanization" => Some(&[
            (MappingKey::A, &["á", "â", "ă", "ā"]),
            (MappingKey::B, &["ḇ"]),
            (MappingKey::C, &["č", "ç"]),
            (MappingKey::D, &["ḑ", "ḍ", "ḏ"]),
            (MappingKey::E, &["ê", "ě", "ĕ", "ē", "é", "ə"]),
            (MappingKey::G, &["ġ", "ǧ", "ğ", "ḡ"]),
            (MappingKey::H, &["ḧ", "ḩ", "ḥ", "ḫ"]),
            (MappingKey::I, &["í", "ı", "î", "ī"]),
            (MappingKey::J, &["ǰ"]),
            (MappingKey::K, &["ḳ", "ḵ"]),
            (MappingKey::L, &["ł"]),
            (MappingKey::N, &["ñ"]),
            (MappingKey::O, &["ó", "ô", "ö", "ŏ", "ō", "ȫ"]),
            (MappingKey::R, &["ṙ", "ṛ"]),
            (MappingKey::S, &["ś", "š", "ş", "ṣ"]),
            (MappingKey::T, &["ẗ", "ţ", "ṭ", "ṯ"]),
            (MappingKey::U, &["ú", "û", "ü", "ū", "ǖ"]),
            (MappingKey::V, &["ṿ"]),
            (MappingKey::Z, &["ż", "ž", "ẓ", "ẕ"]),
        ]),
        "ScottishGaelic" => Some(&[
            (MappingKey::A, &["à"]),
            (MappingKey::E, &["è"]),
            (MappingKey::I, &["ì"]),
            (MappingKey::O, &["ò"]),
            (MappingKey::U, &["ù"]),
        ]),
        "Serbian" => Some(&[
            (MappingKey::C, &["ć", "č"]),
            (MappingKey::D, &["đ"]),
            (MappingKey::S, &["š"]),
            (MappingKey::Z, &["ž"]),
        ]),
        "Slovak" => Some(&[
            (MappingKey::A, &["á", "ä"]),
            (MappingKey::C, &["č"]),
            (MappingKey::D, &["ď"]),
            (MappingKey::E, &["é"]),
            (MappingKey::I, &["í"]),
            (MappingKey::L, &["ľ", "ĺ"]),
            (MappingKey::N, &["ň"]),
            (MappingKey::O, &["ó", "ô"]),
            (MappingKey::R, &["ŕ"]),
            (MappingKey::S, &["š"]),
            (MappingKey::T, &["ť"]),
            (MappingKey::U, &["ú"]),
            (MappingKey::Y, &["ý"]),
            (MappingKey::Z, &["ž"]),
        ]),
        "Slovenian" => Some(&[
            (MappingKey::C, &["č", "ć"]),
            (MappingKey::S, &["š"]),
            (MappingKey::Z, &["ž"]),
        ]),
        "Spanish" => Some(&[
            (MappingKey::A, &["á"]),
            (MappingKey::E, &["é"]),
            (MappingKey::H, &["ḥ"]),
            (MappingKey::I, &["í"]),
            (MappingKey::L, &["ḷ"]),
            (MappingKey::N, &["ñ"]),
            (MappingKey::O, &["ó"]),
            (MappingKey::U, &["ú", "ü"]),
        ]),
        "Swedish" => Some(&[
            (MappingKey::A, &["å", "ä"]),
            (MappingKey::E, &["é"]),
            (MappingKey::O, &["ö"]),
        ]),
        "Turkish" => Some(&[
            (MappingKey::A, &["â"]),
            (MappingKey::C, &["ç"]),
            (MappingKey::E, &["ë"]),
            (MappingKey::G, &["ğ"]),
            (MappingKey::I, &["ı", "İ", "î"]),
            (MappingKey::O, &["ö", "ô"]),
            (MappingKey::S, &["ş"]),
            (MappingKey::U, &["ü", "û"]),
        ]),
        "Vietnamese" => Some(&[
            (MappingKey::A, &["à", "ả", "ã", "á", "ạ", "ă", "ằ", "ẳ", "ẵ", "ắ", "ặ", "â", "ầ", "ẩ", "ẫ", "ấ", "ậ"]),
            (MappingKey::D, &["đ"]),
            (MappingKey::E, &["è", "ẻ", "ẽ", "é", "ẹ", "ê", "ề", "ể", "ễ", "ế", "ệ"]),
            (MappingKey::I, &["ì", "ỉ", "ĩ", "í", "ị"]),
            (MappingKey::O, &["ò", "ỏ", "õ", "ó", "ọ", "ô", "ồ", "ổ", "ỗ", "ố", "ộ", "ơ", "ờ", "ở", "ỡ", "ớ", "ợ"]),
            (MappingKey::U, &["ù", "ủ", "ũ", "ú", "ụ", "ư", "ừ", "ử", "ữ", "ứ", "ự"]),
            (MappingKey::Y, &["ỳ", "ỷ", "ỹ", "ý", "ỵ"]),
        ]),
        "Welsh" => Some(&[
            (MappingKey::A, &["â", "ä", "à", "á"]),
            (MappingKey::E, &["ê", "ë", "è", "é"]),
            (MappingKey::I, &["î", "ï", "ì", "í"]),
            (MappingKey::O, &["ô", "ö", "ò", "ó"]),
            (MappingKey::U, &["û", "ü", "ù", "ú"]),
            (MappingKey::W, &["ŵ", "ẅ", "ẁ", "ẃ"]),
            (MappingKey::Y, &["ŷ", "ÿ", "ỳ", "ý"]),
        ]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_listed_set_resolves_and_is_unique() {
        let mut seen = std::collections::HashSet::new();
        for name in LANGUAGES.iter().chain(SYMBOL_SETS) {
            assert!(get_language_data(name).is_some(), "{name} is listed but unknown");
            assert!(seen.insert(*name), "{name} listed twice");
        }
        assert!(LANGUAGES.windows(2).all(|w| w[0] < w[1]), "LANGUAGES must stay sorted");
    }

    #[test]
    fn hebrew_yiddish_preserve_alphabets_marks_and_config_order() {
        let _guard = test_guard();
        init(&["French".into()]);
        assert!(!get_variants(MappingKey::A, false).contains(&"א".into()));
        for lang in ["Hebrew", "Yiddish"] {
            init(&[lang.into()]);
            let mut alphabet = String::new();
            for (key, _) in get_language_data(lang).unwrap() {
                let choices = get_variants(*key, false);
                assert!(!choices.is_empty());
                assert!(choices.iter().all(|s| !s.is_empty()));
                assert_eq!(choices, get_variants(*key, true));
                alphabet.push_str(&choices.concat());
            }
            for ch in "אבגדהוזחטיכךלמםנןסעפףצץקרשת".chars() {
                assert!(alphabet.contains(ch), "{lang} missing {ch}");
            }
            assert!(get_variants(MappingKey::Period, true).contains(&"\u{05b7}".into()));
        }
        assert!(get_variants(MappingKey::J, true).contains(&"דזש".into()));
        assert!(get_variants(MappingKey::Y, true).contains(&"ײַ".into()));
        init(&["Yiddish".into(), "Hebrew".into()]);
        let choices = get_variants(MappingKey::A, false);
        assert_eq!(choices[0], "אַ");
        assert_eq!(choices.iter().filter(|s| s.as_str() == "אַ").count(), 1);
        assert!(choices.contains(&"ע".into()));
    }

    #[test]
    fn optional_extensions_have_complete_case_stable_choices() {
        let _guard = test_guard();
        init(&["Special".into(), "Currency".into()]);
        assert!(get_variants(MappingKey::Quote, false).is_empty());
        for (name, expected) in [
            ("Typography", "‘’“”„‚«»‹›•◦▪†‡※✓✔✗✘"),
            ("Arrows", "←→↑↓↔↕↗↘↙↖⇒⇐⇔"),
            ("Math", "≔≝≟≢∝≪≲⊂⊆≫≳⊃⊇∩∪∧∨¬∴∵"),
            ("CurrencyExtended", "؋₿₵¤֏₲₾₦₨"),
        ] {
            init(&[name.into()]);
            let mut actual = std::collections::HashSet::new();
            for &(key, _) in get_language_data(name).unwrap() {
                let choices = get_variants(key, false);
                assert_eq!(choices, get_variants(key, true), "{name}: {key:?}");
                for choice in choices {
                    assert_eq!(choice.chars().count(), 1);
                    assert!(actual.insert(choice.chars().next().unwrap()));
                }
            }
            assert_eq!(actual, expected.chars().collect(), "{name}");
        }
        init(&["Typography".into(), "Special".into()]);
        assert_eq!(get_variants(MappingKey::Quote, false)[..4], ["‘", "’", "“", "”"]);
    }

    #[test]
    fn extended_sets_preserve_symbols_and_sequences() {
        let _guard = test_guard();
        init(&["Special".into(), "Currency".into()]);
        for (key, expected) in [
            (MappingKey::Period, "…"), (MappingKey::Minus, "–"),
            (MappingKey::Minus, "—"), (MappingKey::Minus, "⸺"),
            (MappingKey::Minus, "⸻"), (MappingKey::Plus, "≥"),
            (MappingKey::Num1, "½"), (MappingKey::Num8, "∞"),
            (MappingKey::C, "°C"), (MappingKey::F, "°F"),
            (MappingKey::E, "€"), (MappingKey::S, "₪"),
            (MappingKey::V, "V\u{0307}"),
        ] {
            for uppercase in [false, true] {
                assert!(get_variants(key, uppercase).iter().any(|s| s == expected), "{key:?}: {expected}");
            }
        }

        init(&["German".into()]);
        assert!(get_variants(MappingKey::S, true).contains(&"SS".into()));
    }

    #[test]
    fn extended_sets_are_opt_in_and_merge_in_config_order() {
        let _guard = test_guard();
        init(&["French".into()]);
        assert!(get_variants(MappingKey::Minus, false).is_empty());
        init(&["Typography".into(), "Special".into(), "Typography".into()]);
        let variants = get_variants(MappingKey::Period, false);
        assert_eq!(variants[0], "•");
        assert_eq!(variants.iter().filter(|s| *s == "•").count(), 1);
        assert!(variants.contains(&"…".into()));
    }

    #[test]
    fn french_e_has_accents() {
        let _guard = test_guard();
        init(&["French".into()]);
        let v = get_variants(MappingKey::E, false);
        assert!(v.contains(&"é".into()));
        assert!(v.contains(&"è".into()));
        assert!(!v.is_empty());
    }

    #[test]
    fn uppercase_variants() {
        let _guard = test_guard();
        init(&["French".into()]);
        let v = get_variants(MappingKey::E, true);
        assert!(v.iter().any(|s| s == "É" || s.starts_with('É')));
        assert!(v.iter().all(|s| s.chars().next().unwrap().is_uppercase()));
    }

    #[test]
    fn merge_languages_dedupes() {
        let _guard = test_guard();
        init(&["French".into(), "Spanish".into()]);
        let v = get_variants(MappingKey::E, false);
        let mut seen = std::collections::HashSet::new();
        for ch in &v {
            assert!(seen.insert(ch.clone()), "duplicate {ch}");
        }
        // Spanish/French both contribute e accents
        assert!(v.len() >= 2);
    }

    #[test]
    fn unknown_language_does_not_panic() {
        let _guard = test_guard();
        init(&["NotARealLanguage".into()]);
        let v = get_variants(MappingKey::E, false);
        assert!(v.is_empty());
    }

    #[test]
    fn reload_replaces_map() {
        let _guard = test_guard();
        init(&["French".into()]);
        assert!(!get_variants(MappingKey::E, false).is_empty());
        reload(&["NotARealLanguage".into()]);
        assert!(get_variants(MappingKey::E, false).is_empty());
        reload(&["German".into()]);
        let v = get_variants(MappingKey::A, false);
        assert!(v.contains(&"ä".into()) || !v.is_empty());
    }

    #[test]
    fn key_without_entry_returns_empty() {
        let _guard = test_guard();
        init(&["French".into()]);
        // W has no French entry in typical set
        let french_w = get_language_data("French")
            .map(|d| d.iter().any(|(k, _)| *k == MappingKey::W))
            .unwrap_or(false);
        if !french_w {
            assert!(get_variants(MappingKey::W, false).is_empty());
        }
    }
}
