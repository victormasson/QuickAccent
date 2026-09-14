//! Unicode 17.0.0 repertoires, grouped by Unicode syllable/letter names.
//! https://www.unicode.org/Public/17.0.0/ucd/UnicodeData.txt
//! Unicode License V3: see THIRD_PARTY_NOTICES.md.
//! Base choices use lowercase for cased scripts; the existing Shift path supplies capitals.
//! Within each key, order follows the uppercase (or uncased) code point.
use super::{LangData, MappingKey};

pub(super) const CHEROKEE: LangData = &[
    (MappingKey::A, &["ꭰ"]),
    (MappingKey::D, &["ꮣ", "ꮥ", "ꮧ", "ꮩ", "ꮪ", "ꮫ", "ꮬ"]),
    (MappingKey::E, &["ꭱ"]),
    (MappingKey::G, &["ꭶ", "ꭸ", "ꭹ", "ꭺ", "ꭻ", "ꭼ"]),
    (MappingKey::H, &["ꭽ", "ꭾ", "ꭿ", "ꮀ", "ꮁ", "ꮂ", "ꮏ"]),
    (MappingKey::I, &["ꭲ"]),
    (MappingKey::K, &["ꭷ"]),
    (MappingKey::L, &["ꮃ", "ꮄ", "ꮅ", "ꮆ", "ꮇ", "ꮈ"]),
    (MappingKey::M, &["ꮉ", "ꮊ", "ꮋ", "ꮌ", "ꮍ", "ᏽ"]),
    (MappingKey::N, &["ꮎ", "ꮐ", "ꮑ", "ꮒ", "ꮓ", "ꮔ", "ꮕ"]),
    (MappingKey::O, &["ꭳ"]),
    (MappingKey::Q, &["ꮖ", "ꮗ", "ꮘ", "ꮙ", "ꮚ", "ꮛ"]),
    (MappingKey::S, &["ꮜ", "ꮝ", "ꮞ", "ꮟ", "ꮠ", "ꮡ", "ꮢ"]),
    (
        MappingKey::T,
        &[
            "ꮤ", "ꮦ", "ꮨ", "ꮭ", "ꮮ", "ꮯ", "ꮰ", "ꮱ", "ꮲ", "ꮳ", "ꮴ", "ꮵ", "ꮶ", "ꮷ", "ꮸ",
        ],
    ),
    (MappingKey::U, &["ꭴ"]),
    (MappingKey::V, &["ꭵ"]),
    (MappingKey::W, &["ꮹ", "ꮺ", "ꮻ", "ꮼ", "ꮽ", "ꮾ"]),
    (MappingKey::Y, &["ꮿ", "ᏸ", "ᏹ", "ᏺ", "ᏻ", "ᏼ"]),
];

// EH-prefixed Osage consonants live with their unprefixed counterparts (EHKA -> K).
pub(super) const OSAGE: LangData = &[
    (MappingKey::A, &["𐓘", "𐓙", "𐓚", "𐓛"]),
    (MappingKey::B, &["𐓜"]),
    (MappingKey::C, &["𐓝", "𐓞"]),
    (MappingKey::D, &["𐓵"]),
    (MappingKey::E, &["𐓟", "𐓠"]),
    (MappingKey::G, &["𐓹"]),
    (MappingKey::H, &["𐓡", "𐓢"]),
    (MappingKey::I, &["𐓣"]),
    (MappingKey::K, &["𐓤", "𐓥", "𐓦", "𐓸"]),
    (MappingKey::L, &["𐓧"]),
    (MappingKey::M, &["𐓨"]),
    (MappingKey::N, &["𐓩"]),
    (MappingKey::O, &["𐓪", "𐓫"]),
    (MappingKey::P, &["𐓬", "𐓭"]),
    (MappingKey::S, &["𐓮", "𐓯"]),
    (MappingKey::T, &["𐓰", "𐓱", "𐓲", "𐓳", "𐓴"]),
    (MappingKey::U, &["𐓶"]),
    (MappingKey::W, &["𐓷"]),
    (MappingKey::Z, &["𐓺", "𐓻"]),
];

