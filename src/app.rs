mod update;

use smart_default::SmartDefault;
use tui_input::InputRequest;

pub use crate::app::update::update;

use crate::{
    bookmarks::{Bookmark, BookmarkMove, BookmarkState},
    browser::{BrowseLevel, BrowseMove, BrowseState},
    copy_dialog::{CopyDialogState, CopyMove, CopySource},
    help::{HelpMove, HelpState},
    inspector::{InspectorGroup, InspectorMove, InspectorState},
    normalization::{NormalizationMove, NormalizationState},
    preview::{GlyphPreviewState, GlyphPreviewUpdate},
    property::PropertyState,
    search::{SearchMove, SearchState},
    sequence::{SequenceMove, SequenceState},
    unicode::CodePoint,
    viewport::DocumentMove,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    ToggleHelp,
    CloseHelp,
    MoveHelp(HelpMove),
    ResizeHelpViewport {
        viewport_height: usize,
        document_height: usize,
    },
    OpenProperty,
    CloseProperty,
    MoveProperty(DocumentMove),
    ResizePropertyViewport {
        viewport_height: usize,
        document_height: usize,
    },
    OpenPropertyReference,
    CopyPropertyReference,
    MoveCodePoint(CodePointMove),
    MoveInspector(InspectorMove),
    ToggleInspectorProperties,
    CopyInspectorValue,
    ResizeInspectorViewport {
        viewport_height: usize,
        document_height: usize,
        field_ranges: Vec<std::ops::Range<usize>>,
        groups: Vec<InspectorGroup>,
    },
    OpenBrowser(BrowseLevel),
    AdvanceBrowser,
    BackBrowser,
    CloseBrowser,
    MoveBrowser(BrowseMove),
    ResizeBrowserViewport(usize),
    OpenSearch,
    CloseSearch,
    InspectSearchResult,
    EditSearch(InputRequest),
    MoveSearch(SearchMove),
    ResizeSearchViewport(usize),
    OpenBookmarks,
    CloseBookmarks,
    MoveBookmarks(BookmarkMove),
    SwitchBookmarkSort,
    ResizeBookmarksViewport(usize),
    InspectBookmark,
    ReturnToBookmarks,
    ToggleBookmark,
    CompleteBookmark {
        entries: Vec<Bookmark>,
        request: BookmarkRequest,
    },
    MoveSequence(SequenceMove),
    ResizeSequenceViewport(usize),
    ResizeNormalizationOriginalViewport(usize),
    InspectSequenceCodePoint,
    ReturnToSequence,
    OpenNormalization,
    MoveNormalization(NormalizationMove),
    CloseNormalization,
    InspectNormalizationResult,
    ReturnToNormalization,
    CopyNormalizationResult,
    OpenCopyDialog,
    CloseCopyDialog,
    MoveCopyDialog(CopyMove),
    CopyDialogSelection,
    CompleteCopyDialog(FooterStatus),
    UpdateGlyphPreview(GlyphPreviewUpdate),
    ShowFooterStatus(FooterStatus),
}

