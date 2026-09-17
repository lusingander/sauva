Set Shell zsh

Output "./img/demo.gif"

Set FontSize 11
Set Width 800
Set Height 400
Set Padding 15
Set WindowBar Colorful
Set Theme "nord"

Hide

Env SAUVA_CONFIG_FILE "./img/config.toml"
Type "cargo run --release -- --graphics iterm2"
Enter
Sleep 2s

Show

# Inspector

Sleep 2s
Screenshot "./img/inspector.png"
Sleep 100ms
Type@200ms "jjjjjjj"

# Browse

Sleep 500ms
Type "p"

Sleep 1s
Screenshot "./img/plane.png"
Sleep 100ms
Enter

Sleep 500ms
Screenshot "./img/range.png"
Sleep 100ms

Enter

Sleep 1s
Type@200ms "llllll"
Sleep 200ms
Type@200ms "jjjjjj"
Sleep 500ms

Screenshot "./img/code-points.png"
Sleep 100ms

Enter

# Inspector

Sleep 2s

# Filter

Type "/"
Sleep 500ms

Type@300ms "smil"
Sleep 500ms
Ctrl+N
Sleep 300ms
Ctrl+N
Sleep 300ms
Ctrl+N
Sleep 300ms
Ctrl+N
Sleep 300ms

Screenshot "./img/filter.png"
Sleep 100ms

Sleep 500ms
Ctrl+W

Sleep 1s
Type@300ms "hira"

Sleep 1s
Ctrl+N
Sleep 300ms
Ctrl+N
Sleep 300ms
Ctrl+N

Sleep 500ms
Enter

# Inspector

Sleep 1s

Type "g"
Sleep 500ms

Type@300ms "llll"
Sleep 500ms

Type@200ms "jj"

Sleep 500ms
Type "y"
Sleep 1s

# Filter

Type "/"
Sleep 500ms
Ctrl+W
Sleep 500ms

Type "A"
Sleep 1s
Enter

# Inspector

Sleep 1s