// Language/region prefixes do not determine the key: CARRIER GHU -> G, not C.
// Shape-named finals and full stop use Period; glottal stop uses Quote;
// hyphen uses Minus and the Chi sign uses X. These are lookup bindings, not an IME.
pub(super) const CANADIAN: LangData = &[
    (MappingKey::A, &["ᐂ", "ᐊ", "ᐋ", "ᐜ", "ᐮ", "ᖳ"]),
    (MappingKey::B, &["ᖯ"]),
    (
        MappingKey::C,
        &[
            "ᒉ", "ᒊ", "ᒋ", "ᒌ", "ᒍ", "ᒎ", "ᒏ", "ᒐ", "ᒑ", "ᒒ", "ᒓ", "ᒔ", "ᒕ", "ᒖ", "ᒗ", "ᒘ", "ᒙ",
            "ᒚ", "ᒛ", "ᒜ", "ᒝ", "ᒞ", "ᒟ", "ᒠ", "ᒡ", "ᙡ", "ᙢ", "ᙣ", "ᙤ", "ᙥ", "ᙦ",
        ],
    ),
    (
        MappingKey::D,
        &[
            "ᑓ", "ᑔ", "ᘨ", "ᘩ", "ᘪ", "ᘫ", "ᘬ", "ᘭ", "ᙈ", "ᙉ", "ᙊ", "ᙋ", "ᙌ", "ᙍ",
        ],
    ),
    (MappingKey::E, &["ᐁ", "ᐈ", "ᐫ", "ᖰ"]),
    (
        MappingKey::F,
        &["ᕓ", "ᕔ", "ᕕ", "ᕖ", "ᕗ", "ᕘ", "ᕙ", "ᕚ", "ᕛ", "ᕜ", "ᕝ"],
    ),
    (
        MappingKey::G,
        &["ᗄ", "ᗅ", "ᗆ", "ᗇ", "ᗈ", "ᗉ", "ᗯ", "ᗰ", "ᗱ", "ᗲ", "ᗳ", "ᗴ"],
    ),
    (
        MappingKey::H,
        &[
            "ᐶ", "ᐷ", "ᑋ", "ᕴ", "ᕵ", "ᕶ", "ᕷ", "ᕸ", "ᕹ", "ᕺ", "ᕻ", "ᕼ", "ᕽ", "ᗀ", "ᗁ", "ᗂ", "ᗃ",
            "ᗖ", "ᗗ", "ᗘ", "ᗙ", "ᗚ", "ᗛ",
        ],
    ),
    (MappingKey::I, &["ᐃ", "ᐄ", "ᐉ", "ᐬ", "ᖱ"]),
    (
        MappingKey::J,
        &[
            "ᘔ", "ᘕ", "ᘖ", "ᘗ", "ᘘ", "ᘙ", "ᘚ", "ᘛ", "ᘜ", "ᘝ", "ᘞ", "ᘟ", "ᘠ", "ᘡ",
        ],
    ),
    (
        MappingKey::K,
        &[
            "ᑫ", "ᑬ", "ᑭ", "ᑮ", "ᑯ", "ᑰ", "ᑱ", "ᑲ", "ᑳ", "ᑴ", "ᑵ", "ᑶ", "ᑷ", "ᑸ", "ᑹ", "ᑺ", "ᑻ",
            "ᑼ", "ᑽ", "ᑾ", "ᑿ", "ᒀ", "ᒁ", "ᒂ", "ᒃ", "ᒄ", "ᒅ", "ᒆ", "ᒇ", "ᒈ", "ᖼ", "ᖽ", "ᖾ", "ᖿ",
            "ᗵ", "ᗶ", "ᗷ", "ᗸ", "ᗹ", "ᗺ", "ᗻ", "ᗼ", "ᗽ", "ᗾ", "ᗿ", "ᘀ", "ᘁ",
        ],
    ),
    (
        MappingKey::L,
        &[
            "ᓓ", "ᓔ", "ᓕ", "ᓖ", "ᓗ", "ᓘ", "ᓙ", "ᓚ", "ᓛ", "ᓜ", "ᓝ", "ᓞ", "ᓟ", "ᓠ", "ᓡ", "ᓢ", "ᓣ",
            "ᓤ", "ᓥ", "ᓦ", "ᓧ", "ᓨ", "ᓩ", "ᓪ", "ᓫ", "ᓬ", "ᕄ", "ᕊ", "ᕍ", "ᖠ", "ᖡ", "ᖢ", "ᖣ", "ᖤ",
            "ᖥ", "ᖦ", "ᘢ", "ᘣ", "ᘤ", "ᘥ", "ᘦ", "ᘧ", "ᘮ", "ᘯ", "ᘰ", "ᘱ", "ᘲ", "ᘳ",
        ],
    ),
    (
        MappingKey::M,
        &[
            "ᒣ", "ᒤ", "ᒥ", "ᒦ", "ᒧ", "ᒨ", "ᒩ", "ᒪ", "ᒫ", "ᒬ", "ᒭ", "ᒮ", "ᒯ", "ᒰ", "ᒱ", "ᒲ", "ᒳ",
            "ᒴ", "ᒵ", "ᒶ", "ᒷ", "ᒸ", "ᒹ", "ᒺ", "ᒻ", "ᒼ", "ᒽ", "ᒾ", "ᒿ", "ᘈ", "ᘉ", "ᘊ", "ᘋ", "ᘌ",
            "ᘍ",
        ],
    ),
    (MappingKey::Minus, &["᐀"]),
    (
        MappingKey::N,
        &[
            "ᓀ", "ᓁ", "ᓂ", "ᓃ", "ᓄ", "ᓅ", "ᓆ", "ᓇ", "ᓈ", "ᓉ", "ᓊ", "ᓋ", "ᓌ", "ᓍ", "ᓎ", "ᓏ", "ᓐ",
            "ᓑ", "ᓒ", "ᖎ", "ᖏ", "ᖐ", "ᖑ", "ᖒ", "ᖓ", "ᖔ", "ᖕ", "ᖖ", "ᖸ", "ᖹ", "ᖺ", "ᖻ", "ᘂ", "ᘃ",
            "ᘄ", "ᘅ", "ᘆ", "ᘇ", "ᙰ", "ᙱ", "ᙲ", "ᙳ", "ᙴ", "ᙵ", "ᙶ",
        ],
    ),
    (MappingKey::O, &["ᐅ", "ᐆ", "ᐇ", "ᐭ", "ᖲ"]),
    (
        MappingKey::P,
        &[
            "ᐯ", "ᐰ", "ᐱ", "ᐲ", "ᐳ", "ᐴ", "ᐵ", "ᐸ", "ᐹ", "ᐺ", "ᐻ", "ᐼ", "ᐽ", "ᐾ", "ᐿ", "ᑀ", "ᑁ",
            "ᑂ", "ᑃ", "ᑄ", "ᑅ", "ᑆ", "ᑇ", "ᑈ", "ᑉ", "ᑊ", "ᗨ", "ᗩ", "ᗪ", "ᗫ", "ᗬ", "ᗭ", "ᗮ",
        ],
    ),
    (
        MappingKey::Period,
        &[
            "ᐟ", "ᐠ", "ᐡ", "ᐢ", "ᐣ", "ᐤ", "ᐥ", "ᐦ", "ᐧ", "ᐨ", "ᐩ", "ᐪ", "᙮",
        ],
    ),
    (
        MappingKey::Q,
        &["ᕾ", "ᕿ", "ᖀ", "ᖁ", "ᖂ", "ᖃ", "ᖄ", "ᖅ", "ᙯ"],
    ),
    (MappingKey::Quote, &["ᐞ"]),
    (
        MappingKey::R,
        &[
            "ᕂ", "ᕃ", "ᕅ", "ᕆ", "ᕇ", "ᕈ", "ᕉ", "ᕋ", "ᕌ", "ᕎ", "ᕏ", "ᕐ", "ᕑ", "ᕒ", "ᖊ", "ᖋ", "ᖌ",
            "ᖍ", "ᗊ", "ᗋ", "ᗌ", "ᗍ", "ᗎ", "ᗏ",
        ],
    ),
    (
        MappingKey::S,
        &[
            "ᓭ", "ᓮ", "ᓯ", "ᓰ", "ᓱ", "ᓲ", "ᓳ", "ᓴ", "ᓵ", "ᓶ", "ᓷ", "ᓸ", "ᓹ", "ᓺ", "ᓻ", "ᓼ", "ᓽ",
            "ᓾ", "ᓿ", "ᔀ", "ᔁ", "ᔂ", "ᔃ", "ᔄ", "ᔅ", "ᔆ", "ᔇ", "ᔈ", "ᔉ", "ᔊ", "ᔋ", "ᔌ", "ᔍ", "ᔎ",
            "ᔏ", "ᔐ", "ᔑ", "ᔒ", "ᔓ", "ᔔ", "ᔕ", "ᔖ", "ᔗ", "ᔘ", "ᔙ", "ᔚ", "ᔛ", "ᔜ", "ᔝ", "ᔞ", "ᔟ",
            "ᔠ", "ᔡ", "ᔢ", "ᔣ", "ᔤ", "ᔥ", "ᖗ", "ᖘ", "ᖙ", "ᖚ", "ᙎ", "ᙏ", "ᙐ", "ᙑ", "ᙒ", "ᙓ", "ᙔ",
            "ᙕ", "ᙖ", "ᙗ", "ᙘ", "ᙙ", "ᙚ",
        ],
    ),
    (
        MappingKey::T,
        &[
            "ᑌ", "ᑍ", "ᑎ", "ᑏ", "ᑐ", "ᑑ", "ᑒ", "ᑕ", "ᑖ", "ᑗ", "ᑘ", "ᑙ", "ᑚ", "ᑛ", "ᑜ", "ᑝ", "ᑞ",
            "ᑟ", "ᑠ", "ᑡ", "ᑢ", "ᑣ", "ᑤ", "ᑥ", "ᑦ", "ᑧ", "ᑨ", "ᑩ", "ᑪ", "ᒢ", "ᕞ", "ᕟ", "ᕠ", "ᕡ",
            "ᕢ", "ᕣ", "ᕤ", "ᕥ", "ᕦ", "ᕧ", "ᕨ", "ᕩ", "ᕪ", "ᕫ", "ᕬ", "ᕭ", "ᕮ", "ᕯ", "ᕰ", "ᕱ", "ᕲ",
            "ᕳ", "ᖆ", "ᖇ", "ᖈ", "ᖉ", "ᖛ", "ᖜ", "ᖝ", "ᖞ", "ᖟ", "ᖧ", "ᖨ", "ᖩ", "ᖪ", "ᖫ", "ᖬ", "ᖭ",
            "ᖮ", "ᗜ", "ᗝ", "ᗞ", "ᗟ", "ᗠ", "ᗡ", "ᗢ", "ᗣ", "ᗤ", "ᗥ", "ᗦ", "ᗧ", "ᘴ", "ᘵ", "ᘶ", "ᘷ",
            "ᘸ", "ᘹ", "ᘺ", "ᘻ", "ᘼ", "ᘽ", "ᘾ", "ᘿ", "ᙛ", "ᙜ", "ᙝ", "ᙞ", "ᙟ", "ᙠ", "ᙧ", "ᙨ", "ᙩ",
            "ᙪ", "ᙫ", "ᙬ", "ᙷ", "ᙸ", "ᙹ", "ᙺ", "ᙻ", "ᙼ", "ᙽ", "ᙾ",
        ],
    ),
    (
        MappingKey::W,
        &[
            "ᐌ", "ᐍ", "ᐎ", "ᐏ", "ᐐ", "ᐑ", "ᐒ", "ᐓ", "ᐔ", "ᐕ", "ᐖ", "ᐗ", "ᐘ", "ᐙ", "ᐚ", "ᐛ", "ᐝ",
            "ᖴ", "ᖵ", "ᖶ", "ᖷ", "ᗐ", "ᗑ", "ᗒ", "ᗓ", "ᗔ", "ᗕ", "ᙿ",
        ],
    ),
    (MappingKey::X, &["᙭"]),
    (
        MappingKey::Y,
        &[
            "ᔦ", "ᔧ", "ᔨ", "ᔩ", "ᔪ", "ᔫ", "ᔬ", "ᔭ", "ᔮ", "ᔯ", "ᔰ", "ᔱ", "ᔲ", "ᔳ", "ᔴ", "ᔵ", "ᔶ",
            "ᔷ", "ᔸ", "ᔹ", "ᔺ", "ᔻ", "ᔼ", "ᔽ", "ᔾ", "ᔿ", "ᕀ", "ᕁ", "ᘎ", "ᘏ", "ᘐ", "ᘑ", "ᘒ", "ᘓ",
        ],
    ),
    (MappingKey::Z, &["ᙀ", "ᙁ", "ᙂ", "ᙃ", "ᙄ", "ᙅ", "ᙆ", "ᙇ"]),
];

