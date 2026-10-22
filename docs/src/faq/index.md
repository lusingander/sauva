# FAQ

## Why is the glyph preview missing?

Check [Compatibility](../getting-started/compatibility.md) and make sure images were not disabled with `--graphics off`. If a compatible terminal is not detected, try `sauva --graphics force`. For the iTerm2 inline image protocol, use `sauva --graphics iterm2`.

For **Glyph unavailable**, install a font with a usable glyph or adjust the [preferred fonts](../configurations/glyph-preview.md#font-selection). See [Special Characters](../configurations/glyph-preview.md#special-characters) for other preview messages.

If the panel is hidden or reports **Preview area unavailable**, enlarge the terminal.

## Why does the glyph use a different font from my terminal?

Glyph images use their own font settings. Set `glyph_preview.font_families` and `glyph_preview.emoji_font_families` to installed family names. See [Glyph Preview](../configurations/glyph-preview.md#font-selection) for the selection order and examples.

## Why does `sauva 41` open one character?

`41` is interpreted as hexadecimal code point notation for `U+0041`, the letter A.

Use `sauva --text 41` to analyze the literal text `41` in Sequence mode. See [Command Line Options](../getting-started/command-line-options.md#input) for notation rules.

## Why does `echo U+2192 | sauva -` fail?

`echo` adds a newline, and Sauva preserves that newline as part of stdin. It therefore cannot parse the input as code point notation.

Use `printf` to omit the newline:

```sh
printf 'U+2192' | sauva -
```

To examine the literal text and its newline, use:

```sh
echo U+2192 | sauva --text -
```

See [Command Line Options](../getting-started/command-line-options.md#reading-from-stdin) for stdin rules.

## Why does an emoji appear as several rows?

Sequence has one row per code point. An emoji such as `👩‍💻` contains multiple code points grouped into one grapheme cluster. See [Sequence](../features/sequence.md#code-points-and-grapheme-clusters) for an example.

## Why does copying fail?

Copying requires a system clipboard. Check that one is available in the current desktop session; an SSH or headless session may not provide one.

Unavailable property values cannot be copied. See [Copying Values](../features/inspector.md#copying-values) for details.

## Why does my configuration prevent startup?

The startup error identifies the configuration file and the problem. Check the path in [Configurations](../configurations/index.md) and the reported setting in [Config File Format](../configurations/config-file-format.md). For binding errors, see [Custom Keybindings](../keybindings/custom-keybindings.md#conflicts-and-validation).

Use `sauva --print-default-config` for a valid configuration reference.

## Can I use Sauva without glyph images?

Yes. Run `sauva --graphics off`. All text features remain available.

## Which Unicode version does Sauva use?

The bundled data is based on Unicode 17.0.0. See [Unicode Data](../introduction/index.md#unicode-data) for details.
