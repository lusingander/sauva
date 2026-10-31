# Plain Reports

Use `--output plain` to read a Unicode analysis report without starting the TUI. See [Non-interactive Usage](./non-interactive-usage.md) for input, redirection, and exit codes.

## Example

The accent below is U+0301 following U+0041:

```sh
sauva -o plain --text 'Á😀'
```

```text
Input: "A\u0301\uD83D\uDE00"
CP: 3  UTF-8 Bytes: 7  UTF-16 Units: 4  GC: 2

Positions: CP# / GC# are 1-based; byte / unit ranges are 0-based, end-exclusive.
CP#  GC#  Code Point  UTF-8 Bytes  UTF-16 Units  Name
1    1    U+0041      [0,1)        [0,1)         LATIN CAPITAL LETTER A
2    1    U+0301      [1,3)        [1,2)         COMBINING ACUTE ACCENT
3    2    U+1F600     [3,7)        [2,4)         GRINNING FACE

Normalization Forms
  Form  State      CP  UTF-8 Bytes  UTF-16 Units  GC  Result
  NFC   changed    2   6            3             2   "\u00C1\uD83D\uDE00"
  NFD   unchanged  3   7            4             2   "A\u0301\uD83D\uDE00"
  NFKC  changed    2   6            3             2   "\u00C1\uD83D\uDE00"
  NFKD  unchanged  3   7            4             2   "A\u0301\uD83D\uDE00"

Unicode Version: 17.0.0
```

## Report Contents

| Input | Contents |
| --- | --- |
| One code point | The [Inspector's Relevant properties](../features/inspector.md#properties), followed by text counts and normalization results |
| Multiple code points | The original input, text counts, an ordered list of code points with names and positions, and normalization results |
| A surrogate code point | Unicode properties, with reasons for unavailable character, encoding, and text-analysis values |

CP counts code points; GC counts [extended grapheme clusters](../features/sequence.md#code-points-and-grapheme-clusters). UTF-8 counts bytes; UTF-16 counts 16-bit code units. In the code point list, CP# and GC# start at 1. Ranges start at 0, include the start, and exclude the end.

Each [normalization form](../features/normalization.md#forms) includes its changed or unchanged state, complete result, and all four counts. A single character's Decomposition property is its direct mapping, separate from the whole-text normalization results. Numeric values preserve exact fractions such as `1/2`.

Input, Character, and normalized text use the same quoted representation as `json-string`, escaping non-ASCII characters and controls. Reports use English labels, contain no colors or terminal control sequences, and retain all entries and text regardless of terminal width.

Use [JSON Reports](./json-reports.md) for scripts or complete properties for every code point. Plain report alignment and explanatory wording are not a compatibility contract.
