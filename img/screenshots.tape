# Run cargo build --release before recording.

Set Shell zsh

Set FontSize 11
Set Width 800
Set Height 450
Set Padding 15
Set WindowBar Colorful
Set Theme "nord"

Hide

Env SAUVA_CONFIG_FILE "./img/config.toml"
Env TZ "Asia/Tokyo"
Env NO_COLOR ""

# Keep the screenshots' bookmarks separate from personal data.
# Leave this directory in place after recording for inspection.
Type@1ms `sauva_bookmark_demo_dir=$(mktemp -d "${TMPDIR:-/tmp}/sauva-bookmarks.XXXXXX")`
Enter
Type@1ms `export XDG_DATA_HOME="$sauva_bookmark_demo_dir"`
Enter
Type@1ms `mkdir -p "$XDG_DATA_HOME/sauva"`
Enter
Type@1ms `cp ./img/bookmarks.toml "$XDG_DATA_HOME/sauva/bookmarks.toml"`
Enter

Type@1ms "./target/release/sauva --graphics iterm2 U+0042"
Enter
Sleep 1s

Show

# Inspector: bookmark B.

Type "m"
Sleep 300ms

Screenshot "./img/bookmark-added.png"
Sleep 100ms

# Select a seeded bookmark so the screenshot has a fixed registration time.

Type@1ms "Mj"
Sleep 300ms

Screenshot "./img/bookmarks.png"
Sleep 100ms

Hide

Escape
Sleep 100ms

Type "q"
Sleep 100ms

# Inspector copy: show a single-code-point Rust string literal.

Type@1ms "./target/release/sauva --graphics iterm2 U+1F600"
Enter
Sleep 1s

Show

Type "Y"
Right
Right
Sleep 300ms

Screenshot "./img/inspector-copy.png"
Sleep 100ms

Hide

Escape
Sleep 100ms
Type "q"
Sleep 100ms

# Property explanation: show every East Asian Width value and its reference.

Type@1ms "./target/release/sauva --graphics iterm2 U+00E9"
Enter
Sleep 1s

Show

Type@1ms "]jjje"
Sleep 300ms

Screenshot "./img/property-explanation.png"
Sleep 100ms

Hide

Type "q"
Sleep 100ms
