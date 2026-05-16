use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MappingKey {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
}

type LangData = &'static [(MappingKey, &'static [char])];

static COMPILED_MAP: OnceLock<HashMap<MappingKey, Vec<char>>> = OnceLock::new();

pub fn init(languages: &[String]) {
    let map = build_map(languages);
    COMPILED_MAP.set(map).ok();
}

pub fn get_variants(key: MappingKey, uppercase: bool) -> Vec<char> {
    let map = COMPILED_MAP.get().expect("mappings not initialized");
    match map.get(&key) {
        Some(chars) if !chars.is_empty() => {
            if uppercase {
                chars.iter().map(|c| c.to_uppercase().next().unwrap_or(*c)).collect()
            } else {
                chars.clone()
            }
        }
        _ => Vec::new(),
    }
}

fn build_map(languages: &[String]) -> HashMap<MappingKey, Vec<char>> {
    let mut map: HashMap<MappingKey, Vec<char>> = HashMap::new();

    for lang_name in languages {
        if let Some(data) = get_language_data(lang_name) {
            for &(key, chars) in data {
                let entry = map.entry(key).or_default();
                for &ch in chars {
                    if !entry.contains(&ch) {
                        entry.push(ch);
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
        "Catalan" => Some(&[
            (MappingKey::A, &['à', 'á']),
            (MappingKey::C, &['ç']),
            (MappingKey::E, &['è', 'é']),
            (MappingKey::I, &['ì', 'í', 'ï']),
            (MappingKey::N, &['ñ']),
            (MappingKey::O, &['ò', 'ó']),
            (MappingKey::U, &['ù', 'ú', 'ü']),
        ]),
        "CrimeanTatar" => Some(&[
            (MappingKey::A, &['â']),
            (MappingKey::C, &['ç']),
            (MappingKey::G, &['ğ']),
            (MappingKey::I, &['ı', 'İ']),
            (MappingKey::N, &['ñ']),
            (MappingKey::O, &['ö']),
            (MappingKey::S, &['ş']),
            (MappingKey::U, &['ü']),
        ]),
        "Croatian" => Some(&[
            (MappingKey::C, &['ć', 'č']),
            (MappingKey::D, &['đ']),
            (MappingKey::S, &['š']),
            (MappingKey::Z, &['ž']),
        ]),
        "Czech" => Some(&[
            (MappingKey::A, &['á']),
            (MappingKey::C, &['č']),
            (MappingKey::D, &['ď']),
            (MappingKey::E, &['ě', 'é']),
            (MappingKey::I, &['í']),
            (MappingKey::N, &['ň']),
            (MappingKey::O, &['ó']),
            (MappingKey::R, &['ř']),
            (MappingKey::S, &['š']),
            (MappingKey::T, &['ť']),
            (MappingKey::U, &['ů', 'ú']),
            (MappingKey::Y, &['ý']),
            (MappingKey::Z, &['ž']),
        ]),
        "Danish" => Some(&[
            (MappingKey::A, &['å', 'æ']),
            (MappingKey::O, &['ø']),
        ]),
        "Dutch" => Some(&[
            (MappingKey::A, &['á', 'à', 'ä']),
            (MappingKey::C, &['ç']),
            (MappingKey::E, &['é', 'è', 'ë', 'ê']),
            (MappingKey::I, &['í', 'ï', 'î']),
            (MappingKey::N, &['ñ']),
            (MappingKey::O, &['ó', 'ö', 'ô']),
            (MappingKey::U, &['ú', 'ü', 'û']),
        ]),
        "Esperanto" => Some(&[
            (MappingKey::C, &['ĉ']),
            (MappingKey::G, &['ĝ']),
            (MappingKey::H, &['ĥ']),
            (MappingKey::J, &['ĵ']),
            (MappingKey::S, &['ŝ']),
            (MappingKey::U, &['ŭ']),
        ]),
        "Estonian" => Some(&[
            (MappingKey::A, &['ä']),
            (MappingKey::O, &['ö', 'õ']),
            (MappingKey::S, &['š']),
            (MappingKey::U, &['ü']),
            (MappingKey::Z, &['ž']),
        ]),
        "Finnish" => Some(&[
            (MappingKey::A, &['ä', 'å']),
            (MappingKey::O, &['ö']),
        ]),
        "French" => Some(&[
            (MappingKey::A, &['à', 'â', 'á', 'ä', 'ã', 'æ']),
            (MappingKey::C, &['ç']),
            (MappingKey::E, &['é', 'è', 'ê', 'ë']),
            (MappingKey::I, &['î', 'ï', 'í', 'ì']),
            (MappingKey::O, &['ô', 'ö', 'ó', 'ò', 'õ', 'œ']),
            (MappingKey::U, &['û', 'ù', 'ü', 'ú']),
            (MappingKey::Y, &['ÿ', 'ý']),
        ]),
        "German" => Some(&[
            (MappingKey::A, &['ä']),
            (MappingKey::O, &['ö']),
            (MappingKey::S, &['ß']),
            (MappingKey::U, &['ü']),
        ]),
        "Greek" => Some(&[
            (MappingKey::A, &['α', 'ά']),
            (MappingKey::B, &['β']),
            (MappingKey::C, &['χ']),
            (MappingKey::D, &['δ']),
            (MappingKey::E, &['ε', 'έ', 'η', 'ή']),
            (MappingKey::F, &['φ']),
            (MappingKey::G, &['γ']),
            (MappingKey::I, &['ι', 'ί']),
            (MappingKey::K, &['κ']),
            (MappingKey::L, &['λ']),
            (MappingKey::M, &['μ']),
            (MappingKey::N, &['ν']),
            (MappingKey::O, &['ο', 'ό', 'ω', 'ώ']),
            (MappingKey::P, &['π', 'φ', 'ψ']),
            (MappingKey::R, &['ρ']),
            (MappingKey::S, &['σ', 'ς']),
            (MappingKey::T, &['τ', 'θ', 'ϑ']),
            (MappingKey::U, &['υ', 'ύ']),
            (MappingKey::X, &['ξ']),
            (MappingKey::Y, &['υ']),
            (MappingKey::Z, &['ζ']),
        ]),
        "Hungarian" => Some(&[
            (MappingKey::A, &['á']),
            (MappingKey::E, &['é']),
            (MappingKey::I, &['í']),
            (MappingKey::O, &['ó', 'ő', 'ö']),
            (MappingKey::U, &['ú', 'ű', 'ü']),
            (MappingKey::Y, &['ÿ', 'ý']),
        ]),
        "IPA" => Some(&[
            (MappingKey::A, &['ɐ', 'ɑ', 'ɒ', 'ǎ']),
            (MappingKey::B, &['ʙ']),
            (MappingKey::E, &['ɘ', 'ɵ', 'ə', 'ɛ', 'ɜ', 'ɞ']),
            (MappingKey::F, &['ɟ', 'ɸ']),
            (MappingKey::G, &['ɢ', 'ɣ']),
            (MappingKey::H, &['ɦ', 'ʜ']),
            (MappingKey::I, &['ɨ', 'ɪ']),
            (MappingKey::J, &['ʝ']),
            (MappingKey::L, &['ɬ', 'ɮ', 'ꞎ', 'ɭ', 'ʎ', 'ʟ', 'ɺ']),
            (MappingKey::N, &['ɳ', 'ɲ', 'ŋ', 'ɴ']),
            (MappingKey::O, &['ɤ', 'ɔ', 'ɶ', 'ǒ']),
            (MappingKey::R, &['ʁ', 'ɹ', 'ɻ', 'ɾ', 'ɽ', 'ʀ']),
            (MappingKey::S, &['ʃ', 'ʂ', 'ɕ']),
            (MappingKey::U, &['ʉ', 'ʊ', 'ǔ']),
            (MappingKey::V, &['ʋ', 'ⱱ', 'ʌ']),
            (MappingKey::W, &['ɰ', 'ɯ']),
            (MappingKey::Y, &['ʏ']),
            (MappingKey::Z, &['ʒ', 'ʐ', 'ʑ']),
        ]),
        "Iceland" => Some(&[
            (MappingKey::A, &['á', 'æ']),
            (MappingKey::D, &['ð']),
            (MappingKey::E, &['é']),
            (MappingKey::I, &['í']),
            (MappingKey::O, &['ó', 'ö']),
            (MappingKey::T, &['þ']),
            (MappingKey::U, &['ú']),
            (MappingKey::Y, &['ý']),
        ]),
        "Irish" => Some(&[
            (MappingKey::A, &['á']),
            (MappingKey::E, &['é']),
            (MappingKey::I, &['í']),
            (MappingKey::O, &['ó']),
            (MappingKey::U, &['ú']),
        ]),
        "Italian" => Some(&[
            (MappingKey::A, &['à']),
            (MappingKey::E, &['è', 'é', 'ə']),
            (MappingKey::I, &['ì', 'í']),
            (MappingKey::O, &['ò', 'ó']),
            (MappingKey::U, &['ù', 'ú']),
        ]),
        "Kurdish" => Some(&[
            (MappingKey::C, &['ç']),
            (MappingKey::E, &['ê']),
            (MappingKey::I, &['î']),
            (MappingKey::L, &['ł']),
            (MappingKey::N, &['ň']),
            (MappingKey::O, &['ö', 'ô']),
            (MappingKey::R, &['ř']),
            (MappingKey::S, &['ş']),
            (MappingKey::U, &['û', 'ü']),
        ]),
        "Lithuanian" => Some(&[
            (MappingKey::A, &['ą']),
            (MappingKey::C, &['č']),
            (MappingKey::E, &['ę', 'ė']),
            (MappingKey::I, &['į']),
            (MappingKey::S, &['š']),
            (MappingKey::U, &['ų', 'ū']),
            (MappingKey::Z, &['ž']),
        ]),
        "Maltese" => Some(&[
            (MappingKey::A, &['à']),
            (MappingKey::C, &['ċ']),
            (MappingKey::E, &['è']),
            (MappingKey::G, &['ġ']),
            (MappingKey::H, &['ħ']),
            (MappingKey::I, &['ì']),
            (MappingKey::O, &['ò']),
            (MappingKey::U, &['ù']),
            (MappingKey::Z, &['ż']),
        ]),
        "Maori" => Some(&[
            (MappingKey::A, &['ā']),
            (MappingKey::E, &['ē']),
            (MappingKey::I, &['ī']),
            (MappingKey::O, &['ō']),
            (MappingKey::U, &['ū']),
        ]),
        "Norwegian" => Some(&[
            (MappingKey::A, &['å', 'æ']),
            (MappingKey::E, &['é']),
            (MappingKey::O, &['ø']),
        ]),
        "Pinyin" => Some(&[
            (MappingKey::A, &['ā', 'á', 'ǎ', 'à']),
            (MappingKey::C, &['ĉ']),
            (MappingKey::E, &['ē', 'é', 'ě', 'è', 'ê']),
            (MappingKey::I, &['ī', 'í', 'ǐ', 'ì']),
            (MappingKey::M, &['ḿ']),
            (MappingKey::N, &['ń', 'ň', 'ǹ', 'ŋ']),
            (MappingKey::O, &['ō', 'ó', 'ǒ', 'ò']),
            (MappingKey::S, &['ŝ']),
            (MappingKey::U, &['ū', 'ú', 'ǔ', 'ù', 'ü', 'ǖ', 'ǘ', 'ǚ', 'ǜ']),
            (MappingKey::V, &['ü', 'ǖ', 'ǘ', 'ǚ', 'ǜ']),
            (MappingKey::Z, &['ẑ']),
        ]),
        "Polish" => Some(&[
            (MappingKey::A, &['ą']),
            (MappingKey::C, &['ć']),
            (MappingKey::E, &['ę']),
            (MappingKey::L, &['ł']),
            (MappingKey::N, &['ń']),
            (MappingKey::O, &['ó']),
            (MappingKey::S, &['ś']),
            (MappingKey::Z, &['ż', 'ź']),
        ]),
        "Portuguese" => Some(&[
            (MappingKey::A, &['á', 'à', 'â', 'ã']),
            (MappingKey::C, &['ç']),
            (MappingKey::E, &['é', 'ê']),
            (MappingKey::I, &['í']),
            (MappingKey::O, &['ô', 'ó', 'õ']),
            (MappingKey::U, &['ú']),
        ]),
        "ProtoIndoEuropean" => Some(&[
            (MappingKey::A, &['ā']),
            (MappingKey::E, &['ē']),
            (MappingKey::G, &['ǵ']),
            (MappingKey::K, &['ḱ']),
            (MappingKey::O, &['ō']),
        ]),
        "Romanian" => Some(&[
            (MappingKey::A, &['ă', 'â']),
            (MappingKey::I, &['î']),
            (MappingKey::S, &['ș']),
            (MappingKey::T, &['ț']),
        ]),
        "Romanization" => Some(&[
            (MappingKey::A, &['á', 'â', 'ă', 'ā']),
            (MappingKey::B, &['ḇ']),
            (MappingKey::C, &['č', 'ç']),
            (MappingKey::D, &['ḑ', 'ḍ', 'ḏ']),
            (MappingKey::E, &['ê', 'ě', 'ĕ', 'ē', 'é', 'ə']),
            (MappingKey::G, &['ġ', 'ǧ', 'ğ', 'ḡ']),
            (MappingKey::H, &['ḧ', 'ḩ', 'ḥ', 'ḫ']),
            (MappingKey::I, &['í', 'ı', 'î', 'ī']),
            (MappingKey::J, &['ǰ']),
            (MappingKey::K, &['ḳ', 'ḵ']),
            (MappingKey::L, &['ł']),
            (MappingKey::N, &['ñ']),
            (MappingKey::O, &['ó', 'ô', 'ö', 'ŏ', 'ō', 'ȫ']),
            (MappingKey::R, &['ṙ', 'ṛ']),
            (MappingKey::S, &['ś', 'š', 'ş', 'ṣ']),
            (MappingKey::T, &['ẗ', 'ţ', 'ṭ', 'ṯ']),
            (MappingKey::U, &['ú', 'û', 'ü', 'ū', 'ǖ']),
            (MappingKey::V, &['ṿ']),
            (MappingKey::Z, &['ż', 'ž', 'ẓ', 'ẕ']),
        ]),
        "ScottishGaelic" => Some(&[
            (MappingKey::A, &['à']),
            (MappingKey::E, &['è']),
            (MappingKey::I, &['ì']),
            (MappingKey::O, &['ò']),
            (MappingKey::U, &['ù']),
        ]),
        "Serbian" => Some(&[
            (MappingKey::C, &['ć', 'č']),
            (MappingKey::D, &['đ']),
            (MappingKey::S, &['š']),
            (MappingKey::Z, &['ž']),
        ]),
        "Slovak" => Some(&[
            (MappingKey::A, &['á', 'ä']),
            (MappingKey::C, &['č']),
            (MappingKey::D, &['ď']),
            (MappingKey::E, &['é']),
            (MappingKey::I, &['í']),
            (MappingKey::L, &['ľ', 'ĺ']),
            (MappingKey::N, &['ň']),
            (MappingKey::O, &['ó', 'ô']),
            (MappingKey::R, &['ŕ']),
            (MappingKey::S, &['š']),
            (MappingKey::T, &['ť']),
            (MappingKey::U, &['ú']),
            (MappingKey::Y, &['ý']),
            (MappingKey::Z, &['ž']),
        ]),
        "Slovenian" => Some(&[
            (MappingKey::C, &['č', 'ć']),
            (MappingKey::S, &['š']),
            (MappingKey::Z, &['ž']),
        ]),
        "Spanish" => Some(&[
            (MappingKey::A, &['á']),
            (MappingKey::E, &['é']),
            (MappingKey::H, &['ḥ']),
            (MappingKey::I, &['í']),
            (MappingKey::L, &['ḷ']),
            (MappingKey::N, &['ñ']),
            (MappingKey::O, &['ó']),
            (MappingKey::U, &['ú', 'ü']),
        ]),
        "Swedish" => Some(&[
            (MappingKey::A, &['å', 'ä']),
            (MappingKey::E, &['é']),
            (MappingKey::O, &['ö']),
        ]),
        "Turkish" => Some(&[
            (MappingKey::A, &['â']),
            (MappingKey::C, &['ç']),
            (MappingKey::E, &['ë']),
            (MappingKey::G, &['ğ']),
            (MappingKey::I, &['ı', 'İ', 'î']),
            (MappingKey::O, &['ö', 'ô']),
            (MappingKey::S, &['ş']),
            (MappingKey::U, &['ü', 'û']),
        ]),
        "Vietnamese" => Some(&[
            (MappingKey::A, &['à', 'ả', 'ã', 'á', 'ạ', 'ă', 'ằ', 'ẳ', 'ẵ', 'ắ', 'ặ', 'â', 'ầ', 'ẩ', 'ẫ', 'ấ', 'ậ']),
            (MappingKey::D, &['đ']),
            (MappingKey::E, &['è', 'ẻ', 'ẽ', 'é', 'ẹ', 'ê', 'ề', 'ể', 'ễ', 'ế', 'ệ']),
            (MappingKey::I, &['ì', 'ỉ', 'ĩ', 'í', 'ị']),
            (MappingKey::O, &['ò', 'ỏ', 'õ', 'ó', 'ọ', 'ô', 'ồ', 'ổ', 'ỗ', 'ố', 'ộ', 'ơ', 'ờ', 'ở', 'ỡ', 'ớ', 'ợ']),
            (MappingKey::U, &['ù', 'ủ', 'ũ', 'ú', 'ụ', 'ư', 'ừ', 'ử', 'ữ', 'ứ', 'ự']),
            (MappingKey::Y, &['ỳ', 'ỷ', 'ỹ', 'ý', 'ỵ']),
        ]),
        "Welsh" => Some(&[
            (MappingKey::A, &['â', 'ä', 'à', 'á']),
            (MappingKey::E, &['ê', 'ë', 'è', 'é']),
            (MappingKey::I, &['î', 'ï', 'ì', 'í']),
            (MappingKey::O, &['ô', 'ö', 'ò', 'ó']),
            (MappingKey::U, &['û', 'ü', 'ù', 'ú']),
            (MappingKey::W, &['ŵ', 'ẅ', 'ẁ', 'ẃ']),
            (MappingKey::Y, &['ŷ', 'ÿ', 'ỳ', 'ý']),
        ]),
        _ => None,
    }
}
