pub mod cache;
pub mod font;
pub mod runtime;
pub mod settings;

use std::{fmt, sync::Arc};

use swash::{
    FontRef, GlyphId,
    scale::{Render, ScaleContext, Source, StrikeWith, image::Content},
    shape::{Direction, ShapeContext},
    text::{BidiClass, Codepoint as _, Script},
    zeno::{Format, Vector},
};

use crate::{
    glyph::{
        font::{FontCollection, FontInfo, ResolvedFont},
        settings::{GlyphPreviewSettings, RgbaColor},
    },
    unicode::{CodePoint, GeneralCategory, UnicodeDatabase},
};

const PADDING_FRACTION: f32 = 0.1;
const INITIAL_EM_FRACTION: f32 = 0.8;
const FIT_SAFETY_FACTOR: f32 = 0.98;
const MAX_FIT_ATTEMPTS: usize = 4;

pub const MAX_CANVAS_DIMENSION: u16 = 1024;
pub const MAX_CANVAS_PIXELS: usize = 1024 * 1024;
pub const DEFAULT_CACHE_MAX_ENTRIES: usize = 8;
pub const DEFAULT_CACHE_MAX_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanvasSize {
    width: u16,
    height: u16,
}

impl CanvasSize {
    pub fn new(width: u16, height: u16) -> Result<Self, CanvasSizeError> {
        if width == 0 || height == 0 {
            return Err(CanvasSizeError::ZeroDimension);
        }
        if width > MAX_CANVAS_DIMENSION || height > MAX_CANVAS_DIMENSION {
            return Err(CanvasSizeError::DimensionLimit {
                width,
                height,
                maximum: MAX_CANVAS_DIMENSION,
            });
        }

        let pixels = usize::from(width) * usize::from(height);
        if pixels > MAX_CANVAS_PIXELS {
            return Err(CanvasSizeError::PixelLimit {
                pixels,
                maximum: MAX_CANVAS_PIXELS,
            });
        }

        Ok(Self { width, height })
    }

    pub const fn width(self) -> u16 {
        self.width
    }

    pub const fn height(self) -> u16 {
        self.height
    }

    const fn pixel_count(self) -> usize {
        self.width as usize * self.height as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasSizeError {
    ZeroDimension,
    DimensionLimit {
        width: u16,
        height: u16,
        maximum: u16,
    },
    PixelLimit {
        pixels: usize,
        maximum: usize,
    },
}

impl fmt::Display for CanvasSizeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroDimension => formatter.write_str("glyph canvas dimensions must be non-zero"),
            Self::DimensionLimit {
                width,
                height,
                maximum,
            } => write!(
                formatter,
                "glyph canvas {width}x{height} exceeds the {maximum}px dimension limit"
            ),
            Self::PixelLimit { pixels, maximum } => write!(
                formatter,
                "glyph canvas has {pixels} pixels, exceeding the {maximum} pixel limit"
            ),
        }
    }
}

impl std::error::Error for CanvasSizeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphRenderError {
    GlyphDoesNotFit,
}

impl fmt::Display for GlyphRenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GlyphDoesNotFit => {
                formatter.write_str("the rendered glyph does not fit the requested canvas")
            }
        }
    }
}

impl std::error::Error for GlyphRenderError {}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PixelBounds {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphImage {
    size: CanvasSize,
    rgba: Vec<u8>,
    font: Arc<FontInfo>,
}

impl GlyphImage {
    pub const fn size(&self) -> CanvasSize {
        self.size
    }

    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    pub fn font(&self) -> &Arc<FontInfo> {
        &self.font
    }

