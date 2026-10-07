use std::{
    env,
    error::Error,
    ffi::OsString,
    fmt, fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use garde::Validate;
use serde::{Deserialize, Serialize};
use umbra::optional;

use crate::{
    glyph::settings::{GlyphPreviewSettings, OptionalGlyphPreviewSettings},
    keybindings::{KeybindingError, Keybindings, OptionalKeybindings, ResolvedKeymap},
    ui::{
        settings::{OptionalUiSettings, UiSettings},
        theme::{ColorTheme, OptionalColorTheme},
    },
};

const CONFIG_FILE_ENV_NAME: &str = "SAUVA_CONFIG_FILE";
const XDG_CONFIG_HOME_ENV_NAME: &str = "XDG_CONFIG_HOME";
const HOME_ENV_NAME: &str = "HOME";
const APP_DIR_NAME: &str = "sauva";
const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug)]
pub struct RuntimeConfig {
    color_theme: ColorTheme,
    glyph_preview: GlyphPreviewSettings,
    ui: UiSettings,
    keymap: ResolvedKeymap,
}

impl RuntimeConfig {
    pub const fn color_theme(&self) -> &ColorTheme {
        &self.color_theme
    }

    pub const fn glyph_preview(&self) -> &GlyphPreviewSettings {
        &self.glyph_preview
    }

    pub const fn ui(&self) -> &UiSettings {
        &self.ui
    }

    pub const fn keymap(&self) -> &ResolvedKeymap {
        &self.keymap
    }
}

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)]
)]
#[derive(Debug, Default, Serialize, Validate)]
struct Config {
    #[garde(skip)]
    #[nested]
    color: ColorTheme,
    #[garde(dive)]
    #[nested]
    glyph_preview: GlyphPreviewSettings,
    #[garde(dive)]
    #[nested]
    ui: UiSettings,
    #[garde(dive)]
    #[nested]
    keybindings: Keybindings,
}

impl Config {
    fn resolve(self) -> Result<RuntimeConfig, KeybindingError> {
        let keymap = ResolvedKeymap::with_config(self.keybindings)?;
        Ok(RuntimeConfig {
            color_theme: self.color,
            glyph_preview: self.glyph_preview,
            ui: self.ui,
            keymap,
        })
    }
}

fn default_runtime_config() -> RuntimeConfig {
    Config::default()
        .resolve()
        .expect("built-in configuration must be valid")
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ConfigFile {
    Explicit(PathBuf),
    Default(PathBuf),
}

impl ConfigFile {
    fn path(&self) -> &Path {
        match self {
            Self::Explicit(path) | Self::Default(path) => path,
        }
    }

    const fn is_explicit(&self) -> bool {
        matches!(self, Self::Explicit(_))
    }
}

#[derive(Debug)]
pub enum ConfigError {
    EmptyExplicitPath,
    ExplicitFileNotFound(PathBuf),
    Read {
        path: PathBuf,
        source: io::Error,
    },
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    Validate {
        path: PathBuf,
        source: garde::Report,
    },
    Keybindings {
        path: PathBuf,
        source: KeybindingError,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyExplicitPath => {
                write!(formatter, "${CONFIG_FILE_ENV_NAME} must not be empty")
            }
            Self::ExplicitFileNotFound(path) => write!(
                formatter,
                "config file specified by ${CONFIG_FILE_ENV_NAME} was not found: {}",
                path.display()
            ),
            Self::Read { path, source } => {
                write!(
                    formatter,
                    "failed to read config file {}: {source}",
                    path.display()
                )
            }
            Self::Parse { path, source } => {
                write!(
                    formatter,
                    "failed to parse config file {}: {source}",
                    path.display()
                )
            }
            Self::Validate { path, source } => write!(
                formatter,
                "invalid config file {}: {source}",
                path.display()
            ),
            Self::Keybindings { path, source } => write!(
                formatter,
                "invalid config file {}: {source}",
                path.display()
            ),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            Self::Validate { source, .. } => Some(source),
            Self::Keybindings { source, .. } => Some(source),
            Self::EmptyExplicitPath | Self::ExplicitFileNotFound(_) => None,
        }
    }
}

