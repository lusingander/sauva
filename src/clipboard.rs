use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardError {
    Unavailable,
    Busy,
    Failed,
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Unavailable => "Clipboard is unavailable",
            Self::Busy => "Clipboard is busy",
            Self::Failed => "Could not copy to clipboard",
        })
    }
}

pub trait ClipboardWriter {
    fn write_text(&mut self, text: &str) -> Result<(), ClipboardError>;
}

#[derive(Default)]
pub struct SystemClipboard {
    clipboard: Option<arboard::Clipboard>,
}

impl SystemClipboard {
    pub const fn new() -> Self {
        Self { clipboard: None }
    }
}

impl ClipboardWriter for SystemClipboard {
    fn write_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        if self.clipboard.is_none() {
            self.clipboard = Some(arboard::Clipboard::new().map_err(map_error)?);
        }
        self.clipboard
            .as_mut()
            .expect("the clipboard was initialized above")
            .set_text(text.to_owned())
            .map_err(map_error)
    }
}

fn map_error(error: arboard::Error) -> ClipboardError {
    match error {
        arboard::Error::ClipboardNotSupported => ClipboardError::Unavailable,
        arboard::Error::ClipboardOccupied => ClipboardError::Busy,
        _ => ClipboardError::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_actionable_arboard_errors_to_stable_messages() {
        assert_eq!(
            map_error(arboard::Error::ClipboardNotSupported),
            ClipboardError::Unavailable
        );
        assert_eq!(
            map_error(arboard::Error::ClipboardOccupied),
            ClipboardError::Busy
        );
        assert_eq!(
            map_error(arboard::Error::ConversionFailure),
            ClipboardError::Failed
        );
    }
}
