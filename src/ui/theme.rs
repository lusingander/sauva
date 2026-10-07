use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize, Serializer};
use smart_default::SmartDefault;
use umbra::optional;

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, SmartDefault)]
pub struct SelectionColors {
    #[default(Color::Black)]
    #[serde(serialize_with = "serialize_color")]
    pub fg: Color,
    #[default(Color::Cyan)]
    #[serde(serialize_with = "serialize_color")]
    pub bg: Color,
}

impl SelectionColors {
    pub fn style(self) -> Style {
        Style::new().fg(self.fg).bg(self.bg)
    }
}

#[optional(derives = [Debug, Deserialize], attrs = [serde(deny_unknown_fields)])]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, SmartDefault)]
pub struct DifferenceColors {
    #[default(Color::Black)]
    #[serde(serialize_with = "serialize_color")]
    pub fg: Color,
    #[default(Color::Yellow)]
    #[serde(serialize_with = "serialize_color")]
    pub bg: Color,
}

impl DifferenceColors {
    pub fn style(self) -> Style {
        Style::new().fg(self.fg).bg(self.bg)
    }
}

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, SmartDefault)]
pub struct StatusColors {
    #[default(Color::Green)]
    #[serde(serialize_with = "serialize_color")]
    pub info: Color,
    #[default(Color::Yellow)]
    #[serde(serialize_with = "serialize_color")]
    pub warning: Color,
}

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)],
    visibility = pub
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, SmartDefault)]
pub struct ColorTheme {
    #[default(Color::Reset)]
    #[serde(serialize_with = "serialize_color")]
    pub fg: Color,
    #[default(Color::Reset)]
    #[serde(serialize_with = "serialize_color")]
    pub bg: Color,
    #[default(Color::DarkGray)]
    #[serde(serialize_with = "serialize_color")]
    pub muted: Color,
    #[default(Color::Cyan)]
    #[serde(serialize_with = "serialize_color")]
    pub accent: Color,
    #[default(Color::Blue)]
    #[serde(serialize_with = "serialize_color")]
    pub heading: Color,
    #[default(Color::DarkGray)]
    #[serde(serialize_with = "serialize_color")]
    pub border: Color,
    #[default(Color::Yellow)]
    #[serde(serialize_with = "serialize_color")]
    pub r#match: Color,
    #[default(Color::Yellow)]
    #[serde(serialize_with = "serialize_color")]
    pub key: Color,
    #[default(Color::Blue)]
    #[serde(serialize_with = "serialize_color")]
    pub link: Color,
    #[nested]
    pub selection: SelectionColors,
    /// Background emphasis for changed normalization spans, independent of the selection colors.
    #[nested]
    pub difference: DifferenceColors,
    #[nested]
    pub status: StatusColors,
}

fn serialize_color<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value = match color {
        Color::Reset => "reset".to_owned(),
        Color::Black => "black".to_owned(),
        Color::Red => "red".to_owned(),
        Color::Green => "green".to_owned(),
        Color::Yellow => "yellow".to_owned(),
        Color::Blue => "blue".to_owned(),
        Color::Magenta => "magenta".to_owned(),
        Color::Cyan => "cyan".to_owned(),
        Color::Gray => "gray".to_owned(),
        Color::DarkGray => "darkgray".to_owned(),
        Color::LightRed => "light-red".to_owned(),
        Color::LightGreen => "light-green".to_owned(),
        Color::LightYellow => "light-yellow".to_owned(),
        Color::LightBlue => "light-blue".to_owned(),
        Color::LightMagenta => "light-magenta".to_owned(),
        Color::LightCyan => "light-cyan".to_owned(),
        Color::White => "white".to_owned(),
        Color::Rgb(red, green, blue) => format!("#{red:02x}{green:02x}{blue:02x}"),
        Color::Indexed(index) => index.to_string(),
    };
    serializer.serialize_str(&value)
}

impl ColorTheme {
    pub fn base_style(self) -> Style {
        Style::new().fg(self.fg).bg(self.bg)
    }

    pub fn border_style(self) -> Style {
        Style::new().fg(self.border)
    }

    pub fn accent_style(self) -> Style {
        Style::new().fg(self.accent).add_modifier(Modifier::BOLD)
    }

    pub fn heading_style(self) -> Style {
        Style::new().fg(self.heading).add_modifier(Modifier::BOLD)
    }

    pub fn match_style(self, selected: bool) -> Style {
        if selected {
            self.selection.style().add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(self.r#match).add_modifier(Modifier::BOLD)
        }
    }

    pub fn difference_style(self) -> Style {
        self.difference.style().add_modifier(Modifier::UNDERLINED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_config_preserves_unspecified_defaults() {
        let config: OptionalColorTheme = toml::from_str(
            r##"
                fg = "#123456"
                muted = "magenta"
                accent = "light-cyan"
                heading = "light-blue"
                border = "blue"
                match = "light-green"
                key = "light-yellow"
                link = "#654321"

                [selection]
                bg = "red"

                [status]
                warning = "light-red"
            "##,
        )
        .unwrap();

        let theme: ColorTheme = config.into();

        assert_eq!(theme.fg, Color::Rgb(0x12, 0x34, 0x56));
        assert_eq!(theme.bg, Color::Reset);
        assert_eq!(theme.muted, Color::Magenta);
        assert_eq!(theme.accent, Color::LightCyan);
        assert_eq!(theme.heading, Color::LightBlue);
        assert_eq!(theme.border, Color::Blue);
        assert_eq!(theme.r#match, Color::LightGreen);
        assert_eq!(theme.key, Color::LightYellow);
        assert_eq!(theme.link, Color::Rgb(0x65, 0x43, 0x21));
        assert_eq!(theme.selection.fg, Color::Black);
        assert_eq!(theme.selection.bg, Color::Red);
        assert_eq!(theme.status.info, Color::Green);
        assert_eq!(theme.status.warning, Color::LightRed);
    }

    #[test]
    fn selected_match_inherits_the_selection_colors() {
        let theme = ColorTheme {
            selection: SelectionColors {
                fg: Color::White,
                bg: Color::Blue,
            },
            ..ColorTheme::default()
        };

        assert_eq!(
            theme.match_style(true),
            theme.selection.style().add_modifier(Modifier::BOLD)
        );
    }

    #[test]
    fn difference_colors_keep_their_own_defaults_when_partially_configured() {
        let config: OptionalColorTheme = toml::from_str("[difference]\nfg = \"white\"").unwrap();
        let theme: ColorTheme = config.into();
        assert_eq!(theme.difference.fg, Color::White);
        assert_eq!(theme.difference.bg, Color::Yellow);
        assert_eq!(theme.selection.bg, Color::Cyan);
    }

    #[test]
    fn unknown_color_field_is_rejected() {
        let error = toml::from_str::<OptionalColorTheme>("acccent = \"cyan\"").unwrap_err();

        assert!(error.to_string().contains("unknown field `acccent`"));
    }
}
