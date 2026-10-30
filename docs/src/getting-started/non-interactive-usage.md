# Non-interactive Usage

Use `-o, --output <FORMAT>` to print a JSON analysis report or a quoted JSON or Rust string literal and exit. This requires no interactive terminal, fonts, or clipboard and does not load user configuration.

## Usage

```sh
sauva -o json U+00BD
sauva -o json --text - < input.txt > report.json
sauva -o json-string U+2192
sauva --output rust-string --text 'Á😀'
```

Input is required and follows the shared [input interpretation](./command-line-options.md#input) and [`--text`](./command-line-options.md#-t---text-text) rules. Use `-` to [read stdin](./command-line-options.md#reading-from-stdin). Empty input is rejected.

An explicit `--graphics` option cannot be combined with `--output`, including `--graphics auto`. Omitting `--output` starts the TUI, even when stdout is redirected.

Each result ends with one LF. Input line endings are preserved in the parsed text and escaped inside JSON strings or string literals.

## Output Formats

| Format | Output |
| --- | --- |
| `json` | A Unicode analysis report; see [JSON Reports](./json-reports.md) for its contents and examples |
| `json-string` | A quoted JSON string literal, such as `"A\u0301\uD83D\uDE00"` |
| `rust-string` | A quoted Rust string literal, such as `"A\u{301}\u{1F600}"` |

String literals use ASCII escapes for non-ASCII characters and controls. A single character is also a string literal. The original text is preserved without normalization or decoding escape sequences. Surrogate code points cannot be output as strings.

The literal matches the corresponding [copy dialog format](../features/copying.md#formats); clipboard copies do not include the extra LF.

## Exit Codes

Successful output goes to stdout; errors go to stderr.

| Code | Meaning |
| --- | --- |
| `0` | Success, including a downstream reader closing the output pipe |
| `1` | Input or output I/O failure, invalid UTF-8 on stdin, or a surrogate code point that cannot be output as a string |
| `2` | Missing or invalid input or options, including empty input |
