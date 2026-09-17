use std::fmt;

pub mod iterm2;
pub mod iterm2_manager;
pub mod kitty;
mod kitty_diacritics;
pub mod manager;

#[derive(Debug, Clone, Copy)]
pub struct RgbaImage<'a> {
    width: u32,
    height: u32,
    pixels: &'a [u8],
}

impl<'a> RgbaImage<'a> {
    pub fn new(width: u32, height: u32, pixels: &'a [u8]) -> Result<Self, RgbaImageError> {
        if width == 0 || height == 0 {
            return Err(RgbaImageError::ZeroDimension);
        }
        let expected = usize::try_from(width)
            .ok()
            .and_then(|width| usize::try_from(height).ok().map(|height| (width, height)))
            .and_then(|(width, height)| width.checked_mul(height))
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(RgbaImageError::DimensionsOverflow)?;
        if pixels.len() != expected {
            return Err(RgbaImageError::PixelLengthMismatch {
                expected,
                actual: pixels.len(),
            });
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub const fn width(self) -> u32 {
        self.width
    }

    pub const fn height(self) -> u32 {
        self.height
    }

    pub const fn pixels(self) -> &'a [u8] {
        self.pixels
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RgbaImageError {
    ZeroDimension,
    DimensionsOverflow,
    PixelLengthMismatch { expected: usize, actual: usize },
}

impl fmt::Display for RgbaImageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroDimension => write!(formatter, "image dimensions must be non-zero"),
            Self::DimensionsOverflow => write!(formatter, "image dimensions are too large"),
            Self::PixelLengthMismatch { expected, actual } => write!(
                formatter,
                "RGBA pixel length mismatch: expected {expected} bytes, got {actual}"
            ),
        }
    }
}

impl std::error::Error for RgbaImageError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ImageKey<Content, Placement> {
    // The caller must include every input that can change the encoded pixels or dimensions.
    content: Content,
    placement: Placement,
}

impl<Content, Placement> ImageKey<Content, Placement> {
    const fn new(content: Content, placement: Placement) -> Self {
        Self { content, placement }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::image::{RgbaImage, RgbaImageError};

    #[rustfmt::skip]
    #[rstest]
    #[case(0, 2, &[], RgbaImageError::ZeroDimension)]
    #[case(2, 0, &[], RgbaImageError::ZeroDimension)]
    #[case(u32::MAX, u32::MAX, &[], RgbaImageError::DimensionsOverflow)]
    #[case(
        2,
        2,
        &[0; 15],
        RgbaImageError::PixelLengthMismatch {
            expected: 16,
            actual: 15,
        }
    )]
    fn rejects_invalid_rgba_images(
        #[case] width: u32,
        #[case] height: u32,
        #[case] pixels: &[u8],
        #[case] expected: RgbaImageError,
    ) {
        assert_eq!(RgbaImage::new(width, height, pixels).unwrap_err(), expected);
    }
}
