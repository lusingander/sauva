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

## JSON Reports

`json` prints one formatted JSON object containing the original text, code point (CP) and grapheme cluster (GC) counts, UTF-8 byte and UTF-16 code unit counts, every code point's Unicode properties and positions, extended grapheme cluster ranges, and NFC / NFD / NFKC / NFKD results with their counts. Unchanged normalization results are included in full. Normalization does not modify the original text.

Code point and grapheme indices are zero-based. Ranges include the start and exclude the end, using offsets into the entire original text. UTF-16 positions and counts use code units, not bytes or terminal cells.

The structure is the same for single characters and strings, and includes all properties regardless of the Inspector display mode. Surrogate code points such as `U+D800` return properties successfully; text, metrics, graphemes, normalization, and the code point's character, positions, and encoding are `null` because they require a Unicode scalar value.

All JSON strings use ASCII escapes; parsing restores the Unicode text. The fields and types are defined in [output.schema.json](https://github.com/lusingander/sauva/blob/master/output.schema.json). `schema_version` identifies the report contract, while `unicode_version` identifies the Unicode data and algorithms. Additional fields may be introduced within the same schema version; consumers should ignore unknown fields. Key order and indentation are not part of the contract.

To pass a normalization result to another command without adding a newline, use `jq -j`:

```sh
sauva -o json --text - < input.txt |
  jq -j '.normalization.nfc.text' |
  sauva -o rust-string --text -
```

## String Literals

The following examples use `A` + U+0301 + `😀`:

| Format | Output |
| --- | --- |
| `json-string` | `"A\u0301\uD83D\uDE00"` |
| `rust-string` | `"A\u{301}\u{1F600}"` |

Both formats include double quotes and use ASCII escapes for non-ASCII characters and controls. A single character is also a string literal. The original text is preserved without normalization or decoding escape sequences. Surrogate code points cannot be output as strings.

The literal matches the corresponding [copy dialog format](../features/copying.md#formats); clipboard copies do not include the extra LF.

## Exit Codes

Successful output goes to stdout; errors go to stderr.

| Code | Meaning |
| --- | --- |
| `0` | Success, including a downstream reader closing the output pipe |
| `1` | Input or output I/O failure, invalid UTF-8 on stdin, or a surrogate code point that cannot be output as a string |
| `2` | Missing or invalid input or options, including empty input |
