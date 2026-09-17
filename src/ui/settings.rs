use garde::Validate;
use ratatui::text::Line;
use serde::Deserialize;
use umbra::optional;

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)],
    visibility = pub
)]
#[derive(Debug, Clone, Default, PartialEq, Eq, Validate)]
pub struct UiSettings {
    #[garde(custom(validate_selection_cursor))]
    pub selection_cursor: String,
}

impl UiSettings {
    pub fn selection_marker(&self, selected: bool) -> &str {
        if selected && !self.selection_cursor.is_empty() {
            &self.selection_cursor
        } else {
            " "
        }
    }
}

fn validate_selection_cursor(value: &str, _: &()) -> garde::Result {
    if value.is_empty() {
        return Ok(());
    }
    if value.chars().any(char::is_control) || Line::from(value).width() != 1 {
        return Err(garde::Error::new(
            "selection cursor must be empty or occupy exactly one terminal cell",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_an_empty_selection_cursor() {
        assert_eq!(UiSettings::default().selection_cursor, "");
    }

    #[test]
    fn accepts_an_empty_or_single_cell_selection_cursor() {
        for selection_cursor in ["", ">", "▸", " "] {
            let settings = UiSettings {
                selection_cursor: selection_cursor.to_owned(),
            };

            assert!(settings.validate().is_ok(), "{selection_cursor:?}");
        }
    }

    #[test]
    fn rejects_unsafe_or_non_single_cell_selection_cursors() {
        for selection_cursor in ["\n", "\u{301}", "界", ">>"] {
            let settings = UiSettings {
                selection_cursor: selection_cursor.to_owned(),
            };

            assert!(settings.validate().is_err(), "{selection_cursor:?}");
        }
    }

    #[test]
    fn selection_marker_reserves_one_cell_when_hidden() {
        let hidden = UiSettings::default();
        let visible = UiSettings {
            selection_cursor: "▸".to_owned(),
        };

        assert_eq!(hidden.selection_marker(true), " ");
        assert_eq!(visible.selection_marker(false), " ");
        assert_eq!(visible.selection_marker(true), "▸");
    }
}