    fn byte_len(&self) -> usize {
        self.rgba.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlyphPreview {
    Ready(GlyphImage),
    CombiningContext(GlyphImage),
    Blank,
    Missing,
    NotScalar,
}

impl GlyphPreview {
    fn byte_len(&self) -> usize {
        match self {
            Self::Ready(image) | Self::CombiningContext(image) => image.byte_len(),
            Self::Blank | Self::Missing | Self::NotScalar => 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PositionedGlyph {
    id: GlyphId,
    x: f32,
    y: f32,
    advance: f32,
    attached_mark: bool,
}

#[derive(Debug, Clone)]
struct RasterizedGlyph {
    left: i32,
    top: i32,
    width: u32,
    height: u32,
    pixels: RasterizedPixels,
}

#[derive(Debug, Clone)]
enum RasterizedPixels {
    Mask(Vec<u8>),
    Color(Vec<u8>),
}

#[derive(Debug, Clone)]
struct RasterizedRun {
    glyphs: Vec<RasterizedGlyph>,
    advance: f32,
    bounds: RunBounds,
}

#[derive(Debug, Clone, Copy)]
struct RunBounds {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl RunBounds {
    fn width(self) -> u32 {
        self.right.saturating_sub(self.left) as u32
    }

    fn height(self) -> u32 {
        self.bottom.saturating_sub(self.top) as u32
    }
}

pub struct GlyphRenderer {
    fonts: FontCollection,
    shape_context: ShapeContext,
    scale_context: ScaleContext,
    settings: GlyphPreviewSettings,
}

impl GlyphRenderer {
    pub fn new(settings: GlyphPreviewSettings) -> Result<Self, GlyphRenderError> {
        let fonts = FontCollection::new(&settings)?;
        Ok(Self {
            fonts,
            shape_context: ShapeContext::new(),
            scale_context: ScaleContext::new(),
            settings,
        })
    }

    pub fn render(
        &mut self,
        code_point: CodePoint,
        size: CanvasSize,
    ) -> Result<GlyphPreview, GlyphRenderError> {
        let Some(character) = code_point.to_char() else {
            return Ok(GlyphPreview::NotScalar);
        };

        let context = render_context(code_point);
        let text = character.to_string();
        let script = shaping_script(character);
        let direction = shaping_direction(character);
        let insert_dotted_circle = context == GlyphRenderContext::DottedCircle;
        let Some(font) = self.fonts.resolve(character, insert_dotted_circle) else {
            return Ok(GlyphPreview::Missing);
        };

        let available_width = padded_extent(size.width());
        let available_height = padded_extent(size.height());
        let mut pixels_per_em = f32::from(size.width().min(size.height())) * INITIAL_EM_FRACTION;
        let mut run = self.rasterize_run(
            &font,
            &text,
            script,
            direction,
            insert_dotted_circle,
            pixels_per_em,
        );

        for _ in 0..MAX_FIT_ATTEMPTS {
            let Some(current) = run.as_ref() else {
                return Ok(GlyphPreview::Blank);
            };
            let layout_width = layout_width(current);
            let layout_height = current.bounds.height();
            if layout_width <= available_width && layout_height <= available_height {
                break;
            }

            let width_scale = available_width as f32 / layout_width.max(1) as f32;
            let height_scale = available_height as f32 / layout_height.max(1) as f32;
            pixels_per_em *= width_scale.min(height_scale) * FIT_SAFETY_FACTOR;
            run = self.rasterize_run(
                &font,
                &text,
                script,
                direction,
                insert_dotted_circle,
                pixels_per_em,
            );
        }

        let Some(run) = run else {
            return Ok(GlyphPreview::Blank);
        };
        if layout_width(&run) > available_width || run.bounds.height() > available_height {
            return Err(GlyphRenderError::GlyphDoesNotFit);
        }

        let image = compose_run(size, &run, &self.settings, Arc::clone(font.info()))?;
        Ok(if context == GlyphRenderContext::DottedCircle {
            GlyphPreview::CombiningContext(image)
        } else {
            GlyphPreview::Ready(image)
        })
    }

    fn rasterize_run(
        &mut self,
        font: &ResolvedFont,
        text: &str,
        script: Script,
        direction: Direction,
        insert_dotted_circle: bool,
        pixels_per_em: f32,
    ) -> Option<RasterizedRun> {
        let fonts = &self.fonts;
        let shape_context = &mut self.shape_context;
        let scale_context = &mut self.scale_context;
        let fg = self.settings.fg;
        fonts
            .with_font(font, |font| {
                rasterize_run(
                    shape_context,
                    scale_context,
                    font,
                    text,
                    script,
                    direction,
                    insert_dotted_circle,
                    pixels_per_em,
                    fg,
                )
            })
            .flatten()
    }
}

#[allow(clippy::too_many_arguments)]
fn rasterize_run(
    shape_context: &mut ShapeContext,
    scale_context: &mut ScaleContext,
    font: FontRef<'_>,
    text: &str,
    script: Script,
    direction: Direction,
    insert_dotted_circle: bool,
    pixels_per_em: f32,
    fg: RgbaColor,
) -> Option<RasterizedRun> {
    let mut glyphs = Vec::new();
    let mut pen_x = 0.0;
    let mut shaper = shape_context
        .builder(font)
        .script(script)
        .direction(direction)
        .insert_dotted_circles(insert_dotted_circle)
        .size(pixels_per_em)
        .build();
    shaper.add_str(text);
    shaper.shape_with(|cluster| {
        for glyph in cluster.glyphs {
            glyphs.push(PositionedGlyph {
                id: glyph.id,
                x: pen_x + glyph.x,
                y: glyph.y,
                advance: glyph.advance,
                attached_mark: glyph.info.is_mark(),
            });
            pen_x += glyph.advance;
        }
    });

    let manually_position_dotted_circle = insert_dotted_circle
        && glyphs.len() >= 2
        && glyphs[1..].iter().all(|glyph| !glyph.attached_mark);
    if manually_position_dotted_circle {
        let dotted_circle_advance = glyphs[0].advance;
        for glyph in &mut glyphs[1..] {
            glyph.x -= dotted_circle_advance;
        }
        pen_x = dotted_circle_advance;
    }

    let mut rasterized = Vec::new();
    let mut scaler = scale_context
        .builder(font)
        .size(pixels_per_em)
        .hint(true)
        .build();
    let mut renderer = Render::new(&[
        Source::ColorOutline(0),
        Source::ColorBitmap(StrikeWith::BestFit),
        Source::Outline,
    ]);
    renderer.format(Format::Alpha);
    renderer.default_color(fg.channels());

    for glyph in glyphs {
        renderer.offset(Vector::new(glyph.x.fract(), glyph.y.fract()));
        let Some(image) = renderer.render(&mut scaler, glyph.id) else {
            continue;
        };
        let pixels = match image.content {
            Content::Mask if image.data.iter().any(|&alpha| alpha != 0) => {
                RasterizedPixels::Mask(image.data)
            }
            Content::Color
                if image
                    .data
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .any(|channels| channels[3] != 0) =>
            {
                RasterizedPixels::Color(image.data)
            }
            Content::Mask | Content::Color | Content::SubpixelMask => continue,
        };

        let left = glyph.x.floor() as i32 + image.placement.left;
        let top = -(glyph.y.floor() as i32 + image.placement.top);
        rasterized.push(RasterizedGlyph {
            left,
            top,
            width: image.placement.width,
            height: image.placement.height,
            pixels,
        });
    }

    if manually_position_dotted_circle && rasterized.len() >= 2 {
        let dotted_circle = &rasterized[0];
        let dotted_circle_center =
            dotted_circle.left + i32::try_from(dotted_circle.width / 2).unwrap_or(i32::MAX);
        let mark_bounds = rasterized_bounds(&rasterized[1..])?;
        let mark_center =
            mark_bounds.left + i32::try_from(mark_bounds.width() / 2).unwrap_or(i32::MAX);
        let shift = dotted_circle_center.saturating_sub(mark_center);
        for glyph in &mut rasterized[1..] {
            glyph.left = glyph.left.saturating_add(shift);
        }
    }

    let bounds = rasterized_bounds(&rasterized)?;

    Some(RasterizedRun {
        glyphs: rasterized,
        advance: pen_x,
        bounds,
    })
}

fn rasterized_bounds(glyphs: &[RasterizedGlyph]) -> Option<RunBounds> {
    glyphs.iter().fold(None, |bounds, glyph| {
        let right = glyph.left.saturating_add(glyph.width as i32);
        let bottom = glyph.top.saturating_add(glyph.height as i32);
        Some(match bounds {
            Some(current) => RunBounds {
                left: current.left.min(glyph.left),
                top: current.top.min(glyph.top),
                right: current.right.max(right),
                bottom: current.bottom.max(bottom),
            },
            None => RunBounds {
                left: glyph.left,
                top: glyph.top,
                right,
                bottom,
            },
        })
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GlyphRenderContext {
    Standalone,
    DottedCircle,
}

fn render_context(code_point: CodePoint) -> GlyphRenderContext {
    if matches!(
        UnicodeDatabase::general_category(code_point),
        GeneralCategory::NonspacingMark
            | GeneralCategory::SpacingMark
            | GeneralCategory::EnclosingMark
    ) {
        GlyphRenderContext::DottedCircle
    } else {
        GlyphRenderContext::Standalone
    }
}

fn shaping_script(character: char) -> Script {
    let script = character.script();
    if script.is_real() {
        script
    } else {
        Script::Latin
    }
}

fn shaping_direction(character: char) -> Direction {
    match character.bidi_class() {
        BidiClass::AL | BidiClass::R | BidiClass::RLE | BidiClass::RLI | BidiClass::RLO => {
            Direction::RightToLeft
        }
        _ => Direction::LeftToRight,
    }
}

fn padded_extent(extent: u16) -> u32 {
    (f32::from(extent) * (1.0 - PADDING_FRACTION * 2.0)).floor() as u32
}

fn layout_width(run: &RasterizedRun) -> u32 {
    let left = run.bounds.left.min(0);
    let right = run.bounds.right.max(run.advance.ceil() as i32);
    right.saturating_sub(left) as u32
}

fn compose_run(
    size: CanvasSize,
    run: &RasterizedRun,
    settings: &GlyphPreviewSettings,
    font: Arc<FontInfo>,
) -> Result<GlyphImage, GlyphRenderError> {
    let layout_left = run.bounds.left.min(0);
    let layout_right = run.bounds.right.max(run.advance.ceil() as i32);
    let layout_width = layout_right.saturating_sub(layout_left);
    let layout_height = run.bounds.bottom.saturating_sub(run.bounds.top);
    let origin_x = (i32::from(size.width()) - layout_width) / 2 - layout_left;
    let origin_y = (i32::from(size.height()) - layout_height) / 2 - run.bounds.top;

    let byte_count = size
        .pixel_count()
        .checked_mul(4)
        .ok_or(GlyphRenderError::GlyphDoesNotFit)?;
    let bg = settings.bg.channels();
    let mut rgba = Vec::with_capacity(byte_count);
    for _ in 0..size.pixel_count() {
        rgba.extend_from_slice(&bg);
    }

    let mut has_coverage = false;
    for glyph in &run.glyphs {
        for source_y in 0..glyph.height {
            for source_x in 0..glyph.width {
                let source_index = (source_y * glyph.width + source_x) as usize;
                let (source, coverage) = match &glyph.pixels {
                    RasterizedPixels::Mask(alpha) => (settings.fg, alpha[source_index]),
                    RasterizedPixels::Color(rgba) => {
                        let offset = source_index * 4;
                        (
                            RgbaColor::rgba(
                                rgba[offset],
                                rgba[offset + 1],
                                rgba[offset + 2],
                                rgba[offset + 3],
                            ),
                            u8::MAX,
                        )
                    }
                };
                if coverage == 0 {
                    continue;
                }
                has_coverage = true;

                let destination_x = origin_x + glyph.left + source_x as i32;
                let destination_y = origin_y + glyph.top + source_y as i32;
                if destination_x < 0
                    || destination_y < 0
                    || destination_x >= i32::from(size.width())
                    || destination_y >= i32::from(size.height())
                {
                    return Err(GlyphRenderError::GlyphDoesNotFit);
                }
                let destination_index = (destination_y as usize * usize::from(size.width())
                    + destination_x as usize)
                    * 4;
                blend_rgba(
                    &mut rgba[destination_index..destination_index + 4],
                    source,
                    coverage,
                );
            }
        }
    }

    if !has_coverage {
        return Err(GlyphRenderError::GlyphDoesNotFit);
    }
    Ok(GlyphImage { size, rgba, font })
}

fn blend_rgba(destination: &mut [u8], source: RgbaColor, coverage: u8) {
    let source = source.channels();
    let source_alpha = (u32::from(source[3]) * u32::from(coverage) + 127) / 255;
    if source_alpha == 0 {
        return;
    }

    let destination_alpha = u32::from(destination[3]);
    let inverse_source_alpha = 255 - source_alpha;
    let output_alpha_numerator = source_alpha * 255 + destination_alpha * inverse_source_alpha;
    for channel in 0..3 {
        let output_numerator = u32::from(source[channel]) * source_alpha * 255
            + u32::from(destination[channel]) * destination_alpha * inverse_source_alpha;
        destination[channel] =
            ((output_numerator + output_alpha_numerator / 2) / output_alpha_numerator) as u8;
    }
    destination[3] = ((output_alpha_numerator + 127) / 255) as u8;
}

#[cfg(test)]
fn visible_bounds(size: CanvasSize, rgba: &[u8]) -> Option<PixelBounds> {
    let mut left = u16::MAX;
    let mut top = u16::MAX;
    let mut right = 0;
    let mut bottom = 0;
    let mut found = false;

    for y in 0..size.height() {
        for x in 0..size.width() {
            let alpha_index = (usize::from(y) * usize::from(size.width()) + usize::from(x)) * 4 + 3;
            if rgba[alpha_index] == 0 {
                continue;
            }
            found = true;
            left = left.min(x);
            top = top.min(y);
            right = right.max(x);
            bottom = bottom.max(y);
        }
    }

    found.then_some(PixelBounds {
        x: left,
        y: top,
        width: right - left + 1,
        height: bottom - top + 1,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::rstest;
    use swash::{shape::Direction, text::Script};

    use crate::{
        glyph::{
            CanvasSize, CanvasSizeError, GlyphPreview, GlyphRenderer, MAX_CANVAS_DIMENSION,
            MAX_CANVAS_PIXELS, PixelBounds, RasterizedGlyph, RasterizedPixels, RasterizedRun,
            RunBounds, blend_rgba,
            cache::GlyphCache,
            compose_run,
            font::fixture_font_info,
            settings::{GlyphPreviewSettings, RgbaColor},
            visible_bounds,
        },
        unicode::CodePoint,
    };

    fn code_point(value: u32) -> CodePoint {
        CodePoint::new(value).expect("fixture is a valid Unicode code point")
    }

    fn image(preview: &GlyphPreview) -> &crate::glyph::GlyphImage {
        match preview {
            GlyphPreview::Ready(image) | GlyphPreview::CombiningContext(image) => image,
            other => panic!("expected an image preview, got {other:?}"),
        }
    }

    #[rustfmt::skip]
    #[rstest]
    #[case(
        0,
        100,
        CanvasSizeError::ZeroDimension
    )]
    #[case(
        100,
        0,
        CanvasSizeError::ZeroDimension
    )]
    #[case(
        MAX_CANVAS_DIMENSION + 1,
        100,
        CanvasSizeError::DimensionLimit {
            width: MAX_CANVAS_DIMENSION + 1,
            height: 100,
            maximum: MAX_CANVAS_DIMENSION,
        }
    )]
    fn rejects_invalid_canvas_dimensions(
        #[case] width: u16,
        #[case] height: u16,
        #[case] expected: CanvasSizeError,
    ) {
        assert_eq!(CanvasSize::new(width, height), Err(expected));
    }

    #[test]
    fn maximum_canvas_is_bounded() {
        let size = CanvasSize::new(MAX_CANVAS_DIMENSION, MAX_CANVAS_DIMENSION)
            .expect("the documented maximum is valid");
        assert_eq!(size.pixel_count(), MAX_CANVAS_PIXELS);
    }

    #[rstest]
    #[case(0x0041)]
    #[case(0x03a9)]
    #[case(0x0416)]
    fn renders_supported_glyphs(#[case] value: u32) {
        let size = CanvasSize::new(320, 240).expect("fixture canvas is valid");
        let mut renderer =
            GlyphRenderer::new(GlyphPreviewSettings::default()).expect("font system initializes");
        let preview = renderer
            .render(code_point(value), size)
            .expect("supported glyph renders");
        let GlyphPreview::Ready(image) = preview else {
            panic!("expected a ready glyph preview");
        };

        assert_eq!(image.size(), size);
        assert_eq!(image.rgba().len(), size.pixel_count() * 4);
        assert_bounds_are_inside_canvas(&image);
        assert_bounds_are_centered_within(&image, 40);
    }

    #[test]
    fn renders_combining_mark_in_dotted_circle_context() {
        let size = CanvasSize::new(320, 240).expect("fixture canvas is valid");
        let mut renderer =
            GlyphRenderer::new(GlyphPreviewSettings::default()).expect("font system initializes");
        let preview = renderer
            .render(code_point(0x0301), size)
            .expect("combining context renders");

        assert!(matches!(preview, GlyphPreview::CombiningContext(_)));
        let image = image(&preview);
        assert_bounds_are_inside_canvas(image);
        assert_bounds_are_centered_within(image, 40);
        assert_has_padding(image);

        let font = renderer.fonts.resolve('\u{0301}', true).unwrap();
        let run = renderer
            .rasterize_run(
                &font,
                "\u{0301}",
                Script::Latin,
                Direction::LeftToRight,
                true,
                192.0,
            )
            .expect("dotted circle and combining mark have visible glyphs");
        assert_eq!(run.glyphs.len(), 2);
        let dotted_circle = &run.glyphs[0];
        let mark = &run.glyphs[1];
        assert!(
            dotted_circle.left < mark.left + mark.width as i32
                && mark.left < dotted_circle.left + dotted_circle.width as i32,
            "dotted circle and mark must overlap horizontally"
        );
    }

    #[rstest]
    #[case(0x0020, GlyphPreview::Blank)]
    #[case(0x10ffff, GlyphPreview::Missing)]
    #[case(0xd800, GlyphPreview::NotScalar)]
    fn classifies_non_image_results(#[case] value: u32, #[case] expected: GlyphPreview) {
        let size = CanvasSize::new(320, 240).expect("fixture canvas is valid");
        let mut renderer =
            GlyphRenderer::new(GlyphPreviewSettings::default()).expect("font system initializes");

        assert_eq!(renderer.render(code_point(value), size), Ok(expected),);
    }

    #[rstest]
    #[case(80, 64)]
    #[case(320, 240)]
    #[case(640, 320)]
    #[case(1024, 1024)]
    fn renders_deterministically_at_bounded_sizes(#[case] width: u16, #[case] height: u16) {
        let size = CanvasSize::new(width, height).expect("fixture canvas is valid");
        let mut renderer =
            GlyphRenderer::new(GlyphPreviewSettings::default()).expect("font system initializes");

        let first = renderer
            .render(code_point(0x0041), size)
            .expect("first render succeeds");
        let second = renderer
            .render(code_point(0x0041), size)
            .expect("second render succeeds");

        assert_eq!(first, second);
        assert_bounds_are_inside_canvas(image(&first));
        assert_has_padding(image(&first));
    }

    #[test]
    fn representative_system_glyph_has_antialiasing_and_the_configured_color() {
        let size = CanvasSize::new(320, 240).expect("fixture canvas is valid");
        let mut renderer =
            GlyphRenderer::new(GlyphPreviewSettings::default()).expect("font system initializes");
        let preview = renderer
            .render(code_point(0x0041), size)
            .expect("representative glyph renders");
        let image = image(&preview);
        let pixels = image.rgba().as_chunks::<4>().0;

        assert_bounds_are_inside_canvas(image);
        assert_has_padding(image);
        assert!(pixels.iter().any(|pixel| pixel[3] == 0));
        assert!(pixels.iter().any(|pixel| (1..=254).contains(&pixel[3])));
        let fg = GlyphPreviewSettings::default().fg.channels();
        assert!(
            pixels
                .iter()
                .filter(|pixel| pixel[3] > 0)
                .all(|pixel| pixel[..3] == fg[..3])
        );
    }

    #[test]
    fn composites_anti_aliased_foreground_over_a_transparent_background() {
        let mut destination = [0, 0, 0, 0];

        blend_rgba(&mut destination, RgbaColor::rgb(240, 120, 60), 128);

        assert_eq!(destination, [240, 120, 60, 128]);
    }

    #[test]
    fn composites_anti_aliased_foreground_over_an_opaque_background() {
        let mut destination = [0, 0, 255, 255];

        blend_rgba(&mut destination, RgbaColor::rgb(255, 0, 0), 128);

        assert_eq!(destination, [128, 0, 127, 255]);
    }

    #[test]
    fn combines_foreground_alpha_with_glyph_coverage() {
        let mut destination = [0, 0, 0, 0];

        blend_rgba(&mut destination, RgbaColor::rgba(10, 20, 30, 128), 128);

        assert_eq!(destination, [10, 20, 30, 64]);
    }

    #[test]
    fn composites_color_glyph_pixels_over_the_configured_background() {
        let size = CanvasSize::new(1, 1).unwrap();
        let run = RasterizedRun {
            glyphs: vec![RasterizedGlyph {
                left: 0,
                top: 0,
                width: 1,
                height: 1,
                pixels: RasterizedPixels::Color(vec![255, 0, 0, 128]),
            }],
            advance: 1.0,
            bounds: RunBounds {
                left: 0,
                top: 0,
                right: 1,
                bottom: 1,
            },
        };
        let settings = GlyphPreviewSettings {
            bg: RgbaColor::rgb(0, 0, 255),
            ..GlyphPreviewSettings::default()
        };

        let image = compose_run(size, &run, &settings, fixture_font_info()).unwrap();

        assert_eq!(image.rgba(), [128, 0, 127, 255]);
    }

    #[test]
    fn renders_with_configured_fg_and_bg() {
        let settings = GlyphPreviewSettings {
            fg: RgbaColor::rgb(255, 0, 0),
            bg: RgbaColor::rgb(0, 0, 255),
            ..GlyphPreviewSettings::default()
        };
        let size = CanvasSize::new(80, 64).expect("fixture canvas is valid");
        let mut renderer = GlyphRenderer::new(settings.clone()).expect("font system initializes");

        let preview = renderer
            .render(code_point(0x0041), size)
            .expect("configured glyph renders");
        let pixels = image(&preview).rgba().as_chunks::<4>().0;

        assert_eq!(pixels[0], settings.bg.channels());
        assert!(pixels.contains(&settings.fg.channels()));
        assert!(pixels.iter().all(|pixel| pixel[3] == u8::MAX));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn renders_with_a_configured_system_font_on_macos() {
        let settings = GlyphPreviewSettings {
            font_families: vec!["Menlo".to_owned()],
            ..GlyphPreviewSettings::default()
        };
        let size = CanvasSize::new(80, 64).expect("fixture canvas is valid");
        let mut renderer = GlyphRenderer::new(settings).expect("font system initializes");

        let preview = renderer
            .render(code_point(0x0041), size)
            .expect("the configured system font renders");

        assert_eq!(image(&preview).font().family(), "Menlo");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn renders_a_system_emoji_fallback_in_color_on_macos() {
        let size = CanvasSize::new(160, 120).expect("fixture canvas is valid");
        let mut renderer =
            GlyphRenderer::new(GlyphPreviewSettings::default()).expect("font system initializes");

        let preview = renderer
            .render(code_point(0x1f600), size)
            .expect("the system emoji font renders");
        let image = image(&preview);
        let mut visible_colors = image
            .rgba()
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| pixel[3] != 0)
            .map(|pixel| &pixel[..3]);
        let first = visible_colors.next().expect("the emoji has visible pixels");

        assert!(!image.font().family().is_empty());
        assert!(visible_colors.any(|color| color != first));
    }

    #[test]
    fn allows_a_fully_transparent_configured_image() {
        let transparent = RgbaColor::rgba(0, 0, 0, 0);
        let settings = GlyphPreviewSettings {
            fg: transparent,
            bg: transparent,
            ..GlyphPreviewSettings::default()
        };
        let size = CanvasSize::new(80, 64).expect("fixture canvas is valid");
        let mut renderer = GlyphRenderer::new(settings.clone()).expect("font system initializes");

        let preview = renderer
            .render(code_point(0x0041), size)
            .expect("transparent glyph renders");

        assert!(image(&preview).rgba().iter().all(|channel| *channel == 0));
    }

    #[test]
    fn cache_reuses_an_entry_and_separates_canvas_sizes() {
        let mut cache = GlyphCache::with_settings(GlyphPreviewSettings::default())
            .expect("font system initializes");
        let small = CanvasSize::new(160, 120).expect("fixture canvas is valid");
        let large = CanvasSize::new(320, 240).expect("fixture canvas is valid");

        let first = cache
            .get_or_render(code_point(0x0041), small)
            .expect("first render succeeds");
        let hit = cache
            .get_or_render(code_point(0x0041), small)
            .expect("cache hit succeeds");
        let resized = cache
            .get_or_render(code_point(0x0041), large)
            .expect("resized render succeeds");

        assert!(Arc::ptr_eq(&first, &hit));
        assert!(!Arc::ptr_eq(&first, &resized));
    }

    #[test]
    fn cache_evicts_least_recently_used_entry_by_count() {
        let mut cache = GlyphCache::with_limits(GlyphPreviewSettings::default(), 2, usize::MAX)
            .expect("font system initializes");
        let size = CanvasSize::new(80, 64).expect("fixture canvas is valid");
        let first = cache
            .get_or_render(code_point(0x0041), size)
            .expect("first render succeeds");
        let second = cache
            .get_or_render(code_point(0x0042), size)
            .expect("second render succeeds");
        let first_hit = cache
            .get_or_render(code_point(0x0041), size)
            .expect("first cache hit succeeds");
        cache
            .get_or_render(code_point(0x0043), size)
            .expect("third render succeeds");
        let second_again = cache
            .get_or_render(code_point(0x0042), size)
            .expect("evicted entry renders again");

        assert!(Arc::ptr_eq(&first, &first_hit));
        assert!(!Arc::ptr_eq(&second, &second_again));
    }

    #[test]
    fn cache_evicts_entries_by_image_bytes() {
        let size = CanvasSize::new(80, 64).expect("fixture canvas is valid");
        let one_image = size.pixel_count() * 4;
        let mut cache = GlyphCache::with_limits(GlyphPreviewSettings::default(), 8, one_image)
            .expect("font system initializes");
        let first = cache
            .get_or_render(code_point(0x0041), size)
            .expect("first render succeeds");
        cache
            .get_or_render(code_point(0x0042), size)
            .expect("second render succeeds");
        let first_again = cache
            .get_or_render(code_point(0x0041), size)
            .expect("evicted entry renders again");

        assert!(!Arc::ptr_eq(&first, &first_again));
        assert!(cache.total_bytes() <= one_image);
    }

    #[test]
    fn oversized_cache_value_is_returned_without_being_stored() {
        let size = CanvasSize::new(80, 64).expect("fixture canvas is valid");
        let mut cache = GlyphCache::with_limits(GlyphPreviewSettings::default(), 8, 1)
            .expect("font system initializes");

        let first = cache
            .get_or_render(code_point(0x0041), size)
            .expect("first render succeeds");
        let second = cache
            .get_or_render(code_point(0x0041), size)
            .expect("second render succeeds");

        assert!(!Arc::ptr_eq(&first, &second));
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.total_bytes(), 0);
    }

    fn assert_bounds_are_inside_canvas(image: &crate::glyph::GlyphImage) {
        let bounds = bounds(image);
        assert!(u32::from(bounds.x) + u32::from(bounds.width) <= u32::from(image.size().width()));
        assert!(u32::from(bounds.y) + u32::from(bounds.height) <= u32::from(image.size().height()));
        assert_eq!(
            image.rgba()[0..4],
            GlyphPreviewSettings::default().bg.channels()
        );
    }

    fn assert_bounds_are_centered_within(image: &crate::glyph::GlyphImage, tolerance: i32) {
        let bounds = bounds(image);
        let left = i32::from(bounds.x);
        let right = i32::from(image.size().width() - bounds.x - bounds.width);
        let top = i32::from(bounds.y);
        let bottom = i32::from(image.size().height() - bounds.y - bounds.height);
        assert!(
            (left - right).abs() <= tolerance,
            "left={left}, right={right}"
        );
        assert!(
            (top - bottom).abs() <= tolerance,
            "top={top}, bottom={bottom}"
        );
    }

    fn assert_has_padding(image: &crate::glyph::GlyphImage) {
        let bounds = bounds(image);
        let minimum_x = (f32::from(image.size().width()) * 0.08).floor() as u16;
        let minimum_y = (f32::from(image.size().height()) * 0.08).floor() as u16;
        assert!(bounds.x >= minimum_x, "bounds={bounds:?}");
        assert!(bounds.y >= minimum_y, "bounds={bounds:?}");
        assert!(
            image.size().width() - bounds.x - bounds.width >= minimum_x,
            "bounds={bounds:?}"
        );
        assert!(
            image.size().height() - bounds.y - bounds.height >= minimum_y,
            "bounds={bounds:?}"
        );
    }

    fn bounds(image: &crate::glyph::GlyphImage) -> PixelBounds {
        visible_bounds(image.size(), image.rgba()).expect("rendered images have visible pixels")
    }
}