pub(super) const CANADIAN_EXTENDED: LangData = &[
    (MappingKey::A, &["ᢱ", "ᢲ"]),
    (MappingKey::C, &["ᣗ"]),
    (MappingKey::G, &["ᣭ", "ᣮ", "ᣯ", "ᣰ"]),
    (MappingKey::H, &["ᣬ"]),
    (MappingKey::J, &["ᣱ", "ᣲ"]),
    (MappingKey::K, &["ᢸ", "ᢹ", "ᣖ"]),
    (MappingKey::L, &["ᢽ", "ᣡ", "ᣢ", "ᣳ"]),
    (MappingKey::M, &["ᢺ", "ᣘ"]),
    (
        MappingKey::N,
        &["ᢻ", "ᢼ", "ᣆ", "ᣇ", "ᣈ", "ᣉ", "ᣊ", "ᣋ", "ᣌ", "ᣍ", "ᣙ"],
    ),
    (MappingKey::O, &["ᢰ"]),
    (MappingKey::P, &["ᢴ", "ᢵ", "ᢶ", "ᣔ"]),
    (MappingKey::Period, &["ᣞ", "ᣟ"]),
    (
        MappingKey::R,
        &["ᣅ", "ᣎ", "ᣏ", "ᣐ", "ᣑ", "ᣒ", "ᣓ", "ᣠ", "ᣴ"],
    ),
    (
        MappingKey::S,
        &["ᢾ", "ᢿ", "ᣀ", "ᣁ", "ᣂ", "ᣚ", "ᣛ", "ᣪ", "ᣫ", "ᣵ"],
    ),
    (
        MappingKey::T,
        &["ᢷ", "ᣕ", "ᣣ", "ᣤ", "ᣥ", "ᣦ", "ᣧ", "ᣨ", "ᣩ"],
    ),
    (MappingKey::W, &["ᢳ", "ᣜ", "ᣝ"]),
    (MappingKey::Y, &["ᣃ", "ᣄ"]),
];

