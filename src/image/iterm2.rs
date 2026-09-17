use std::{fmt::Write as _, io};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use ratatui::layout::Rect;

use crate::image::RgbaImage;

pub fn encode_rgba(image: RgbaImage<'_>, area: Rect, generation: u64) -> io::Result<String> {
    let png = encode_png(image)?;
    let name = STANDARD.encode(format!("sauva-{generation}"));
    let mut sequence = String::from("\x1b7");

    for _ in 0..area.height {
        write!(sequence, "\x1b[{}X\x1b[1B", area.width).expect("writing to a String cannot fail");
    }
    write!(sequence, "\x1b[{}A", area.height).expect("writing to a String cannot fail");
    write!(
        sequence,
        "\x1b]1337;File=name={name};inline=1;size={};width={};height={};preserveAspectRatio=0:",
        png.len(),
        area.width,
        area.height,
    )
    .expect("writing to a String cannot fail");
    sequence.push_str(&STANDARD.encode(png));
    sequence.push('\x07');
    sequence.push_str("\x1b8");
    Ok(sequence)
}

fn encode_png(image: RgbaImage<'_>) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut output, image.width(), image.height());
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut png = encoder.write_header().map_err(io::Error::other)?;
        png.write_image_data(image.pixels())
            .map_err(io::Error::other)?;
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use ratatui::layout::Rect;

    use crate::image::{RgbaImage, iterm2::encode_rgba};

    const RGBA: [u8; 16] = [
        255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 64, 255, 255, 0, 0,
    ];

    #[test]
    fn encodes_png_with_cell_erasure_and_placement_dimensions() {
        let image = RgbaImage::new(2, 2, &RGBA).unwrap();

        let sequence = encode_rgba(image, Rect::new(3, 4, 4, 2), 7).unwrap();

        assert!(sequence.starts_with("\x1b7\x1b[4X\x1b[1B\x1b[4X\x1b[1B\x1b[2A\x1b]1337;"));
        assert!(sequence.contains("File=name=c2F1dmEtNw==;inline=1;size="));
        assert!(sequence.contains(";width=4;height=2;preserveAspectRatio=0:"));
        assert!(sequence.ends_with("\x07\x1b8"));

        let encoded = sequence
            .rsplit_once(':')
            .expect("the inline image sequence must have a payload")
            .1
            .trim_end_matches("\x07\x1b8");
        let png = STANDARD.decode(encoded).unwrap();
        let decoder = png::Decoder::new(Cursor::new(png));
        let mut reader = decoder.read_info().unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut pixels).unwrap();
        assert_eq!((info.width, info.height), (2, 2));
        assert_eq!(&pixels[..info.buffer_size()], RGBA);
    }
}
