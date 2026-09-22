#[cfg(test)]
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::crossterm::event::{KeyEvent, KeyEventKind};
use tui_input::backend::crossterm::to_input_request;

use crate::{
    app::{Action, AppState, CodePointMove, View},
    browser::{BrowseLevel, BrowseMove},
    help::HelpMove,
    inspector::InspectorMove,
    keybindings::{Command, Context, KeyChord, ResolvedKeymap},
    search::SearchMove,
};

pub fn action_for_key(state: &AppState, key: KeyEvent, keymap: &ResolvedKeymap) -> Option<Action> {
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return None;
    }

    let chord = KeyChord::from_event(key);
    let base_context = context_for_state(state);
    let resolved = if state.help().is_open() {
        keymap
            .resolve(Context::Help, chord)
            .map(|command| (Context::Help, command))
            .or_else(|| {
                keymap
                    .resolve(Context::Global, chord)
                    .map(|command| (Context::Global, command))
            })
    } else {
        keymap
            .resolve(Context::Global, chord)
            .map(|command| (Context::Global, command))
            .or_else(|| {
                keymap
                    .resolve(base_context, chord)
                    .map(|command| (base_context, command))
            })
    };

    if let Some((context, command)) = resolved {
        if key.kind == KeyEventKind::Repeat && !command.is_repeatable() {
            return None;
        }
        return action_for_command(context, command);
    }

    if !state.help().is_open() && state.view() == View::Search {
        return to_input_request(&ratatui::crossterm::event::Event::Key(key))
            .map(Action::EditSearch);
    }

    None
}

pub fn context_for_state(state: &AppState) -> Context {
    match state.view() {
        View::Inspector => Context::Inspector,
        View::Search => Context::Search,
        View::Browser => match state
            .browse()
            .expect("the browser view always has browse state")
            .level()
        {
            BrowseLevel::Plane => Context::BrowsePlane,
            BrowseLevel::Range => Context::BrowseRange,
            BrowseLevel::CodePointTable => Context::BrowseCodePoints,
        },
    }
}

