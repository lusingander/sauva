# Normalization

From [Sequence](./sequence.md), press <kbd>n</kbd> to compare the input with its NFC, NFD, NFKC, and NFKD normalized forms.

```sh
sauva 'Á①ﬃ'
```

![Normalization comparison](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/sequence-normalization.png)

## Forms

| Form | Transformation |
| --- | --- |
| NFC | Canonical decomposition followed by canonical composition |
| NFD | Canonical decomposition |
| NFKC | Compatibility decomposition followed by canonical composition |
| NFKD | Compatibility decomposition |

Canonical normalization handles equivalent representations, such as a precomposed accented letter and a letter followed by a combining mark. Compatibility normalization also converts characters such as circled digits and ligatures, and can remove distinctions in their original presentation. See [Unicode Normalization Forms](https://www.unicode.org/reports/tr15/) for the specification.

Examples of individual transformations:

| Input | Form | Result |
| --- | --- | --- |
| `A` + `U+0301` | NFC | `Á` (`U+00C1`) |
| `Á` (`U+00C1`) | NFD | `A` + `U+0301` |
| `①` (`U+2460`) | NFKC | `1` (`U+0031`) |
| `ﬃ` (`U+FB03`) | NFKD | `ffi` |

## Comparison

The comparison screen shows:

- The original text and its code point sequence.
- Whether each form changes the input.
- Code point (`CP`), UTF-8 byte (`Bytes`), and grapheme cluster (`GC`) counts for each form.
- The selected form's result, code point sequence, and number of changed regions.

Changed regions use the configured difference colors and underlining. Spaces and invisible characters are represented with display aids; the copied result contains the actual normalized text.

## Inspect a Result

Select a form and press <kbd>Enter</kbd> to open its code point list.

![Normalization result](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/sequence-normalization-result.png)

The result is shown alongside the original code point list. As you select a result code point, the corresponding region in the original is highlighted and brought into view. This makes compositions and expansions easier to follow.

Press <kbd>Enter</kbd> again to inspect a result code point. <kbd>Backspace</kbd> in the Inspector returns to the result list; <kbd>Backspace</kbd> or <kbd>Esc</kbd> in the result list returns to the form comparison.

The original input is preserved throughout these operations.

Use <kbd>[</kbd> / <kbd>]</kbd> to select the first code point of the previous / next grapheme cluster in the normalized result. Boundaries follow the result text: for example, normalizing `ﬃ` to `ffi` creates three clusters that can be visited separately. The corresponding original region follows the selection. Navigation stops at the first and last clusters, and <kbd>[</kbd> always targets the previous cluster even from the middle of the current one.

## Operations

| Keys | In form comparison | In a result list |
| --- | --- | --- |
| <kbd>j</kbd> / <kbd>k</kbd>, <kbd>Down</kbd> / <kbd>Up</kbd> | Select a form | Select a code point |
| <kbd>[</kbd> / <kbd>]</kbd> | — | Select the first code point of the previous / next grapheme cluster |
| <kbd>g</kbd> / <kbd>G</kbd> | Select NFC / NFKD | Select the first / last code point |
| <kbd>Enter</kbd> | Open the selected result | Inspect the selected code point |
| <kbd>y</kbd> | Copy the exact normalized text | — |
| <kbd>Backspace</kbd>, <kbd>Esc</kbd> | Return to the original sequence | Return to form comparison |
| <kbd>q</kbd> | Quit | Quit |

Copying includes whitespace and newlines in the normalized result. Select a form and press <kbd>y</kbd> in the comparison screen to copy the whole result.
