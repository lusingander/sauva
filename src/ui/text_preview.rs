use crate::unicode::{CodePoint, GeneralCategory, UnicodeDatabase};

/// Escape terminal controls and invisible formatting while preserving shaping
/// within clusters. This representation is never used as clipboard data.
pub(super) fn safe_cluster(text: &str) -> String {
    use GeneralCategory as G;
    if text.len() > 1024 {
        return format!("<cluster: {} CP>", text.chars().count());
    }
    let has_base = text.chars().any(|character| {
        let record = UnicodeDatabase::lookup(CodePoint::from(character));
        !record.is_default_ignorable()
            && !matches!(
                record.general_category(),
                G::NonspacingMark
                    | G::SpacingMark
                    | G::EnclosingMark
                    | G::Control
                    | G::Format
                    | G::SpaceSeparator
                    | G::LineSeparator
                    | G::ParagraphSeparator
                    | G::Unassigned
                    | G::PrivateUse
            )
    });
    let mut output = String::new();
    for character in text.chars() {
        let point = CodePoint::from(character);
        let record = UnicodeDatabase::lookup(point);
        let shaping =
            matches!(point.value(), 0x200c | 0x200d | 0xfe00..=0xfe0f | 0xe0100..=0xe01ef);
        match character {
            ' ' => output.push('␠'),
            '\t' => output.push_str("<TAB>"),
            '\r' => output.push_str("<CR>"),
            '\n' => output.push_str("<LF>"),
            _ if shaping && has_base => output.push(character),
            _ if record.is_default_ignorable() => output.push_str(&format!("<{point}>")),
            _ if matches!(
                record.general_category(),
                G::NonspacingMark | G::SpacingMark | G::EnclosingMark
            ) =>
            {
                if output.is_empty() && !has_base {
                    output.push('◌');
                }
                output.push(character);
            }
            _ => {
                let display = record.display_representation();
                if display.as_str() == character.to_string() {
                    output.push(character);
                } else {
                    output.push_str(&format!("<{point}>"));
                }
            }
        }
    }
    output
}
