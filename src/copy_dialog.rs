use crate::sequence::SequenceState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyTarget {
    CodePoint,
    Grapheme,
    WholeText,
}

impl CopyTarget {
    pub const ALL: [Self; 3] = [Self::CodePoint, Self::Grapheme, Self::WholeText];

    pub const fn label(self) -> &'static str {
        match self {
            Self::CodePoint => "Code Point",
            Self::Grapheme => "Grapheme Cluster",
            Self::WholeText => "Whole Text",
        }
    }

    /// Borrow exact source text; display aids and escaping belong to the UI.
    pub fn text(self, sequence: &SequenceState) -> &str {
        let analysis = sequence.analysis();
        let point = &sequence.code_points()[sequence.selected_index()];
        match self {
            Self::CodePoint => &analysis.source()[point.byte_range()],
            Self::Grapheme => {
                &analysis.source()[analysis.graphemes()[point.grapheme_index()].byte_range()]
            }
            Self::WholeText => analysis.source(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyMove {
    Previous,
    Next,
    First,
    Last,
}

#[derive(Debug, Clone, Default)]
pub struct CopyDialogState {
    selected_index: usize,
    error: Option<String>,
}

impl CopyDialogState {
    pub const fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn selected_target(&self) -> CopyTarget {
        CopyTarget::ALL[self.selected_index]
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn move_selection(&mut self, movement: CopyMove) {
        let last = CopyTarget::ALL.len() - 1;
        self.selected_index = match movement {
            CopyMove::Previous => self.selected_index.saturating_sub(1),
            CopyMove::Next => (self.selected_index + 1).min(last),
            CopyMove::First => 0,
            CopyMove::Last => last,
        };
        self.clear_error();
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::sequence::SequenceMove;

    #[rstest]
    #[case("A\u{0301}")]
    #[case("\u{0301}\u{0300}")]
    #[case("👩🏽‍💻")]
    #[case("🇯🇵")]
    #[case("❤️")]
    #[case("\r\n")]
    fn copies_the_whole_cluster_from_each_member_even_outside_the_viewport(#[case] cluster: &str) {
        let source = format!("\n{cluster}Y");
        let mut sequence = SequenceState::new(source.clone());
        sequence.resize_viewport(1);
        for member in cluster.chars() {
            sequence.move_selection(SequenceMove::Next);
            assert_eq!(CopyTarget::CodePoint.text(&sequence), member.to_string());
            assert_eq!(CopyTarget::Grapheme.text(&sequence), cluster);
            assert_eq!(CopyTarget::WholeText.text(&sequence), source);
        }
    }

    #[test]
    fn distinguishes_repeated_clusters_and_preserves_all_controls() {
        let source = "A\u{0301}\t\r\n\u{1b}\u{202e}A\u{0301}";
        let mut sequence = SequenceState::new(source.to_owned());
        sequence.move_selection(SequenceMove::Last);
        assert_eq!(CopyTarget::CodePoint.text(&sequence), "\u{0301}");
        assert_eq!(CopyTarget::Grapheme.text(&sequence), "A\u{0301}");
        assert_eq!(CopyTarget::WholeText.text(&sequence), source);
    }
}
