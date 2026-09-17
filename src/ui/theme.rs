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
pub struct InspectorColors {
    #[default(Color::Cyan)]
    pub section_heading: Color,
    #[default(Color::DarkGray)]
    pub field_label: Color,
    #[nested]
    pub selection: SelectionColors,
}

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub struct KeyValueColors {
    #[default(Color::DarkGray)]
    pub label: Color,
}

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub struct ListColors {
    #[nested]
    pub selection: SelectionColors,
}

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub struct CodePointTableColors {
    #[nested]
    pub selection: SelectionColors,
}

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub struct SearchMatchColors {
    #[default(Color::Yellow)]
    pub fg: Color,
    #[default(Color::White)]
    pub selected_fg: Color,
}

impl SearchMatchColors {
    pub fn style(self, selected: bool) -> Style {
        let fg = if selected { self.selected_fg } else { self.fg };
        Style::new().fg(fg).add_modifier(Modifier::BOLD)
    }
}

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub struct SearchMessageColors {
    #[default(Color::Cyan)]
    pub example_label: Color,
    #[default(Color::DarkGray)]
    pub detail: Color,
}

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub struct SelectionPreviewColors {
    #[default(Color::DarkGray)]
    pub empty: Color,
}

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub struct GlyphPreviewColors {
    #[default(Color::DarkGray)]
    pub metadata_label: Color,
    #[default(Color::Yellow)]
    pub status_heading: Color,
    #[default(Color::DarkGray)]
    pub status_detail: Color,
}

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub struct HelpColors {
    #[default(Color::Yellow)]
    pub key: Color,
    #[default(Color::Blue)]
    pub link: Color,
    #[default(Color::DarkGray)]
    pub divider: Color,
}

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)]
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub struct FooterColors {
    #[default(Color::DarkGray)]
    pub short_help: Color,
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
    #[nested]
    pub inspector: InspectorColors,
    #[nested]
    pub key_value: KeyValueColors,
    #[nested]
    pub list: ListColors,
    #[nested]
    pub code_point_table: CodePointTableColors,
    #[nested]
    pub search_match: SearchMatchColors,
    #[nested]
    pub search_message: SearchMessageColors,
    #[nested]
    pub selection_preview: SelectionPreviewColors,
    #[nested]
    pub glyph_preview: GlyphPreviewColors,
    #[nested]
    pub help: HelpColors,
    #[nested]
    pub footer: FooterColors,
}

impl ColorTheme {
    pub fn base_style(self) -> Style {
        Style::new().fg(self.fg).bg(self.bg)
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

                [inspector]
                section_heading = "light-cyan"

                [inspector.selection]
                bg = "red"

                [key_value]
                label = "magenta"

                [list.selection]
                fg = "blue"

                [code_point_table.selection]
                bg = "green"

                [search_match]
                selected_fg = "light-green"

                [search_message]
                example_label = "blue"

                [selection_preview]
                empty = "light-blue"

                [glyph_preview]
                status_detail = "cyan"

                [help]
                link = "#654321"

                [footer]
                warning = "red"
            "##,
        )
        .unwrap();

        let theme: ColorTheme = config.into();

        assert_eq!(theme.fg, Color::Rgb(0x12, 0x34, 0x56));
        assert_eq!(theme.inspector.field_label, Color::DarkGray);
        assert_eq!(theme.inspector.section_heading, Color::LightCyan);
        assert_eq!(theme.inspector.selection.fg, Color::Black);
        assert_eq!(theme.inspector.selection.bg, Color::Red);
        assert_eq!(theme.key_value.label, Color::Magenta);
        assert_eq!(theme.list.selection.fg, Color::Blue);
        assert_eq!(theme.list.selection.bg, Color::Cyan);
        assert_eq!(theme.code_point_table.selection.bg, Color::Green);
        assert_eq!(theme.help.link, Color::Rgb(0x65, 0x43, 0x21));
        assert_eq!(theme.search_match.fg, Color::Yellow);
        assert_eq!(theme.search_match.selected_fg, Color::LightGreen);
        assert_eq!(theme.search_message.example_label, Color::Blue);
        assert_eq!(theme.search_message.detail, Color::DarkGray);
        assert_eq!(theme.selection_preview.empty, Color::LightBlue);
        assert_eq!(theme.glyph_preview.status_detail, Color::Cyan);
        assert_eq!(theme.footer.info, Color::Green);
        assert_eq!(theme.footer.warning, Color::Red);
    }

    #[test]
    fn unknown_color_field_is_rejected() {
        let error = toml::from_str::<OptionalColorTheme>("acccent = \"cyan\"").unwrap_err();

        assert!(error.to_string().contains("unknown field `acccent`"));
    }
}
