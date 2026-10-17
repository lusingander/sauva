# Inspector

The Inspector shows the properties of one Unicode code point. Open it at startup with a character or code point argument, or select a code point from Search, Browse, or Sequence.

```sh
sauva あ
sauva U+2192
```

![Inspector](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/inspector.png)

## Properties

The fixed **Properties** heading shows the current display mode. The Inspector starts in **Relevant** mode; press <kbd>a</kbd> to toggle between **Relevant** and **All**. The mode is kept while navigating code points or returning from another screen.

| Section | Fields |
| --- | --- |
| Identity | Character, Code Point, Primary Name, Aliases, Plane, Block |
| Classification | General Category, Script, Age, East Asian Width, Canonical Combining Class, Bidi Class, Default Ignorable |
| Case | Uppercase Mapping, Lowercase Mapping, Titlecase Mapping, Case Folding |
| Numeric | Numeric Type, Numeric Value |
| Encoding | UTF-8, UTF-16, HTML Decimal, HTML Hex, Rust char, Unicode Escape |
| Normalization | Decomposition Type, Decomposition |
| Data | Unicode Version |

In **Relevant** mode:

- **Aliases**, **Decomposition Type**, and **Decomposition** appear only when present. Empty sections are omitted.
- **Canonical Combining Class** appears for combining marks, including those with a value of `0`, and for any code point with a nonzero value.
- **Default Ignorable** appears only when its value is `Yes`.
- **Case** fields appear only when their mapping differs from the selected code point.
- **Numeric Type** and **Numeric Value** appear together when a numeric value exists, including `0`.
- **Encoding** shows the available representations. For surrogates, it shows one explanation instead of unavailable fields.

The remaining fields are always shown, including default values for unassigned code points. **All** mode shows every field, including `None`, `No`, `0`, and unavailable values. Unchanged case mappings show the selected code point itself. Code points without a numeric value show Numeric Type `None` and Numeric Value `NaN`.

Some useful distinctions:

- **Age** is the Unicode version in which the character was introduced. **Unicode Version** is the version of Sauva's bundled data.
- **Aliases** lists formal Unicode name aliases and their types. These aliases can also be searched.
- **East Asian Width** is a Unicode property. Actual terminal cell width also depends on the terminal and its rendering rules.
- **Case** shows full Unicode mappings as code points and names, including mappings to multiple characters such as `ß` → `SS`. These are the default mappings for a single code point; language-specific and context-dependent rules are not applied. **Case Folding** shows the default full mapping used for caseless matching.
- **Numeric Type** distinguishes `Decimal`, `Digit`, and `Numeric`. **Numeric Value** preserves exact integers and fractions, such as `0`, `4`, and `1/2`, including values for numeric ideographs.
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
| <kbd>e</kbd> | Explain the selected property |
| <kbd>y</kbd> | Copy the selected value |
| <kbd>a</kbd> | Toggle Relevant / All properties |
| <kbd>/</kbd> | Open Search |
| <kbd>p</kbd>, <kbd>r</kbd>, <kbd>b</kbd>, <kbd>c</kbd> | Open Browse at the corresponding level |
| <kbd>Backspace</kbd> | Return to the original or normalized sequence, when one is open |
| <kbd>q</kbd>, <kbd>Esc</kbd> | Quit |

Adjacent navigation follows code point values, even when the Inspector was opened from a sequence. <kbd>Backspace</kbd> returns to the sequence's existing selection.

Code point navigation and display mode changes keep the same property selected when possible. If that property is hidden, selection moves to a nearby property, preferring the same section.

## Property Explanations

Select a property and press <kbd>e</kbd> to read its meaning, possible values or format, notes, and a specification reference. Explanations are bundled with Sauva and can be read offline, including for unavailable values shown in **All** mode.

The page header identifies the property. **Current value** shows the selected code point and its value; it scrolls with the explanation. Use the same scrolling keys as Help, and press <kbd>Esc</kbd> to return to the Inspector's existing selection and position. <kbd>F1</kbd> opens operation help and returns to the same reading position when closed.

Press <kbd>o</kbd> to open the specification in your default browser, or <kbd>y</kbd> to copy its URL. Opening the reference uses the external browser; reading the explanation does not access the network.

## Copying Values

Select a property and press <kbd>y</kbd> to copy it to the system clipboard. Selecting **Character** copies the actual character, including invisible characters and combining marks. Display aids such as a dotted circle or a `<SPACE>` label are not included in that value.

Unavailable values, such as an encoding for a surrogate, cannot be copied. A status message reports whether copying succeeded or why it failed.

Case mappings copy the displayed code points and names, preserving their order and repeated characters. Numeric Value copies its exact number or fraction; `NaN` cannot be copied.

## Special Code Points

Sauva can inspect the full code point range, including unassigned code points, private-use characters, and surrogates.

Surrogates are not Unicode scalar values, so UTF-8 / UTF-16 encodings and character images are unavailable for them. Empty or invisible characters may have property data without visible pixels in the preview.
