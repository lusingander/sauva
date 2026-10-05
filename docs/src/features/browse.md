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

Blocks have different sizes and may leave gaps between them. When opening the block list from a code point in a gap, Sauva selects the next block.

![Block browser](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/block.png)

![Block code point browser](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/block-code-points.png)

## Operations

- Use <kbd>j</kbd> / <kbd>k</kbd> or <kbd>Down</kbd> / <kbd>Up</kbd> in lists.
- Use <kbd>h</kbd>, <kbd>j</kbd>, <kbd>k</kbd>, <kbd>l</kbd> or the arrow keys in a code point grid.
- Press <kbd>Enter</kbd> to open the next level or inspect the selected code point.
- Press <kbd>g</kbd> / <kbd>G</kbd> to jump to the first / last item. In a grid, this is the first / last code point in the range or block.
- Use <kbd>Ctrl-U</kbd> / <kbd>Ctrl-D</kbd> for larger steps in ranges, blocks, and code point grids.

<kbd>Backspace</kbd> returns to the previous browse level. At the level where browsing was opened, it returns to the Inspector. <kbd>Esc</kbd> cancels browsing immediately and returns to the Inspector without changing its selected code point. Press <kbd>q</kbd> to quit.

Code point grids include unassigned and special code points, so an entry may have no visible glyph. The Inspector provides the available property details for each entry.
