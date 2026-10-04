# Custom Keybindings

Customize bindings in the `[keybindings.<context>]` sections of [the configuration file](../configurations/index.md).

```toml
[keybindings.global]
help = ["f2"]

[keybindings.inspector]
next_code_point = ["l", "right", "n"]
back = ["backspace"]
browse_planes = []
```

- Each array replaces the keys for one command in one context.
- Omitted commands keep their built-in bindings.
- Multiple keys can be assigned to one command.
- An empty array disables that command in that context.
- Help and the footer display the resulting bindings.

Use `sauva --print-default-config` to see every context, command, and default binding. The command names are also listed in [Keybindings](./index.md).

## Contexts

| Context | Applies to |
| --- | --- |
| `global` | Every screen; supports `quit` and `help` |
| `inspector` | Code point properties |
| `search` | Search results |
| `sequence` | The original input sequence |
| `normalization` | Normalization form comparison |
| `normalization_result` | A normalized sequence |
| `browse_plane` | Plane list |
| `browse_range` | Range list |
| `browse_block` | Block list |
| `browse_code_points` | Code point grid |
| `help` | Contextual help |

Global bindings remain active in all contexts. When help is open, only its own bindings and the global bindings are active.

Disabling a local command does not disable the global binding for the same action. For example, disabling `keybindings.inspector.quit` leaves the default global `Ctrl-C` binding available.

## Key Formats

### Characters

Use a single non-whitespace character, such as `a`, `1`, `/`, or `-`.

Uppercase ASCII letters represent Shift plus that letter. For example, `G` and `shift-g` describe the same key.

### Named Keys

| Name | Key |
| --- | --- |
| `space` | Space |
| `enter` | Enter |
| `esc` | Escape |
| `tab` | Tab |
| `backspace` | Backspace |
| `delete` | Delete |
| `left`, `right`, `up`, `down` | Arrow keys |
| `home`, `end` | Home and End |
| `pageup`, `pagedown` | Page Up and Page Down |
| `f1` through `f12` | Function keys |

Names and modifier prefixes are lowercase.

### Modifiers

Use `ctrl-`, `alt-`, or `shift-` followed by a single character. Modifiers can be combined, without repeating the same modifier:

```toml
[keybindings.inspector]
next_code_point = ["ctrl-n", "alt-l"]
last = ["shift-g"]
```

Modifiers apply only to character keys. Forms such as `ctrl-left`, `shift-tab`, or `alt-f1` are not supported. Use a lowercase ASCII letter after a modifier; `ctrl-G` is invalid.

The terminal must deliver the chosen key combination to Sauva. Some combinations are handled by the terminal itself.

## Conflicts and Validation

Sauva rejects invalid bindings at startup, including:

- Unknown contexts, command names, or key names.
- Duplicate keys for a command, including equivalent forms such as `G` and `shift-g`.
- A key assigned to two different commands in the same context.
- A local binding that uses a global key for a different command.
- Global or search bindings that would intercept search input editing.

For example, binding `j` to `search.next_result` is rejected because `j` is needed for typing the query. Use an arrow key or another combination that is not an input editing key.

When moving a key from one command to another, also remove it from the original command's binding. For example:

```toml
[keybindings.inspector]
next_code_point = ["right"]
copy_value = ["y", "l"]
```
