# Keybindings

Press <kbd>F1</kbd> to open or close contextual help. Help and the footer show the active bindings, including your custom settings.

The tables below list the built-in bindings and the command names used in [Custom Keybindings](./custom-keybindings.md).

## Global

These bindings apply in every screen, including help.

| Command | Keys | Action |
| --- | --- | --- |
| `quit` | <kbd>Ctrl-C</kbd> | Quit Sauva |
| `help` | <kbd>F1</kbd> | Open or close help |

## Inspector

Context: `inspector`.

| Command | Keys | Action |
| --- | --- | --- |
| `quit` | <kbd>q</kbd>, <kbd>Esc</kbd> | Quit |
| `previous_code_point` | <kbd>h</kbd>, <kbd>Left</kbd> | Inspect the previous code point |
| `next_code_point` | <kbd>l</kbd>, <kbd>Right</kbd> | Inspect the next code point |
| `move_up` | <kbd>k</kbd>, <kbd>Up</kbd> | Select the previous property |
| `move_down` | <kbd>j</kbd>, <kbd>Down</kbd> | Select the next property |
| `previous_group` | <kbd>[</kbd> | Select the first property of the previous section |
| `next_group` | <kbd>]</kbd> | Select the first property of the next section |
| `page_up` | <kbd>Ctrl-U</kbd> | Move properties up one page |
| `page_down` | <kbd>Ctrl-D</kbd> | Move properties down one page |
| `first` | <kbd>g</kbd> | Select the first property |
| `last` | <kbd>G</kbd> | Select the last property |
| `copy_value` | <kbd>y</kbd> | Copy the selected property value |
| `search` | <kbd>/</kbd> | Open search |
| `browse_planes` | <kbd>p</kbd> | Browse planes |
| `browse_ranges` | <kbd>r</kbd> | Browse ranges in the current plane |
| `browse_blocks` | <kbd>b</kbd> | Browse blocks |
| `browse_code_points` | <kbd>c</kbd> | Browse code points in the current range |
| `back` | <kbd>Backspace</kbd> | Return to the sequence, when one is open |

## Search

Context: `search`.

| Command | Keys | Action |
| --- | --- | --- |
| `previous_result` | <kbd>Up</kbd>, <kbd>Ctrl-P</kbd> | Select the previous result |
| `next_result` | <kbd>Down</kbd>, <kbd>Ctrl-N</kbd> | Select the next result |
| `inspect_result` | <kbd>Enter</kbd> | Open the selected result in the Inspector |
| `close` | <kbd>Esc</kbd> | Close search |

Character keys edit the query, including <kbd>j</kbd>, <kbd>k</kbd>, and <kbd>q</kbd>. Use the arrow keys or <kbd>Ctrl-P</kbd> / <kbd>Ctrl-N</kbd> to select results. Search input editing keys are reserved and cannot be assigned to commands.

## Sequence

Context: `sequence`.

| Command | Keys | Action |
| --- | --- | --- |
| `quit` | <kbd>q</kbd>, <kbd>Esc</kbd> | Quit |
| `move_up` | <kbd>k</kbd>, <kbd>Up</kbd> | Select the previous code point |
| `move_down` | <kbd>j</kbd>, <kbd>Down</kbd> | Select the next code point |
| `previous_group` | <kbd>[</kbd> | Select the first code point of the previous grapheme cluster |
| `next_group` | <kbd>]</kbd> | Select the first code point of the next grapheme cluster |
| `first` | <kbd>g</kbd> | Select the first code point |
| `last` | <kbd>G</kbd> | Select the last code point |
| `activate` | <kbd>Enter</kbd> | Inspect the selected code point |
| `normalize` | <kbd>n</kbd> | Compare normalization forms |
| `open_copy_dialog` | <kbd>Y</kbd> | Open copy candidates |

## Normalization

Context: `normalization`.

| Command | Keys | Action |
| --- | --- | --- |
| `quit` | <kbd>q</kbd> | Quit |
| `close` | <kbd>Esc</kbd> | Return to the original sequence |
| `back` | <kbd>Backspace</kbd> | Return to the original sequence |
| `move_up` | <kbd>k</kbd>, <kbd>Up</kbd> | Select the previous normalization form |
| `move_down` | <kbd>j</kbd>, <kbd>Down</kbd> | Select the next normalization form |
| `first` | <kbd>g</kbd> | Select NFC |
| `last` | <kbd>G</kbd> | Select NFKD |
| `activate` | <kbd>Enter</kbd> | Inspect the selected normalization result |
| `copy_value` | <kbd>y</kbd> | Copy the exact normalized text |

## Normalization Result

Context: `normalization_result`.

