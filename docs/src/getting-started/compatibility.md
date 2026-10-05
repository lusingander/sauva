# Compatibility

## Operating Systems

macOS and Linux are supported. Windows is not currently supported.

## Terminal Graphics

Sauva uses the [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) with [Unicode placeholders](https://sw.kovidgoyal.net/kitty/graphics-protocol/#unicode-placeholders) for glyph images.

| Terminal | Graphics mode | Support |
| --- | --- | --- |
| [kitty](https://sw.kovidgoyal.net/kitty/) | `auto` | Supported |
| [Ghostty](https://ghostty.org) | `auto` | Supported |
| Other terminals supporting Kitty graphics and Unicode placeholders | `force` | Can be tried; compatibility is not guaranteed |
| Terminals supporting the iTerm2 inline image protocol | `iterm2` | Explicit opt-in; compatibility and future support are not guaranteed |
| Other terminals | `off` or `auto` | Text information remains available without glyph images |

Automatic detection recognizes `TERM=xterm-kitty` and `TERM=xterm-ghostty`.

If a compatible terminal is not detected, try:

```sh
sauva --graphics force
```

This skips detection; it does not add graphics support to the terminal. A terminal multiplexer or a changed `TERM` value may affect detection and image display.

The [iTerm2 inline image protocol](https://iterm2.com/documentation-images.html) is available only when selected explicitly:

```sh
sauva --graphics iterm2
```

Sixel is not implemented. To disable images, use:

```sh
sauva --graphics off
```

## Fonts

Sauva does not bundle fonts. The available glyphs and their appearance depend on the fonts installed on the system.

- macOS uses Core Text to find system fonts.
- Linux uses Fontconfig when available. If Fontconfig cannot be loaded, Sauva uses a best-effort font database lookup.

Preferred text and emoji fonts can be configured separately. See [Glyph Preview](../configurations/glyph-preview.md) for the selection order and settings.

The font used for a glyph image can differ from the terminal's own text font. Terminal-rendered characters and the image preview can therefore look different.

## Layout

The TUI adapts to the terminal size. Some preview panels are hidden at smaller widths, while the main text view remains available. See [Requirements](./requirements.md) for the minimum size.
