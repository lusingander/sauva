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

- Use <kbd>h</kbd> / <kbd>l</kbd> or <kbd>Left</kbd> / <kbd>Right</kbd> to inspect adjacent code points.
- Use <kbd>j</kbd> / <kbd>k</kbd> or <kbd>Down</kbd> / <kbd>Up</kbd> to select a property, then <kbd>y</kbd> to copy its value.
- Press <kbd>/</kbd> to [search](../features/search.md), or <kbd>p</kbd>, <kbd>r</kbd>, <kbd>b</kbd>, or <kbd>c</kbd> to [browse](../features/browse.md).
- Press <kbd>F1</kbd> for help with the current screen.
- Press <kbd>q</kbd> or <kbd>Esc</kbd> to quit from the Inspector. <kbd>Ctrl-C</kbd> quits from any screen.

## Analyze a String

Pass a string containing multiple code points to open [Sequence](../features/sequence.md) mode:

```sh
sauva 'Á👩‍💻'
```

The list preserves the input order and groups code points into grapheme clusters.

1. Use <kbd>j</kbd> / <kbd>k</kbd> or <kbd>Down</kbd> / <kbd>Up</kbd> to select a code point.
2. Press <kbd>Enter</kbd> to open it in the Inspector, then <kbd>Backspace</kbd> to return.
3. Press <kbd>n</kbd> in Sequence to compare [normalization forms](../features/normalization.md).

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
