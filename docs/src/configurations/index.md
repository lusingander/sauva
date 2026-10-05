# Configurations

Sauva uses a TOML configuration file to customize colors, glyph preview fonts, presentation, and keybindings.

## Configuration File Location

The configuration path is selected in this order:

1. `$SAUVA_CONFIG_FILE`, if set.
2. `$XDG_CONFIG_HOME/sauva/config.toml`, if `XDG_CONFIG_HOME` is nonempty.
3. `$HOME/.config/sauva/config.toml`.

Only the selected file is loaded. For example, when `XDG_CONFIG_HOME` is set, Sauva does not look in `$HOME/.config` if that file is missing.

If the default file does not exist, all built-in settings are used. An empty `SAUVA_CONFIG_FILE`, a missing file explicitly selected with it, or an invalid configuration causes startup to fail.

All settings are optional. Unspecified settings keep their built-in defaults. Unknown fields are rejected, so misspelled setting names cause an error instead of being ignored.

## Create a Configuration

Print the complete default configuration:

```sh
sauva --print-default-config
```

To create a file at the default location when `XDG_CONFIG_HOME` is unset:

```sh
mkdir -p ~/.config/sauva
sauva --print-default-config > ~/.config/sauva/config.toml
```

This writes to the specified file; use a new file path if you already have a configuration you want to keep.

You can also use a custom path:

```sh
sauva --print-default-config > ./config.toml
SAUVA_CONFIG_FILE=./config.toml sauva
```

Printing defaults does not load the local configuration or start the TUI, so it also works when an existing configuration is invalid.

----

- [Config File Format](./config-file-format.md): all configuration sections, values, and defaults.
- [Glyph Preview](./glyph-preview.md): font selection and image colors.
- [Custom Keybindings](../keybindings/custom-keybindings.md): command bindings for each screen.
