# Config File Format

All configuration settings are optional. The following example contains the default colors, glyph preview settings, and UI settings:

```toml
[color]
fg = "reset"
bg = "reset"
muted = "darkgray"
accent = "cyan"
heading = "blue"
border = "darkgray"
match = "yellow"
key = "yellow"
link = "blue"

[color.selection]
fg = "black"
bg = "cyan"

[color.difference]
fg = "black"
bg = "yellow"

[color.status]
info = "green"
warning = "yellow"

[glyph_preview]
font_families = []
emoji_font_families = []
fg = "#f5f7fa"
bg = "#00000000"

[ui]
selection_cursor = ""
input_cursor = "native"
```

Use `sauva --print-default-config` to print the full configuration, including every default keybinding.

The configuration structure is also described by [config.schema.json](https://github.com/lusingander/sauva/blob/master/config.schema.json). Sauva validates the configuration at startup, including terminal cell widths and keybinding conflicts.

## `color`

These settings control the TUI's colors:

| Setting | Default | Use |
| --- | --- | --- |
| `fg` | `"reset"` | Main text |
| `bg` | `"reset"` | Main background |
| `muted` | `"darkgray"` | Secondary text and labels |
| `accent` | `"cyan"` | Accented text and grapheme markers |
| `heading` | `"blue"` | Section headings |
| `border` | `"darkgray"` | Borders and scrollbars |
| `match` | `"yellow"` | Search matches |
| `key` | `"yellow"` | Key labels in help and the footer |
| `link` | `"blue"` | Link text |
| `selection.fg` | `"black"` | Selected item text |
| `selection.bg` | `"cyan"` | Selected item background |
| `difference.fg` | `"black"` | Text in changed normalization spans |
| `difference.bg` | `"yellow"` | Background of changed normalization spans |
| `status.info` | `"green"` | Informational status messages |
| `status.warning` | `"yellow"` | Warning status messages |

Changed normalization spans are also underlined. Their colors are configured separately from the selection colors.

UI color values are strings in one of these formats:

- ANSI names: `reset`, `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `gray`, `darkgray`, `light-red`, `light-green`, `light-yellow`, `light-blue`, `light-magenta`, `light-cyan`, or `white`.
- RGB hexadecimal: `"#RRGGBB"`.
- Indexed color: a quoted number from `"0"` to `"255"`.

`reset` uses the terminal's default color. UI colors do not accept an alpha channel.

For example:

```toml
[color]
heading = "#89b4fa"
muted = "245"

[color.selection]
fg = "black"
bg = "light-blue"
```

## `glyph_preview`

| Setting | Type | Default |
| --- | --- | --- |
| `font_families` | Array of font family names | `[]` |
| `emoji_font_families` | Array of font family names | `[]` |
| `fg` | RGB or RGBA color string | `"#f5f7fa"` |
| `bg` | RGB or RGBA color string | `"#00000000"` |

Font lists are tried in the order given. Empty or whitespace-only family names are rejected. Image colors accept `#RRGGBB` or `#RRGGBBAA`, including an alpha channel.

See [Glyph Preview](./glyph-preview.md) for examples and the complete font selection order.

## `ui`

### `selection_cursor`

The marker shown before a selected Inspector property, list entry, or code point.

- Type: string.
- Default: `""` (no visible marker).
- Must be empty or occupy exactly one terminal cell, without control characters.

```toml
[ui]
selection_cursor = "▸"
```

### `input_cursor`

The cursor shown in the search input.

- Default: `"native"`, using the terminal's native cursor.
- Use `{ text = "..." }` for a text cursor. Its text must occupy exactly one terminal cell, without control characters.

```toml
[ui]
input_cursor = { text = "|" }
```

## `keybindings`

Bindings are grouped by screen context. Each command accepts an array of key strings that replaces its built-in binding. An empty array disables the command in that context.

```toml
[keybindings.global]
help = ["f2"]

[keybindings.inspector]
next_code_point = ["l", "right", "n"]
browse_planes = []
```

See [Custom Keybindings](../keybindings/custom-keybindings.md) for contexts, command names, key formats, and validation rules.
