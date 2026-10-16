use garde::Validate;
use jiff::{Timestamp, fmt::strtime, tz::TimeZone};
use ratatui::text::Line;
use serde::{Deserialize, Serialize};
use umbra::optional;

#[optional(
    derives = [Debug, Deserialize],
    attrs = [serde(deny_unknown_fields)],
    visibility = pub
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Validate)]
pub struct UiSettings {
    #[garde(custom(validate_input_cursor))]
    pub input_cursor: InputCursor,
    #[garde(custom(validate_bookmark_date_format))]
    pub bookmark_date_format: String,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            input_cursor: InputCursor::default(),
            bookmark_date_format: "%Y-%m-%d %H:%M %Z".to_owned(),
        }
    }
}

pub fn format_bookmark_date(
    timestamp: Timestamp,
    format: &str,
    time_zone: TimeZone,
) -> Result<String, jiff::Error> {
    strtime::format(format, &timestamp.to_zoned(time_zone))
}

fn validate_bookmark_date_format(value: &str, _: &()) -> garde::Result {
    let formatted = format_bookmark_date(Timestamp::UNIX_EPOCH, value, TimeZone::UTC)
        .map_err(|error| garde::Error::new(format!("invalid strftime format: {error}")))?;
    if formatted.trim().is_empty() || formatted.chars().any(char::is_control) {
        return Err(garde::Error::new(
            "bookmark date format must produce nonempty text without control characters",
        ));
    }
    Ok(())
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
    fn bookmark_dates_support_strftime_and_reject_invalid_or_multiline_formats() {
        let timestamp: Timestamp = "2026-10-16T05:32:10Z".parse().unwrap();
        let zone = TimeZone::get("Asia/Tokyo").unwrap();
        assert_eq!(
            format_bookmark_date(
                timestamp,
                &UiSettings::default().bookmark_date_format,
                zone.clone()
            )
            .unwrap(),
            "2026-10-16 14:32 JST"
        );
        assert_eq!(
            format_bookmark_date(timestamp, "%d/%m/%Y %H:%M:%S %:z", zone).unwrap(),
            "16/10/2026 14:32:10 +09:00"
        );
        for format in ["", " ", "%", "%?", "%n", "%t", "%Y\n%m", "\u{1b}[31m%Y"] {
            let settings = UiSettings {
                bookmark_date_format: format.to_owned(),
                ..UiSettings::default()
            };
            assert!(settings.validate().is_err(), "{format:?}");
        }
        let settings = UiSettings {
            bookmark_date_format: "%F %T %z".to_owned(),
            ..UiSettings::default()
        };
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validates_text_input_cursor_width() {
        assert!(UiSettings::default().validate().is_ok());
        for text in ["|", "▏", " "] {
            let settings = UiSettings {
                input_cursor: InputCursor::Text(text.to_owned()),
                ..UiSettings::default()
            };
            assert!(settings.validate().is_ok(), "{text:?}");
        }
        for text in ["", "\n", "\u{301}", "界", "||"] {
            let settings = UiSettings {
                input_cursor: InputCursor::Text(text.to_owned()),
                ..UiSettings::default()
            };
            assert!(settings.validate().is_err(), "{text:?}");
        }
    }
}
