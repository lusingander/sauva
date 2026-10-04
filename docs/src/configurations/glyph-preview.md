# Glyph Preview

The glyph preview renders the selected code point as an image using a font installed on your system. When space is available, the preview panel also shows the font family, style, and version.

The image font is selected independently of the terminal's text font. Configure it in the `[glyph_preview]` section:

```toml
[glyph_preview]
font_families = ["Iosevka", "Noto Sans"]
emoji_font_families = ["Noto Color Emoji"]
fg = "#f5f7fa"
bg = "#00000000"
```

## Font Selection

For each code point, Sauva tries:

1. Configured normal fonts in `font_families`, in order.
2. The system default text font.
3. Configured emoji fonts in `emoji_font_families`, in order.
4. System fallback fonts.

Missing families and fonts without a usable glyph are skipped. Both lists default to `[]`, so system fonts are used without any configuration.

Sauva does not install fonts. Specify family names for fonts already installed on the machine. The chosen font can vary between code points and between operating systems.

For OS-specific font lookup and terminal support, see [Compatibility](../getting-started/compatibility.md).

## Image Colors

- `fg` is the glyph foreground color; the default is `"#f5f7fa"`.
- `bg` is the image background color; the default is `"#00000000"`, fully transparent.

Values accept `#RRGGBB` or `#RRGGBBAA`. For example, use an opaque background on a light terminal:

```toml
[glyph_preview]
fg = "#202020"
bg = "#ffffff"
```

Color font glyphs, such as emoji, can use their own palette.

## Special Characters

Combining marks can be shown with a dotted circle to make their position visible. The dotted circle is a preview aid; it is not part of the character's value or the text copied from the Inspector.

Some code points cannot produce a visible image:

| Message | Meaning |
| --- | --- |
| `No visible pixels` | The resolved glyph is blank, as can happen with a space or invisible character |
| `Glyph unavailable` | No usable glyph was found in the available fonts |
| `Not a Unicode scalar value` | The selected code point is a surrogate and cannot be rendered |
| `Graphics disabled` | Images were disabled with `--graphics off` |
| `Graphics unavailable` | Automatic detection did not select a graphics protocol |

Unicode details remain available in these cases. See [FAQ](../faq/index.md) for troubleshooting.
