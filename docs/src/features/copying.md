# Copying

The copy dialog copies a character or string as text, a quoted string literal, or a numeric sequence. It is available from the Inspector, Sequence, and both Normalization screens.

## Scope

| Screen | Available scopes |
| --- | --- |
| Inspector | The current Code Point, regardless of the selected property |
| Sequence | The selected Code Point, its Grapheme Cluster, or the Whole Input |
| Normalization comparison | The selected form's Whole Result |
| Normalization Result | The selected Code Point, its Grapheme Cluster, or the Whole Result |

A grapheme cluster includes all its code points, including members outside the visible list. Copies from Normalization use the selected result; they do not revert to the original input or apply another normalization.

In the Inspector, property copying remains separate. For example, copying a case mapping takes its displayed code points and names, while the dialog represents the current character itself. The dialog's Rust String is a string literal even for one character; the Rust char property remains a character literal.

## Formats

The following examples use `A` + U+0301 + `😀`:

| Format | Clipboard content |
| --- | --- |
| Text | The actual text `Á😀` |
| JSON String | `"A\u0301\uD83D\uDE00"` |
| Rust String | `"A\u{301}\u{1F600}"` |
| Code Point List | `U+0041 U+0301 U+1F600` |
| UTF-8 Hex | `41 CC 81 F0 9F 98 80` |
| UTF-16 Units | `0041 0301 D83D DE00` |

JSON String and Rust String include quotes, preserve printable ASCII, and escape non-ASCII characters and controls. Literal backslashes are escaped rather than interpreted. These formats preserve whitespace and line endings without requiring invisible characters to survive a paste into another application.

Numeric sequences use uppercase hexadecimal values separated by single spaces. UTF-8 Hex copies a textual representation of bytes. UTF-16 Units copies 16-bit code units, without choosing a byte order or producing UTF-16LE / BE bytes.

Copies preserve the selected text without trimming, decoding escapes, or normalizing it. No separator newline or BOM is added. Existing U+FEFF characters are preserved. Preview labels, dotted circles, and ellipses are not copied; a shortened preview still copies the complete value.

Unassigned code points, private-use characters, and noncharacters support all six formats when they are Unicode scalar values. Surrogates support Code Point List only, such as `U+D800`.

## Operations

| Keys | Action |
| --- | --- |
| <kbd>Y</kbd> | Inspector, Sequence, or Normalization: open the copy dialog |
| <kbd>Up</kbd> / <kbd>Down</kbd> | Copy dialog: select a scope when multiple scopes are available |
| <kbd>Left</kbd> / <kbd>Right</kbd> | Copy dialog: cycle through the available formats |
| <kbd>Enter</kbd> | Copy dialog: copy the selected scope and format |

See [Keybindings](../keybindings/index.md#copy-dialog) for all copy dialog controls and [Custom Keybindings](../keybindings/custom-keybindings.md) to change them.
