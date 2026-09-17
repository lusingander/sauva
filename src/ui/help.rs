use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Padding, Paragraph},
};

use crate::{
    app::AppState,
    input::context_for_state,
    keybindings::{Command, Context, KeyLabelStyle, ResolvedKeymap, format_key},
    ui::{
        layout,
        scrollbar::{self, ViewportScrollbar},
        theme::ColorTheme,
    },
};

const KEY_COLUMN_WIDTH: usize = 28;
const ABOUT_HEIGHT: u16 = 7;
const DIVIDER_HEIGHT: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PackageMetadata {
    name: &'static str,
    description: &'static str,
    version: &'static str,
    repository: &'static str,
}

const CARGO_PACKAGE: PackageMetadata = PackageMetadata {
    name: env!("CARGO_PKG_NAME"),
    description: env!("CARGO_PKG_DESCRIPTION"),
    version: env!("CARGO_PKG_VERSION"),
    repository: env!("CARGO_PKG_REPOSITORY"),
};

#[cfg(test)]
const FIXTURE_PACKAGE: PackageMetadata = PackageMetadata {
    name: "sauva",
    description: "Terminal Unicode Explorer",
    version: "1.2.3",
    repository: "https://example.com/sauva",
};

#[cfg(not(test))]
const DISPLAY_PACKAGE: PackageMetadata = CARGO_PACKAGE;
#[cfg(test)]
const DISPLAY_PACKAGE: PackageMetadata = FIXTURE_PACKAGE;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ViewportMetrics {
    pub viewport_height: usize,
    pub document_height: usize,
}

#[derive(Debug, Clone, Copy)]
struct HelpItem {
    command: Command,
    description: &'static str,
}

#[derive(Debug, Clone)]
struct ShortHelpItem {
    commands: Vec<Command>,
    description: &'static str,
    priority: u8,
}

#[derive(Debug, Clone)]
struct RenderedShortHelpItem {
    text: String,
    priority: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HelpSections {
    about: Rect,
    divider: Rect,
    help: Rect,
}

pub fn viewport_metrics(area: Rect, state: &AppState, keymap: &ResolvedKeymap) -> ViewportMetrics {
    let Some(layout) = layout::calculate(area) else {
        return ViewportMetrics::default();
    };
    let sections = help_sections(layout.main);
    let content = section_content_area(sections.help);
    let context = context_for_state(state);
    let document = document(
        context,
        keymap,
        Style::new(),
        Style::new(),
        usize::from(content.width),
    );
    let document_height = document.lines.len();

    ViewportMetrics {
        viewport_height: usize::from(content.height),
        document_height,
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    keymap: &ResolvedKeymap,
    color_theme: &ColorTheme,
) {
    let context = context_for_state(state);
    let title = format!(" Help · {} ", context_label(context));
    let block = Block::bordered()
        .title(title)
        .padding(Padding::horizontal(1));
    let sections = help_sections(area);
    let help_block = section_block();
    let help_content = help_block.inner(sections.help);
    let document = document(
        context,
        keymap,
        Style::new().fg(color_theme.fg),
        Style::new().fg(color_theme.help.key),
        usize::from(help_content.width),
    );
    let document_height = document.lines.len();
    let visible = state.help().visible_range();
    let start = visible.start.min(document_height);
    let end = visible.end.max(start).min(document_height);

    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(about_document(
            DISPLAY_PACKAGE,
            Style::new().fg(color_theme.fg),
            Style::new().fg(color_theme.help.link),
        ))
        .block(section_block()),
        sections.about,
    );
    frame.render_widget(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::new().fg(color_theme.help.divider)),
        sections.divider,
    );
    frame.render_widget(
        Paragraph::new(document)
            .block(help_block)
            .scroll((u16::try_from(start).unwrap_or(u16::MAX), 0)),
        sections.help,
    );
    frame.render_widget(
        ViewportScrollbar::new(document_height, start..end).style(color_theme.base_style()),
        scrollbar::area_after(help_content),
    );
}

