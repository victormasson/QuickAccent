# Character sets

Enable the sets you want in `~/.config/quickaccent/config.toml`:

```toml
languages = ["French", "Special", "Currency", "Typography", "Arrows", "Math", "CurrencyExtended"]
```

The language list reloads automatically. Sets merge in configuration order,
with duplicate choices removed. Put your most-used set first. Existing configs
keep their current sets.

Hold a base key for your configured hold delay, press your activation key
(Space by default), cycle with Space or arrows, then release the base key.
Punctuation and number bindings use physical US key positions. On macOS, letters
also use physical US positions. Non-US layouts may have different legends.
Shift changes case, not the punctuation binding; use `=` without Shift for Plus.

## PowerToys sets

`Special` and `Currency` use the corresponding
[PowerToys tables](https://github.com/microsoft/PowerToys/blob/main/src/modules/poweraccent/PowerAccent.Common/CharacterMappings.cs)
(see [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md)).
Other PowerToys language tables are not imported by this change.

- `.`: ellipses and combining accents.
- `-`: en/em/two-em/three-em dashes, nonbreaking hyphen, mathematical minus.
- `=`: comparisons and operators, including ≤ ≥ ≠ ≈ ± ≡.
- `,`: primes, angle quotation marks and mathematical symbols.
- `/`: ÷ √ ‽ ⸘; keypad divide: ÷ √; keypad multiply: × ⋅ ˣ ₓ.
- Backslash: grave accent and tilde.
- Number row: subscripts, superscripts and fractions; `0` includes °, `8` includes ∞.
- Letters: examples include `c` → © °C, `f` → °F, `r` → ®, `t` → ™, `s` → § ∑ ∫.
- Currency examples: `e` → €, `p` → £ ₽ ₱, `r` → ₹ ៛ ﷼, `s` → $ ₪, `y` → ¥.

Some currency choices are abbreviation letters, matching PowerToys. Standalone
combining accents appear on a dotted circle in the picker; only the accent is
inserted, attaching to the preceding character. Display direction marks are
never inserted. Shift preserves complete multi-character choices such as °C.

## Optional extensions

Each extension can be enabled independently. These are all its bindings, in
picker order. The quote key is the US apostrophe/double-quote key, without Shift.

| Set | Base key | Choices |
| --- | --- | --- |
| Typography | `'` | ‘ ’ “ ” „ ‚ « » ‹ › |
| Typography | `.` | • ◦ ▪ |
| Typography | `t` | † ‡ ※ |
| Typography | `v` | ✓ ✔ |
| Typography | `x` | ✗ ✘ |
| Arrows | `-` | ← → ↑ ↓ ↔ ↕ ↗ ↘ ↙ ↖ |
| Arrows | `=` | ⇒ ⇐ ⇔ |
| Math | `=` | ≔ ≝ ≟ ≢ ∝ |
| Math | `,` (the `<` key) | ≪ ≲ ⊂ ⊆ |
| Math | `.` (the `>` key) | ≫ ≳ ⊃ ⊇ |
| Math | `i`, `u` | ∩ / ∪ |
| Math | `a`, `o`, `n` | ∧ / ∨ / ¬ |
| Math | `t` | ∴ ∵ |
| CurrencyExtended | `a` | ؋ (afghani) |
| CurrencyExtended | `b` | ₿ (bitcoin) |
| CurrencyExtended | `c` | ₵ (cedi), ¤ (generic currency) |
| CurrencyExtended | `d` | ֏ (dram) |
| CurrencyExtended | `g` | ₲ (guaraní) |
| CurrencyExtended | `l` | ₾ (lari) |
| CurrencyExtended | `n` | ₦ (naira) |
| CurrencyExtended | `r` | ₨ (rupee) |

## Phonetic Hebrew and Yiddish

Add `Hebrew` and/or `Yiddish` to `languages`, for example:

```toml
languages = ["French", "Hebrew", "Yiddish"]
```

Keep your Latin input source selected. These are individual-character pickers,
not automatic transliterators: each held Latin key offers the choices below.
Final forms (`ך ם ן ף ץ`) must be selected explicitly. Hebrew and Yiddish have no
uppercase; Shift preserves their letters, vowel marks, and multi-letter choices.

| Latin key | Hebrew | Yiddish |
| --- | --- | --- |
| a | א ע אַ אָ | אַ אָ א ײַ |
| b | ב בּ | ב בּ בֿ |
| c | צ ץ ח צ׳ | צ ץ טש |
| d | ד דּ | ד דזש |
| e | אֶ אֵ ע | ע ײ |
| f | פ ף פֿ | פֿ ף |
| g | ג גּ ג׳ | ג |
| h | ה ח הּ | ה ח |
| i | י אִ | י יִ |
| j | ג׳ י | דזש י |
| k | כ ך כּ ךּ ק | ק כּ כ ך |
| l | ל | ל |
| m | מ ם | מ ם |
| n | נ ן | נ ן |
| o | וֹ אֹ | אָ ױ |
| p | פּ פ ף | פּ פ ף |
| q | ק | ק |
| r | ר | ר |
| s | ס ש שׁ שׂ | ס ש שׂ ת |
| t | ת ט תּ צ ץ | ט תּ ת צ ץ טש |
| u | וּ אֻ | ו וּ |
| v | ו ב בֿ | װ בֿ |
| w | ו וו | װ |
| x | ח כ ך | כ ך ח |
| y | י | י ײ ײַ ױ |
| z | ז ז׳ | ז זש |

- **Comma:** geresh `׳`, gershayim `״`, and quotation marks.
- **Hyphen:** maqaf `־`.
- **Period:** combining vowel/letter marks. Hebrew includes sheva, hataf vowels,
  hiriq, tsere, segol, patah, qamats, holam, holam haser, qubuts, dagesh, meteg,
  rafe, shin/sin dots, and qamats qatan. Yiddish offers patah, qamats, hiriq,
  dagesh, rafe, and sin dot.

To add a standalone mark, first insert the Hebrew letter, then use the period
picker. The mark combines with the preceding character. Pointed-letter choices
already include their marks. Yiddish uses Unicode double-vav, vav-yod, and
double-yod characters; `דזש`, `טש`, and `זש` are multi-letter sequences.
Your editor controls right-to-left paragraph direction and font support.

## Cherokee, Osage, and Canadian syllabics

Enable any of these five independent sets in `~/.config/quickaccent/config.toml`:

```toml
languages = [
    "Cherokee",
    "Osage",
    "CanadianAboriginalSyllabics",
    "CanadianAboriginalSyllabicsExtended",
    "CanadianAboriginalSyllabicsExtendedA",
]
```

You can combine them with existing sets. Nothing is enabled by default. As with
other sets, configuration order determines which choices come first and repeated
characters are removed. Keep a Latin input source selected: these are lookup
bindings on the existing letter keys, not native keyboard layouts or automatic
transliteration. On macOS, letter bindings use physical US key positions; on
Linux they follow the existing logical-letter mapping.

Coverage is pinned to [Unicode 17.0.0](https://www.unicode.org/Public/17.0.0/ucd/UnicodeData.txt):

| Set | Base choices | Coverage |
| --- | ---: | --- |
| Cherokee | 86 | All 86 lowercase letters (U+AB70–ABBF and U+13F8–13FD), with all 86 uppercase partners at U+13A0–13F5 |
| Osage | 36 | All 36 lowercase letters at U+104D8–104FB and uppercase partners at U+104B0–104D3 |
| CanadianAboriginalSyllabics | 640 | Every assigned character at U+1400–167F |
| CanadianAboriginalSyllabicsExtended | 70 | Every assigned character at U+18B0–18F5; U+18F6–18FF are unassigned |
| CanadianAboriginalSyllabicsExtendedA | 16 | Every assigned character at U+11AB0–11ABF |

### Finding a character

Use the **first letter of the syllable or letter name in Unicode**, ignoring
language/region qualifiers. Within a key, choices follow Unicode code-point order
(uppercase code-point order for Cherokee and Osage). The links below provide the
complete official character names.

- **[Cherokee](https://www.unicode.org/charts/nameslist/n_13A0.html):** vowel names
  A, E, I, O, U and V use those keys. GA/GE/GI/GO/GU/GV use G; KA uses K;
  QUA–QUV use Q; HNA uses H; NAH uses N; DLA uses D; TLA and TSA families
  both use T. The archaic MV syllable is included under M. For example, A offers
  `ꭰ`, G starts with `ꭶ`, and Q starts with `ꮖ`.
- **[Osage](https://www.unicode.org/charts/nameslist/n_104B0.html):** A/AI/AIN/AH
  use A; BRA uses B; CHA and EHCHA use C; HA/HYA use H; KA/EHKA/KYA/KHA use K;
  PA/EHPA use P; TA/EHTA/TSA/EHTSA/TSHA use T; DHA uses D; GHA uses G;
  SA/SHA use S; ZA/ZHA use Z. E/EIN, I, LA, MA, NA, O/OIN, U and WA use
  their respective initial letter. The EH prefix is ignored for consonants.
- **Canadian syllabics ([main](https://www.unicode.org/charts/nameslist/n_1400.html),
  [Extended](https://www.unicode.org/charts/nameslist/n_18B0.html),
  [Extended-A](https://www.unicode.org/charts/nameslist/n_11AB0.html)):** use the
  final syllable-name token's first letter. PE/PI/PO/PA use P, NWI uses N,
  SHRI and SPE use S, and CARRIER GHU uses G. For Extended-A, Nattilik H-series
  choices are under H; Nattilik SHR-series and historic SP-series choices are
  under S. Language qualifiers such as CARRIER, NATTILIK and WOODS-CREE do not
  determine the key. Named finals use their letter (FINAL TH uses T).

Canadian punctuation and shape-named finals use these exceptions:

| Key | Choices |
| --- | --- |
| `-` | U+1400 Canadian syllabics hyphen `᐀` |
| `'` | U+141E glottal stop `ᐞ` |
| `.` | Shape-named finals U+141F–142A, full stop U+166E `᙮`, and Extended finals U+18DE/18DF `ᣞ`/`ᣟ` |
| X | U+166D Chi sign `᙭` |

These Canadian blocks contain multiple languages, regional alternatives and
historic forms. Unicode names are a consistent lookup aid, **not a universal
pronunciation guide**. For example, Nattilik names containing O/OO represent
U/UU in that orthography. A language-specific keyboard may be preferable for
sustained writing.

### Case, navigation, and fonts

Cherokee and Osage start with lowercase choices. Hold Shift at activation or tap
Shift while choosing to select their actual Unicode uppercase letters; tapping
Shift again toggles back. Canadian syllabics have no uppercase/lowercase
distinction, so Shift leaves those choices unchanged.

The picker uses system font fallback. Install fonts covering the enabled
scripts if characters appear as missing-glyph boxes. Noto Sans Cherokee, Noto
Sans Osage, and a current Noto Sans Canadian Aboriginal are examples; verify
Extended-A support specifically when choosing a Canadian-syllabics font.
Your destination editor also needs suitable font coverage. Supplementary-plane
Osage and Extended-A characters are inserted as complete Unicode characters,
without display-only direction marks or substitutions.
