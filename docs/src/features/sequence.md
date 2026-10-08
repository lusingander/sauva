# Sequence

Sequence mode analyzes a string as an ordered list of code points, grouped into grapheme clusters. It is useful for examining combining marks, emoji sequences, and invisible characters.

## Start in Sequence Mode

Pass text containing multiple code points:

```sh
sauva 'Á👩‍💻'
```

Use `--text` to prevent code point notation parsing:

```sh
sauva --text 41
sauva --text U+2192
```

Read text from a pipe or file:

```sh
printf 'Á👩‍💻' | sauva --text -
sauva --text - < input.txt
```

A single code point opens the Inspector. Sequence opens for multiple code points, even when they form a single visible character. Empty input is rejected.

See [Command Line Options](../getting-started/command-line-options.md) for the complete input rules.

![Sequence and normalization demo](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/demo-sequence.gif)

## Code Points and Grapheme Clusters

A code point is one Unicode value, such as `U+0041`. A grapheme cluster groups code points into a unit that often corresponds to a user-perceived character. Sauva uses extended grapheme cluster boundaries as described in [Unicode Text Segmentation](https://www.unicode.org/reports/tr29/).

For example, the string `Á👩‍💻` contains five code points and two grapheme clusters:

| Cluster | Code points |
| --- | --- |
| `Á` | `U+0041` LATIN CAPITAL LETTER A + `U+0301` COMBINING ACUTE ACCENT |
| `👩‍💻` | `U+1F469` WOMAN + `U+200D` ZERO WIDTH JOINER + `U+1F4BB` PERSONAL COMPUTER |

The accented A in the command above is `U+0041` followed by `U+0301`.

## Reading the Screen

![Sequence](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/sequence.png)

- Each row shows the input position, code point, display representation, and Unicode name.
- The left gutter numbers grapheme clusters and connects their member code points. A dot marks a cluster containing one code point.
- The selected cluster is emphasized in the gutter.
- The heading shows code point and grapheme counts, abbreviated as `CP` and `GC` in compact layouts.
- When space is available, the selection panel shows the selected code point's properties and position within its cluster. The glyph image renders that individual code point.

Input order, repeated characters, spaces, and newlines are preserved. Labels and dotted circles make otherwise hard-to-see characters visible in the list.

## Operations

| Keys | Action |
| --- | --- |
| <kbd>j</kbd> / <kbd>k</kbd>, <kbd>Down</kbd> / <kbd>Up</kbd> | Select a code point |
| <kbd>[</kbd> / <kbd>]</kbd> | Select the first code point of the previous / next grapheme cluster |
| <kbd>g</kbd> / <kbd>G</kbd> | Select the first / last code point |
| <kbd>Enter</kbd> | Open the selected code point in the Inspector |
| <kbd>n</kbd> | Compare normalization forms |
| <kbd>Y</kbd> | Open copy candidates |
| <kbd>q</kbd>, <kbd>Esc</kbd> | Quit |

![Sequence Inspector](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/sequence-inspector.png)

<kbd>Backspace</kbd> in the Inspector returns to the sequence. From Sequence, press <kbd>n</kbd> to open [Normalization](./normalization.md).

## Copying

Press <kbd>Y</kbd> to choose what to copy:

- **Code Point**: the selected code point as text.
- **Grapheme Cluster**: the entire cluster containing the selected code point, including members outside the visible list.
- **Whole Input**: the original input in its original order.

Select a candidate with <kbd>j</kbd> / <kbd>k</kbd> or the arrow keys, then press <kbd>Enter</kbd> to copy. <kbd>Esc</kbd> cancels. The preview shows the candidate's text and code point notation; labels, dotted circles, and ellipses are display aids and are not copied. Whitespace, newlines, and invisible characters are preserved exactly.

The glyph image is hidden while the dialog or its help is open and restored when you return to Sequence. <kbd>F1</kbd> opens the shared Copy Dialog help; closing help returns to the same candidate. A successful copy closes the dialog and shows confirmation in the footer. If copying fails, the dialog stays open with an error so you can retry.