fn about_document(
    package: PackageMetadata,
    normal_style: Style,
    link_style: Style,
) -> Text<'static> {
    Text::from(vec![
        Line::from(Span::styled(
            format!("{} - {}", package.name, package.description),
            normal_style,
        )),
        Line::default(),
        Line::from(Span::styled(
            format!("Version: {}", package.version),
            normal_style,
        )),
        Line::default(),
        Line::from(Span::styled(package.repository, link_style)),
    ])
}

pub fn footer(context: Context, width: u16, keymap: &ResolvedKeymap) -> String {
    let mut items = short_help_items(context)
        .into_iter()
        .filter_map(|item| render_short_help_item(context, item, keymap))
        .collect::<Vec<_>>();

    while footer_width(&items) > usize::from(width) {
        let Some(index) = items
            .iter()
            .enumerate()
            .max_by_key(|(index, item)| (item.priority, *index))
            .map(|(index, _)| index)
        else {
            break;
        };
        items.remove(index);
    }

    if items.is_empty() {
        String::new()
    } else {
        format!(
            " {} ",
            items
                .iter()
                .map(|item| item.text.as_str())
                .collect::<Vec<_>>()
                .join("  ")
        )
    }
}

fn document(
    context: Context,
    keymap: &ResolvedKeymap,
    normal_style: Style,
    key_style: Style,
    width: usize,
) -> Text<'static> {
    let mut lines = help_items(context)
        .into_iter()
        .filter_map(|item| help_lines(context, item, keymap, normal_style, key_style, width))
        .flatten()
        .collect::<Vec<_>>();

    if context == Context::Search {
        lines.push(Line::default());
        lines.extend(wrap_plain_text(
            "Type normally to edit the query; command keys take precedence.",
            width,
        ));
    }

    Text::from(lines)
}

fn help_lines(
    context: Context,
    item: HelpItem,
    keymap: &ResolvedKeymap,
    normal_style: Style,
    key_style: Style,
    width: usize,
) -> Option<Vec<Line<'static>>> {
    let keys = keymap.keys_for_active(context, item.command);
    if keys.is_empty() {
        return None;
    }
    let labels = keys
        .into_iter()
        .map(|key| format_key(key, KeyLabelStyle::Full))
        .collect::<Vec<_>>();
    let key_text = format!("<{}>", labels.join(">  <"));
    let key_width = Line::from(key_text.as_str()).width();
    let key_column_width = KEY_COLUMN_WIDTH.max(key_width);
    if width <= key_column_width + 1 {
        let mut lines = vec![Line::from(key_spans(&labels, normal_style, key_style))];
        lines.extend(wrap_plain_text(item.description, width));
        return Some(lines);
    }

    let description_width = width - key_column_width - 1;
    let mut description_lines = wrap_words(item.description, description_width).into_iter();
    let first_description = description_lines.next().unwrap_or_default();
    let mut first_line = key_spans(&labels, normal_style, key_style);
    first_line.push(Span::styled(
        " ".repeat(key_column_width - key_width + 1),
        normal_style,
    ));
    first_line.push(Span::styled(first_description, normal_style));
    let mut lines = vec![Line::from(first_line)];
    lines.extend(description_lines.map(|description| {
        Line::from(Span::styled(
            format!("{}{}", " ".repeat(key_column_width + 1), description),
            normal_style,
        ))
    }));
    Some(lines)
}

fn key_spans(labels: &[String], normal_style: Style, key_style: Style) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (index, label) in labels.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled("  ", normal_style));
        }
        spans.push(Span::styled("<", normal_style));
        spans.push(Span::styled(label.clone(), key_style));
        spans.push(Span::styled(">", normal_style));
    }
    spans
}

fn wrap_plain_text(text: &str, width: usize) -> Vec<Line<'static>> {
    wrap_words(text, width).into_iter().map(Line::raw).collect()
}