impl Action {
    fn clears_footer_status(&self) -> bool {
        !matches!(
            self,
            Self::ResizeHelpViewport { .. }
                | Self::ResizePropertyViewport { .. }
                | Self::ResizeInspectorViewport { .. }
                | Self::ResizeBrowserViewport(_)
                | Self::ResizeSearchViewport(_)
                | Self::ResizeBookmarksViewport(_)
                | Self::ResizeSequenceViewport(_)
                | Self::ResizeNormalizationOriginalViewport(_)
                | Self::UpdateGlyphPreview(_)
                | Self::ShowFooterStatus(_)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodePointMove {
    Previous,
    Next,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Inspector,
    Browser,
    Search,
    Bookmarks,
    Sequence,
    Normalization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FooterStatusLevel {
    Info,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FooterStatus {
    level: FooterStatusLevel,
    message: String,
}

impl FooterStatus {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            level: FooterStatusLevel::Info,
            message: message.into(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: FooterStatusLevel::Warning,
            message: message.into(),
        }
    }

    pub const fn level(&self) -> FooterStatusLevel {
        self.level
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardRequest {
    label: &'static str,
    success_message: String,
    value: Option<String>,
    from_copy_dialog: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BookmarkRequest {
    pub code_point: CodePoint,
    pub bookmarked: bool,
}

impl ClipboardRequest {
    pub fn success_message(&self) -> &str {
        &self.success_message
    }

    pub const fn label(&self) -> &'static str {
        self.label
    }

    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }

    pub const fn is_from_copy_dialog(&self) -> bool {
        self.from_copy_dialog
    }
}

#[derive(Debug, Clone, SmartDefault)]
pub struct AppState {
    #[default(true)]
    running: bool,
    #[default(View::Inspector)]
    view: View,
    #[default(CodePoint::from('A'))]
    selected: CodePoint,
    inspector: InspectorState,
    property: Option<PropertyState>,
    reference_request: Option<&'static str>,
    browse: Option<BrowseState>,
    search: Option<SearchState>,
    bookmarks: BookmarkState,
    inspecting_bookmark: bool,
    bookmark_origin: Option<CodePoint>,
    bookmark_request: Option<BookmarkRequest>,
    sequence: Option<SequenceState>,
    normalization: Option<NormalizationState>,
    showing_normalization_result: bool,
    help: HelpState,
    copy_dialog: Option<CopyDialogState>,
    glyph_preview: GlyphPreviewState,
    clipboard_request: Option<ClipboardRequest>,
    footer_status: Option<FooterStatus>,
}

impl AppState {
    pub fn with_selected(selected: CodePoint) -> Self {
        Self {
            selected,
            ..Self::default()
        }
    }

    pub fn with_sequence(source: String) -> Self {
        let sequence = SequenceState::new(source);
        let selected = sequence.selected();
        Self {
            view: View::Sequence,
            selected,
            sequence: Some(sequence),
            ..Self::default()
        }
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn view(&self) -> View {
        self.view
    }

    pub fn selected(&self) -> CodePoint {
        self.selected
    }

    pub fn property(&self) -> Option<&PropertyState> {
        self.property.as_ref()
    }

    pub fn take_reference_request(&mut self) -> Option<&'static str> {
        self.reference_request.take()
    }

    pub const fn inspector(&self) -> &InspectorState {
        &self.inspector
    }

    pub fn browse(&self) -> Option<&BrowseState> {
        self.browse.as_ref()
    }

    pub fn search(&self) -> Option<&SearchState> {
        self.search.as_ref()
    }

    pub const fn bookmarks(&self) -> &BookmarkState {
        &self.bookmarks
    }

    pub fn set_bookmarks(&mut self, entries: Vec<Bookmark>) {
        self.bookmarks.replace(entries);
    }

    pub const fn inspecting_bookmark(&self) -> bool {
        self.inspecting_bookmark
    }

    pub fn has_inspector_parent(&self) -> bool {
        self.inspecting_bookmark || self.sequence().is_some()
    }

    pub fn take_bookmark_request(&mut self) -> Option<BookmarkRequest> {
        self.bookmark_request.take()
    }

    pub fn sequence(&self) -> Option<&SequenceState> {
        if self.showing_normalization_result {
            self.normalization
                .as_ref()
                .map(|state| &state.comparison().result)
        } else {
            self.sequence.as_ref()
        }
    }

    pub fn original_sequence(&self) -> Option<&SequenceState> {
        self.sequence.as_ref()
    }

    fn sequence_mut(&mut self) -> Option<&mut SequenceState> {
        if self.showing_normalization_result {
            self.normalization
                .as_mut()
                .map(|state| &mut state.comparison_mut().result)
        } else {
            self.sequence.as_mut()
        }
    }

    pub const fn showing_normalization_result(&self) -> bool {
        self.showing_normalization_result
    }

    pub fn normalization(&self) -> Option<&NormalizationState> {
        self.normalization.as_ref()
    }

    pub const fn help(&self) -> HelpState {
        self.help
    }

    pub fn copy_dialog(&self) -> Option<&CopyDialogState> {
        self.copy_dialog.as_ref()
    }

    pub fn copy_source(&self) -> Option<CopySource<'_>> {
        match self.view {
            View::Inspector => Some(CopySource::Inspector(self.selected)),
            View::Sequence => {
                let sequence = self.sequence()?;
                Some(CopySource::Sequence {
                    analysis: sequence.analysis(),
                    selected_index: sequence.selected_index(),
                    form: self
                        .showing_normalization_result
                        .then(|| self.normalization.as_ref().unwrap().form()),
                })
            }
            View::Normalization => {
                let normalization = self.normalization.as_ref()?;
                Some(CopySource::Normalization {
                    analysis: normalization.comparison().result.analysis(),
                    form: normalization.form(),
                })
            }
            _ => None,
        }
    }

    pub const fn glyph_preview(&self) -> &GlyphPreviewState {
        &self.glyph_preview
    }

    pub fn take_clipboard_request(&mut self) -> Option<ClipboardRequest> {
        self.clipboard_request.take()
    }

    pub const fn footer_status(&self) -> Option<&FooterStatus> {
        self.footer_status.as_ref()
    }

    pub fn preview_code_point(&self) -> Option<CodePoint> {
        if self.help.is_open() || self.copy_dialog.is_some() || self.property.is_some() {
            return None;
        }

        match self.view {
            View::Inspector => Some(self.selected),
            View::Browser => self.browse.as_ref().and_then(|browse| {
                (browse.level() == BrowseLevel::CodePointTable).then(|| browse.cursor())
            }),
            View::Search => self
                .search
                .as_ref()
                .and_then(SearchState::selected_result)
                .map(|result| result.code_point()),
            View::Bookmarks => self.bookmarks.selected().map(Bookmark::code_point),
            View::Sequence => self.sequence().map(SequenceState::selected),
            View::Normalization => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn property_page_preserves_the_inspector_and_sequence_while_owning_input() {
        use crate::{inspector::InspectorMove, viewport::DocumentMove};
        let mut state = super::AppState::with_sequence("éß".to_owned());
        super::update(&mut state, super::Action::InspectSequenceCodePoint);
        let metrics = crate::ui::inspector::viewport_metrics(
            ratatui::layout::Rect::new(0, 0, 60, 16),
            &state,
        );
        super::update(
            &mut state,
            super::Action::ResizeInspectorViewport {
                viewport_height: metrics.viewport_height,
                document_height: metrics.document_height,
                field_ranges: metrics.field_ranges,
                groups: metrics.groups,
            },
        );
        super::update(
            &mut state,
            super::Action::MoveInspector(InspectorMove::Last),
        );
        let inspector = state.inspector().clone();
        let code_point = state.selected();
        super::update(&mut state, super::Action::OpenProperty);
        assert_eq!(
            state.property().unwrap().field().id(),
            crate::inspector::InspectorFieldId::UnicodeVersion
        );
        assert!(state.preview_code_point().is_none());
        super::update(
            &mut state,
            super::Action::ResizePropertyViewport {
                viewport_height: 3,
                document_height: 100,
            },
        );
        super::update(&mut state, super::Action::MoveProperty(DocumentMove::Last));
        super::update(
            &mut state,
            super::Action::MoveCodePoint(super::CodePointMove::Next),
        );
        super::update(&mut state, super::Action::ToggleInspectorProperties);
        super::update(&mut state, super::Action::ReturnToSequence);
        assert_eq!(state.selected(), code_point);
        assert_eq!(state.inspector(), &inspector);
        assert_eq!(state.view(), super::View::Inspector);
        super::update(&mut state, super::Action::ToggleHelp);
        super::update(&mut state, super::Action::CopyPropertyReference);
        super::update(&mut state, super::Action::CloseProperty);
        assert!(state.property().is_some());
        assert!(state.take_clipboard_request().is_none());
        super::update(&mut state, super::Action::CloseHelp);
        assert_eq!(
            state.property().unwrap().viewport().visible_range(),
            97..100
        );
        super::update(
            &mut state,
            super::Action::ResizePropertyViewport {
                viewport_height: 4,
                document_height: 60,
            },
        );
        assert_eq!(state.property().unwrap().viewport().visible_range(), 56..60);
        super::update(&mut state, super::Action::CloseProperty);
        assert_eq!(state.inspector(), &inspector);
        assert_eq!(state.preview_code_point(), Some(code_point));
        super::update(&mut state, super::Action::ReturnToSequence);
        assert_eq!(state.view(), super::View::Sequence);
        assert_eq!(state.sequence().unwrap().analysis().source(), "éß");
    }

    use super::*;

    #[test]
    fn bookmarks_request_persistence_before_changing_state_and_keep_status_on_resize() {
        let mut state = AppState::default();
        update(&mut state, Action::SwitchBookmarkSort);
        assert_eq!(
            state.bookmarks().sort(),
            crate::bookmarks::BookmarkSort::NewestFirst
        );
        update(&mut state, Action::ToggleBookmark);
        assert!(!state.bookmarks().contains(state.selected()));
        assert!(state.footer_status().is_none());
        let request = state.take_bookmark_request().unwrap();
        assert_eq!(
            request,
            BookmarkRequest {
                code_point: CodePoint::from('A'),
                bookmarked: true
            }
        );
        update(
            &mut state,
            Action::CompleteBookmark {
                entries: vec![Bookmark::new(
                    request.code_point,
                    jiff::Timestamp::UNIX_EPOCH,
                )],
                request,
            },
        );
        assert!(state.bookmarks().contains(state.selected()));
        assert_eq!(
            state.footer_status().unwrap().message(),
            "Bookmarked U+0041"
        );
        update(&mut state, Action::OpenBookmarks);
        update(&mut state, Action::SwitchBookmarkSort);
        assert_eq!(
            state.bookmarks().sort(),
            crate::bookmarks::BookmarkSort::OldestFirst
        );
        assert_eq!(state.preview_code_point(), Some(state.selected()));
        assert!(state.take_bookmark_request().is_none());
        update(&mut state, Action::ToggleHelp);
        update(&mut state, Action::SwitchBookmarkSort);
        assert_eq!(
            state.bookmarks().sort(),
            crate::bookmarks::BookmarkSort::OldestFirst
        );
        update(&mut state, Action::CloseHelp);
        update(&mut state, Action::ToggleBookmark);
        let request = state.take_bookmark_request().unwrap();
        assert!(!request.bookmarked);
        update(
            &mut state,
            Action::CompleteBookmark {
                entries: Vec::new(),
                request,
            },
        );
        update(&mut state, Action::ResizeBookmarksViewport(10));
        assert_eq!(
            state.footer_status().unwrap().message(),
            "Removed bookmark U+0041"
        );
        assert!(state.bookmarks().selected().is_none());
        assert!(state.preview_code_point().is_none());
        assert_eq!(
            state.bookmarks().sort(),
            crate::bookmarks::BookmarkSort::OldestFirst
        );
        update(&mut state, Action::InspectBookmark);
        assert_eq!(state.view(), View::Bookmarks);
        update(&mut state, Action::ToggleBookmark);
        assert!(state.take_bookmark_request().is_none());
    }

    #[test]
    fn bookmark_navigation_restores_its_source_and_preserves_a_sequence_parent() {
        let mut state = AppState::with_sequence("AB".to_owned());
        update(&mut state, Action::InspectSequenceCodePoint);
        state.set_bookmarks(vec![Bookmark::new(
            CodePoint::from('→'),
            jiff::Timestamp::UNIX_EPOCH,
        )]);
        update(&mut state, Action::OpenBookmarks);
        update(&mut state, Action::SwitchBookmarkSort);
        assert_eq!(state.selected(), CodePoint::from('A'));
        assert_eq!(state.preview_code_point(), Some(CodePoint::from('→')));
        update(&mut state, Action::CloseBookmarks);
        assert_eq!(state.selected(), CodePoint::from('A'));
        update(&mut state, Action::OpenBookmarks);
        assert_eq!(
            state.bookmarks().sort(),
            crate::bookmarks::BookmarkSort::OldestFirst
        );
        update(&mut state, Action::InspectBookmark);
        assert!(state.inspecting_bookmark());
        assert_eq!(state.selected(), CodePoint::from('→'));
        update(&mut state, Action::ReturnToBookmarks);
        assert_eq!(
            state.bookmarks().sort(),
            crate::bookmarks::BookmarkSort::OldestFirst
        );
        update(&mut state, Action::CloseBookmarks);
        assert!(!state.inspecting_bookmark());
        assert_eq!(state.selected(), CodePoint::from('A'));
        update(&mut state, Action::ReturnToSequence);
        assert_eq!(state.view(), View::Sequence);
        assert_eq!(state.sequence().unwrap().selected_index(), 0);
        update(&mut state, Action::ToggleBookmark);
        assert!(state.take_bookmark_request().is_none());
    }

    #[test]
    fn normalization_reference_follows_result_without_moving_the_input_cursor_or_viewport() {
        let mut state = AppState::with_sequence("ﬃ".repeat(50));
        update(&mut state, Action::ResizeSequenceViewport(4));
        for _ in 0..10 {
            update(&mut state, Action::MoveSequence(SequenceMove::Next));
        }
        let original_index = state.sequence().unwrap().selected_index();
        let original_visible = state.sequence().unwrap().visible_range();
        update(&mut state, Action::OpenNormalization);
        for _ in 0..2 {
            update(
                &mut state,
                Action::MoveNormalization(NormalizationMove::Next),
            );
        }
        update(&mut state, Action::InspectNormalizationResult);
        update(&mut state, Action::ResizeNormalizationOriginalViewport(4));
        update(&mut state, Action::ResizeSequenceViewport(4));
        let original = state.original_sequence().unwrap().analysis();
        let comparison = state.normalization().unwrap().comparison();
        assert_eq!(comparison.original_selection(original), 0..1);
        assert_eq!(comparison.original_visible_range(original), 0..4);
        update(&mut state, Action::MoveSequence(SequenceMove::Last));
        let original = state.original_sequence().unwrap().analysis();
        let comparison = state.normalization().unwrap().comparison();
        assert_eq!(comparison.original_selection(original), 49..50);
        assert_eq!(comparison.original_visible_range(original), 46..50);
        assert_eq!(
            state.original_sequence().unwrap().selected_index(),
            original_index
        );
        assert_eq!(
            state.original_sequence().unwrap().visible_range(),
            original_visible
        );
        update(&mut state, Action::ReturnToNormalization);
        update(&mut state, Action::CloseNormalization);
        assert_eq!(state.sequence().unwrap().selected_index(), original_index);
        assert_eq!(state.sequence().unwrap().visible_range(), original_visible);
    }

    #[test]
    fn reference_highlights_all_source_members_of_a_composed_grapheme() {
        let mut state = AppState::with_sequence("A\u{0301}B".to_owned());
        update(&mut state, Action::OpenNormalization);
        update(&mut state, Action::InspectNormalizationResult);
        update(&mut state, Action::ResizeNormalizationOriginalViewport(2));
        let original = state.original_sequence().unwrap().analysis();
        let comparison = state.normalization().unwrap().comparison();
        assert_eq!(comparison.original_selection(original), 0..2);
        assert_eq!(comparison.original_visible_range(original), 0..2);
        update(&mut state, Action::MoveSequence(SequenceMove::Next));
        let original = state.original_sequence().unwrap().analysis();
        let comparison = state.normalization().unwrap().comparison();
        assert_eq!(comparison.original_selection(original), 2..3);
        assert_eq!(comparison.original_visible_range(original), 1..3);
    }

    #[test]
    fn result_group_jumps_follow_normalized_boundaries_and_preserve_the_original_selection() {
        let mut state = AppState::with_sequence("A\u{0301}ﬃ👩‍💻B".to_owned());
        update(&mut state, Action::MoveSequence(SequenceMove::Next));
        update(&mut state, Action::OpenNormalization);
        update(
            &mut state,
            Action::MoveNormalization(NormalizationMove::Next),
        );
        update(
            &mut state,
            Action::MoveNormalization(NormalizationMove::Next),
        );
        update(&mut state, Action::InspectNormalizationResult);
        update(&mut state, Action::ResizeSequenceViewport(2));
        update(&mut state, Action::ResizeNormalizationOriginalViewport(2));

        for (index, original_range) in [(1, 2..3), (2, 2..3), (3, 2..3), (4, 3..6)] {
            update(&mut state, Action::MoveSequence(SequenceMove::NextGroup));
            assert_eq!(state.sequence().unwrap().selected_index(), index);
            assert_eq!(state.sequence().unwrap().visible_range(), index..index + 2);
            let original = state.original_sequence().unwrap().analysis();
            let comparison = state.normalization().unwrap().comparison();
            assert_eq!(comparison.original_selection(original), original_range);
            assert!(
                comparison
                    .original_visible_range(original)
                    .contains(&original_range.start)
            );
            assert_eq!(state.original_sequence().unwrap().selected_index(), 1);
            assert_eq!(
                state.preview_code_point(),
                Some(state.sequence().unwrap().selected())
            );
        }
        update(&mut state, Action::MoveSequence(SequenceMove::Next));
        update(
            &mut state,
            Action::MoveSequence(SequenceMove::PreviousGroup),
        );
        assert_eq!(state.sequence().unwrap().selected_index(), 3);
        update(&mut state, Action::InspectSequenceCodePoint);
        update(&mut state, Action::ReturnToSequence);
        assert_eq!(state.sequence().unwrap().selected_index(), 3);
        update(&mut state, Action::ReturnToNormalization);
        update(&mut state, Action::CloseNormalization);
        assert_eq!(state.sequence().unwrap().selected_index(), 1);
    }

    #[test]
    fn normalization_preserves_the_source_selection_and_has_no_glyph_target() {
        let mut state = AppState::with_sequence("A\u{0301} ①👩‍💻".to_owned());
        update(&mut state, Action::MoveSequence(SequenceMove::Last));
        let original = state.sequence().unwrap().analysis().clone();
        update(&mut state, Action::OpenNormalization);
        assert_eq!(state.view(), View::Normalization);
        assert_eq!(state.preview_code_point(), None);
        update(
            &mut state,
            Action::MoveNormalization(NormalizationMove::Last),
        );
        update(&mut state, Action::MoveSequence(SequenceMove::First));
        assert_eq!(state.sequence().unwrap().selected_index(), 6);
        update(&mut state, Action::CloseNormalization);
        assert_eq!(state.view(), View::Sequence);
        assert_eq!(state.sequence().unwrap().analysis(), &original);
        assert_eq!(state.sequence().unwrap().selected_index(), 6);
        update(&mut state, Action::OpenNormalization);
        assert_eq!(
            state.normalization().unwrap().form(),
            crate::unicode::text::NormalizationForm::Nfkd
        );
    }

    #[test]
    fn normalization_actions_are_scoped_to_their_views() {
        let mut state = AppState::default();
        update(&mut state, Action::OpenNormalization);
        update(
            &mut state,
            Action::MoveNormalization(NormalizationMove::Next),
        );
        update(&mut state, Action::CloseNormalization);
        update(&mut state, Action::InspectNormalizationResult);
        update(&mut state, Action::ReturnToNormalization);
        update(&mut state, Action::CopyNormalizationResult);
        assert_eq!(state.view(), View::Inspector);
        assert!(state.normalization().is_none());
        assert!(state.take_clipboard_request().is_none());
    }

    #[test]
    fn normalization_result_navigation_restores_both_cursors_and_the_form() {
        let mut state = AppState::with_sequence("A\u{0301} ①👩‍💻".to_owned());
        update(&mut state, Action::MoveSequence(SequenceMove::Last));
        update(&mut state, Action::OpenNormalization);
        for _ in 0..2 {
            update(
                &mut state,
                Action::MoveNormalization(NormalizationMove::Next),
            );
        }
        update(&mut state, Action::InspectNormalizationResult);
        assert!(state.showing_normalization_result());
        assert_eq!(state.sequence().unwrap().analysis().source(), "Á 1👩‍💻");
        update(&mut state, Action::ResizeSequenceViewport(2));
        update(&mut state, Action::MoveSequence(SequenceMove::Last));
        update(&mut state, Action::InspectSequenceCodePoint);
        assert_eq!(state.selected().value(), 0x1f4bb);
        update(&mut state, Action::MoveCodePoint(CodePointMove::Previous));
        update(&mut state, Action::ReturnToSequence);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x1f4bb);
        assert_eq!(state.sequence().unwrap().selected_index(), 5);
        assert_eq!(state.sequence().unwrap().visible_range(), 4..6);
        update(&mut state, Action::OpenNormalization);
        assert_eq!(state.view(), View::Sequence); // Do not normalize a derived result again.
        update(&mut state, Action::ReturnToNormalization);
        assert_eq!(state.view(), View::Normalization);
        assert!(!state.showing_normalization_result());
        assert_eq!(state.sequence().unwrap().selected_index(), 6);
        update(&mut state, Action::InspectNormalizationResult);
        assert_eq!(state.sequence().unwrap().selected_index(), 5);
        update(&mut state, Action::ReturnToNormalization);
        update(&mut state, Action::CloseNormalization);
        assert_eq!(
            state.sequence().unwrap().analysis().source(),
            "A\u{0301} ①👩‍💻"
        );
        assert_eq!(state.sequence().unwrap().selected_index(), 6);
    }

    #[test]
    fn each_normalization_form_keeps_its_own_result_selection() {
        let mut state = AppState::with_sequence("A\u{0301} ①".to_owned());
        update(&mut state, Action::OpenNormalization);
        update(&mut state, Action::InspectNormalizationResult);
        update(&mut state, Action::MoveSequence(SequenceMove::Last));
        update(&mut state, Action::ReturnToNormalization);
        update(
            &mut state,
            Action::MoveNormalization(NormalizationMove::Next),
        );
        update(&mut state, Action::InspectNormalizationResult);
        assert_eq!(state.sequence().unwrap().selected_index(), 0);
        update(&mut state, Action::MoveSequence(SequenceMove::Last));
        assert_eq!(state.sequence().unwrap().selected_index(), 3);
        update(&mut state, Action::ReturnToNormalization);
        update(
            &mut state,
            Action::MoveNormalization(NormalizationMove::Previous),
        );
        update(&mut state, Action::InspectNormalizationResult);
        assert_eq!(state.sequence().unwrap().selected_index(), 2);
    }

    #[test]
    fn inspecting_a_single_code_point_result_stays_in_sequence_navigation() {
        let mut state = AppState::with_sequence("A\u{0301}".to_owned());
        update(&mut state, Action::OpenNormalization);
        update(&mut state, Action::InspectNormalizationResult);
        assert_eq!(state.sequence().unwrap().code_points().len(), 1);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x00c1);
        update(&mut state, Action::InspectSequenceCodePoint);
        update(&mut state, Action::ReturnToSequence);
        update(&mut state, Action::ReturnToNormalization);
        assert_eq!(state.view(), View::Normalization);
    }

    #[test]
    fn copy_requests_preserve_exact_normalized_and_unchanged_text() {
        let source = "e\u{0301}\t\r\n\u{1b}\u{202e}①👩‍💻";
        let mut state = AppState::with_sequence(source.to_owned());
        update(&mut state, Action::OpenNormalization);
        for form in crate::unicode::text::NormalizationForm::ALL {
            update(&mut state, Action::CopyNormalizationResult);
            let request = state.take_clipboard_request().unwrap();
            assert_eq!(
                request.value(),
                Some(state.sequence().unwrap().analysis().normalized_text(form))
            );
            assert_eq!(
                request.success_message(),
                format!("Copied {} Result", form.label())
            );
            assert!(state.take_clipboard_request().is_none());
            update(
                &mut state,
                Action::MoveNormalization(NormalizationMove::Next),
            );
        }
    }

    #[test]
    fn copy_dialog_and_help_preserve_the_source_selection_and_own_input() {
        let mut state = AppState::with_sequence("A\u{0301}👩‍💻".to_owned());
        update(&mut state, Action::ResizeSequenceViewport(1));
        update(&mut state, Action::MoveSequence(SequenceMove::Next));
        let original_visible = state.sequence().unwrap().visible_range();
        update(&mut state, Action::OpenCopyDialog);
        assert_eq!(state.preview_code_point(), None);
        update(&mut state, Action::MoveCopyDialog(CopyMove::Next));
        update(&mut state, Action::ToggleHelp);
        update(&mut state, Action::MoveCopyDialog(CopyMove::Last));
        update(&mut state, Action::CloseCopyDialog);
        update(&mut state, Action::MoveSequence(SequenceMove::Last));
        update(&mut state, Action::CopyDialogSelection);
        assert!(state.take_clipboard_request().is_none());
        assert_eq!(state.copy_dialog().unwrap().selected_index(), 1);
        update(&mut state, Action::CloseHelp);
        update(&mut state, Action::CopyDialogSelection);
        let request = state.take_clipboard_request().unwrap();
        assert_eq!(request.value(), Some("A\u{0301}"));
        assert!(request.is_from_copy_dialog());
        update(
            &mut state,
            Action::CompleteCopyDialog(FooterStatus::info(request.success_message())),
        );
        assert!(state.copy_dialog().is_none());
        assert_eq!(state.sequence().unwrap().selected_index(), 1);
        assert_eq!(state.sequence().unwrap().visible_range(), original_visible);
        assert_eq!(
            state.preview_code_point(),
            Some(state.sequence().unwrap().selected())
        );
    }

    #[test]
    fn copy_dialog_uses_the_normalized_result_and_leaves_the_original_untouched() {
        let mut state = AppState::with_sequence("A\u{0301}ﬃ".to_owned());
        update(&mut state, Action::MoveSequence(SequenceMove::Last));
        update(&mut state, Action::OpenNormalization);
        update(
            &mut state,
            Action::MoveNormalization(NormalizationMove::Next),
        );
        update(
            &mut state,
            Action::MoveNormalization(NormalizationMove::Next),
        );
        update(&mut state, Action::InspectNormalizationResult);
        update(&mut state, Action::MoveSequence(SequenceMove::Next));
        update(&mut state, Action::OpenCopyDialog);
        update(&mut state, Action::CopyDialogSelection);
        assert_eq!(state.take_clipboard_request().unwrap().value(), Some("f"));
        update(&mut state, Action::MoveCopyDialog(CopyMove::Next));
        update(&mut state, Action::CopyDialogSelection);
        assert_eq!(state.take_clipboard_request().unwrap().value(), Some("f"));
        update(&mut state, Action::MoveCopyDialog(CopyMove::Last));
        update(&mut state, Action::CopyDialogSelection);
        let request = state.take_clipboard_request().unwrap();
        assert_eq!(request.value(), Some("Áffi"));
        assert!(request.success_message().contains("NFKC Result"));
        update(&mut state, Action::CloseCopyDialog);
        assert!(state.showing_normalization_result());
        assert_eq!(state.sequence().unwrap().selected_index(), 1);
        assert_eq!(
            state.original_sequence().unwrap().analysis().source(),
            "A\u{0301}ﬃ"
        );
        assert_eq!(state.original_sequence().unwrap().selected_index(), 2);
    }

    #[test]
    fn copy_dialog_supports_single_code_point_results() {
        let mut state = AppState::with_sequence("A\u{0301}".to_owned());
        update(&mut state, Action::OpenNormalization);
        update(&mut state, Action::OpenCopyDialog);
        assert!(state.copy_dialog().is_none());
        update(&mut state, Action::InspectNormalizationResult);
        update(&mut state, Action::OpenCopyDialog);
        for _ in 0..3 {
            update(&mut state, Action::CopyDialogSelection);
            assert_eq!(state.take_clipboard_request().unwrap().value(), Some("Á"));
            update(&mut state, Action::MoveCopyDialog(CopyMove::Next));
        }
    }

    #[test]
    fn inspector_copy_formats_ignore_the_selected_property_and_preserve_its_quick_copy() {
        let mut state = AppState::with_selected(CodePoint::from('😀'));
        resize_inspector(&mut state, 5);
        update(&mut state, Action::MoveInspector(InspectorMove::Last));
        let field_index = state.inspector().selected_index();
        update(&mut state, Action::CopyInspectorValue);
        let property = state.take_clipboard_request().unwrap();
        assert_eq!(property.label(), "Unicode Version");
        update(&mut state, Action::OpenCopyDialog);
        for expected in [
            "😀",
            r#""\uD83D\uDE00""#,
            r#""\u{1F600}""#,
            "U+1F600",
            "F0 9F 98 80",
            "D83D DE00",
        ] {
            update(&mut state, Action::MoveCopyDialog(CopyMove::Last));
            update(&mut state, Action::CopyDialogSelection);
            assert_eq!(
                state.take_clipboard_request().unwrap().value(),
                Some(expected)
            );
            update(&mut state, Action::MoveCopyDialog(CopyMove::NextFormat));
        }
        update(&mut state, Action::CloseCopyDialog);
        assert_eq!(state.inspector().selected_index(), field_index);
        update(&mut state, Action::CopyInspectorValue);
        assert_eq!(state.take_clipboard_request().unwrap(), property);
    }

    #[test]
    fn sequence_inspector_copies_its_current_code_point_instead_of_the_parent_selection() {
        let mut state = AppState::with_sequence("A\u{301}".to_owned());
        update(&mut state, Action::MoveSequence(SequenceMove::Next));
        update(&mut state, Action::InspectSequenceCodePoint);
        update(&mut state, Action::MoveCodePoint(CodePointMove::Next));
        update(&mut state, Action::OpenCopyDialog);
        update(&mut state, Action::CopyDialogSelection);
        assert_eq!(
            state.take_clipboard_request().unwrap().value(),
            Some("\u{302}")
        );
        update(&mut state, Action::CloseCopyDialog);
        update(&mut state, Action::ReturnToSequence);
        assert_eq!(state.sequence().unwrap().selected().value(), 0x301);
    }

    #[test]
    fn inspector_surrogate_copy_is_fixed_to_code_point_notation() {
        let mut state = AppState::with_selected(CodePoint::new(0xd800).unwrap());
        update(&mut state, Action::OpenCopyDialog);
        update(&mut state, Action::MoveCopyDialog(CopyMove::NextFormat));
        update(&mut state, Action::MoveCopyDialog(CopyMove::Last));
        update(&mut state, Action::CopyDialogSelection);
        assert_eq!(
            state.take_clipboard_request().unwrap().value(),
            Some("U+D800")
        );
    }

    #[test]
    fn starts_running() {
        let state = AppState::default();

        assert!(state.is_running());
        assert_eq!(state.selected().value(), 0x0041);
        assert_eq!(state.inspector().offset(), 0);
        assert_eq!(state.browse(), None);
        assert!(state.search().is_none());
        assert!(!state.help().is_open());
        assert_eq!(
            state.glyph_preview().status(),
            crate::preview::GlyphPreviewStatus::Detecting
        );
    }

    #[test]
    fn help_opens_without_changing_the_base_view_and_resets_when_closed() {
        let mut state = AppState::default();
        update(&mut state, Action::OpenSearch);

        update(&mut state, Action::ToggleHelp);
        update(
            &mut state,
            Action::ResizeHelpViewport {
                viewport_height: 5,
                document_height: 20,
            },
        );
        update(&mut state, Action::MoveHelp(HelpMove::Last));

        assert!(state.help().is_open());
        assert_eq!(state.help().offset(), 15);
        assert_eq!(state.view(), View::Search);
        assert_eq!(state.preview_code_point(), None);

        update(&mut state, Action::CloseHelp);

        assert!(!state.help().is_open());
        assert_eq!(state.help().offset(), 0);
        assert_eq!(state.view(), View::Search);
    }

    #[test]
    fn quit_action_stops_the_app() {
        let mut state = AppState::default();

        update(&mut state, Action::Quit);

        assert!(!state.is_running());
    }

    #[test]
    fn preview_target_follows_only_code_point_selections() {
        let mut state = AppState::default();
        assert_eq!(state.preview_code_point().unwrap().value(), 0x0041);

        update(&mut state, Action::OpenBrowser(BrowseLevel::Plane));
        assert_eq!(state.preview_code_point(), None);
        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.preview_code_point(), None);
        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x0041);
        update(&mut state, Action::AdvanceBrowser);

        update(&mut state, Action::OpenSearch);
        assert_eq!(state.preview_code_point(), None);
        edit_search_query(&mut state, "U+2192");
        assert_eq!(state.preview_code_point().unwrap().value(), 0x2192);
        assert_eq!(state.selected().value(), 0x0041);
    }

    #[test]
    fn starts_with_the_requested_selection() {
        let selected = CodePoint::new(0x1f600).unwrap();
        let state = AppState::with_selected(selected);

        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected(), selected);
    }

    #[test]
    fn inspector_movement_is_limited_to_the_inspector() {
        let mut state = AppState::default();
        resize_inspector(&mut state, 5);

        update(
            &mut state,
            Action::MoveInspector(InspectorMove::PageForward),
        );
        assert_eq!(state.inspector().selected_index(), 5);
        assert_eq!(state.inspector().offset(), 1);

        update(&mut state, Action::OpenSearch);
        update(&mut state, Action::MoveInspector(InspectorMove::NextField));
        assert_eq!(state.inspector().selected_index(), 5);
        assert_eq!(state.inspector().offset(), 1);
    }

    #[test]
    fn inspector_moves_to_adjacent_code_points_and_preserves_the_selected_field() {
        let mut state = AppState::with_selected(CodePoint::new(0xd7ff).unwrap());
        resize_inspector(&mut state, 5);
        update(&mut state, Action::MoveInspector(InspectorMove::Last));

        update(&mut state, Action::MoveCodePoint(CodePointMove::Next));
        assert_eq!(state.selected().value(), 0xd800);
        assert_eq!(
            selected_inspector_field(&state),
            crate::inspector::InspectorFieldId::UnicodeVersion
        );
        assert_eq!(state.inspector().offset(), 12);

        update(&mut state, Action::MoveCodePoint(CodePointMove::Previous));
        assert_eq!(state.selected().value(), 0xd7ff);
    }

    #[test]
    fn property_modes_preserve_selection_and_copy_and_survive_other_views() {
        use crate::inspector::{InspectorDisplayMode, InspectorField, InspectorFieldId};

        let mut state = AppState::with_selected(CodePoint::from('!'));
        assert_eq!(
            state.inspector().display_mode(),
            InspectorDisplayMode::Relevant
        );
        resize_inspector(&mut state, 5);
        let fields =
            InspectorField::for_code_point(state.selected(), state.inspector().display_mode());
        let index = fields
            .iter()
            .position(|field| field.id() == InspectorFieldId::GeneralCategory)
            .unwrap();
        for _ in 0..index {
            update(&mut state, Action::MoveInspector(InspectorMove::NextField));
        }
        for _ in 0..2 {
            update(&mut state, Action::ToggleInspectorProperties);
            update(&mut state, Action::CopyInspectorValue);
            let request = state.take_clipboard_request().unwrap();
            assert_eq!(request.label(), "General Category");
            assert_eq!(request.value(), Some("Po — Other Punctuation"));
        }
        update(&mut state, Action::MoveCodePoint(CodePointMove::Previous));
        update(&mut state, Action::CopyInspectorValue);
        assert_eq!(
            state.take_clipboard_request().unwrap().value(),
            Some("Zs — Space Separator")
        );

        update(&mut state, Action::ToggleInspectorProperties);
        update(&mut state, Action::OpenSearch);
        update(&mut state, Action::ToggleInspectorProperties);
        update(&mut state, Action::CloseSearch);
        assert_eq!(state.inspector().display_mode(), InspectorDisplayMode::All);
        update(&mut state, Action::CopyInspectorValue);
        assert_eq!(
            state.take_clipboard_request().unwrap().label(),
            "General Category"
        );
    }

    #[test]
    fn adjacent_code_point_movement_stops_at_the_unicode_code_space_boundaries() {
        let mut first = AppState::with_selected(CodePoint::new(CodePoint::MIN_VALUE).unwrap());
        let mut last = AppState::with_selected(CodePoint::new(CodePoint::MAX_VALUE).unwrap());

        update(&mut first, Action::MoveCodePoint(CodePointMove::Previous));
        update(&mut last, Action::MoveCodePoint(CodePointMove::Next));

        assert_eq!(first.selected().value(), CodePoint::MIN_VALUE);
        assert_eq!(last.selected().value(), CodePoint::MAX_VALUE);
    }

    #[test]
    fn adjacent_code_point_movement_is_limited_to_the_inspector() {
        let selected = CodePoint::new(0x0041).unwrap();
        let mut browser = AppState::with_selected(selected);
        let mut search = AppState::with_selected(selected);
        update(&mut browser, Action::OpenBrowser(BrowseLevel::Plane));
        update(&mut search, Action::OpenSearch);

        update(&mut browser, Action::MoveCodePoint(CodePointMove::Next));
        update(&mut search, Action::MoveCodePoint(CodePointMove::Next));

        assert_eq!(browser.selected(), selected);
        assert_eq!(search.selected(), selected);
    }

    #[test]
    fn search_inspection_preserves_the_selected_inspector_field() {
        let mut state = AppState::default();
        resize_inspector(&mut state, 5);
        update(&mut state, Action::MoveInspector(InspectorMove::Last));
        let offset = state.inspector().offset();

        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "A");
        update(&mut state, Action::InspectSearchResult);
        assert_eq!(state.selected().value(), 0x0041);
        assert_eq!(state.inspector().offset(), offset);

        update(&mut state, Action::OpenSearch);
        update(&mut state, Action::EditSearch(InputRequest::DeleteLine));
        edit_search_query(&mut state, "U+2192");
        update(&mut state, Action::InspectSearchResult);
        assert_eq!(state.selected().value(), 0x2192);
        assert_eq!(
            selected_inspector_field(&state),
            crate::inspector::InspectorFieldId::UnicodeVersion
        );
        assert_eq!(state.inspector().offset(), offset);
    }

