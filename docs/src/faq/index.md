# FAQ

## Why is the glyph preview missing?

First, check [Compatibility](../getting-started/compatibility.md). Automatic graphics detection enables images for kitty and Ghostty. If a compatible terminal is not detected, try `sauva --graphics force`. For the iTerm2 inline image protocol, explicitly use `sauva --graphics iterm2`.

Also check the preview's message:

- **Graphics disabled**: start without `--graphics off`.
- **Graphics unavailable**: automatic detection did not select a protocol.
- **Glyph unavailable**: install a font with a usable glyph, or adjust the [preferred fonts](../configurations/glyph-preview.md).
- **No visible pixels**: the resolved glyph is blank. Spaces and invisible characters can have this result.
- **Not a Unicode scalar value**: surrogates cannot be rendered.
- **Preview area unavailable**: resize the terminal and try again.

Some preview panels are hidden at smaller terminal widths. Enlarge the terminal to make room for them. Text information remains available without images.

## Why does the glyph use a different font from my terminal?

Sauva selects fonts independently for glyph images. It tries configured normal fonts, the system default text font, configured emoji fonts, and system fallback fonts, in that order.

Set `glyph_preview.font_families` and `glyph_preview.emoji_font_families` to installed family names. The preview's Font and Version fields identify the font used. See [Glyph Preview](../configurations/glyph-preview.md).

## Why does `sauva 41` open one character?

An input containing only hexadecimal digits is interpreted as code point notation. `41` selects `U+0041`, the letter A.

Use `sauva --text 41` to open the two literal characters in Sequence mode. A single-character argument is always treated as that character; use `sauva U+A` to inspect code point `U+000A`.

The search field has its own input rules. See [Command Line Options](../getting-started/command-line-options.md) and [Search](../features/search.md).

## Why does `echo U+2192 | sauva -` fail?

`echo` adds a newline, and Sauva preserves that newline as part of stdin. It therefore cannot parse the input as code point notation.

Use:

```sh
printf 'U+2192' | sauva -
```

To examine the literal text and its newline, use:

```sh
echo U+2192 | sauva --text -
```

Stdin must contain nonempty, valid UTF-8. It is read only when `-` is explicitly supplied, and Sauva waits for EOF before starting. Keyboard input still requires an interactive terminal.

## Why does an emoji appear as several rows?

Sequence has one row per code point. An emoji such as `👩‍💻` contains multiple code points grouped into one grapheme cluster. The gutter connects those rows and shows their shared cluster number.

The glyph image previews the selected code point. See [Sequence](../features/sequence.md) for an example.

## Why does copying fail?

Sauva uses the system clipboard. The footer reports when it is unavailable, busy, or unable to accept the text. Check that a system clipboard is available in the current desktop session; an SSH or headless session may not provide one.

A field marked unavailable, such as the encoding of a surrogate, has no copyable value. In Normalization, press `y` in the form comparison screen to copy the whole normalized result.

## Why does my configuration prevent startup?

The startup error identifies the configuration file and the problem. Check for:

- An empty `SAUVA_CONFIG_FILE` or a nonexistent file selected by it.
- Invalid TOML or unknown field names.
- Unsupported color formats or cursor text wider than one terminal cell.
- Invalid, duplicate, conflicting, or input-reserved keybindings.

`sauva --print-default-config` works without loading the local file and provides a valid starting point. See [Configurations](../configurations/index.md) and [Custom Keybindings](../keybindings/custom-keybindings.md).

## Can I use Sauva without glyph images?

Yes. Run `sauva --graphics off` to use properties, Search, Browse, Sequence, and Normalization without image previews.

## Which Unicode version does Sauva use?

The bundled data is based on Unicode 17.0.0. The Inspector's Unicode Version field shows this data version, while Age shows the version in which the selected character was introduced. An installed font may cover a different set of characters.
