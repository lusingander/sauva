# Bookmarks

Save code points to revisit them across sessions. In the Inspector, press <kbd>m</kbd> to add the current code point.

![Bookmarked code point](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/bookmark-added.png)

## Bookmark List

Press <kbd>M</kbd> in the Inspector to open Bookmarks.

![Bookmark list](https://raw.githubusercontent.com/lusingander/sauva/refs/heads/master/img/bookmarks.png)

| Keys | Action |
| --- | --- |
| <kbd>j</kbd> / <kbd>k</kbd>, <kbd>Down</kbd> / <kbd>Up</kbd> | Select a bookmark |
| <kbd>Enter</kbd> | Inspect the selected code point |
| <kbd>m</kbd> | Remove the selected bookmark |
| <kbd>s</kbd> | Switch sort order |
| <kbd>Esc</kbd> | Close Bookmarks and return to the Inspector |
| <kbd>q</kbd> | Quit |

Bookmarks can be sorted by registration time or code point, in ascending or descending order.

Press <kbd>Backspace</kbd> in the Inspector to return to the bookmark list. See [Keybindings](../keybindings/index.md#bookmarks) for all Bookmarks controls.

## Registration Time

Customize the display of registration times with [`ui.bookmark_date_format`](../configurations/config-file-format.md#bookmark_date_format).

## Storage

Bookmarks are saved in `$XDG_DATA_HOME/sauva/bookmarks.toml`, or `~/.local/share/sauva/bookmarks.toml` when `XDG_DATA_HOME` is unset or empty.

Any code point that Sauva can inspect can be bookmarked, including unassigned code points and surrogates.
