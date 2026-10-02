use std::io;

use ratatui::{
    DefaultTerminal,
    crossterm::{
        event::{self, Event},
        terminal::window_size,
    },
    layout::Rect,
};

use crate::{
    app::{Action, AppState, FooterStatus, View, update},
    clipboard::{ClipboardWriter, SystemClipboard},
    glyph::{
        runtime::{GlyphPreviewRuntime, TerminalPixelMetrics},
        settings::GlyphPreviewSettings,
    },
    graphics::{self, GraphicsMode},
    image::kitty,
    input::action_for_key,
    keybindings::ResolvedKeymap,
    ui::{
        self, inspector,
        layout::{browser_list_height, search_result_height, sequence_list_height},
        render,
        settings::UiSettings,
        theme::ColorTheme,
    },
};

pub fn run(
    state: &mut AppState,
    graphics_mode: GraphicsMode,
    glyph_preview: &GlyphPreviewSettings,
    color_theme: &ColorTheme,
    ui: &UiSettings,
    keymap: &ResolvedKeymap,
) -> io::Result<()> {
    let availability = graphics::detect(graphics_mode);
    let mut preview_runtime =
        GlyphPreviewRuntime::with_settings(availability, glyph_preview.clone());
    let mut clipboard = SystemClipboard::new();
    update(
        state,
        Action::UpdateGlyphPreview(preview_runtime.configure_update()),
    );
    let _color_output = preview_runtime
        .uses_kitty_placeholders()
        .then(kitty::enable_placeholder_colors);

    ratatui::run(|terminal| {
        let run_result = run_event_loop(
            terminal,
            state,
            &mut preview_runtime,
            &mut clipboard,
            color_theme,
            ui,
            keymap,
        );
        let preview_cleanup_result = preview_runtime.cleanup(terminal.backend_mut());

        run_result.and(preview_cleanup_result)
    })
}

fn run_event_loop(
    terminal: &mut DefaultTerminal,
    state: &mut AppState,
    preview_runtime: &mut GlyphPreviewRuntime,
    clipboard: &mut impl ClipboardWriter,
    color_theme: &ColorTheme,
    ui: &UiSettings,
    keymap: &ResolvedKeymap,
) -> io::Result<()> {
    while state.is_running() {
        let size = terminal.size()?;
        let area = Rect::new(0, 0, size.width, size.height);
        let preview_request = ui::glyph_preview_request(area, state);
        let preview_code_point = preview_request
            .map(|request| request.code_point)
            .unwrap_or_else(|| state.selected());
        let preview_placeholder = preview_request.map(|request| request.placeholder);
        let terminal_pixels = window_size().ok().map(|size| TerminalPixelMetrics {
            columns: size.columns,
            rows: size.rows,
            width: size.width,
            height: size.height,
        });
        let preview_update = preview_runtime.synchronize(
            terminal.backend_mut(),
            preview_code_point,
            preview_placeholder,
            terminal_pixels,
        );
        update(state, Action::UpdateGlyphPreview(preview_update));
        resize_active_view(state, area, keymap);
        terminal.draw(|frame| {
            render::render(frame, state, color_theme, ui, keymap);
            preview_runtime.render_image(frame.buffer_mut());
        })?;

        if let Event::Key(key) = event::read()?
            && let Some(action) = action_for_key(state, key, keymap)
        {
            update(state, action);
            handle_clipboard_request(state, clipboard);
        }
    }

    Ok(())
}

fn handle_clipboard_request(state: &mut AppState, clipboard: &mut impl ClipboardWriter) {
    let Some(request) = state.take_clipboard_request() else {
        return;
    };
    let status = match request.value() {
        None => FooterStatus::warning(format!("No value to copy: {}", request.label())),
        Some(value) => match clipboard.write_text(value) {
            Ok(()) => FooterStatus::info(request.success_message()),
            Err(error) => FooterStatus::warning(error.to_string()),
        },
    };
    update(state, Action::ShowFooterStatus(status));
}