fn wrap_words(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_owned()
        } else {
            format!("{current} {word}")
        };
        if Line::from(candidate.as_str()).width() <= width {
            current = candidate;
        } else {
            if !current.is_empty() {
                lines.push(current);
            }
            current = word.to_owned();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn help_items(context: Context) -> Vec<HelpItem> {
    use Command as C;

    let mut items = match context {
        Context::Inspector => vec![
            item(C::PreviousCodePoint, "Previous code point"),
            item(C::NextCodePoint, "Next code point"),
            item(C::MoveUp, "Select the previous property"),
            item(C::MoveDown, "Select the next property"),
            item(C::PageUp, "Move properties up one page"),
            item(C::PageDown, "Move properties down one page"),
            item(C::First, "Select the first property"),
            item(C::Last, "Select the last property"),
            item(C::CopyValue, "Copy the selected property value"),
            item(C::OpenSearch, "Open search"),
            item(C::BrowsePlanes, "Browse planes"),
            item(C::BrowseRanges, "Browse ranges"),
            item(C::BrowseCodePoints, "Browse code points"),
            item(C::Quit, "Quit"),
        ],
        Context::Search => vec![
            item(C::PreviousResult, "Select the previous result"),
            item(C::NextResult, "Select the next result"),
            item(C::InspectResult, "Inspect the selected result"),
            item(C::Close, "Close search"),
            item(C::Quit, "Quit"),
        ],
        Context::BrowsePlane => vec![
            item(C::MoveUp, "Select the previous plane"),
            item(C::MoveDown, "Select the next plane"),
            item(C::First, "Select the first plane"),
            item(C::Last, "Select the last plane"),
            item(C::Activate, "Open ranges in the selected plane"),
            item(C::Close, "Close browse"),
            item(C::Quit, "Quit"),
        ],
        Context::BrowseRange => vec![
            item(C::MoveUp, "Select the previous range"),
            item(C::MoveDown, "Select the next range"),
            item(C::PageUp, "Move backward by a large step"),
            item(C::PageDown, "Move forward by a large step"),
            item(C::First, "Select the first range"),
            item(C::Last, "Select the last range"),
            item(C::Activate, "Open code points in the selected range"),
            item(C::Back, "Return to planes"),
            item(C::Close, "Close browse"),
            item(C::Quit, "Quit"),
        ],
        Context::BrowseCodePoints => vec![
            item(C::MoveLeft, "Move to the code point on the left"),
            item(C::MoveRight, "Move to the code point on the right"),
            item(C::MoveUp, "Move to the row above"),
            item(C::MoveDown, "Move to the row below"),
            item(C::PageUp, "Move backward by a large step"),
            item(C::PageDown, "Move forward by a large step"),
            item(C::First, "Move to the first code point in the range"),
            item(C::Last, "Move to the last code point in the range"),
            item(C::Activate, "Inspect the selected code point"),
            item(C::Back, "Return to ranges"),
            item(C::Close, "Close browse"),
            item(C::Quit, "Quit"),
        ],
        Context::Help => vec![
            item(C::MoveUp, "Move help up one line"),
            item(C::MoveDown, "Move help down one line"),
            item(C::PageUp, "Move help up one page"),
            item(C::PageDown, "Move help down one page"),
            item(C::First, "Move to the start of help"),
            item(C::Last, "Move to the end of help"),
            item(C::Close, "Close help"),
            item(C::Quit, "Quit"),
        ],
        Context::Global => vec![item(C::Quit, "Quit")],
    };
    items.push(item(C::Help, "Open or close help"));
    items
}

fn item(command: Command, description: &'static str) -> HelpItem {
    HelpItem {
        command,
        description,
    }
}

#[rustfmt::skip]
fn short_help_items(context: Context) -> Vec<ShortHelpItem> {
    use Command as C;

    match context {
        Context::Inspector => vec![
            short(&[C::PreviousCodePoint, C::NextCodePoint], "Point", 1),
            short(&[C::MoveUp, C::MoveDown], "Field", 1),
            short(&[C::PageUp, C::PageDown], "Page", 3),
            short(&[C::First, C::Last], "Ends", 3),
            short(&[C::CopyValue], "Copy", 1),
            short(&[C::OpenSearch], "Search", 2),
            short(&[C::BrowsePlanes, C::BrowseRanges, C::BrowseCodePoints], "Browse", 2),
            short(&[C::Quit], "Quit", 0),
            short(&[C::Help], "Help", 0),
        ],
        Context::Search => vec![
            short(&[C::PreviousResult, C::NextResult], "Move", 1),
            short(&[C::InspectResult], "Inspect", 1),
            short(&[C::Close], "Close", 0),
            short(&[C::Quit], "Quit", 2),
            short(&[C::Help], "Help", 0),
        ],
        Context::BrowsePlane => vec![
            short(&[C::MoveUp, C::MoveDown], "Move", 1),
            short(&[C::First, C::Last], "Ends", 3),
            short(&[C::Activate], "Ranges", 1),
            short(&[C::Close], "Close", 0),
            short(&[C::Quit], "Quit", 2),
            short(&[C::Help], "Help", 0),
        ],
        Context::BrowseRange => vec![
            short(&[C::MoveUp, C::MoveDown], "Move", 1),
            short(&[C::PageUp, C::PageDown], "Jump", 3),
            short(&[C::First, C::Last], "Ends", 3),
            short(&[C::Activate], "Points", 1),
            short(&[C::Back], "Back", 1),
            short(&[C::Close], "Close", 0),
            short(&[C::Quit], "Quit", 2),
            short(&[C::Help], "Help", 0),
        ],
        Context::BrowseCodePoints => vec![
            short(&[C::MoveLeft, C::MoveRight, C::MoveUp, C::MoveDown], "Move", 1),
            short(&[C::PageUp, C::PageDown], "Jump", 3),
            short(&[C::First, C::Last], "Ends", 3),
            short(&[C::Activate], "Inspect", 1),
            short(&[C::Back], "Back", 1),
            short(&[C::Close], "Close", 0),
            short(&[C::Quit], "Quit", 2),
            short(&[C::Help], "Help", 0),
        ],
        Context::Help => vec![
            short(&[C::MoveUp, C::MoveDown], "Move", 1),
            short(&[C::PageUp, C::PageDown], "Page", 3),
            short(&[C::First, C::Last], "Ends", 3),
            short(&[C::Close, C::Help], "Close", 0),
            short(&[C::Quit], "Quit", 1),
        ],
        Context::Global => vec![
            short(&[C::Quit], "Quit", 0), short(&[C::Help], "Help", 0),
        ],
    }
}

fn short(commands: &[Command], description: &'static str, priority: u8) -> ShortHelpItem {
    ShortHelpItem {
        commands: commands.to_vec(),
        description,
        priority,
    }
}

fn render_short_help_item(
    context: Context,
    item: ShortHelpItem,
    keymap: &ResolvedKeymap,
) -> Option<RenderedShortHelpItem> {
    let labels = item
        .commands
        .into_iter()
        .filter_map(|command| keymap.keys_for_active(context, command).first().copied())
        .map(|key| format_key(key, KeyLabelStyle::Compact))
        .collect::<Vec<_>>();
    if labels.is_empty() {
        return None;
    }

    Some(RenderedShortHelpItem {
        text: format!("{}: {}", labels.join("/"), item.description),
        priority: item.priority,
    })
}

fn footer_width(items: &[RenderedShortHelpItem]) -> usize {
    if items.is_empty() {
        return 0;
    }
    let content = items
        .iter()
        .map(|item| item.text.as_str())
        .collect::<Vec<_>>()
        .join("  ");
    Line::from(content).width() + 2
}

fn context_label(context: Context) -> &'static str {
    match context {
        Context::Global => "Global",
        Context::Inspector => "Inspector",
        Context::Search => "Search",
        Context::BrowsePlane => "Browse Planes",
        Context::BrowseRange => "Browse Ranges",
        Context::BrowseCodePoints => "Browse Code Points",
        Context::Help => "Help",
    }
}

