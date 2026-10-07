# Inspector

The Inspector shows the properties of one Unicode code point. Open it at startup with a character or code point argument, or select a code point from Search, Browse, or Sequence.

```sh
sauva あ
sauva U+2192
```

![Inspector](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/inspector.png)

## Properties

| Section | Fields |
| --- | --- |
| Identity | Character, Code Point, Primary Name, Aliases, Plane, Block |
| Classification | General Category, Script, Age, East Asian Width, Canonical Combining Class, Bidi Class, Default Ignorable |
| Encoding | UTF-8, UTF-16, HTML Decimal, HTML Hex, Rust char, Unicode Escape |
| Normalization | Decomposition Type, Decomposition |
| Data | Unicode Version |

Some useful distinctions:

- **Age** is the Unicode version in which the character was introduced. **Unicode Version** is the version of Sauva's bundled data.
- **Aliases** lists formal Unicode name aliases and their types. These aliases can also be searched.
- **East Asian Width** is a Unicode property. Actual terminal cell width also depends on the terminal and its rendering rules.
- **Decomposition** shows the character's Unicode decomposition mapping. To compare the normalized forms of a string, use [Normalization](./normalization.md) from Sequence.

The glyph panel renders the selected code point and shows the font used, when space is available. See [Glyph Preview](../configurations/glyph-preview.md) for font settings and special cases.

## Operations

| Keys | Action |
| --- | --- |
| <kbd>h</kbd> / <kbd>l</kbd>, <kbd>Left</kbd> / <kbd>Right</kbd> | Inspect the previous / next code point |
| <kbd>j</kbd> / <kbd>k</kbd>, <kbd>Down</kbd> / <kbd>Up</kbd> | Select a property |
| <kbd>[</kbd> / <kbd>]</kbd> | Select the first property of the previous / next section |
| <kbd>Ctrl-U</kbd> / <kbd>Ctrl-D</kbd> | Move through properties by page |
| <kbd>g</kbd> / <kbd>G</kbd> | Select the first / last property |
| <kbd>y</kbd> | Copy the selected value |
| <kbd>/</kbd> | Open Search |
| <kbd>p</kbd>, <kbd>r</kbd>, <kbd>b</kbd>, <kbd>c</kbd> | Open Browse at the corresponding level |
| <kbd>Backspace</kbd> | Return to the original or normalized sequence, when one is open |
| <kbd>q</kbd>, <kbd>Esc</kbd> | Quit |

Adjacent navigation follows code point values, even when the Inspector was opened from a sequence. <kbd>Backspace</kbd> returns to the sequence's existing selection.

Section navigation selects a property and brings its section heading into view. <kbd>[</kbd> always moves to the previous section, even from the middle of the current one. Both keys stop when there is no adjacent section; they do not wrap. Sections containing unavailable values are included.

## Copying Values

Select a property and press <kbd>y</kbd> to copy it to the system clipboard. Selecting **Character** copies the actual character, including invisible characters and combining marks. Display aids such as a dotted circle or a `<SPACE>` label are not included in that value.

Unavailable values, such as an encoding for a surrogate, cannot be copied. A status message reports whether copying succeeded or why it failed.

## Special Code Points

Sauva can inspect the full code point range, including unassigned code points, private-use characters, and surrogates.

Surrogates are not Unicode scalar values, so UTF-8 / UTF-16 encodings and character images are unavailable for them. Empty or invisible characters may have property data without visible pixels in the preview.