fn resize_active_view(state: &mut AppState, area: Rect, keymap: &ResolvedKeymap) {
    if state.help().is_open() {
        let metrics = crate::ui::help::viewport_metrics(area, state, keymap);
        update(
            state,
            Action::ResizeHelpViewport {
                viewport_height: metrics.viewport_height,
                document_height: metrics.document_height,
            },
        );
        return;
    }

    let action = match state.view() {
        View::Inspector => {
            let metrics = inspector::viewport_metrics(area, state);
            Action::ResizeInspectorViewport {
                viewport_height: metrics.viewport_height,
                document_height: metrics.document_height,
                field_ranges: metrics.field_ranges,
            }
        }
        View::Browser => Action::ResizeBrowserViewport(browser_list_height(area)),
        View::Search => Action::ResizeSearchViewport(search_result_height(area)),
        View::Sequence => Action::ResizeSequenceViewport(sequence_list_height(area)),
        View::Normalization => return,
    };
    update(state, action);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::FooterStatusLevel, clipboard::ClipboardError, fixtures, inspector::InspectorMove,
    };

    struct TestClipboard {
        result: Result<(), ClipboardError>,
        writes: Vec<String>,
    }

    impl TestClipboard {
        fn succeeding() -> Self {
            Self {
                result: Ok(()),
                writes: Vec::new(),
            }
        }

        fn failing(error: ClipboardError) -> Self {
            Self {
                result: Err(error),
                writes: Vec::new(),
            }
        }
    }

    impl ClipboardWriter for TestClipboard {
        fn write_text(&mut self, text: &str) -> Result<(), ClipboardError> {
            self.writes.push(text.to_owned());
            self.result
        }
    }

    #[test]
    fn reports_a_successful_character_copy_without_echoing_the_character() {
        let mut state = fixtures::startup();
        let mut clipboard = TestClipboard::succeeding();
        update(&mut state, Action::CopyInspectorValue);

        handle_clipboard_request(&mut state, &mut clipboard);

        assert_eq!(clipboard.writes, ["A"]);
        let status = state.footer_status().unwrap();
        assert_eq!(status.level(), FooterStatusLevel::Info);
        assert_eq!(status.message(), "Copied Character: U+0041");
    }

    #[test]
    fn copies_normalized_text_without_copying_escapes_or_echoing_controls() {
        let mut state = AppState::with_sequence("A\u{0301}\t\n\u{1b}①".to_owned());
        let mut clipboard = TestClipboard::succeeding();
        update(&mut state, Action::OpenNormalization);
        update(&mut state, Action::CopyNormalizationResult);
        handle_clipboard_request(&mut state, &mut clipboard);
        assert_eq!(clipboard.writes, ["Á\t\n\u{1b}①"]);
        assert_eq!(
            state.footer_status().unwrap().message(),
            "Copied NFC Result"
        );
        assert_eq!(
            state.footer_status().unwrap().level(),
            FooterStatusLevel::Info
        );
        update(
            &mut state,
            Action::MoveNormalization(crate::normalization::NormalizationMove::Next),
        );
        assert!(state.footer_status().is_none());
        let mut failing = TestClipboard::failing(ClipboardError::Unavailable);
        update(&mut state, Action::CopyNormalizationResult);
        handle_clipboard_request(&mut state, &mut failing);
        assert_eq!(
            state.footer_status().unwrap().message(),
            "Clipboard is unavailable"
        );
    }

    #[test]
    fn reports_an_unavailable_value_without_calling_the_clipboard() {
        let mut state = fixtures::surrogate();
        let mut clipboard = TestClipboard::succeeding();
        update(&mut state, Action::CopyInspectorValue);

        handle_clipboard_request(&mut state, &mut clipboard);

        assert!(clipboard.writes.is_empty());
        let status = state.footer_status().unwrap();
        assert_eq!(status.level(), FooterStatusLevel::Warning);
        assert_eq!(status.message(), "No value to copy: Character");
    }

    #[test]
    fn reports_clipboard_failures_as_warnings() {
        let mut state = fixtures::startup();
        let mut clipboard = TestClipboard::failing(ClipboardError::Busy);
        update(&mut state, Action::CopyInspectorValue);

        handle_clipboard_request(&mut state, &mut clipboard);

        let status = state.footer_status().unwrap();
        assert_eq!(status.level(), FooterStatusLevel::Warning);
        assert_eq!(status.message(), "Clipboard is busy");
    }

    #[test]
    fn copies_all_lines_of_the_current_property() {
        let mut state = fixtures::details_aliases();
        update(
            &mut state,
            Action::ResizeInspectorViewport {
                viewport_height: 10,
                document_height: 22,
                field_ranges: (0..22).map(|index| index..index + 1).collect(),
            },
        );
        for _ in 0..3 {
            update(&mut state, Action::MoveInspector(InspectorMove::NextField));
        }
        let mut clipboard = TestClipboard::succeeding();

        update(&mut state, Action::CopyInspectorValue);
        handle_clipboard_request(&mut state, &mut clipboard);

        assert_eq!(clipboard.writes, ["NULL — control\nNUL — abbreviation"]);
        assert_eq!(state.footer_status().unwrap().message(), "Copied Aliases");
    }
}