fn content_area(area: Rect) -> Rect {
    Block::bordered()
        .padding(Padding::horizontal(1))
        .inner(area)
}

fn help_sections(area: Rect) -> HelpSections {
    let [about, divider, help] = Layout::vertical([
        Constraint::Length(ABOUT_HEIGHT),
        Constraint::Length(DIVIDER_HEIGHT),
        Constraint::Min(0),
    ])
    .areas(content_area(area));

    HelpSections {
        about,
        divider,
        help,
    }
}

fn section_block() -> Block<'static> {
    Block::default().padding(Padding::uniform(1))
}

fn section_content_area(area: Rect) -> Rect {
    section_block().inner(area)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fixtures, keybindings::OptionalKeybindings, ui::layout::MINIMUM_SIZE};

    #[test]
    fn about_formats_all_fixture_metadata() {
        let text = about_document(FIXTURE_PACKAGE, Style::new(), Style::new());
        let rendered = text
            .lines
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            rendered,
            vec![
                "sauva - Terminal Unicode Explorer",
                "",
                "Version: 1.2.3",
                "",
                "https://example.com/sauva",
            ]
        );
    }

    #[test]
    fn cargo_metadata_is_complete_and_fits_the_minimum_width() {
        let text = about_document(CARGO_PACKAGE, Style::new(), Style::new());
        let (width, height) = MINIMUM_SIZE;
        let shell = layout::calculate(Rect::new(0, 0, width, height)).unwrap();
        let content = section_content_area(help_sections(shell.main).about);

        assert!(
            [
                CARGO_PACKAGE.name,
                CARGO_PACKAGE.description,
                CARGO_PACKAGE.version,
                CARGO_PACKAGE.repository,
            ]
            .into_iter()
            .all(|value| !value.is_empty())
        );
        assert_eq!(usize::from(content.height), text.lines.len());
        assert!(
            text.lines
                .iter()
                .all(|line| line.width() <= usize::from(content.width))
        );
    }

    #[test]
    fn viewport_metrics_exclude_the_fixed_about_area() {
        let state = fixtures::startup();
        let keymap = ResolvedKeymap::default();
        let (width, height) = MINIMUM_SIZE;

        let metrics = viewport_metrics(Rect::new(0, 0, width, height), &state, &keymap);

        assert_eq!(metrics.viewport_height, 3);
        assert!(metrics.document_height > metrics.viewport_height);
    }

    #[test]
    fn footer_prunes_lower_priority_items_to_fit() {
        let keymap = ResolvedKeymap::default();
        let wide = footer(Context::Inspector, 140, &keymap);
        let narrow = footer(Context::Inspector, 60, &keymap);

        assert!(Line::from(wide.as_str()).width() <= 140);
        assert!(Line::from(narrow.as_str()).width() <= 60);
        assert!(wide.contains("C-u/C-d: Page"));
        assert!(!narrow.contains("C-u/C-d: Page"));
        assert!(narrow.contains("q: Quit"));
        assert!(narrow.contains("F1: Help"));
    }

    #[test]
    fn full_help_resolves_context_and_global_keys() {
        let text = document(
            Context::Inspector,
            &ResolvedKeymap::default(),
            Style::new(),
            Style::new(),
            80,
        );
        let rendered = text
            .lines
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("<q>  <Esc>  <Ctrl-c>"));
        assert!(rendered.contains("<h>  <Left>"));
        assert!(rendered.contains("<F1>"));
        assert!(rendered.contains("Browse code points"));
    }

    #[test]
    fn search_help_is_context_specific_and_explains_raw_input() {
        let text = document(
            Context::Search,
            &ResolvedKeymap::default(),
            Style::new(),
            Style::new(),
            80,
        );
        let rendered = text
            .lines
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("Select the previous result"));
        assert!(rendered.contains("Type normally to edit the query"));
        assert!(!rendered.contains("Browse code points"));
    }

    #[test]
    fn help_uses_configured_keys_and_omits_disabled_commands() {
        let config: OptionalKeybindings = toml::from_str(
            r#"
                [inspector]
                next_code_point = ["n"]
                search = []
            "#,
        )
        .unwrap();
        let keymap = ResolvedKeymap::with_config(config.into()).unwrap();
        let footer = footer(Context::Inspector, 140, &keymap);
        let help = document(Context::Inspector, &keymap, Style::new(), Style::new(), 80)
            .lines
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(footer.contains("h/n: Point"));
        assert!(!footer.contains("/: Search"));
        assert!(help.contains("<n>"));
        assert!(!help.contains("<l>  <Right>"));
        assert!(!help.contains("Open search"));
    }
}
