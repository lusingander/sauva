use std::{
    fmt,
    io::{self, Write},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use flate2::{Compression, write::ZlibEncoder};
use ratatui::{crossterm::style::Colored, style::Color};

use crate::image::{RgbaImage, kitty_diacritics::ROW_COLUMN_DIACRITICS};

const PLACEHOLDER: char = '\u{10eeee}';
const MAX_IMAGE_ID: u32 = 0x00ff_ffff;
const PAYLOAD_CHUNK_SIZE: usize = 4096;
// A stable non-zero ID makes a resized virtual placement replace its predecessor.
// This also avoids anonymous-placement accumulation in affected Ghostty releases.
const VIRTUAL_PLACEMENT_ID: u32 = 1;

pub const MAX_PLACEHOLDER_DIMENSION: u16 = ROW_COLUMN_DIACRITICS.len() as u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageId(u32);

impl ImageId {
    pub const fn new(value: u32) -> Option<Self> {
        if value == 0 || value > MAX_IMAGE_ID {
            None
        } else {
            Some(Self(value))
        }
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VirtualPlacement {
    columns: u16,
    rows: u16,
}

impl VirtualPlacement {
    pub const fn new(columns: u16, rows: u16) -> Result<Self, KittyImageError> {
        if columns == 0 || rows == 0 {
            return Err(KittyImageError::ZeroPlacementDimension);
        }
        if columns > MAX_PLACEHOLDER_DIMENSION || rows > MAX_PLACEHOLDER_DIMENSION {
            return Err(KittyImageError::PlacementDimensionLimit {
                columns,
                rows,
                maximum: MAX_PLACEHOLDER_DIMENSION,
            });
        }
        Ok(Self { columns, rows })
    }

    pub const fn columns(self) -> u16 {
        self.columns
    }

    pub const fn rows(self) -> u16 {
        self.rows
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KittyImageError {
    ZeroPlacementDimension,
    PlacementDimensionLimit {
        columns: u16,
        rows: u16,
        maximum: u16,
    },
}

impl fmt::Display for KittyImageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroPlacementDimension => {
                write!(formatter, "placement dimensions must be non-zero")
            }
            Self::PlacementDimensionLimit {
                columns,
                rows,
                maximum,
            } => write!(
                formatter,
                "placement {columns}x{rows} exceeds the placeholder limit {maximum}x{maximum}"
            ),
        }
    }
}

impl std::error::Error for KittyImageError {}

pub fn transmit_rgba(
    writer: &mut impl Write,
    image_id: ImageId,
    image: RgbaImage<'_>,
    placement: VirtualPlacement,
) -> io::Result<()> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(image.pixels())?;
    let compressed = encoder.finish()?;
    let encoded = STANDARD.encode(compressed);
    let chunks = encoded.as_bytes().chunks(PAYLOAD_CHUNK_SIZE);
    let chunk_count = chunks.len();

    for (index, chunk) in chunks.enumerate() {
        let more = u8::from(index + 1 < chunk_count);
        if index == 0 {
            write!(
                writer,
                "\x1b_Ga=T,f=32,s={},v={},i={},p={VIRTUAL_PLACEMENT_ID},U=1,c={},r={},o=z,m={more},q=2;",
                image.width(),
                image.height(),
                image_id.value(),
                placement.columns(),
                placement.rows(),
            )?;
        } else {
            write!(writer, "\x1b_Gm={more},q=2;")?;
        }
        writer.write_all(chunk)?;
        writer.write_all(b"\x1b\\")?;
    }
    writer.flush()
}

pub fn delete(writer: &mut impl Write, image_id: ImageId) -> io::Result<()> {
    write!(writer, "\x1b_Ga=d,d=I,i={},q=2\x1b\\", image_id.value())?;
    writer.flush()
}

pub fn placeholder(row: u16, column: u16) -> Option<String> {
    let row = *ROW_COLUMN_DIACRITICS.get(usize::from(row))?;
    let column = *ROW_COLUMN_DIACRITICS.get(usize::from(column))?;
    Some(format!("{PLACEHOLDER}{row}{column}"))
}

pub const fn image_id_color(image_id: ImageId) -> Color {
    let [_, red, green, blue] = image_id.value().to_be_bytes();
    Color::Rgb(red, green, blue)
}

/// Keeps ANSI colors enabled while a Unicode placeholder is rendered.
///
/// Kitty uses the foreground color as image metadata, so suppressing colors via
/// `NO_COLOR` would make the placeholder lose its image ID.
pub fn enable_placeholder_colors() -> impl Drop {
    let were_disabled = Colored::ansi_color_disabled_memoized();
    Colored::set_ansi_color_disabled(false);
    ColorOutputGuard { were_disabled }
}

struct ColorOutputGuard {
    were_disabled: bool,
}

impl Drop for ColorOutputGuard {
    fn drop(&mut self) {
        Colored::set_ansi_color_disabled(self.were_disabled);
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use flate2::read::ZlibDecoder;
    use ratatui::style::Color;
    use rstest::rstest;

    use crate::image::{
        RgbaImage,
        kitty::{
            ImageId, KittyImageError, MAX_PLACEHOLDER_DIMENSION, PAYLOAD_CHUNK_SIZE,
            VirtualPlacement, delete, image_id_color, placeholder, transmit_rgba,
        },
    };

    const TEST_IMAGE_ID: ImageId = match ImageId::new(0x55_45_58) {
        Some(image_id) => image_id,
        None => panic!("the test image ID must be valid"),
    };
    const TEST_PLACEMENT: VirtualPlacement = match VirtualPlacement::new(2, 2) {
        Ok(placement) => placement,
        Err(_) => panic!("the test placement must be valid"),
    };
    const TEST_RGBA: [u8; 16] = [
        255, 0, 0, 255, // opaque red
        0, 255, 0, 128, // translucent green
        0, 0, 255, 64, // translucent blue
        255, 255, 0, 0, // transparent yellow
    ];

    #[rstest]
    #[case(0)]
    #[case(0x0100_0000)]
    #[case(u32::MAX)]
    fn rejects_image_ids_that_cannot_be_encoded_in_the_placeholder_color(#[case] value: u32) {
        assert_eq!(ImageId::new(value), None);
    }

    #[test]
    fn accepts_image_id_boundaries() {
        assert_eq!(ImageId::new(1).map(ImageId::value), Some(1));
        assert_eq!(
            ImageId::new(0x00ff_ffff).map(ImageId::value),
            Some(0x00ff_ffff)
        );
    }

    #[rustfmt::skip]
    #[rstest]
    #[case(
        0,
        2,
        KittyImageError::ZeroPlacementDimension
    )]
    #[case(
        2,
        0,
        KittyImageError::ZeroPlacementDimension
    )]
    #[case(
        MAX_PLACEHOLDER_DIMENSION + 1,
        2,
        KittyImageError::PlacementDimensionLimit {
            columns: MAX_PLACEHOLDER_DIMENSION + 1,
            rows: 2,
            maximum: MAX_PLACEHOLDER_DIMENSION,
        }
    )]
    fn rejects_invalid_virtual_placements(
        #[case] columns: u16,
        #[case] rows: u16,
        #[case] expected: KittyImageError,
    ) {
        assert_eq!(VirtualPlacement::new(columns, rows).unwrap_err(), expected);
    }

    #[test]
    fn transmits_compressed_rgba() {
        let mut output = Vec::new();
        let image = RgbaImage::new(2, 2, &TEST_RGBA).unwrap();

        transmit_rgba(&mut output, TEST_IMAGE_ID, image, TEST_PLACEMENT).unwrap();

        let packets = parse_packets(&output);
        assert_eq!(packets.len(), 1);
        assert_eq!(
            packets[0].0,
            "a=T,f=32,s=2,v=2,i=5588312,p=1,U=1,c=2,r=2,o=z,m=0,q=2"
        );
        assert_eq!(decompress_payload(&packets), TEST_RGBA);
    }

    #[test]
    fn chunks_a_large_compressed_payload_into_protocol_sized_packets() {
        let pixels = pseudo_random_bytes(192 * 128 * 4);
        let image = RgbaImage::new(192, 128, &pixels).unwrap();
        let mut output = Vec::new();

        transmit_rgba(&mut output, TEST_IMAGE_ID, image, TEST_PLACEMENT).unwrap();

        let packets = parse_packets(&output);
        assert!(packets.len() > 1);
        assert_eq!(
            packets[0].0,
            "a=T,f=32,s=192,v=128,i=5588312,p=1,U=1,c=2,r=2,o=z,m=1,q=2"
        );
        for (index, (control, payload)) in packets.iter().enumerate() {
            assert!(payload.len() <= PAYLOAD_CHUNK_SIZE);
            if index + 1 < packets.len() {
                assert_eq!(payload.len() % 4, 0);
                assert!(control.ends_with("m=1,q=2"));
            } else {
                assert_eq!(control, "m=0,q=2");
            }
            if index > 0 && index + 1 < packets.len() {
                assert_eq!(control, "m=1,q=2");
            }
        }
        assert_eq!(decompress_payload(&packets), pixels);
    }

    #[test]
    fn deletes_the_image_and_its_placements() {
        let mut output = Vec::new();

        delete(&mut output, TEST_IMAGE_ID).unwrap();

        assert_eq!(
            String::from_utf8(output).unwrap(),
            "\x1b_Ga=d,d=I,i=5588312,q=2\x1b\\"
        );
    }

    #[test]
    fn builds_placeholders_across_the_official_diacritic_table() {
        assert_eq!(placeholder(0, 0), Some("\u{10eeee}\u{0305}\u{0305}".into()));
        assert_eq!(
            placeholder(MAX_PLACEHOLDER_DIMENSION - 1, MAX_PLACEHOLDER_DIMENSION - 1),
            Some("\u{10eeee}\u{1d244}\u{1d244}".into())
        );
        assert_eq!(placeholder(MAX_PLACEHOLDER_DIMENSION, 0), None);
        assert_eq!(placeholder(0, MAX_PLACEHOLDER_DIMENSION), None);
        assert_eq!(MAX_PLACEHOLDER_DIMENSION, 297);
    }

    #[test]
    fn encodes_the_image_id_as_placeholder_foreground_color() {
        assert_eq!(image_id_color(TEST_IMAGE_ID), Color::Rgb(0x55, 0x45, 0x58));
    }

    fn parse_packets(output: &[u8]) -> Vec<(String, Vec<u8>)> {
        output
            .split(|byte| *byte == b'\\')
            .filter(|packet| !packet.is_empty())
            .map(|packet| {
                assert!(packet.starts_with(b"\x1b_G"));
                assert_eq!(packet.last(), Some(&0x1b));
                let body = &packet[3..packet.len() - 1];
                let separator = body.iter().position(|byte| *byte == b';').unwrap();
                (
                    String::from_utf8(body[..separator].to_vec()).unwrap(),
                    body[separator + 1..].to_vec(),
                )
            })
            .collect()
    }

    fn decompress_payload(packets: &[(String, Vec<u8>)]) -> Vec<u8> {
        let encoded: Vec<u8> = packets
            .iter()
            .flat_map(|(_, payload)| payload.iter().copied())
            .collect();
        let compressed = STANDARD.decode(encoded).unwrap();
        let mut decoder = ZlibDecoder::new(Cursor::new(compressed));
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded).unwrap();
        decoded
    }

    fn pseudo_random_bytes(length: usize) -> Vec<u8> {
        let mut state = 0x1234_5678_u32;
        (0..length)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                state as u8
            })
            .collect()
    }
}