pub fn load() -> Result<RuntimeConfig, ConfigError> {
    let config_file = resolve_config_file(
        env::var_os(CONFIG_FILE_ENV_NAME),
        env::var_os(XDG_CONFIG_HOME_ENV_NAME),
        env::var_os(HOME_ENV_NAME),
    )?;
    load_config(config_file.as_ref())
}

pub fn default_toml() -> Result<String, toml::ser::Error> {
    toml::to_string(&Config::default())
}

fn resolve_config_file(
    explicit_path: Option<OsString>,
    xdg_config_home: Option<OsString>,
    home: Option<OsString>,
) -> Result<Option<ConfigFile>, ConfigError> {
    if let Some(path) = explicit_path {
        if path.is_empty() {
            return Err(ConfigError::EmptyExplicitPath);
        }
        return Ok(Some(ConfigFile::Explicit(PathBuf::from(path))));
    }

    let config_home = xdg_config_home
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            home.filter(|value| !value.is_empty())
                .map(PathBuf::from)
                .map(|path| path.join(".config"))
        });

    Ok(config_home.map(|path| ConfigFile::Default(path.join(APP_DIR_NAME).join(CONFIG_FILE_NAME))))
}

fn load_config(config_file: Option<&ConfigFile>) -> Result<RuntimeConfig, ConfigError> {
    let Some(config_file) = config_file else {
        return Ok(default_runtime_config());
    };
    let path = config_file.path();
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(source) if source.kind() == ErrorKind::NotFound && !config_file.is_explicit() => {
            return Ok(default_runtime_config());
        }
        Err(source) if source.kind() == ErrorKind::NotFound => {
            return Err(ConfigError::ExplicitFileNotFound(path.to_owned()));
        }
        Err(source) => {
            return Err(ConfigError::Read {
                path: path.to_owned(),
                source,
            });
        }
    };
    let config =
        toml::from_str::<OptionalConfig>(&content).map_err(|source| ConfigError::Parse {
            path: path.to_owned(),
            source,
        })?;
    let config: Config = config.into();
    config.validate().map_err(|source| ConfigError::Validate {
        path: path.to_owned(),
        source,
    })?;
    config.resolve().map_err(|source| ConfigError::Keybindings {
        path: path.to_owned(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};
    use ratatui::style::Color;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn explicit_path_takes_precedence() {
        let path = resolve_config_file(
            Some("custom.toml".into()),
            Some("/xdg".into()),
            Some("/home/user".into()),
        )
        .unwrap();

        assert_eq!(path, Some(ConfigFile::Explicit("custom.toml".into())));
    }

    #[test]
    fn xdg_path_takes_precedence_over_home() {
        let path =
            resolve_config_file(None, Some("/xdg".into()), Some("/home/user".into())).unwrap();

        assert_eq!(
            path,
            Some(ConfigFile::Default("/xdg/sauva/config.toml".into()))
        );
    }

    #[test]
    fn home_path_is_used_without_xdg() {
        let path = resolve_config_file(None, None, Some("/home/user".into())).unwrap();

        assert_eq!(
            path,
            Some(ConfigFile::Default(
                "/home/user/.config/sauva/config.toml".into()
            ))
        );
    }

    #[test]
    fn empty_explicit_path_is_rejected() {
        let error = resolve_config_file(Some(OsString::new()), None, None).unwrap_err();

        assert!(matches!(error, ConfigError::EmptyExplicitPath));
    }

    #[test]
    fn generated_default_config_is_complete_and_loadable() {
        let generated = default_toml().unwrap();

        for table in [
            "[color]",
            "[color.selection]",
            "[color.difference]",
            "[color.status]",
            "[glyph_preview]",
            "[ui]",
            "[keybindings.global]",
            "[keybindings.inspector]",
            "[keybindings.search]",
            "[keybindings.sequence]",
            "[keybindings.normalization]",
            "[keybindings.normalization_result]",
            "[keybindings.browse_plane]",
            "[keybindings.browse_range]",
            "[keybindings.browse_block]",
            "[keybindings.browse_code_points]",
            "[keybindings.help]",
        ] {
            assert!(generated.contains(table), "missing {table}");
        }
        assert!(generated.contains("fg = \"#f5f7fa\""));
        assert!(generated.contains("input_cursor = \"native\""));

        let parsed: OptionalConfig = toml::from_str(&generated).unwrap();
        let parsed: Config = parsed.into();
        parsed.validate().unwrap();
        assert_eq!(toml::to_string(&parsed).unwrap(), generated);
        parsed.resolve().unwrap();
    }

    #[test]
    fn missing_default_file_uses_defaults() {
        let directory = tempdir().unwrap();
        let file = ConfigFile::Default(directory.path().join("missing.toml"));

        let config = load_config(Some(&file)).unwrap();

        assert_eq!(config.color_theme(), &ColorTheme::default());
    }

    #[test]
    fn missing_explicit_file_is_rejected() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("missing.toml");
        let file = ConfigFile::Explicit(path.clone());

        let error = load_config(Some(&file)).unwrap_err();

        assert!(matches!(error, ConfigError::ExplicitFileNotFound(value) if value == path));
    }

    #[test]
    fn partial_color_config_is_loaded() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(
            &path,
            r##"
                [color]
                fg = "#123456"
                muted = "light-cyan"
                accent = "light-magenta"
                heading = "light-blue"
                border = "white"
                match = "yellow"
                key = "light-yellow"
                link = "light-blue"

                [color.status]
                info = "light-green"

                [color.selection]
                fg = "white"
                bg = "blue"
            "##,
        )
        .unwrap();

        let config = load_config(Some(&ConfigFile::Explicit(path))).unwrap();

        assert_eq!(config.color_theme().fg, Color::Rgb(0x12, 0x34, 0x56));
        assert_eq!(config.color_theme().bg, Color::Reset);
        assert_eq!(config.color_theme().muted, Color::LightCyan);
        assert_eq!(config.color_theme().accent, Color::LightMagenta);
        assert_eq!(config.color_theme().heading, Color::LightBlue);
        assert_eq!(config.color_theme().border, Color::White);
        assert_eq!(config.color_theme().r#match, Color::Yellow);
        assert_eq!(config.color_theme().key, Color::LightYellow);
        assert_eq!(config.color_theme().link, Color::LightBlue);
        assert_eq!(config.color_theme().status.info, Color::LightGreen);
        assert_eq!(config.color_theme().status.warning, Color::Yellow);
        assert_eq!(config.color_theme().selection.fg, Color::White);
        assert_eq!(config.color_theme().selection.bg, Color::Blue);
    }

    #[test]
    fn glyph_preview_colors_are_loaded() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(
            &path,
            r##"
                [glyph_preview]
                fg = "#10203040"
                bg = "#506070"
            "##,
        )
        .unwrap();

        let configured = load_config(Some(&ConfigFile::Explicit(path))).unwrap();

        assert_eq!(
            configured.glyph_preview(),
            &GlyphPreviewSettings {
                fg: crate::glyph::settings::RgbaColor::rgba(0x10, 0x20, 0x30, 0x40),
                bg: crate::glyph::settings::RgbaColor::rgb(0x50, 0x60, 0x70),
                ..GlyphPreviewSettings::default()
            }
        );
    }

    #[test]
    fn glyph_preview_font_families_are_loaded_in_order() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(
            &path,
            r#"
                [glyph_preview]
                font_families = ["Iosevka", "Noto Sans"]
                emoji_font_families = []
            "#,
        )
        .unwrap();

        let configured = load_config(Some(&ConfigFile::Explicit(path))).unwrap();

        assert_eq!(
            configured.glyph_preview().font_families,
            ["Iosevka", "Noto Sans"]
        );
        assert!(configured.glyph_preview().emoji_font_families.is_empty());
    }

    #[test]
    fn invalid_glyph_preview_color_includes_the_config_path() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(
            &path,
            r#"
                [glyph_preview]
                bg = "transparent"
            "#,
        )
        .unwrap();

        let error = load_config(Some(&ConfigFile::Explicit(path.clone()))).unwrap_err();
        let message = error.to_string();

        assert!(matches!(error, ConfigError::Parse { .. }));
        assert!(message.contains(&path.display().to_string()));
        assert!(message.contains("expected #RRGGBB or #RRGGBBAA"));
    }

    #[test]
    fn empty_ui_config_uses_defaults() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(
            &path,
            r#"
                [ui]
            "#,
        )
        .unwrap();

        let configured = load_config(Some(&ConfigFile::Explicit(path))).unwrap();
        let defaults = default_runtime_config();

        assert_eq!(
            configured.ui().input_cursor,
            crate::ui::settings::InputCursor::Native
        );
        assert_eq!(
            defaults.ui().input_cursor,
            crate::ui::settings::InputCursor::Native
        );
    }

    #[test]
    fn text_input_cursor_is_loaded_from_config() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(
            &path,
            r#"
                [ui]
                input_cursor = { text = "|" }
            "#,
        )
        .unwrap();

        let configured = load_config(Some(&ConfigFile::Explicit(path.clone()))).unwrap();
        assert_eq!(
            configured.ui().input_cursor,
            crate::ui::settings::InputCursor::Text("|".to_owned())
        );

        fs::write(
            &path,
            r#"
                [ui]
                input_cursor = "native"
            "#,
        )
        .unwrap();
        let configured = load_config(Some(&ConfigFile::Explicit(path))).unwrap();
        assert_eq!(
            configured.ui().input_cursor,
            crate::ui::settings::InputCursor::Native
        );
    }

    #[test]
    fn invalid_text_input_cursor_reports_its_setting() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(
            &path,
            r#"
                [ui]
                input_cursor = { text = "界" }
            "#,
        )
        .unwrap();

        let error = load_config(Some(&ConfigFile::Explicit(path.clone()))).unwrap_err();
        let message = error.to_string();
        assert!(matches!(error, ConfigError::Validate { .. }));
        assert!(message.contains(&path.display().to_string()));
        assert!(message.contains("ui.input_cursor"));
    }

    #[test]
    fn unknown_top_level_field_is_rejected_with_the_path() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(&path, "colour = {}\n").unwrap();

        let error = load_config(Some(&ConfigFile::Explicit(path.clone()))).unwrap_err();
        let message = error.to_string();

        assert!(message.contains(&path.display().to_string()));
        assert!(message.contains("unknown field `colour`"));
    }

    #[test]
    fn partial_keybinding_config_is_loaded() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(
            &path,
            r#"
                [keybindings.inspector]
                next_code_point = ["right", "n"]
                search = []
            "#,
        )
        .unwrap();

        let config = load_config(Some(&ConfigFile::Explicit(path))).unwrap();
        let keymap = config.keymap();

        assert_eq!(
            keymap.resolve(
                crate::keybindings::Context::Inspector,
                crate::keybindings::KeyChord::new(KeyCode::Char('n'), KeyModifiers::NONE)
            ),
            Some(crate::keybindings::Command::NextCodePoint)
        );
        assert!(
            keymap
                .keys_for(
                    crate::keybindings::Context::Inspector,
                    crate::keybindings::Command::OpenSearch,
                )
                .is_empty()
        );
    }

    #[test]
    fn keybinding_errors_include_the_config_path_and_setting() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(
            &path,
            r#"
                [keybindings.inspector]
                next_code_point = ["h"]
            "#,
        )
        .unwrap();

        let error = load_config(Some(&ConfigFile::Explicit(path.clone()))).unwrap_err();
        let message = error.to_string();

        assert!(message.contains(&path.display().to_string()));
        assert!(message.contains("keybindings.inspector.next_code_point"));
        assert!(message.contains("keybindings.inspector.previous_code_point"));
    }

    #[test]
    fn structurally_invalid_keybindings_are_rejected_before_resolution() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(
            &path,
            r#"
                [keybindings.inspector]
                next_code_point = [""]
            "#,
        )
        .unwrap();

        let error = load_config(Some(&ConfigFile::Explicit(path))).unwrap_err();
        let message = error.to_string();

        assert!(matches!(error, ConfigError::Validate { .. }));
        assert!(message.contains("keybindings.inspector.next_code_point"));
        assert!(message.contains("invalid key ``"));
    }
}
