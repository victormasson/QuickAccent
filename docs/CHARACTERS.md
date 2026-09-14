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

## Picker pagination

For long character sets, see [Picker pages](../README.md#picker-pages) to configure pagination via `items_per_page`.

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
