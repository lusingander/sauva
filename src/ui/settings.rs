use garde::Validate;
use ratatui::text::Line;
use serde::{Deserialize, Serialize};
use umbra::optional;

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)],
    visibility = pub
)]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Validate)]
pub struct UiSettings {
    #[garde(custom(validate_input_cursor))]
    pub input_cursor: InputCursor,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InputCursor {
    #[default]
    Native,
    Text(String),
}

fn validate_input_cursor(value: &InputCursor, _: &()) -> garde::Result {
    let InputCursor::Text(text) = value else {
        return Ok(());
    };
    if text.chars().any(char::is_control) || Line::from(text.as_str()).width() != 1 {
        return Err(garde::Error::new(
            "input cursor text must occupy exactly one terminal cell",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_text_input_cursor_width() {
        assert!(UiSettings::default().validate().is_ok());
        for text in ["|", "▏", " "] {
            let settings = UiSettings {
                input_cursor: InputCursor::Text(text.to_owned()),
            };
            assert!(settings.validate().is_ok(), "{text:?}");
        }
        for text in ["", "\n", "\u{301}", "界", "||"] {
            let settings = UiSettings {
                input_cursor: InputCursor::Text(text.to_owned()),
            };
            assert!(settings.validate().is_err(), "{text:?}");
        }
    }
}
