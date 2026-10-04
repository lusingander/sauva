# Basic Usage

## Inspect a Code Point

Run Sauva without arguments to open the default code point:

```sh
sauva
```

You can also specify a character or code point:

```sh
sauva あ
sauva U+2192
sauva 0x1F600
```

The [Inspector](../features/inspector.md) shows the selected code point's properties and glyph preview.

- Use `h` / `l` or Left / Right to inspect adjacent code points.
- Use `j` / `k` or Down / Up to select a property, then `y` to copy its value.
- Press `/` to [search](../features/search.md), or `p`, `r`, `b`, or `c` to [browse](../features/browse.md).
- Press `F1` for help with the current screen.
- Press `q` or `Esc` to quit from the Inspector. `Ctrl-C` quits from any screen.

## Analyze a String

Pass a string containing multiple code points to open [Sequence](../features/sequence.md) mode:

```sh
sauva 'Á👩‍💻'
```

The list preserves the input order and groups code points into grapheme clusters.

1. Use `j` / `k` or Down / Up to select a code point.
2. Press `Enter` to open it in the Inspector, then `Backspace` to return.
3. Press `n` in Sequence to compare [normalization forms](../features/normalization.md).

![Sequence and normalization demo](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/demo-sequence.gif)

Use `--text` when a string would otherwise be interpreted as code point notation:

```sh
sauva --text 41
sauva --text U+2192
```

A single code point opens the Inspector, even with `--text`.

## Read from a Pipe or File

Specify `-` to read UTF-8 input from stdin:

```sh
printf 'Á👩‍💻' | sauva -
printf 'U+2192' | sauva -
sauva --text - < input.txt
```

Sauva waits for EOF before opening the TUI. Whitespace and trailing newlines are preserved. For code point notation, use `printf` without a trailing newline; use `--text -` to inspect text exactly as supplied.

For all input rules and options, see [Command Line Options](./command-line-options.md). The complete controls are listed in [Keybindings](../keybindings/index.md).
