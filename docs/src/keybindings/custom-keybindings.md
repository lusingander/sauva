# Custom Keybindings

Customize bindings in the `[keybindings.<context>]` sections of [the configuration file](../configurations/index.md).

```toml
[keybindings.global]
help = ["f2"]

[keybindings.inspector]
next_code_point = ["l", "right", "n"]
back = ["backspace"]
browse_planes = []

[keybindings.sequence]
open_copy_dialog = ["C"]

[keybindings.normalization_result]
open_copy_dialog = ["C"]

[keybindings.copy_dialog]
activate = ["enter", "y"]
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
| `copy_dialog` | Copy candidate list in Sequence and Normalization Result |
| `browse_plane` | Plane list |
| `browse_range` | Range list |
| `browse_block` | Block list |
| `browse_code_points` | Code point grid |
| `help` | Contextual help |

Global bindings remain active in all contexts. When help is open, only its own bindings and the global bindings are active.

Disabling a local command does not disable the global binding for the same action. For example, disabling `keybindings.inspector.quit` leaves the default global <kbd>Ctrl-C</kbd> binding available.

## Key Formats

### Characters

Use a single non-whitespace character, such as <kbd>a</kbd>, <kbd>1</kbd>, <kbd>/</kbd>, or <kbd>-</kbd>.

Uppercase ASCII letters represent <kbd>Shift</kbd> plus that letter. For example, `G` and `shift-g` both describe <kbd>Shift-G</kbd>.

### Named Keys

| Name | Key |
| --- | --- |
| `space` | <kbd>Space</kbd> |
| `enter` | <kbd>Enter</kbd> |
| `esc` | <kbd>Escape</kbd> |
| `tab` | <kbd>Tab</kbd> |
| `backspace` | <kbd>Backspace</kbd> |
| `delete` | <kbd>Delete</kbd> |
| `left`, `right`, `up`, `down` | <kbd>Left</kbd>, <kbd>Right</kbd>, <kbd>Up</kbd>, <kbd>Down</kbd> |
| `home`, `end` | <kbd>Home</kbd> and <kbd>End</kbd> |
| `pageup`, `pagedown` | <kbd>Page Up</kbd> and <kbd>Page Down</kbd> |
| `f1` through `f12` | <kbd>F1</kbd> through <kbd>F12</kbd> |

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

For example, binding <kbd>j</kbd> to `search.next_result` is rejected because <kbd>j</kbd> is needed for typing the query. Use an arrow key or another combination that is not an input editing key.

When moving a key from one command to another, also remove it from the original command's binding. The same applies if new built-in bindings conflict with existing overrides. For example:

```toml
[keybindings.inspector]
next_code_point = ["right"]
copy_value = ["y", "l"]
```
