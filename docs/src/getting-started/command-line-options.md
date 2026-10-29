# Command Line Options

```text
Sauva - Terminal Unicode Explorer 🪄

Usage: sauva [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Text or code point to inspect, or - to read stdin

Options:
  -t, --text <TEXT>           Treat the input as literal text, without code point notation parsing; - reads stdin
      --output <FORMAT>       Print a result and exit [possible values: json-string, rust-string]
  -g, --graphics <MODE>       Control glyph preview graphics [default: auto] [possible values: auto, force, iterm2, off]
      --print-default-config  Print the complete default configuration to standard output
  -h, --help                  Print help
  -V, --version               Print version
```

## `INPUT`

Input is interpreted in this order:

1. An input containing a single code point selects that character.
2. `U+` or `0x` followed by one to six hexadecimal digits selects a code point. The prefixes are case-insensitive.
3. An argument made entirely of hexadecimal digits is interpreted as a code point. It must contain two to six digits.
4. Other nonempty text is treated as a string, even if it looks like one character.

Code point values must be between `U+0000` and `U+10FFFF`. Empty input is rejected.

Without `--output`, one code point opens the Inspector and multiple code points open Sequence mode.

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

Empty text is rejected.

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

Sauva reads UTF-8 input until EOF. All whitespace, including trailing LF and CRLF, is preserved. Empty input and invalid UTF-8 are rejected.

In particular, `echo U+2192 | sauva -` is rejected because the newline is part of the code point notation. Use `printf 'U+2192' | sauva -` for notation, or `echo U+2192 | sauva --text -` to inspect the literal text including its newline.

Stdin is read only when `-` is explicitly supplied. The TUI still requires an interactive terminal for keyboard input. Use [`--output`](#--output-format) for noninteractive output.

Since `-` selects stdin, use `sauva U+002D` to inspect the hyphen character itself.

## `--output <FORMAT>`

Print one quoted string literal to stdout and exit:

```sh
sauva --output json-string U+2192
sauva --output rust-string --text 'Á😀'
sauva --output json-string --text - < input.txt > literal.json
```

The following examples use `A` + U+0301 + `😀`:

| Format | Output |
| --- | --- |
| `json-string` | `"A\u0301\uD83D\uDE00"` |
| `rust-string` | `"A\u{301}\u{1F600}"` |

Both formats include double quotes and use ASCII escapes for non-ASCII characters and controls. A single character is also a string literal. The original text is preserved without normalization or decoding escape sequences.

Input is required, using the same [`INPUT`](#input) and [`--text`](#-t---text-text) rules. Empty input is rejected. Surrogate code points such as `U+D800` cannot be output as strings.

Each result ends with one LF outside the closing quote. Newlines from the input are escaped inside the literal. The literal matches the corresponding [copy dialog format](../features/copying.md#formats); clipboard copies do not include the extra LF.

This mode requires no interactive terminal, fonts, or clipboard and does not load user configuration. It cannot be combined with an explicit `--graphics` option, including `--graphics auto`. Omitting `--output` starts the TUI, even when stdout is redirected.

### Exit Codes

These exit codes apply to `--output`. Successful output goes to stdout; errors go to stderr.

| Code | Meaning |
| --- | --- |
| `0` | Success, including a downstream reader closing the output pipe |
| `1` | Input or output I/O failure, invalid UTF-8, or a surrogate code point that cannot be output as a string |
| `2` | Missing or invalid input or options, including empty input |

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
