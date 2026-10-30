# Non-interactive Usage

Use `-o, --output <FORMAT>` to print a quoted JSON or Rust string literal and exit. This requires no interactive terminal, fonts, or clipboard and does not load user configuration.

## Usage

```sh
sauva -o json-string U+2192
sauva --output rust-string --text 'Á😀'
sauva -o json-string --text - < input.txt > literal.json
```

Input is required and follows the shared [input interpretation](./command-line-options.md#input) and [`--text`](./command-line-options.md#-t---text-text) rules. Use `-` to [read stdin](./command-line-options.md#reading-from-stdin). Empty input is rejected. Surrogate code points such as `U+D800` cannot be output as strings.

An explicit `--graphics` option cannot be combined with `--output`, including `--graphics auto`. Omitting `--output` starts the TUI, even when stdout is redirected.

## Output Formats

The following examples use `A` + U+0301 + `😀`:

| Format | Output |
| --- | --- |
| `json-string` | `"A\u0301\uD83D\uDE00"` |
| `rust-string` | `"A\u{301}\u{1F600}"` |

Both formats include double quotes and use ASCII escapes for non-ASCII characters and controls. A single character is also a string literal. The original text is preserved without normalization or decoding escape sequences.

Each result ends with one LF outside the closing quote. Newlines from the input are escaped inside the literal. The literal matches the corresponding [copy dialog format](../features/copying.md#formats); clipboard copies do not include the extra LF.

## Exit Codes

Successful output goes to stdout; errors go to stderr.

| Code | Meaning |
| --- | --- |
| `0` | Success, including a downstream reader closing the output pipe |
| `1` | Input or output I/O failure, invalid UTF-8 on stdin, or a surrogate code point that cannot be output as a string |
| `2` | Missing or invalid input or options, including empty input |