pub(super) const CANADIAN_EXTENDED_A: LangData = &[
    (MappingKey::H, &["𑪰", "𑪱", "𑪲", "𑪳", "𑪴", "𑪵"]),
    (
        MappingKey::S,
        &["𑪶", "𑪷", "𑪸", "𑪹", "𑪺", "𑪻", "𑪼", "𑪽", "𑪾", "𑪿"],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mappings::{get_language_data, get_variants, init, test_guard};
    use std::collections::HashSet;

    #[test]
    fn requested_unicode_repertoires_are_complete_and_case_correct() {
        let _guard = test_guard();
        // Independent assigned ranges from Unicode 17.0.0; reserved slots are excluded.
        for (name, lower, upper) in [
            (
                "Cherokee",
                vec![0xAB70..=0xABBF, 0x13F8..=0x13FD],
                vec![0x13A0..=0x13F5],
            ),
            ("Osage", vec![0x104D8..=0x104FB], vec![0x104B0..=0x104D3]),
            (
                "CanadianAboriginalSyllabics",
                vec![0x1400..=0x167F],
                vec![0x1400..=0x167F],
            ),
            (
                "CanadianAboriginalSyllabicsExtended",
                vec![0x18B0..=0x18F5],
                vec![0x18B0..=0x18F5],
            ),
            (
                "CanadianAboriginalSyllabicsExtendedA",
                vec![0x11AB0..=0x11ABF],
                vec![0x11AB0..=0x11ABF],
            ),
        ] {
            assert!(crate::mappings::LANGUAGES.contains(&name), "{name} missing from Settings");
            init(&[name.into()]);
            let data = get_language_data(name).unwrap();
            for (uppercase, ranges) in [(false, lower), (true, upper)] {
                let expected: HashSet<char> = ranges
                    .into_iter()
                    .flatten()
                    .map(|cp| char::from_u32(cp).unwrap())
                    .collect();
                let mut actual = HashSet::new();
                for &(key, choices) in data {
                    let variants = get_variants(key, uppercase);
                    assert_eq!(variants.len(), choices.len());
                    for variant in variants {
                        assert_eq!(variant.chars().count(), 1, "{name}: {variant:?}");
                        assert!(
                            actual.insert(variant.chars().next().unwrap()),
                            "duplicate in {name}"
                        );
                    }
                    let lower_choices = get_variants(key, false);
                    let upper_choices = get_variants(key, true);
                    assert_eq!(
                        upper_choices,
                        lower_choices
                            .iter()
                            .map(|v| v.to_uppercase())
                            .collect::<Vec<_>>()
                    );
                }
                assert_eq!(actual, expected, "{name}, uppercase={uppercase}");
            }
        }
    }

    #[test]
    fn script_sets_are_opt_in_and_deduplicate() {
        let _guard = test_guard();
        init(&["French".into()]);
        assert!(!get_variants(MappingKey::A, false).contains(&"ꭰ".into()));
        for canonical in [
            "CanadianAboriginalSyllabics",
            "CanadianAboriginalSyllabicsExtended",
            "CanadianAboriginalSyllabicsExtendedA",
        ] {
            init(&[canonical.into(), canonical.into()]);
            for &(key, choices) in get_language_data(canonical).unwrap() {
                assert_eq!(get_variants(key, false), choices);
            }
        }
        init(&["Osage".into(), "Cherokee".into(), "Osage".into()]);
        assert_eq!(
            get_variants(MappingKey::A, false),
            ["𐓘", "𐓙", "𐓚", "𐓛", "ꭰ"]
        );
    }

    #[test]
    fn name_based_bindings_and_supplementary_plane_commits_work() {
        use crate::config::ActivationKey;
        use crate::state_machine::{GrabEvent, KeyInput, StateMachine};
        let _guard = test_guard();
        for (name, key, expected) in [
            ("Cherokee", MappingKey::Q, "ꮖ"),
            ("Cherokee", MappingKey::M, "ᏽ"),
            ("Osage", MappingKey::K, "𐓥"),
            ("CanadianAboriginalSyllabics", MappingKey::G, "ᗄ"),
            ("CanadianAboriginalSyllabics", MappingKey::T, "ᙾ"),
            ("CanadianAboriginalSyllabics", MappingKey::Period, "᙮"),
            ("CanadianAboriginalSyllabicsExtended", MappingKey::R, "ᣴ"),
            ("CanadianAboriginalSyllabicsExtendedA", MappingKey::H, "𑪰"),
            ("CanadianAboriginalSyllabicsExtendedA", MappingKey::S, "𑪿"),
        ] {
            init(&[name.into()]);
            let variants = get_variants(key, false);
            let target = variants.iter().position(|v| v == expected).unwrap();
            for uppercase in [false, true] {
                let input = KeyInput::Letter(key);
                let mut state = StateMachine::new(0, 0, ActivationKey::Space);
                state.handle_key_press(input, false);
                assert!(matches!(
                    state.handle_key_press(KeyInput::Space, false),
                    (true, Some(GrabEvent::ShowOverlay { index: 0, .. }))
                ));
                for index in 1..=target {
                    assert!(
                        matches!(state.handle_key_press(KeyInput::RightArrow, false),
                        (true, Some(GrabEvent::UpdateSelection(i))) if i == index)
                    );
                }
                if uppercase {
                    state.update_shift(true);
                }
                let expected = if uppercase {
                    expected.to_uppercase()
                } else {
                    expected.to_string()
                };
                assert!(matches!(state.handle_key_release(input),
                    (true, Some(GrabEvent::InjectChar(value))) if value == expected));

                // Also exercise the deferred Linux path, which emits the full string directly.
                let mut state = StateMachine::new(0, 0, ActivationKey::Space);
                state.deferred_press(input, Some(30), uppercase, false);
                state.deferred_press(KeyInput::Space, Some(57), uppercase, false);
                for _ in 0..target {
                    state.deferred_press(KeyInput::RightArrow, Some(106), uppercase, false);
                }
                assert_eq!(
                    state.deferred_release(Some(30), uppercase).inject,
                    Some(expected)
                );
            }
        }
    }
}
