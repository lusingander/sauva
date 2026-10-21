# Bookmarks

Save code points to revisit them across sessions. In the Inspector, press <kbd>m</kbd> to add the current code point. The header shows `* Bookmarked`, and the footer reports the result.

![Bookmarked code point](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/bookmark-added.png)

## Bookmark List

Press <kbd>M</kbd> in the Inspector to open Bookmarks. Entries show the character, code point, and Unicode name. The selected entry's details include its registration time; narrow terminals show these details below the list.

![Bookmark list](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/bookmarks.png)

| Keys | Action |
| --- | --- |
| <kbd>j</kbd> / <kbd>k</kbd>, <kbd>Down</kbd> / <kbd>Up</kbd> | Select a bookmark |
| <kbd>Ctrl-U</kbd> / <kbd>Ctrl-D</kbd> | Move up / down one page |
| <kbd>g</kbd> / <kbd>G</kbd> | Select the first / last bookmark |
| <kbd>Enter</kbd> | Inspect the selected code point |
| <kbd>m</kbd> | Remove the selected bookmark |
| <kbd>s</kbd> | Switch sort order |
| <kbd>Esc</kbd> | Close Bookmarks and restore the Inspector where it was opened |
| <kbd>q</kbd> | Quit |

Bookmarks can be sorted by registration time or code point, in ascending or descending order.

Press <kbd>Backspace</kbd> in the Inspector to return to the bookmark list. Closing the list restores the original Inspector, including its return path to Sequence when one is open. Keybindings can be changed in the `inspector` and `bookmarks` contexts; see [Custom Keybindings](../keybindings/custom-keybindings.md).

## Registration Time

Times use the local time zone and default to a format such as `2026-10-16 14:32 JST`. Set `ui.bookmark_date_format` to use a different [strftime format](https://docs.rs/jiff/latest/jiff/fmt/strtime/index.html):

```toml
[ui]
bookmark_date_format = "%Y-%m-%d %H:%M:%S %:z"
```

The format must produce nonempty text without control characters. Invalid formats are reported when Sauva starts.

## Storage

Bookmarks are saved in `$XDG_DATA_HOME/sauva/bookmarks.toml`, or `~/.local/share/sauva/bookmarks.toml` when `XDG_DATA_HOME` is unset or empty. This file is separate from the configuration. Adding or removing an entry saves it immediately; a save failure is reported in the footer and leaves the current bookmark state unchanged.

Any code point that Sauva can inspect can be bookmarked, including unassigned code points and surrogates. Removing and adding an entry again gives it a new registration time.
