Set Shell zsh

Output "./img/demo-bookmarks.gif"

Set FontSize 11
Set Width 800
Set Height 400
Set Padding 15
Set WindowBar Colorful
Set Theme "nord"

Hide

Env SAUVA_CONFIG_FILE "./img/config.toml"
Env TZ "Asia/Tokyo"
Env NO_COLOR ""

# Keep the demo's bookmarks separate from personal data.
# Leave this directory in place after recording for inspection.
Type@1ms `sauva_bookmark_demo_dir=$(mktemp -d "${TMPDIR:-/tmp}/sauva-bookmarks.XXXXXX")`
Enter
Type@1ms `export XDG_DATA_HOME="$sauva_bookmark_demo_dir"`
Enter
Type@1ms `mkdir -p "$XDG_DATA_HOME/sauva"`
Enter
Type@1ms `cat > "$XDG_DATA_HOME/sauva/bookmarks.toml" <<'BOOKMARKS'`
Enter
Type@1ms "[[bookmarks]]"
Enter
Type@1ms `code_point = "U+2192"`
Enter
Type@1ms `added_at = "2026-10-15T03:00:00Z"`
Enter
Type@1ms "[[bookmarks]]"
Enter
Type@1ms `code_point = "U+00D7"`
Enter
Type@1ms `added_at = "2026-10-14T03:00:00Z"`
Enter
Type@1ms "[[bookmarks]]"
Enter
Type@1ms `code_point = "U+2212"`
Enter
Type@1ms `added_at = "2026-10-13T03:00:00Z"`
Enter
Type@1ms "[[bookmarks]]"
Enter
Type@1ms `code_point = "U+0301"`
Enter
Type@1ms `added_at = "2026-10-12T03:00:00Z"`
Enter
Type@1ms "[[bookmarks]]"
Enter
Type@1ms `code_point = "U+200B"`
Enter
Type@1ms `added_at = "2026-10-11T03:00:00Z"`
Enter
Type@1ms "BOOKMARKS"
Enter

Type "cargo run --release -- --graphics iterm2 U+0041"
Enter
Sleep 2s

Show

# Inspector: add and remove a bookmark.

Sleep 2s

Type "m"
Sleep 1s

Screenshot "./img/bookmark-added.png"
Sleep 100ms

Type "m"
Sleep 1s

# The list contains only the entries with fixed registration times.

Type "M"
Sleep 1s

Screenshot "./img/bookmarks.png"
Sleep 100ms

Type@400ms "jjjj"
Sleep 500ms

Type "g"
Sleep 500ms

Enter
Sleep 1s

Backspace
Sleep 2s

Hide

Type "q"
Sleep 500ms
