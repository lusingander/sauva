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

Each normalized result can be opened as a code point list.

![Normalization result](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/sequence-normalization-result.png)

The result is shown alongside the original code point list. As you select a result code point, the corresponding region in the original is highlighted and brought into view. This makes compositions and expansions easier to follow.

Result code points can be opened in the [Inspector](./inspector.md) to read their properties.

## Copying

Quick copying from the form comparison includes the whole normalized result as text. The [copy dialog](./copying.md) also offers string literals, code point lists, and encodings for that result.

The result list offers the selected code point, its entire grapheme cluster, or the whole normalized result in the same dialog. Each scope uses the normalized text; the original input remains unchanged.

## Operations

| Keys | In form comparison | In a result list |
| --- | --- | --- |
| <kbd>Up</kbd> / <kbd>Down</kbd> | Select a form | Select a code point |
| <kbd>Enter</kbd> | Open the selected result | Open the selected code point in the Inspector |
| <kbd>y</kbd> | Copy the whole normalized result | — |
| <kbd>Y</kbd> | Open the copy dialog for the whole result | Open the copy dialog |
| <kbd>Esc</kbd> | Return to the original sequence | Return to form comparison |

<kbd>Backspace</kbd> in the Inspector returns to the result list.

See [Keybindings](../keybindings/index.md#normalization) for all comparison, result list, and copy dialog controls.
