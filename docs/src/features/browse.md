# Browse

Browse Unicode by numeric location or by a named block. From the Inspector, use these shortcuts:

| Key | Starting view |
| --- | --- |
| <kbd>p</kbd> | Planes |
| <kbd>r</kbd> | Ranges in the current plane |
| <kbd>b</kbd> | Blocks |
| <kbd>c</kbd> | Code points in the current range |

## Planes and Ranges

This path explores Unicode by code point value:

```text
Plane → Range → Code Point → Inspector
```

A plane contains 65,536 code points. Sauva divides each plane into ranges of 256 code points for browsing. These ranges are numeric subdivisions, independent of named Unicode blocks.

![Plane browser](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/plane.png)

![Range browser](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/range.png)

![Code point browser](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/code-points.png)

## Blocks

This path explores named blocks from the Unicode Character Database:

```text
Block → Code Point → Inspector
```

Blocks have different sizes and may leave gaps between them.

![Block browser](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/block.png)

![Block code point browser](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/block-code-points.png)

Code point grids include unassigned and special code points, so an entry may have no visible glyph. The Inspector provides the available property details for each entry.

## Operations

| Keys | Action |
| --- | --- |
| Arrow keys | Select an item or code point |
| <kbd>Enter</kbd> | Open the next level or inspect the selected code point |
| <kbd>Backspace</kbd> | Return to the previous browse level, or to the Inspector at the starting level |
| <kbd>Esc</kbd> | Close Browse and return to the Inspector |

See [Keybindings](../keybindings/index.md#browse-lists) for all Browse controls.
