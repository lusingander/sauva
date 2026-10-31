# JSON Reports

Use `--output json` to print a Unicode analysis report as one JSON object. Single characters and strings use the same structure. See [Non-interactive Usage](./non-interactive-usage.md) for input, redirection, and exit codes.

## Example

The accent below is U+0301 following U+0041:

```sh
sauva -o json --text 'Á😀'
```

The input has three code points and two grapheme clusters. It occupies seven UTF-8 bytes and four UTF-16 code units. NFC combines the first two code points into `Á`, leaving two code points in the result.

<details>
<summary>Show the complete JSON output</summary>

```json
{
  "schema_version": 1,
  "sauva_version": "0.6.0",
  "unicode_version": "17.0.0",
  "text": "A\u0301\uD83D\uDE00",
  "metrics": {
    "code_points": 3,
    "utf8_bytes": 7,
    "utf16_units": 4,
    "graphemes": 2
  },
  "code_points": [
    {
      "index": 0,
      "value": 65,
      "code_point": "U+0041",
      "character": "A",
      "utf8_range": [
        0,
        1
      ],
      "utf16_range": [
        0,
        1
      ],
      "grapheme_index": 0,
      "properties": {
        "name": "LATIN CAPITAL LETTER A",
        "aliases": [],
        "plane": 0,
        "block": "Basic Latin",
        "general_category": "Lu",
        "script": "Latin",
        "age": "1.1",
        "east_asian_width": "Na",
        "canonical_combining_class": 0,
        "bidi_class": "L",
        "default_ignorable": false,
        "case_mapping": {
          "uppercase": [
            65
          ],
          "lowercase": [
            97
          ],
          "titlecase": [
            65
          ],
          "case_folding": [
            97
          ]
        },
        "numeric_type": "None",
        "numeric_value": null,
        "decomposition": null,
        "encoding": {
          "utf8": [
            65
          ],
          "utf16": [
            65
          ]
        }
      }
    },
    {
      "index": 1,
      "value": 769,
      "code_point": "U+0301",
      "character": "\u0301",
      "utf8_range": [
        1,
        3
      ],
      "utf16_range": [
        1,
        2
      ],
      "grapheme_index": 0,
      "properties": {
        "name": "COMBINING ACUTE ACCENT",
        "aliases": [],
        "plane": 0,
        "block": "Combining Diacritical Marks",
        "general_category": "Mn",
        "script": "Inherited",
        "age": "1.1",
        "east_asian_width": "A",
        "canonical_combining_class": 230,
        "bidi_class": "NSM",
        "default_ignorable": false,
        "case_mapping": {
          "uppercase": [
            769
          ],
          "lowercase": [
            769
          ],
          "titlecase": [
            769
          ],
          "case_folding": [
            769
          ]
        },
        "numeric_type": "None",
        "numeric_value": null,
        "decomposition": null,
        "encoding": {
          "utf8": [
            204,
            129
          ],
          "utf16": [
            769
          ]
        }
      }
    },
    {
      "index": 2,
      "value": 128512,
      "code_point": "U+1F600",
      "character": "\uD83D\uDE00",
      "utf8_range": [
        3,
        7
      ],
      "utf16_range": [
        2,
        4
      ],
      "grapheme_index": 1,
      "properties": {
        "name": "GRINNING FACE",
        "aliases": [],
        "plane": 1,
        "block": "Emoticons",
        "general_category": "So",
        "script": "Common",
        "age": "6.1",
        "east_asian_width": "W",
        "canonical_combining_class": 0,
        "bidi_class": "ON",
        "default_ignorable": false,
        "case_mapping": {
          "uppercase": [
            128512
          ],
          "lowercase": [
            128512
          ],
          "titlecase": [
            128512
          ],
          "case_folding": [
            128512
          ]
        },
        "numeric_type": "None",
        "numeric_value": null,
        "decomposition": null,
        "encoding": {
          "utf8": [
            240,
            159,
            152,
            128
          ],
          "utf16": [
            55357,
            56832
          ]
        }
      }
    }
  ],
  "graphemes": [
    {
      "index": 0,
      "code_point_range": [
        0,
        2
      ],
      "utf8_range": [
        0,
        3
      ],
      "utf16_range": [
        0,
        2
      ]
    },
    {
      "index": 1,
      "code_point_range": [
        2,
        3
      ],
      "utf8_range": [
        3,
        7
      ],
      "utf16_range": [
        2,
        4
      ]
    }
  ],
  "normalization": {
    "nfc": {
      "changed": true,
      "text": "\u00C1\uD83D\uDE00",
      "metrics": {
        "code_points": 2,
        "utf8_bytes": 6,
        "utf16_units": 3,
        "graphemes": 2
      }
    },
    "nfd": {
      "changed": false,
      "text": "A\u0301\uD83D\uDE00",
      "metrics": {
        "code_points": 3,
        "utf8_bytes": 7,
        "utf16_units": 4,
        "graphemes": 2
      }
    },
    "nfkc": {
      "changed": true,
      "text": "\u00C1\uD83D\uDE00",
      "metrics": {
        "code_points": 2,
        "utf8_bytes": 6,
        "utf16_units": 3,
        "graphemes": 2
      }
    },
    "nfkd": {
      "changed": false,
      "text": "A\u0301\uD83D\uDE00",
      "metrics": {
        "code_points": 3,
        "utf8_bytes": 7,
        "utf16_units": 4,
        "graphemes": 2
      }
    }
  }
}
```

</details>

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