#[rustfmt::skip]
fn action_for_command(context: Context, command: Command) -> Option<Action> {
    use Command as C;
    use Context as X;

    match (context, command) {
        (_, C::Quit) => Some(Action::Quit),
        (_, C::Help) => Some(Action::ToggleHelp),
        (X::Help, C::Close) => Some(Action::CloseHelp),
        (X::Help, C::MoveUp) => Some(Action::MoveHelp(HelpMove::LineBackward)),
        (X::Help, C::MoveDown) => Some(Action::MoveHelp(HelpMove::LineForward)),
        (X::Help, C::PageUp) => Some(Action::MoveHelp(HelpMove::PageBackward)),
        (X::Help, C::PageDown) => Some(Action::MoveHelp(HelpMove::PageForward)),
        (X::Help, C::First) => Some(Action::MoveHelp(HelpMove::First)),
        (X::Help, C::Last) => Some(Action::MoveHelp(HelpMove::Last)),
        (X::Inspector, C::PreviousCodePoint) => Some(Action::MoveCodePoint(CodePointMove::Previous)),
        (X::Inspector, C::NextCodePoint) => Some(Action::MoveCodePoint(CodePointMove::Next)),
        (X::Inspector, C::MoveUp) => Some(Action::MoveInspector(InspectorMove::PreviousField)),
        (X::Inspector, C::MoveDown) => Some(Action::MoveInspector(InspectorMove::NextField)),
        (X::Inspector, C::PageUp) => Some(Action::MoveInspector(InspectorMove::PageBackward)),
        (X::Inspector, C::PageDown) => Some(Action::MoveInspector(InspectorMove::PageForward)),
        (X::Inspector, C::First) => Some(Action::MoveInspector(InspectorMove::First)),
        (X::Inspector, C::Last) => Some(Action::MoveInspector(InspectorMove::Last)),
        (X::Inspector, C::CopyValue) => Some(Action::CopyInspectorValue),
        (X::Inspector, C::OpenSearch) => Some(Action::OpenSearch),
        (X::Inspector, C::BrowsePlanes) => Some(Action::OpenBrowser(BrowseLevel::Plane)),
        (X::Inspector, C::BrowseRanges) => Some(Action::OpenBrowser(BrowseLevel::Range)),
        (X::Inspector, C::BrowseCodePoints) => Some(Action::OpenBrowser(BrowseLevel::CodePointTable)),
        (X::Search, C::PreviousResult) => Some(Action::MoveSearch(SearchMove::Previous)),
        (X::Search, C::NextResult) => Some(Action::MoveSearch(SearchMove::Next)),
        (X::Search, C::InspectResult) => Some(Action::InspectSearchResult),
        (X::Search, C::Close) => Some(Action::CloseSearch),
        (X::BrowsePlane | X::BrowseRange | X::BrowseCodePoints, C::MoveUp) => Some(Action::MoveBrowser(BrowseMove::Up)),
        (X::BrowsePlane | X::BrowseRange | X::BrowseCodePoints, C::MoveDown) => Some(Action::MoveBrowser(BrowseMove::Down)),
        (X::BrowsePlane | X::BrowseRange | X::BrowseCodePoints, C::First) => Some(Action::MoveBrowser(BrowseMove::First)),
        (X::BrowsePlane | X::BrowseRange | X::BrowseCodePoints, C::Last) => Some(Action::MoveBrowser(BrowseMove::Last)),
        (X::BrowseRange | X::BrowseCodePoints, C::PageUp) => Some(Action::MoveBrowser(BrowseMove::LargeBackward)),
        (X::BrowseRange | X::BrowseCodePoints, C::PageDown) => Some(Action::MoveBrowser(BrowseMove::LargeForward)),
        (X::BrowseCodePoints, C::MoveLeft) => Some(Action::MoveBrowser(BrowseMove::Left)),
        (X::BrowseCodePoints, C::MoveRight) => Some(Action::MoveBrowser(BrowseMove::Right)),
        (X::BrowsePlane | X::BrowseRange | X::BrowseCodePoints, C::Activate) => Some(Action::AdvanceBrowser),
        (X::BrowsePlane | X::BrowseRange | X::BrowseCodePoints, C::Back) => Some(Action::BackBrowser),
        (X::BrowsePlane | X::BrowseRange | X::BrowseCodePoints, C::Close) => Some(Action::CloseBrowser),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use tui_input::InputRequest;

    use super::*;

    fn action_for_key(state: &AppState, key: KeyEvent) -> Option<Action> {
        super::action_for_key(state, key, &ResolvedKeymap::default())
    }

    #[test]
    fn lowercase_q_requests_quit() {
        let state = AppState::new();
        let action = action_for_key(
            &state,
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
        );

        assert_eq!(action, Some(Action::Quit));
    }

    #[test]
    fn escape_requests_quit() {
        let state = AppState::new();
        let action = action_for_key(&state, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

        assert_eq!(action, Some(Action::Quit));
    }

    #[test]
    fn control_c_requests_quit_in_every_state() {
        let states = [
            AppState::new(),
            browser_state(BrowseLevel::Plane),
            search_state(),
        ];
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

        for state in states {
            assert_eq!(action_for_key(&state, key), Some(Action::Quit));
        }
    }

    #[rstest]
    #[case('p', BrowseLevel::Plane)]
    #[case('r', BrowseLevel::Range)]
    #[case('c', BrowseLevel::CodePointTable)]
    fn browser_shortcuts_open_the_requested_level(#[case] key: char, #[case] level: BrowseLevel) {
        let state = AppState::new();

        assert_eq!(
            action_for_key(
                &state,
                KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE)
            ),
            Some(Action::OpenBrowser(level))
        );
    }

    #[rstest]
    #[case('b')]
    #[case('o')]
    fn former_shortcuts_are_unassigned(#[case] key: char) {
        let state = AppState::new();

        assert_eq!(
            action_for_key(
                &state,
                KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE)
            ),
            None
        );
    }

    #[test]
    fn slash_opens_search_from_the_inspector() {
        let state = AppState::new();

        assert_eq!(
            action_for_key(
                &state,
                KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)
            ),
            Some(Action::OpenSearch)
        );
    }

    #[test]
    fn configured_keys_replace_defaults_in_runtime_dispatch() {
        let config: crate::keybindings::OptionalKeybindings = toml::from_str(
            r#"
                [inspector]
                next_code_point = ["n"]
            "#,
        )
        .unwrap();
        let keymap = ResolvedKeymap::with_config(config.into()).unwrap();
        let state = AppState::new();

        assert_eq!(
            super::action_for_key(
                &state,
                KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
                &keymap,
            ),
            Some(Action::MoveCodePoint(CodePointMove::Next))
        );
        assert_eq!(
            super::action_for_key(
                &state,
                KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
                &keymap,
            ),
            None
        );
    }

    #[rstest]
    #[case(KeyCode::Up, KeyModifiers::NONE, InspectorMove::PreviousField)]
    #[case(KeyCode::Char('k'), KeyModifiers::NONE, InspectorMove::PreviousField)]
    #[case(KeyCode::Down, KeyModifiers::NONE, InspectorMove::NextField)]
    #[case(KeyCode::Char('j'), KeyModifiers::NONE, InspectorMove::NextField)]
    #[case(KeyCode::Char('u'), KeyModifiers::CONTROL, InspectorMove::PageBackward)]
    #[case(KeyCode::Char('d'), KeyModifiers::CONTROL, InspectorMove::PageForward)]
    #[case(KeyCode::Char('g'), KeyModifiers::NONE, InspectorMove::First)]
    #[case(KeyCode::Char('G'), KeyModifiers::SHIFT, InspectorMove::Last)]
    fn inspector_scroll_keys_map_to_viewport_movements(
        #[case] code: KeyCode,
        #[case] modifiers: KeyModifiers,
        #[case] movement: InspectorMove,
    ) {
        assert_eq!(
            action_for_key(&AppState::new(), KeyEvent::new(code, modifiers)),
            Some(Action::MoveInspector(movement))
        );
    }

    #[rstest]
    #[case(KeyCode::Left, CodePointMove::Previous)]
    #[case(KeyCode::Char('h'), CodePointMove::Previous)]
    #[case(KeyCode::Right, CodePointMove::Next)]
    #[case(KeyCode::Char('l'), CodePointMove::Next)]
    fn inspector_horizontal_keys_move_to_adjacent_code_points(
        #[case] code: KeyCode,
        #[case] movement: CodePointMove,
    ) {
        assert_eq!(
            action_for_key(&AppState::new(), KeyEvent::new(code, KeyModifiers::NONE)),
            Some(Action::MoveCodePoint(movement))
        );
    }

    #[test]
    fn adjacent_code_point_movement_repeats_but_release_does_not() {
        let state = AppState::new();
        let repeated =
            KeyEvent::new_with_kind(KeyCode::Right, KeyModifiers::NONE, KeyEventKind::Repeat);
        let released =
            KeyEvent::new_with_kind(KeyCode::Right, KeyModifiers::NONE, KeyEventKind::Release);

        assert_eq!(
            action_for_key(&state, repeated),
            Some(Action::MoveCodePoint(CodePointMove::Next))
        );
        assert_eq!(action_for_key(&state, released), None);
    }

    #[test]
    fn inspector_scroll_repeats_but_release_does_not() {
        let state = AppState::new();
        let repeated =
            KeyEvent::new_with_kind(KeyCode::Down, KeyModifiers::NONE, KeyEventKind::Repeat);
        let released =
            KeyEvent::new_with_kind(KeyCode::Down, KeyModifiers::NONE, KeyEventKind::Release);

        assert_eq!(
            action_for_key(&state, repeated),
            Some(Action::MoveInspector(InspectorMove::NextField))
        );
        assert_eq!(action_for_key(&state, released), None);
    }

    #[test]
    fn y_copies_the_inspector_value_but_does_not_repeat() {
        let state = AppState::new();
        let pressed = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        let repeated =
            KeyEvent::new_with_kind(KeyCode::Char('y'), KeyModifiers::NONE, KeyEventKind::Repeat);

        assert_eq!(
            action_for_key(&state, pressed),
            Some(Action::CopyInspectorValue)
        );
        assert_eq!(action_for_key(&state, repeated), None);
    }

    #[test]
    fn key_release_has_no_action() {
        let state = AppState::new();
        let action = action_for_key(
            &state,
            KeyEvent::new_with_kind(
                KeyCode::Char('q'),
                KeyModifiers::NONE,
                KeyEventKind::Release,
            ),
        );

        assert_eq!(action, None);
    }

    #[rstest]
    #[case(KeyCode::Up, KeyModifiers::NONE, SearchMove::Previous)]
    #[case(KeyCode::Char('p'), KeyModifiers::CONTROL, SearchMove::Previous)]
    #[case(KeyCode::Down, KeyModifiers::NONE, SearchMove::Next)]
    #[case(KeyCode::Char('n'), KeyModifiers::CONTROL, SearchMove::Next)]
    fn search_movement_keys_map_to_previous_and_next(
        #[case] code: KeyCode,
        #[case] modifiers: KeyModifiers,
        #[case] movement: SearchMove,
    ) {
        assert_eq!(
            action_for_key(&search_state(), KeyEvent::new(code, modifiers)),
            Some(Action::MoveSearch(movement))
        );
    }

    #[rstest]
    #[case(KeyCode::Enter, Action::InspectSearchResult)]
    #[case(KeyCode::Esc, Action::CloseSearch)]
    fn search_enter_and_escape_map_to_search_actions(
        #[case] code: KeyCode,
        #[case] expected: Action,
    ) {
        assert_eq!(
            action_for_key(&search_state(), KeyEvent::new(code, KeyModifiers::NONE)),
            Some(expected)
        );
    }

    #[rstest]
    #[case('q')]
    #[case('j')]
    #[case('k')]
    #[case('/')]
    fn search_character_keys_edit_the_query(#[case] character: char) {
        assert_eq!(
            action_for_key(
                &search_state(),
                KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE)
            ),
            Some(Action::EditSearch(InputRequest::InsertChar(character)))
        );
    }

    #[test]
    fn search_movement_repeats_but_release_does_not() {
        let repeated = KeyEvent::new_with_kind(
            KeyCode::Char('n'),
            KeyModifiers::CONTROL,
            KeyEventKind::Repeat,
        );
        let released = KeyEvent::new_with_kind(
            KeyCode::Char('n'),
            KeyModifiers::CONTROL,
            KeyEventKind::Release,
        );

        assert_eq!(
            action_for_key(&search_state(), repeated),
            Some(Action::MoveSearch(SearchMove::Next))
        );
        assert_eq!(action_for_key(&search_state(), released), None);
    }

    #[test]
    fn f1_toggles_help_in_every_base_view() {
        for state in [
            AppState::new(),
            browser_state(BrowseLevel::Plane),
            search_state(),
        ] {
            assert_eq!(
                action_for_key(&state, KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE)),
                Some(Action::ToggleHelp)
            );
        }
    }

    #[test]
    fn help_keys_do_not_reach_the_base_view_or_search_input() {
        let mut state = search_state();
        crate::app::update(&mut state, Action::ToggleHelp);

        assert_eq!(
            action_for_key(
                &state,
                KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)
            ),
            Some(Action::MoveHelp(HelpMove::LineForward))
        );
        assert_eq!(
            action_for_key(&state, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            Some(Action::CloseHelp)
        );
        assert_eq!(
            action_for_key(
                &state,
                KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)
            ),
            None
        );
    }

    #[rstest]
    #[case(KeyCode::Backspace, InputRequest::DeletePrevChar)]
    #[case(KeyCode::Delete, InputRequest::DeleteNextChar)]
    #[case(KeyCode::Left, InputRequest::GoToPrevChar)]
    #[case(KeyCode::Right, InputRequest::GoToNextChar)]
    #[case(KeyCode::Home, InputRequest::GoToStart)]
    #[case(KeyCode::End, InputRequest::GoToEnd)]
    fn search_editing_keys_are_delegated_to_tui_input(
        #[case] code: KeyCode,
        #[case] request: InputRequest,
    ) {
        assert_eq!(
            action_for_key(&search_state(), KeyEvent::new(code, KeyModifiers::NONE)),
            Some(Action::EditSearch(request))
        );
    }

    #[rustfmt::skip]
    #[rstest]
    #[case(
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        Action::AdvanceBrowser
    )]
    #[case(
        KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
        Action::CloseBrowser
    )]
    #[case(
        KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
        Action::Quit
    )]
    fn browser_enter_escape_and_q_map_to_navigation_and_app_actions(
        #[case] key: KeyEvent,
        #[case] expected: Action,
    ) {
        let state = browser_state(BrowseLevel::Range);

        assert_eq!(action_for_key(&state, key), Some(expected));
    }

    #[rstest]
    #[case(BrowseLevel::Range, Some(Action::BackBrowser))]
    #[case(BrowseLevel::CodePointTable, Some(Action::BackBrowser))]
    #[case(BrowseLevel::Plane, Some(Action::BackBrowser))]
    fn backspace_returns_to_the_previous_screen_or_inspector(
        #[case] level: BrowseLevel,
        #[case] expected: Option<Action>,
    ) {
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);

        assert_eq!(action_for_key(&browser_state(level), key), expected);
    }

    #[rustfmt::skip]
    #[rstest]
    #[case::plane_up(
        BrowseLevel::Plane,
        KeyCode::Up,
        KeyModifiers::NONE,
        Some(BrowseMove::Up)
    )]
    #[case::plane_down(
        BrowseLevel::Plane,
        KeyCode::Char('j'),
        KeyModifiers::NONE,
        Some(BrowseMove::Down)
    )]
    #[case::plane_first(
        BrowseLevel::Plane,
        KeyCode::Char('g'),
        KeyModifiers::NONE,
        Some(BrowseMove::First)
    )]
    #[case::plane_last(
        BrowseLevel::Plane,
        KeyCode::Char('G'),
        KeyModifiers::SHIFT,
        Some(BrowseMove::Last)
    )]
    #[case::plane_left_unassigned(
        BrowseLevel::Plane,
        KeyCode::Left,
        KeyModifiers::NONE,
        None
    )]
    #[case::plane_jump_unassigned(
        BrowseLevel::Plane,
        KeyCode::Char('u'),
        KeyModifiers::CONTROL,
        None
    )]
    #[case::range_up(
        BrowseLevel::Range,
        KeyCode::Char('k'),
        KeyModifiers::NONE,
        Some(BrowseMove::Up)
    )]
    #[case::range_down(
        BrowseLevel::Range,
        KeyCode::Down,
        KeyModifiers::NONE,
        Some(BrowseMove::Down)
    )]
    #[case::range_backward(
        BrowseLevel::Range,
        KeyCode::Char('u'),
        KeyModifiers::CONTROL,
        Some(BrowseMove::LargeBackward)
    )]
    #[case::range_forward(
        BrowseLevel::Range,
        KeyCode::Char('d'),
        KeyModifiers::CONTROL,
        Some(BrowseMove::LargeForward)
    )]
    #[case::range_right_unassigned(
        BrowseLevel::Range,
        KeyCode::Right,
        KeyModifiers::NONE,
        None
    )]
    #[case::table_left(
        BrowseLevel::CodePointTable,
        KeyCode::Left,
        KeyModifiers::NONE,
        Some(BrowseMove::Left)
    )]
    #[case::table_right(
        BrowseLevel::CodePointTable,
        KeyCode::Char('l'),
        KeyModifiers::NONE,
        Some(BrowseMove::Right)
    )]
    #[case::table_up(
        BrowseLevel::CodePointTable,
        KeyCode::Up,
        KeyModifiers::NONE,
        Some(BrowseMove::Up)
    )]
    #[case::table_down(
        BrowseLevel::CodePointTable,
        KeyCode::Char('j'),
        KeyModifiers::NONE,
        Some(BrowseMove::Down)
    )]
    #[case::table_backward(
        BrowseLevel::CodePointTable,
        KeyCode::Char('u'),
        KeyModifiers::CONTROL,
        Some(BrowseMove::LargeBackward)
    )]
    #[case::table_forward(
        BrowseLevel::CodePointTable,
        KeyCode::Char('d'),
        KeyModifiers::CONTROL,
        Some(BrowseMove::LargeForward)
    )]
    fn browser_movement_keys_map_by_level(
        #[case] level: BrowseLevel,
        #[case] code: KeyCode,
        #[case] modifiers: KeyModifiers,
        #[case] movement: Option<BrowseMove>,
    ) {
        let state = browser_state(level);

        assert_eq!(
            action_for_key(&state, KeyEvent::new(code, modifiers)),
            movement.map(Action::MoveBrowser)
        );
    }

    #[test]
    fn navigation_repeats_but_release_does_not() {
        let state = browser_state(BrowseLevel::CodePointTable);
        let repeated =
            KeyEvent::new_with_kind(KeyCode::Right, KeyModifiers::NONE, KeyEventKind::Repeat);
        let released =
            KeyEvent::new_with_kind(KeyCode::Right, KeyModifiers::NONE, KeyEventKind::Release);

        assert_eq!(
            action_for_key(&state, repeated),
            Some(Action::MoveBrowser(BrowseMove::Right))
        );
        assert_eq!(action_for_key(&state, released), None);
    }

    #[rstest]
    #[case(KeyCode::Home)]
    #[case(KeyCode::End)]
    #[case(KeyCode::PageUp)]
    #[case(KeyCode::PageDown)]
    fn unassigned_navigation_keys_do_not_have_actions(#[case] code: KeyCode) {
        let state = browser_state(BrowseLevel::CodePointTable);

        assert_eq!(
            action_for_key(&state, KeyEvent::new(code, KeyModifiers::NONE)),
            None
        );
    }

    fn browser_state(level: BrowseLevel) -> AppState {
        let mut state = AppState::new();
        crate::app::update(&mut state, Action::OpenBrowser(level));
        state
    }

    fn search_state() -> AppState {
        let mut state = AppState::new();
        crate::app::update(&mut state, Action::OpenSearch);
        state
    }
}