    #[test]
    fn selection_change_invalidates_a_prepared_glyph_preview_only_when_needed() {
        use crate::{
            glyph::CanvasSize,
            graphics::{GraphicsAvailability, GraphicsProtocol},
            image::kitty::ImageId,
            preview::{GlyphPreviewGeometry, GlyphPreviewStatus},
        };

        let mut state = AppState::default();
        let image_id = ImageId::new(7).unwrap();
        let canvas = CanvasSize::new(80, 64).unwrap();
        let geometry = GlyphPreviewGeometry::new(10, 4, canvas);
        update(
            &mut state,
            Action::UpdateGlyphPreview(GlyphPreviewUpdate::Configure {
                availability: GraphicsAvailability::Available(GraphicsProtocol::Kitty),
                image_id: Some(image_id),
            }),
        );
        update(
            &mut state,
            Action::UpdateGlyphPreview(GlyphPreviewUpdate::Prepared {
                image_id: Some(image_id),
                geometry,
                status: GlyphPreviewStatus::Ready,
                font: None,
            }),
        );

        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "A");
        update(&mut state, Action::InspectSearchResult);
        assert_eq!(state.glyph_preview().status(), GlyphPreviewStatus::Ready);

        update(&mut state, Action::MoveCodePoint(CodePointMove::Next));
        assert_eq!(state.selected().value(), 0x0042);
        assert_eq!(state.glyph_preview().status(), GlyphPreviewStatus::Pending);
        assert_eq!(state.glyph_preview().geometry(), Some(geometry));
        assert_eq!(state.glyph_preview().image_id(), Some(image_id));
    }

    #[test]
    fn closing_and_reopening_other_views_preserves_the_inspector_offset() {
        let mut state = AppState::default();
        resize_inspector(&mut state, 5);
        update(&mut state, Action::MoveInspector(InspectorMove::Last));
        let offset = state.inspector().offset();

        update(&mut state, Action::OpenSearch);
        update(&mut state, Action::CloseSearch);
        assert_eq!(state.inspector().offset(), offset);

        update(&mut state, Action::OpenBrowser(BrowseLevel::Plane));
        update(&mut state, Action::CloseBrowser);
        assert_eq!(state.inspector().offset(), offset);
    }

    #[test]
    fn opening_a_browser_level_uses_the_shared_selection() {
        let selected = CodePoint::new(0x1_23ab).unwrap();
        let mut state = AppState::with_selected(selected);

        update(&mut state, Action::OpenBrowser(BrowseLevel::CodePointTable));

        assert_eq!(state.view(), View::Browser);
        assert_eq!(state.browse().unwrap().level(), BrowseLevel::CodePointTable);
        assert_eq!(state.browse().unwrap().cursor(), selected);
        assert_eq!(state.selected(), selected);
    }

    #[test]
    fn plane_and_range_movement_do_not_change_the_shared_selection() {
        let selected = CodePoint::new(0x1_23ab).unwrap();
        let mut state = AppState::with_selected(selected);
        update(&mut state, Action::OpenBrowser(BrowseLevel::Plane));

        update(&mut state, Action::MoveBrowser(BrowseMove::Down));
        assert_eq!(state.browse().unwrap().cursor().value(), 0x2_23ab);
        assert_eq!(state.selected(), selected);

        update(&mut state, Action::AdvanceBrowser);
        update(&mut state, Action::MoveBrowser(BrowseMove::Down));
        assert_eq!(state.browse().unwrap().cursor().value(), 0x2_24ab);
        assert_eq!(state.selected(), selected);
    }

    #[test]
    fn table_movement_is_tentative_until_enter() {
        let mut state = AppState::with_selected(CodePoint::new(0x0041).unwrap());
        resize_inspector(&mut state, 5);
        update(&mut state, Action::MoveInspector(InspectorMove::Last));
        let offset = state.inspector().offset();
        update(&mut state, Action::OpenBrowser(BrowseLevel::Plane));
        update(&mut state, Action::AdvanceBrowser);
        update(&mut state, Action::MoveBrowser(BrowseMove::Down));

        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.selected().value(), 0x0041);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x0141);
        assert_eq!(
            selected_inspector_field(&state),
            crate::inspector::InspectorFieldId::UnicodeVersion
        );
        assert_eq!(state.inspector().offset(), offset);

        update(&mut state, Action::MoveBrowser(BrowseMove::Right));
        assert_eq!(state.browse().unwrap().cursor().value(), 0x0142);
        assert_eq!(state.selected().value(), 0x0041);
        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.selected().value(), 0x0142);
        assert_eq!(state.view(), View::Inspector);
    }

    #[test]
    fn closing_or_backing_out_of_browse_discards_the_tentative_selection() {
        let original = CodePoint::new(0x0041).unwrap();
        let mut state = AppState::with_selected(original);
        update(&mut state, Action::OpenBrowser(BrowseLevel::CodePointTable));
        update(&mut state, Action::MoveBrowser(BrowseMove::Right));
        assert_eq!(state.preview_code_point().unwrap().value(), 0x0042);
        update(&mut state, Action::CloseBrowser);
        assert_eq!(state.selected(), original);
        assert_eq!(state.preview_code_point(), Some(original));

        update(&mut state, Action::OpenBrowser(BrowseLevel::CodePointTable));
        update(&mut state, Action::MoveBrowser(BrowseMove::Right));
        update(&mut state, Action::BackBrowser);
        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected(), original);
        assert_eq!(state.preview_code_point(), Some(original));
    }

    #[test]
    fn moving_a_tentative_cursor_invalidates_its_glyph_preview() {
        use crate::{
            glyph::CanvasSize,
            graphics::{GraphicsAvailability, GraphicsProtocol},
            preview::{GlyphPreviewGeometry, GlyphPreviewStatus},
        };

        let mut state = AppState::default();
        update(&mut state, Action::OpenBrowser(BrowseLevel::CodePointTable));
        update(
            &mut state,
            Action::UpdateGlyphPreview(GlyphPreviewUpdate::Configure {
                availability: GraphicsAvailability::Available(GraphicsProtocol::Kitty),
                image_id: None,
            }),
        );
        update(
            &mut state,
            Action::UpdateGlyphPreview(GlyphPreviewUpdate::Prepared {
                image_id: None,
                geometry: GlyphPreviewGeometry::new(10, 4, CanvasSize::new(80, 64).unwrap()),
                status: GlyphPreviewStatus::Ready,
                font: None,
            }),
        );

        update(&mut state, Action::MoveBrowser(BrowseMove::Right));
        assert_eq!(state.glyph_preview().status(), GlyphPreviewStatus::Pending);
        assert_eq!(state.selected().value(), 0x0041);
    }

    #[test]
    fn backing_from_a_table_preserves_its_tentative_cursor_in_the_previous_list() {
        let original = CodePoint::new(0x0041).unwrap();
        let mut state = AppState::with_selected(original);
        update(&mut state, Action::OpenBrowser(BrowseLevel::Range));
        update(&mut state, Action::MoveBrowser(BrowseMove::Down));
        update(&mut state, Action::AdvanceBrowser);
        update(&mut state, Action::MoveBrowser(BrowseMove::Right));
        update(&mut state, Action::BackBrowser);

        assert_eq!(state.browse().unwrap().level(), BrowseLevel::Range);
        assert_eq!(state.browse().unwrap().cursor().value(), 0x0142);
        assert_eq!(state.selected(), original);
    }

    #[test]
    fn block_browse_from_an_unmapped_code_point_commits_only_on_table_enter() {
        let original = CodePoint::new(0x2fe0).unwrap();
        let mut state = AppState::with_selected(original);
        update(&mut state, Action::OpenBrowser(BrowseLevel::Block));
        assert_eq!(state.browse().unwrap().cursor().value(), 0x2ff0);
        assert_eq!(state.selected(), original);
        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x2ff0);
        update(&mut state, Action::MoveBrowser(BrowseMove::Right));
        update(&mut state, Action::BackBrowser);
        assert_eq!(state.browse().unwrap().level(), BrowseLevel::Block);
        assert_eq!(state.selected(), original);
        update(&mut state, Action::AdvanceBrowser);
        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected().value(), 0x2ff1);
    }

    #[test]
    fn browser_enter_and_back_actions_follow_the_hierarchy() {
        let mut state = AppState::default();
        update(&mut state, Action::OpenBrowser(BrowseLevel::Plane));

        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.browse().unwrap().level(), BrowseLevel::Range);
        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.browse().unwrap().level(), BrowseLevel::CodePointTable);
        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.view(), View::Inspector);

        update(&mut state, Action::OpenBrowser(BrowseLevel::Plane));
        update(&mut state, Action::AdvanceBrowser);
        update(&mut state, Action::AdvanceBrowser);
        update(&mut state, Action::BackBrowser);
        assert_eq!(state.browse().unwrap().level(), BrowseLevel::Range);
        update(&mut state, Action::BackBrowser);
        assert_eq!(state.browse().unwrap().level(), BrowseLevel::Plane);
        update(&mut state, Action::BackBrowser);
        assert_eq!(state.view(), View::Inspector);

        update(&mut state, Action::OpenBrowser(BrowseLevel::CodePointTable));
        update(&mut state, Action::BackBrowser);
        assert_eq!(state.view(), View::Inspector);

        update(&mut state, Action::OpenBrowser(BrowseLevel::Range));
        update(&mut state, Action::AdvanceBrowser);
        update(&mut state, Action::BackBrowser);
        assert_eq!(state.browse().unwrap().level(), BrowseLevel::Range);
        update(&mut state, Action::BackBrowser);
        assert_eq!(state.view(), View::Inspector);
    }

    #[test]
    fn reopening_code_points_uses_the_selection_from_search() {
        let mut state = AppState::default();
        update(&mut state, Action::OpenBrowser(BrowseLevel::CodePointTable));
        update(&mut state, Action::MoveBrowser(BrowseMove::Right));
        assert_eq!(state.selected().value(), 0x0041);
        update(&mut state, Action::AdvanceBrowser);
        assert_eq!(state.selected().value(), 0x0042);

        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "→");
        update(&mut state, Action::InspectSearchResult);
        assert_eq!(state.selected().value(), 0x2192);

        update(&mut state, Action::OpenBrowser(BrowseLevel::CodePointTable));

        assert_eq!(state.view(), View::Browser);
        assert_eq!(state.browse().unwrap().level(), BrowseLevel::CodePointTable);
        assert_eq!(state.browse().unwrap().cursor().value(), 0x2192);
        assert_eq!(state.selected().value(), 0x2192);
    }

    #[test]
    fn first_open_lazily_initializes_an_empty_search_without_changing_selection() {
        let selected = CodePoint::new(0x1f600).unwrap();
        let mut state = AppState::with_selected(selected);

        update(&mut state, Action::OpenSearch);

        assert_eq!(state.view(), View::Search);
        assert_eq!(state.selected(), selected);
        assert_eq!(state.search().unwrap().input().value(), "");
        assert_eq!(state.search().unwrap().selected_result(), None);
    }

    #[test]
    fn sequence_selection_opens_the_inspector_and_returns_to_the_same_position() {
        let mut state = AppState::with_sequence("A→B".to_owned());
        update(&mut state, Action::ResizeSequenceViewport(2));
        update(&mut state, Action::MoveSequence(SequenceMove::Next));

        update(&mut state, Action::InspectSequenceCodePoint);
        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected().value(), 0x2192);

        update(&mut state, Action::MoveCodePoint(CodePointMove::Next));
        assert_eq!(state.selected().value(), 0x2193);
        update(&mut state, Action::ReturnToSequence);

        assert_eq!(state.view(), View::Sequence);
        assert_eq!(state.sequence().unwrap().selected_index(), 1);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x2192);
    }

    #[test]
    fn search_from_a_sequence_inspector_does_not_change_the_sequence_position() {
        let mut state = AppState::with_sequence("A→B".to_owned());
        update(&mut state, Action::MoveSequence(SequenceMove::Next));
        update(&mut state, Action::InspectSequenceCodePoint);
        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "Ω");
        update(&mut state, Action::InspectSearchResult);

        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected().value(), 0x03a9);
        assert_eq!(state.sequence().unwrap().selected_index(), 1);

        update(&mut state, Action::ReturnToSequence);
        assert_eq!(state.view(), View::Sequence);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x2192);
    }

    #[test]
    fn inspecting_and_searching_preserve_the_original_sequence_analysis() {
        let mut state = AppState::with_sequence("A\u{0301} 👩‍💻".to_owned());
        let analysis = state.sequence().unwrap().analysis().clone();
        update(&mut state, Action::ResizeSequenceViewport(2));
        update(&mut state, Action::MoveSequence(SequenceMove::Last));
        update(&mut state, Action::MoveSequence(SequenceMove::Previous));
        update(&mut state, Action::InspectSequenceCodePoint);
        assert_eq!(state.selected().value(), 0x200d);

        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "Ω");
        update(&mut state, Action::InspectSearchResult);
        assert_eq!(state.selected().value(), 0x03a9);
        update(&mut state, Action::ReturnToSequence);

        assert_eq!(state.view(), View::Sequence);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x200d);
        let sequence = state.sequence().unwrap();
        assert_eq!(sequence.selected_index(), 4);
        assert_eq!(sequence.visible_range(), 4..6);
        assert_eq!(sequence.analysis(), &analysis);
    }

    #[test]
    fn browse_from_a_sequence_inspector_returns_through_the_inspector() {
        let mut state = AppState::with_sequence("AB".to_owned());
        update(&mut state, Action::InspectSequenceCodePoint);
        update(&mut state, Action::OpenBrowser(BrowseLevel::CodePointTable));
        update(&mut state, Action::MoveBrowser(BrowseMove::Right));
        update(&mut state, Action::AdvanceBrowser);

        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected().value(), 0x0042);
        assert_eq!(state.sequence().unwrap().selected_index(), 0);

        update(&mut state, Action::ReturnToSequence);
        assert_eq!(state.view(), View::Sequence);
        assert_eq!(state.preview_code_point().unwrap().value(), 0x0041);
    }

    #[test]
    fn search_cannot_open_from_the_browser() {
        let mut browser = AppState::default();
        update(&mut browser, Action::OpenBrowser(BrowseLevel::Plane));

        update(&mut browser, Action::OpenSearch);

        assert_eq!(browser.view(), View::Browser);
        assert!(browser.search().is_none());
    }

    #[test]
    fn editing_and_moving_search_results_do_not_change_the_shared_selection() {
        let mut state = AppState::default();
        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "rightwards arrow");
        let first = state.search().unwrap().selected_result().unwrap();

        update(&mut state, Action::MoveSearch(SearchMove::Next));

        assert_ne!(state.search().unwrap().selected_result(), Some(first));
        assert_eq!(state.selected().value(), 0x0041);
    }

    #[test]
    fn inspecting_a_search_result_updates_the_shared_selection_and_closes_search() {
        let mut state = AppState::default();
        resize_inspector(&mut state, 5);
        update(&mut state, Action::MoveInspector(InspectorMove::Last));
        let offset = state.inspector().offset();
        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "→");

        update(&mut state, Action::InspectSearchResult);

        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected().value(), 0x2192);
        assert_eq!(
            selected_inspector_field(&state),
            crate::inspector::InspectorFieldId::UnicodeVersion
        );
        assert_eq!(state.inspector().offset(), offset);
        assert_eq!(state.search().unwrap().input().value(), "→");
    }

    #[test]
    fn search_accepts_short_explicit_code_point_notation() {
        let mut state = AppState::default();
        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "U+A");

        update(&mut state, Action::InspectSearchResult);

        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected().value(), 0x000a);
    }

    #[test]
    fn inspecting_without_a_result_keeps_search_open_and_preserves_selection() {
        let mut state = AppState::default();
        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "not a unicode name");

        update(&mut state, Action::InspectSearchResult);

        assert_eq!(state.view(), View::Search);
        assert_eq!(state.selected().value(), 0x0041);
    }

    #[test]
    fn closing_and_reopening_search_retains_query_selection_and_viewport() {
        let mut state = AppState::default();
        update(&mut state, Action::OpenSearch);
        edit_search_query(&mut state, "rightwards arrow");
        update(&mut state, Action::ResizeSearchViewport(2));
        update(&mut state, Action::MoveSearch(SearchMove::Next));
        update(&mut state, Action::MoveSearch(SearchMove::Next));
        let selected = state.search().unwrap().selected_result();
        let visible = state.search().unwrap().visible_result_range();

        update(&mut state, Action::CloseSearch);
        assert_eq!(state.view(), View::Inspector);
        assert_eq!(state.selected().value(), 0x0041);
        update(&mut state, Action::OpenSearch);

        assert_eq!(state.view(), View::Search);
        assert_eq!(state.search().unwrap().input().value(), "rightwards arrow");
        assert_eq!(state.search().unwrap().selected_result(), selected);
        assert_eq!(state.search().unwrap().visible_result_range(), visible);
    }

    #[test]
    fn footer_status_survives_internal_updates_and_clears_on_the_next_user_action() {
        let mut state = AppState::default();
        update(
            &mut state,
            Action::ShowFooterStatus(FooterStatus::info("Copied Code Point")),
        );

        resize_inspector(&mut state, 10);
        update(
            &mut state,
            Action::UpdateGlyphPreview(GlyphPreviewUpdate::Hidden),
        );
        assert_eq!(
            state.footer_status().map(FooterStatus::message),
            Some("Copied Code Point")
        );

        update(&mut state, Action::MoveInspector(InspectorMove::NextField));
        assert_eq!(state.footer_status(), None);
    }

    fn selected_inspector_field(state: &AppState) -> crate::inspector::InspectorFieldId {
        crate::inspector::InspectorField::for_code_point(
            state.selected(),
            state.inspector().display_mode(),
        )[state.inspector().selected_index()]
        .id()
    }

    fn resize_inspector(state: &mut AppState, viewport_height: usize) {
        let document_height = crate::inspector::InspectorField::for_code_point(
            state.selected(),
            state.inspector().display_mode(),
        )
        .len();
        update(
            state,
            Action::ResizeInspectorViewport {
                viewport_height,
                document_height,
                field_ranges: (0..document_height).map(|index| index..index + 1).collect(),
                groups: Vec::new(),
            },
        );
    }

    fn edit_search_query(state: &mut AppState, value: &str) {
        for character in value.chars() {
            update(
                state,
                Action::EditSearch(InputRequest::InsertChar(character)),
            );
        }
    }
}
