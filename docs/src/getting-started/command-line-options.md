# Command Line Options

```text
Sauva - Terminal Unicode Explorer 🪄

Usage: sauva [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Text or code point to inspect, or - to read stdin

Options:
  -t, --text <TEXT>           Treat the input as literal text, without code point notation parsing; - reads stdin
  -g, --graphics <MODE>       Control glyph preview graphics [default: auto] [possible values: auto, force, iterm2, off]
      --print-default-config  Print the complete default configuration to standard output
  -h, --help                  Print help
  -V, --version               Print version
```

## `INPUT`

Input is interpreted in this order:

1. A single character opens that character in the Inspector.
2. `U+` or `0x` followed by one to six hexadecimal digits selects a code point. The prefixes are case-insensitive.
3. An argument made entirely of hexadecimal digits is interpreted as a code point. It must contain two to six digits.
4. Other nonempty text opens Sequence mode if it contains multiple code points.

Code point values must be between `U+0000` and `U+10FFFF`.

| Example | Interpretation |
| --- | --- |
| `sauva あ` | Inspect `U+3042` |
| `sauva A` | Inspect the letter A (`U+0041`) |
| `sauva U+A` | Inspect `U+000A` |
| `sauva 41` | Inspect `U+0041` |
| `sauva 1F600` | Inspect `U+1F600` |
| `sauva 'Á👩‍💻'` | Analyze a sequence of code points |

A hexadecimal-only argument that fails notation parsing is rejected; it does not fall back to literal text. For example, use `--text` for a string of seven hexadecimal digits.

## `-t, --text <TEXT>`

Treat the input as literal text without parsing code point notation:

```sh
sauva --text 41
sauva -t U+2192
```

These inputs open Sequence with the characters `4`, `1`, or `U`, `+`, `2`, `1`, `9`, `2`, respectively. A single character still opens the Inspector. Empty text is rejected.

`INPUT` and `--text` cannot be supplied together.

## Reading from Stdin

Use `-` as `INPUT` to read stdin using the same interpretation rules. Use `--text -` to read literal text instead.

```sh
printf 'U+2192' | sauva -
printf 'Á👩‍💻' | sauva -
printf '41' | sauva --text -
printf 'A\nB\n' | sauva -
sauva --text - < input.txt
```

Sauva reads UTF-8 input until EOF before starting the TUI. All whitespace, including trailing LF and CRLF, is preserved. Empty input and invalid UTF-8 are rejected.

In particular, `echo U+2192 | sauva -` is rejected because the newline is part of the code point notation. Use `printf 'U+2192' | sauva -` for notation, or `echo U+2192 | sauva --text -` to inspect the literal text including its newline.

Stdin is read only when `-` is explicitly supplied. An interactive terminal is still required for keyboard input.

Since `-` selects stdin, use `sauva U+002D` to inspect the hyphen character itself.

## `-g, --graphics <MODE>`

| Mode | Behavior |
| --- | --- |
| `auto` | Use the Kitty graphics protocol in detected kitty and Ghostty terminals; otherwise omit glyph images |
| `force` | Use the Kitty graphics protocol without terminal detection |
| `iterm2` | Use the iTerm2 inline image protocol; never selected automatically |
| `off` | Disable glyph images |

See [Compatibility](./compatibility.md) for support details.

## `--print-default-config`

Print the complete built-in configuration, including default keybindings, and exit:

```sh
sauva --print-default-config
sauva --print-default-config > config.toml
```

This option does not start the TUI or load your local configuration. It must be used on its own.

See [Configurations](../configurations/index.md) for where to put the file.

## `-h, --help` and `-V, --version`

Print command line help or the installed application version and exit.
