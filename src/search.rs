use std::ops::Range;

use tui_input::{Input, InputRequest};

use crate::{
    unicode::{CodePoint, CodePointNotationError, UnicodeDatabase, parse_code_point_notation},
    viewport::ListViewport,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMove {
    Previous,
    Next,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchDirectMatchKind {
    CodePointNotation,
    LiteralCharacter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SearchNameMatchKind {
    Exact,
    Prefix,
    Substring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchResult {
    code_point: CodePoint,
    direct_match: Option<SearchDirectMatchKind>,
    name_match: Option<SearchNameMatchKind>,
}

impl SearchResult {
    const fn new(
        code_point: CodePoint,
        direct_match: Option<SearchDirectMatchKind>,
        name_match: Option<SearchNameMatchKind>,
    ) -> Self {
        Self {
            code_point,
            direct_match,
            name_match,
        }
    }

    #[cfg(test)]
    pub const fn new_for_test(
        code_point: CodePoint,
        direct_match: Option<SearchDirectMatchKind>,
        name_match: Option<SearchNameMatchKind>,
    ) -> Self {
        Self::new(code_point, direct_match, name_match)
    }

    pub const fn code_point(self) -> CodePoint {
        self.code_point
    }

    pub const fn direct_match(self) -> Option<SearchDirectMatchKind> {
        self.direct_match
    }

    pub const fn name_match(self) -> Option<SearchNameMatchKind> {
        self.name_match
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchOutcome {
    Empty,
    Results {
        name_query: Option<String>,
        results: Vec<SearchResult>,
    },
    InvalidNotation(CodePointNotationError),
}

impl SearchOutcome {
    pub fn results(&self) -> &[SearchResult] {
        match self {
            Self::Results { results, .. } => results,
            Self::Empty | Self::InvalidNotation(_) => &[],
        }
    }

    pub fn name_query(&self) -> Option<&str> {
        match self {
            Self::Results { name_query, .. } => name_query.as_deref(),
            Self::Empty | Self::InvalidNotation(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchState {
    input: Input,
    outcome: SearchOutcome,
    selected_index: Option<usize>,
    viewport: ListViewport,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            input: Input::default(),
            outcome: SearchOutcome::Empty,
            selected_index: None,
            viewport: ListViewport::new(),
        }
    }

    pub const fn input(&self) -> &Input {
        &self.input
    }

    pub const fn outcome(&self) -> &SearchOutcome {
        &self.outcome
    }

    pub const fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn selected_result(&self) -> Option<SearchResult> {
        self.selected_index
            .and_then(|index| self.outcome.results().get(index))
            .copied()
    }

    pub fn visible_result_range(&self) -> Range<usize> {
        self.viewport.visible_range(self.outcome.results().len())
    }

    pub fn edit(&mut self, request: InputRequest) {
        let value_changed = self
            .input
            .handle(request)
            .is_some_and(|change| change.value);
        if !value_changed {
            return;
        }

        self.outcome = search(self.input.value());
        self.selected_index = (!self.outcome.results().is_empty()).then_some(0);
        self.viewport
            .ensure_visible(0, self.outcome.results().len());
    }

    pub fn move_selection(&mut self, movement: SearchMove) -> bool {
        let Some(selected) = self.selected_index else {
            return false;
        };
        let item_count = self.outcome.results().len();
        let next = match movement {
            SearchMove::Previous => selected.saturating_sub(1),
            SearchMove::Next => (selected + 1).min(item_count - 1),
        };
        if next == selected {
            return false;
        }

        self.selected_index = Some(next);
        self.viewport.ensure_visible(next, item_count);
        true
    }

    pub fn resize_viewport(&mut self, height: usize) {
        self.viewport.resize(
            height,
            self.selected_index.unwrap_or(0),
            self.outcome.results().len(),
        );
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn search(input: &str) -> SearchOutcome {
    let query = input.trim_matches(|character: char| character.is_ascii_whitespace());
    if query.is_empty() {
        return SearchOutcome::Empty;
    }

    if has_code_point_prefix(query) {
        return exact_notation_result(query);
    }

    let mut characters = query.chars();
    if let (Some(character), None) = (characters.next(), characters.next()) {
        return exact_result(
            SearchDirectMatchKind::LiteralCharacter,
            CodePoint::from(character),
        );
    }

    if is_bare_code_point_notation(query) {
        return combined_notation_and_name_results(query);
    }

    primary_name_results(query)
}

fn has_code_point_prefix(query: &str) -> bool {
    query.starts_with("U+") || query.starts_with("u+")
}

fn is_bare_code_point_notation(query: &str) -> bool {
    (4..=6).contains(&query.len()) && query.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn exact_notation_result(query: &str) -> SearchOutcome {
    match parse_code_point_notation(query) {
        Ok(code_point) => exact_result(SearchDirectMatchKind::CodePointNotation, code_point),
        Err(error) => SearchOutcome::InvalidNotation(error),
    }
}

fn exact_result(direct_match: SearchDirectMatchKind, code_point: CodePoint) -> SearchOutcome {
    SearchOutcome::Results {
        name_query: None,
        results: vec![SearchResult::new(code_point, Some(direct_match), None)],
    }
}

fn primary_name_results(query: &str) -> SearchOutcome {
    combined_name_results(query, None)
}

fn combined_notation_and_name_results(query: &str) -> SearchOutcome {
    match parse_code_point_notation(query) {
        Ok(code_point) => combined_name_results(
            query,
            Some((code_point, SearchDirectMatchKind::CodePointNotation)),
        ),
        Err(error) => SearchOutcome::InvalidNotation(error),
    }
}

fn combined_name_results(
    query: &str,
    direct_result: Option<(CodePoint, SearchDirectMatchKind)>,
) -> SearchOutcome {
    let query = normalize_name_query(query);
    let mut direct = None;
    let mut exact = Vec::new();
    let mut prefix = Vec::new();
    let mut substring = Vec::new();
    for (code_point, name) in UnicodeDatabase::primary_names() {
        let direct_match = direct_result.and_then(|(direct_code_point, kind)| {
            (code_point == direct_code_point).then_some(kind)
        });
        let name_match = name_match_kind(name, &query);
        let Some(result) = (direct_match.is_some() || name_match.is_some())
            .then(|| SearchResult::new(code_point, direct_match, name_match))
        else {
            continue;
        };

        if direct_match.is_some() {
            direct = Some(result);
        } else {
            match name_match.expect("a non-direct result has a name match") {
                SearchNameMatchKind::Exact => exact.push(result),
                SearchNameMatchKind::Prefix => prefix.push(result),
                SearchNameMatchKind::Substring => substring.push(result),
            }
        }
    }
    if direct.is_none()
        && let Some((code_point, direct_match)) = direct_result
    {
        direct = Some(SearchResult::new(code_point, Some(direct_match), None));
    }
    let results = direct
        .into_iter()
        .chain(exact)
        .chain(prefix)
        .chain(substring)
        .collect();

    SearchOutcome::Results {
        name_query: Some(query),
        results,
    }
}

fn normalize_name_query(query: &str) -> String {
    let mut normalized = String::with_capacity(query.len());
    for word in query.split_ascii_whitespace() {
        if !normalized.is_empty() {
            normalized.push(' ');
        }
        normalized.push_str(word);
    }
    normalized.make_ascii_uppercase();
    normalized
}

fn name_match_kind(name: &str, query: &str) -> Option<SearchNameMatchKind> {
    if name == query {
        Some(SearchNameMatchKind::Exact)
    } else if name.starts_with(query) {
        Some(SearchNameMatchKind::Prefix)
    } else if name.contains(query) {
        Some(SearchNameMatchKind::Substring)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("A", 0x0041)]
    #[case("1", 0x0031)]
    #[case("→", 0x2192)]
    #[case("あ", 0x3042)]
    fn keeps_single_characters_as_literal_only(
        #[case] input: &str,
        #[case] expected_code_point: u32,
    ) {
        let SearchOutcome::Results {
            name_query,
            results,
        } = search(input)
        else {
            panic!("expected literal search results");
        };

        assert_eq!(name_query, None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code_point().value(), expected_code_point);
        assert_eq!(
            results[0].direct_match(),
            Some(SearchDirectMatchKind::LiteralCharacter)
        );
        assert_eq!(results[0].name_match(), None);
    }

    #[rstest]
    #[case("U+0041", 0x0041)]
    #[case("u+1f600", 0x1f600)]
    #[case("U+D800", 0xd800)]
    fn keeps_explicit_notation_as_a_direct_result(
        #[case] input: &str,
        #[case] expected_code_point: u32,
    ) {
        let SearchOutcome::Results {
            name_query,
            results,
        } = search(input)
        else {
            panic!("expected notation search results");
        };

        assert_eq!(name_query, None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].code_point().value(), expected_code_point);
        assert_eq!(
            results[0].direct_match(),
            Some(SearchDirectMatchKind::CodePointNotation)
        );
        assert_eq!(results[0].name_match(), None);
    }

    #[rstest]
    #[case("0041", 0x0041)]
    #[case("2192", 0x2192)]
    #[case("FACE", 0xface)]
    fn prioritizes_valid_bare_notation_and_keeps_name_matches(
        #[case] input: &str,
        #[case] expected_code_point: u32,
    ) {
        let SearchOutcome::Results {
            name_query,
            results,
        } = search(input)
        else {
            panic!("expected combined search results");
        };

        assert_eq!(name_query.as_deref(), Some(input));
        assert_eq!(results[0].code_point().value(), expected_code_point);
        assert_eq!(
            results[0].direct_match(),
            Some(SearchDirectMatchKind::CodePointNotation)
        );
        assert!(
            results
                .iter()
                .skip(1)
                .any(|result| result.name_match().is_some())
        );
    }

    #[rstest]
    #[case("AB")]
    #[case("123")]
    #[case("0000041")]
    #[case("latin capital letter a")]
    fn classifies_other_multi_character_queries_as_primary_names(#[case] input: &str) {
        let expected_query = normalize_name_query(input);
        let SearchOutcome::Results { name_query, .. } = search(input) else {
            panic!("expected primary name results");
        };

        assert_eq!(name_query.as_deref(), Some(expected_query.as_str()));
    }

    #[rstest]
    #[case("")]
    #[case(" ")]
    #[case("\t\r\n")]
    fn distinguishes_empty_queries(#[case] input: &str) {
        assert_eq!(search(input), SearchOutcome::Empty);
    }

    #[rustfmt::skip]
    #[rstest]
    #[case(
        "U+",
        CodePointNotationError::InvalidFormat
    )]
    #[case(
        "U+G",
        CodePointNotationError::InvalidFormat
    )]
    #[case(
        "U+110000",
        CodePointNotationError::OutOfRange { value: 0x11_0000 }
    )]
    #[case(
        "110000",
        CodePointNotationError::OutOfRange { value: 0x11_0000 }
    )]
    fn explicit_and_bare_notation_errors_do_not_fall_back_to_name_search(
        #[case] input: &str,
        #[case] expected: CodePointNotationError,
    ) {
        assert_eq!(search(input), SearchOutcome::InvalidNotation(expected));
    }

    #[test]
    fn normalizes_ascii_case_and_internal_whitespace_for_name_search() {
        let SearchOutcome::Results {
            name_query,
            results,
        } = search(" \tlatin   capital letter a\r\n")
        else {
            panic!("expected primary name results");
        };

        assert_eq!(name_query.as_deref(), Some("LATIN CAPITAL LETTER A"));
        assert_eq!(results[0].code_point().value(), 0x0041);
        assert_eq!(results[0].direct_match(), None);
        assert_eq!(results[0].name_match(), Some(SearchNameMatchKind::Exact));
    }

    #[test]
    fn orders_name_results_by_match_quality_then_code_point() {
        let SearchOutcome::Results {
            name_query,
            results,
        } = search("rightwards arrow")
        else {
            panic!("expected primary name results");
        };

        assert_eq!(name_query.as_deref(), Some("RIGHTWARDS ARROW"));
        assert_eq!(results[0].code_point().value(), 0x2192);
        assert_eq!(results[0].name_match(), Some(SearchNameMatchKind::Exact));
        assert!(results.windows(2).all(|pair| {
            (pair[0].name_match().unwrap(), pair[0].code_point())
                <= (pair[1].name_match().unwrap(), pair[1].code_point())
        }));
    }

    #[test]
    fn combines_duplicate_direct_and_name_matches_into_one_result() {
        let SearchOutcome::Results { results, .. } = search("face") else {
            panic!("expected combined search results");
        };

        assert_eq!(results[0].code_point().value(), 0xface);
        assert_eq!(
            results[0].direct_match(),
            Some(SearchDirectMatchKind::CodePointNotation)
        );
        assert_eq!(
            results[0].name_match(),
            Some(SearchNameMatchKind::Substring)
        );
        assert_eq!(
            results
                .iter()
                .filter(|result| result.code_point().value() == 0xface)
                .count(),
            1
        );
    }

    #[test]
    fn returns_an_empty_result_list_for_an_unknown_primary_name() {
        let SearchOutcome::Results {
            name_query,
            results,
        } = search("not a unicode name")
        else {
            panic!("expected primary name results");
        };

        assert_eq!(name_query.as_deref(), Some("NOT A UNICODE NAME"));
        assert!(results.is_empty());
    }

    #[test]
    fn search_state_starts_empty_without_a_selection() {
        let state = SearchState::new();

        assert_eq!(state.input().value(), "");
        assert_eq!(state.outcome(), &SearchOutcome::Empty);
        assert_eq!(state.selected_index(), None);
        assert_eq!(state.selected_result(), None);
    }

    #[test]
    fn editing_a_query_refreshes_results_and_selects_the_first() {
        let mut state = SearchState::new();

        state.edit(InputRequest::InsertChar('→'));

        assert_eq!(state.input().value(), "→");
        assert_eq!(state.selected_index(), Some(0));
        assert_eq!(
            state.selected_result().unwrap().code_point().value(),
            0x2192
        );
    }

    #[test]
    fn selection_movement_stops_at_result_boundaries_and_updates_the_viewport() {
        let mut state = search_state_with_query("rightwards arrow");
        state.resize_viewport(2);

        assert!(!state.move_selection(SearchMove::Previous));
        assert!(state.move_selection(SearchMove::Next));
        assert!(state.move_selection(SearchMove::Next));
        assert_eq!(state.selected_index(), Some(2));
        assert_eq!(state.visible_result_range(), 1..3);

        while state.move_selection(SearchMove::Next) {}
        let last = state.selected_index().unwrap();
        assert_eq!(last, state.outcome().results().len() - 1);
        assert!(!state.move_selection(SearchMove::Next));
    }

    #[test]
    fn changing_the_query_resets_selection_and_viewport_but_cursor_movement_does_not() {
        let mut state = search_state_with_query("arrow");
        state.resize_viewport(2);
        state.move_selection(SearchMove::Next);
        state.move_selection(SearchMove::Next);
        let selected = state.selected_result();

        state.edit(InputRequest::GoToPrevChar);
        assert_eq!(state.selected_result(), selected);
        assert_eq!(state.visible_result_range(), 1..3);

        state.edit(InputRequest::GoToEnd);
        state.edit(InputRequest::InsertChar(' '));
        assert_eq!(state.selected_index(), Some(0));
        assert_eq!(state.visible_result_range().start, 0);
    }

    #[test]
    fn a_query_without_results_clears_the_result_selection() {
        let state = search_state_with_query("not a unicode name");

        assert!(state.outcome().results().is_empty());
        assert_eq!(state.selected_index(), None);
        assert_eq!(state.selected_result(), None);
    }

    fn search_state_with_query(query: &str) -> SearchState {
        let mut state = SearchState::new();
        for character in query.chars() {
            state.edit(InputRequest::InsertChar(character));
        }
        state
    }
}
