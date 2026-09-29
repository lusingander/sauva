use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use smart_default::SmartDefault;
use umbra::optional;

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub struct SelectionColors {
    #[default(Color::Black)]
    pub fg: Color,
    #[default(Color::Cyan)]
    pub bg: Color,
}

impl SelectionColors {
    pub fn style(self) -> Style {
        Style::new().fg(self.fg).bg(self.bg)
    }
}

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub struct StatusColors {
    #[default(Color::Green)]
    pub info: Color,
    #[default(Color::Yellow)]
    pub warning: Color,
}

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)],
    visibility = pub
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub struct ColorTheme {
    #[default(Color::Reset)]
    pub fg: Color,
    #[default(Color::Reset)]
    pub bg: Color,
    #[default(Color::DarkGray)]
    pub muted: Color,
    #[default(Color::Cyan)]
    pub accent: Color,
    #[default(Color::DarkGray)]
    pub border: Color,
    #[default(Color::Yellow)]
    pub r#match: Color,
    #[default(Color::Yellow)]
    pub key: Color,
    #[default(Color::Blue)]
    pub link: Color,
    #[nested]
    pub selection: SelectionColors,
    #[nested]
    pub status: StatusColors,
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

    pub fn match_style(self, selected: bool) -> Style {
        if selected {
            self.selection.style().add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(self.r#match).add_modifier(Modifier::BOLD)
        }
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
    fn unknown_color_field_is_rejected() {
        let error = toml::from_str::<OptionalColorTheme>("acccent = \"cyan\"").unwrap_err();

        assert!(error.to_string().contains("unknown field `acccent`"));
    }
}
