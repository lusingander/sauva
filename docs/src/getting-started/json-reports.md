# JSON Reports

Use `--output json` to print a Unicode analysis report as one JSON object. Single characters and strings use the same structure. See [Non-interactive Usage](./non-interactive-usage.md) for input, redirection, and exit codes.

```sh
sauva -o json --text 'Á😀'
```

## Report Contents

| Field | Contents |
| --- | --- |
| `text` | The original text |
| `metrics` | Code point, UTF-8 byte, UTF-16 code unit, and grapheme cluster counts |
| `code_points` | Each code point's value, Unicode properties, and position |
| `graphemes` | Extended grapheme cluster ranges |
| `normalization` | NFC, NFD, NFKC, and NFKD results with their counts |

Code points and grapheme clusters follow the input order, including repeated characters. All Unicode properties are included for each code point.

## Counts and Positions

The counts distinguish [code points and grapheme clusters](../features/sequence.md#code-points-and-grapheme-clusters). UTF-8 counts bytes; UTF-16 counts 16-bit code units.

Code point and grapheme indices are zero-based. Ranges include the start and exclude the end, using offsets into the entire original text.

## Properties and Normalization

Case mappings contain code point integers and use Unicode's default full mappings for each character, independently of locale and surrounding text. Unchanged mappings contain the original code point.

Numeric values are strings preserving exact integers and fractions, such as `"0"` and `"1/2"`. A character's `decomposition` is its direct decomposition mapping; the whole-text normalized results are in `normalization`.

Each [normalization form](../features/normalization.md#forms) includes `changed`, the complete result `text`, and `metrics`, even when unchanged. The original input remains in the report's top-level `text`.

To pass a normalization result to another command without adding a newline, use `jq -j`:

```sh
sauva -o json --text - < input.txt |
  jq -j '.normalization.nfc.text' |
  sauva -o rust-string --text -
```

## Unavailable Values

`null` means a value is unavailable. For example, a code point without a numeric value has `numeric_value: null`; a code point without aliases has `aliases: []`.

Surrogate code points such as `U+D800` return properties successfully. Their text, text-analysis data, character, positions, and encoding are `null` because they cannot be represented as Unicode scalar values.

## JSON and Schema

All JSON strings use ASCII escapes. Parsing the JSON restores the Unicode text.

The complete fields and types are defined in [output.schema.json](https://github.com/lusingander/sauva/blob/master/output.schema.json). `schema_version` identifies the report contract, `sauva_version` the running binary, and `unicode_version` the Unicode data and algorithms.

Additional fields may be introduced within the same schema version; consumers should ignore unknown fields. Key order and indentation are not part of the contract.