| Command | Keys | Action |
| --- | --- | --- |
| `quit` | <kbd>q</kbd> | Quit |
| `close` | <kbd>Esc</kbd> | Return to normalization comparison |
| `back` | <kbd>Backspace</kbd> | Return to normalization comparison |
| `move_up` | <kbd>k</kbd>, <kbd>Up</kbd> | Select the previous code point |
| `move_down` | <kbd>j</kbd>, <kbd>Down</kbd> | Select the next code point |
| `previous_group` | <kbd>[</kbd> | Select the first code point of the previous grapheme cluster |
| `next_group` | <kbd>]</kbd> | Select the first code point of the next grapheme cluster |
| `first` | <kbd>g</kbd> | Select the first code point |
| `last` | <kbd>G</kbd> | Select the last code point |
| `activate` | <kbd>Enter</kbd> | Inspect the selected code point |
| `open_copy_dialog` | <kbd>Y</kbd> | Open copy candidates for the normalized text |

Use <kbd>Backspace</kbd> in the Inspector to return to this result.

## Copy Dialog

Context: `copy_dialog`. These bindings apply to the candidate list opened from Sequence or Normalization Result. The source screen's bindings are inactive while the dialog is open.

| Command | Keys | Action |
| --- | --- | --- |
| `move_up` | <kbd>k</kbd>, <kbd>Up</kbd> | Select the previous candidate |
| `move_down` | <kbd>j</kbd>, <kbd>Down</kbd> | Select the next candidate |
| `first` | <kbd>g</kbd>, <kbd>Home</kbd> | Select Code Point |
| `last` | <kbd>G</kbd>, <kbd>End</kbd> | Select the whole input or result |
| `activate` | <kbd>Enter</kbd> | Copy the selected candidate |
| `close` | <kbd>Esc</kbd> | Cancel and return to the source screen |

Global <kbd>F1</kbd> opens the shared Copy Dialog help. Closing help restores the candidate selection. Global <kbd>Ctrl-C</kbd> still quits.

## Browse Lists

Contexts: `browse_plane`, `browse_range`, and `browse_block`.

| Command | Keys | Action |
| --- | --- | --- |
| `quit` | <kbd>q</kbd> | Quit |
| `close` | <kbd>Esc</kbd> | Cancel browsing and return to the Inspector |
| `back` | <kbd>Backspace</kbd> | Return to the previous browse level, or to the Inspector at the top |
| `move_up` | <kbd>k</kbd>, <kbd>Up</kbd> | Select the previous item |
| `move_down` | <kbd>j</kbd>, <kbd>Down</kbd> | Select the next item |
| `first` | <kbd>g</kbd> | Select the first item |
| `last` | <kbd>G</kbd> | Select the last item |
| `activate` | <kbd>Enter</kbd> | Open ranges in a plane, or code points in a range or block |

`browse_range` and `browse_block` additionally support:

| Command | Keys | Action |
| --- | --- | --- |
| `page_up` | <kbd>Ctrl-U</kbd> | Move backward by a large step |
| `page_down` | <kbd>Ctrl-D</kbd> | Move forward by a large step |

## Browse Code Points

Context: `browse_code_points`.

| Command | Keys | Action |
| --- | --- | --- |
| `quit` | <kbd>q</kbd> | Quit |
| `close` | <kbd>Esc</kbd> | Cancel browsing and return to the Inspector |
| `back` | <kbd>Backspace</kbd> | Return to the previous browse level |
| `move_left` | <kbd>h</kbd>, <kbd>Left</kbd> | Move left |
| `move_right` | <kbd>l</kbd>, <kbd>Right</kbd> | Move right |
| `move_up` | <kbd>k</kbd>, <kbd>Up</kbd> | Move up one row |
| `move_down` | <kbd>j</kbd>, <kbd>Down</kbd> | Move down one row |
| `page_up` | <kbd>Ctrl-U</kbd> | Move backward by a large step |
| `page_down` | <kbd>Ctrl-D</kbd> | Move forward by a large step |
| `first` | <kbd>g</kbd> | Select the first code point in the range or block |
| `last` | <kbd>G</kbd> | Select the last code point in the range or block |
| `activate` | <kbd>Enter</kbd> | Inspect the selected code point |

## Help

Context: `help`. The underlying screen's commands are inactive while help is open; global bindings remain active.

| Command | Keys | Action |
| --- | --- | --- |
| `close` | <kbd>Esc</kbd> | Close help |
| `move_up` | <kbd>k</kbd>, <kbd>Up</kbd> | Scroll up one line |
| `move_down` | <kbd>j</kbd>, <kbd>Down</kbd> | Scroll down one line |
| `page_up` | <kbd>Ctrl-U</kbd> | Scroll up one page |
| `page_down` | <kbd>Ctrl-D</kbd> | Scroll down one page |
| `first` | <kbd>g</kbd> | Go to the beginning |
| `last` | <kbd>G</kbd> | Go to the end |
