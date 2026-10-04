# Keybindings

Press `F1` to open or close contextual help. Help and the footer show the active bindings, including your custom settings.

The tables below list the built-in bindings and the command names used in [Custom Keybindings](./custom-keybindings.md).

## Global

These bindings apply in every screen, including help.

| Command | Keys | Action |
| --- | --- | --- |
| `quit` | `Ctrl-C` | Quit Sauva |
| `help` | `F1` | Open or close help |

## Inspector

Context: `inspector`.

| Command | Keys | Action |
| --- | --- | --- |
| `quit` | `q`, `Esc` | Quit |
| `previous_code_point` | `h`, Left | Inspect the previous code point |
| `next_code_point` | `l`, Right | Inspect the next code point |
| `move_up` | `k`, Up | Select the previous property |
| `move_down` | `j`, Down | Select the next property |
| `page_up` | `Ctrl-U` | Move properties up one page |
| `page_down` | `Ctrl-D` | Move properties down one page |
| `first` | `g` | Select the first property |
| `last` | `G` | Select the last property |
| `copy_value` | `y` | Copy the selected property value |
| `search` | `/` | Open search |
| `browse_planes` | `p` | Browse planes |
| `browse_ranges` | `r` | Browse ranges in the current plane |
| `browse_blocks` | `b` | Browse blocks |
| `browse_code_points` | `c` | Browse code points in the current range |
| `back` | Backspace | Return to the sequence, when one is open |

## Search

Context: `search`.

| Command | Keys | Action |
| --- | --- | --- |
| `previous_result` | Up, `Ctrl-P` | Select the previous result |
| `next_result` | Down, `Ctrl-N` | Select the next result |
| `inspect_result` | Enter | Open the selected result in the Inspector |
| `close` | Esc | Close search |

Character keys edit the query, including `j`, `k`, and `q`. Use the arrow keys or `Ctrl-P` / `Ctrl-N` to select results. Search input editing keys are reserved and cannot be assigned to commands.

## Sequence

Context: `sequence`.

| Command | Keys | Action |
| --- | --- | --- |
| `quit` | `q`, Esc | Quit |
| `move_up` | `k`, Up | Select the previous code point |
| `move_down` | `j`, Down | Select the next code point |
| `first` | `g` | Select the first code point |
| `last` | `G` | Select the last code point |
| `activate` | Enter | Inspect the selected code point |
| `normalize` | `n` | Compare normalization forms |

## Normalization

Context: `normalization`.

| Command | Keys | Action |
| --- | --- | --- |
| `quit` | `q` | Quit |
| `close` | Esc | Return to the original sequence |
| `back` | Backspace | Return to the original sequence |
| `move_up` | `k`, Up | Select the previous normalization form |
| `move_down` | `j`, Down | Select the next normalization form |
| `first` | `g` | Select NFC |
| `last` | `G` | Select NFKD |
| `activate` | Enter | Inspect the selected normalization result |
| `copy_value` | `y` | Copy the exact normalized text |

## Normalization Result

Context: `normalization_result`.

| Command | Keys | Action |
| --- | --- | --- |
| `quit` | `q` | Quit |
| `close` | Esc | Return to normalization comparison |
| `back` | Backspace | Return to normalization comparison |
| `move_up` | `k`, Up | Select the previous code point |
| `move_down` | `j`, Down | Select the next code point |
| `first` | `g` | Select the first code point |
| `last` | `G` | Select the last code point |
| `activate` | Enter | Inspect the selected code point |

Use Backspace in the Inspector to return to this result.

## Browse Lists

Contexts: `browse_plane`, `browse_range`, and `browse_block`.

| Command | Keys | Action |
| --- | --- | --- |
| `quit` | `q` | Quit |
| `close` | Esc | Cancel browsing and return to the Inspector |
| `back` | Backspace | Return to the previous browse level, or to the Inspector at the top |
| `move_up` | `k`, Up | Select the previous item |
| `move_down` | `j`, Down | Select the next item |
| `first` | `g` | Select the first item |
| `last` | `G` | Select the last item |
| `activate` | Enter | Open ranges in a plane, or code points in a range or block |

`browse_range` and `browse_block` additionally support:

| Command | Keys | Action |
| --- | --- | --- |
| `page_up` | `Ctrl-U` | Move backward by a large step |
| `page_down` | `Ctrl-D` | Move forward by a large step |

## Browse Code Points

Context: `browse_code_points`.

| Command | Keys | Action |
| --- | --- | --- |
| `quit` | `q` | Quit |
| `close` | Esc | Cancel browsing and return to the Inspector |
| `back` | Backspace | Return to the previous browse level |
| `move_left` | `h`, Left | Move left |
| `move_right` | `l`, Right | Move right |
| `move_up` | `k`, Up | Move up one row |
| `move_down` | `j`, Down | Move down one row |
| `page_up` | `Ctrl-U` | Move backward by a large step |
| `page_down` | `Ctrl-D` | Move forward by a large step |
| `first` | `g` | Select the first code point in the range or block |
| `last` | `G` | Select the last code point in the range or block |
| `activate` | Enter | Inspect the selected code point |

## Help

Context: `help`. The underlying screen's commands are inactive while help is open; global bindings remain active.

| Command | Keys | Action |
| --- | --- | --- |
| `close` | Esc | Close help |
| `move_up` | `k`, Up | Scroll up one line |
| `move_down` | `j`, Down | Scroll down one line |
| `page_up` | `Ctrl-U` | Scroll up one page |
| `page_down` | `Ctrl-D` | Scroll down one page |
| `first` | `g` | Go to the beginning |
| `last` | `G` | Go to the end |
