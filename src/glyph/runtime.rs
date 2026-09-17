use std::{
    io::{self, Write},
    sync::Arc,
};

use ratatui::{buffer::Buffer, layout::Rect};

use crate::{
    glyph::{
        CanvasSize, GlyphPreview, MAX_CANVAS_DIMENSION, cache::GlyphCache,
        settings::GlyphPreviewSettings,
    },
    graphics::{GraphicsAvailability, GraphicsProtocol},
    image::{
        RgbaImage,
        iterm2_manager::Iterm2ImageManager,
        kitty::{ImageId, VirtualPlacement},
        manager::KittyImageManager,
    },
    preview::{GlyphPreviewError, GlyphPreviewGeometry, GlyphPreviewStatus, GlyphPreviewUpdate},
    unicode::CodePoint,
};

const FALLBACK_CELL_WIDTH: u16 = 8;
const FALLBACK_CELL_HEIGHT: u16 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalPixelMetrics {
    pub columns: u16,
    pub rows: u16,
    pub width: u16,
    pub height: u16,
}

pub struct GlyphPreviewRuntime {
    availability: GraphicsAvailability,
    cache: Option<GlyphCache>,
    image: Option<ImageManager>,
    initialization_failed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GlyphImageKey {
    code_point: CodePoint,
    canvas: CanvasSize,
}

enum ImageManager {
    Kitty(KittyImageManager<GlyphImageKey>),
    Iterm2(Iterm2ImageManager<GlyphImageKey>),
}

impl ImageManager {
    const fn image_id(&self) -> Option<ImageId> {
        match self {
            Self::Kitty(image) => Some(image.image_id()),
            Self::Iterm2(_) => None,
        }
    }

    fn cleanup(&mut self, writer: &mut impl Write) -> io::Result<bool> {
        match self {
            Self::Kitty(image) => image.cleanup(writer),
            Self::Iterm2(image) => Ok(image.hide()),
        }
    }

    fn render(&self, buffer: &mut Buffer) {
        if let Self::Iterm2(image) = self {
            image.render(buffer);
        }
    }
}

impl GlyphPreviewRuntime {
    #[cfg(test)]
    pub fn new(availability: GraphicsAvailability) -> Self {
        Self::with_settings(availability, GlyphPreviewSettings::default())
    }

    pub fn with_settings(
        availability: GraphicsAvailability,
        settings: GlyphPreviewSettings,
    ) -> Self {
        Self::with_image_id(availability, None, settings)
    }

    fn with_image_id(
        availability: GraphicsAvailability,
        image_id: Option<ImageId>,
        settings: GlyphPreviewSettings,
    ) -> Self {
        if !availability.is_available() {
            return Self {
                availability,
                cache: None,
                image: None,
                initialization_failed: false,
            };
        }

        let (cache, initialization_failed) = match GlyphCache::with_settings(settings) {
            Ok(cache) => (Some(cache), false),
            Err(_) => (None, true),
        };
        let image = match availability {
            GraphicsAvailability::Available(GraphicsProtocol::Kitty) => Some(ImageManager::Kitty(
                image_id.map_or_else(KittyImageManager::new, KittyImageManager::with_id),
            )),
            GraphicsAvailability::Available(GraphicsProtocol::Iterm2) => {
                Some(ImageManager::Iterm2(Iterm2ImageManager::new()))
            }
            GraphicsAvailability::Unavailable(_) => None,
        };
        Self {
            availability,
            cache,
            image,
            initialization_failed,
        }
    }

    pub fn configure_update(&self) -> GlyphPreviewUpdate {
        GlyphPreviewUpdate::Configure {
            availability: self.availability,
            image_id: self.image.as_ref().and_then(ImageManager::image_id),
        }
    }

    pub const fn uses_kitty_placeholders(&self) -> bool {
        matches!(self.image, Some(ImageManager::Kitty(_)))
    }

    pub fn synchronize(
        &mut self,
        writer: &mut impl Write,
        code_point: CodePoint,
        placeholder: Option<Rect>,
        terminal_pixels: Option<TerminalPixelMetrics>,
    ) -> GlyphPreviewUpdate {
        let Some(placeholder) = placeholder else {
            return self.hide(writer);
        };
        let Some(image) = self.image.as_mut() else {
            return self.configure_update();
        };
        if self.initialization_failed {
            return GlyphPreviewUpdate::Failed {
                geometry: None,
                error: GlyphPreviewError::Rendering,
            };
        }

        let geometry = match preview_geometry(placeholder, terminal_pixels) {
            Ok(geometry) => geometry,
            Err(()) => {
                return cleanup_then(
                    image,
                    writer,
                    GlyphPreviewUpdate::Failed {
                        geometry: None,
                        error: GlyphPreviewError::InvalidGeometry,
                    },
                );
            }
        };
        let key = GlyphImageKey {
            code_point,
            canvas: geometry.canvas(),
        };
        let Some(cache) = self.cache.as_mut() else {
            return GlyphPreviewUpdate::Failed {
                geometry: Some(geometry),
                error: GlyphPreviewError::Rendering,
            };
        };
        let preview = match cache.get_or_render(code_point, geometry.canvas()) {
            Ok(preview) => preview,
            Err(_) => {
                return cleanup_then(
                    image,
                    writer,
                    GlyphPreviewUpdate::Failed {
                        geometry: Some(geometry),
                        error: GlyphPreviewError::Rendering,
                    },
                );
            }
        };

        let status = match preview.as_ref() {
            GlyphPreview::Ready(glyph) => {
                return transmit(
                    image,
                    writer,
                    key,
                    placeholder,
                    geometry,
                    glyph,
                    GlyphPreviewStatus::Ready,
                );
            }
            GlyphPreview::CombiningContext(glyph) => {
                return transmit(
                    image,
                    writer,
                    key,
                    placeholder,
                    geometry,
                    glyph,
                    GlyphPreviewStatus::CombiningContext,
                );
            }
            GlyphPreview::Blank => GlyphPreviewStatus::Blank,
            GlyphPreview::Missing => GlyphPreviewStatus::Missing,
            GlyphPreview::NotScalar => GlyphPreviewStatus::NotScalar,
        };
        let update = GlyphPreviewUpdate::Prepared {
            image_id: image.image_id(),
            geometry,
            status,
            font: None,
        };
        cleanup_then(image, writer, update)
    }

    pub fn cleanup(&mut self, writer: &mut impl Write) -> io::Result<()> {
        if let Some(image) = self.image.as_mut() {
            image.cleanup(writer)?;
        }
        Ok(())
    }

    pub fn render_image(&self, buffer: &mut Buffer) {
        if let Some(image) = self.image.as_ref() {
            image.render(buffer);
        }
    }

    fn hide(&mut self, writer: &mut impl Write) -> GlyphPreviewUpdate {
        let update = GlyphPreviewUpdate::Hidden;
        match self.image.as_mut() {
            Some(image) => cleanup_then(image, writer, update),
            None => update,
        }
    }
}

fn transmit(
    image: &mut ImageManager,
    writer: &mut impl Write,
    key: GlyphImageKey,
    area: Rect,
    geometry: GlyphPreviewGeometry,
    glyph: &crate::glyph::GlyphImage,
    status: GlyphPreviewStatus,
) -> GlyphPreviewUpdate {
    let size = glyph.size();
    let rgba = match RgbaImage::new(
        u32::from(size.width()),
        u32::from(size.height()),
        glyph.rgba(),
    ) {
        Ok(rgba) => rgba,
        Err(_) => {
            return GlyphPreviewUpdate::Failed {
                geometry: Some(geometry),
                error: GlyphPreviewError::Rendering,
            };
        }
    };
    let result = match image {
        ImageManager::Kitty(image) => {
            let placement = match VirtualPlacement::new(geometry.columns(), geometry.rows()) {
                Ok(placement) => placement,
                Err(_) => {
                    return GlyphPreviewUpdate::Failed {
                        geometry: Some(geometry),
                        error: GlyphPreviewError::InvalidGeometry,
                    };
                }
            };
            image.show(writer, key, rgba, placement)
        }
        ImageManager::Iterm2(image) => image.prepare(key, rgba, area),
    };
    if result.is_err() {
        return GlyphPreviewUpdate::Failed {
            geometry: Some(geometry),
            error: GlyphPreviewError::Transmission,
        };
    }
    GlyphPreviewUpdate::Prepared {
        image_id: image.image_id(),
        geometry,
        status,
        font: Some(Arc::clone(glyph.font())),
    }
}

fn cleanup_then(
    image: &mut ImageManager,
    writer: &mut impl Write,
    update: GlyphPreviewUpdate,
) -> GlyphPreviewUpdate {
    match image.cleanup(writer) {
        Ok(_) => update,
        Err(_) => GlyphPreviewUpdate::Failed {
            geometry: None,
            error: GlyphPreviewError::Transmission,
        },
    }
}

pub fn preview_geometry(
    placeholder: Rect,
    terminal_pixels: Option<TerminalPixelMetrics>,
) -> Result<GlyphPreviewGeometry, ()> {
    if placeholder.width == 0 || placeholder.height == 0 {
        return Err(());
    }
    let (cell_width, cell_height) = cell_pixel_size(terminal_pixels);
    let width = u32::from(placeholder.width)
        .checked_mul(u32::from(cell_width))
        .ok_or(())?;
    let height = u32::from(placeholder.height)
        .checked_mul(u32::from(cell_height))
        .ok_or(())?;
    let (width, height) = bounded_canvas_dimensions(width, height);
    let canvas = CanvasSize::new(width, height).map_err(|_| ())?;
    Ok(GlyphPreviewGeometry::new(
        placeholder.width,
        placeholder.height,
        canvas,
    ))
}

fn bounded_canvas_dimensions(width: u32, height: u32) -> (u16, u16) {
    let largest = width.max(height);
    let maximum = u32::from(MAX_CANVAS_DIMENSION);
    if largest <= maximum {
        return (width as u16, height as u16);
    }

    let scaled_width = u64::from(width) * u64::from(maximum) / u64::from(largest);
    let scaled_height = u64::from(height) * u64::from(maximum) / u64::from(largest);
    (scaled_width.max(1) as u16, scaled_height.max(1) as u16)
}

fn cell_pixel_size(metrics: Option<TerminalPixelMetrics>) -> (u16, u16) {
    metrics
        .filter(|metrics| {
            metrics.columns > 0 && metrics.rows > 0 && metrics.width > 0 && metrics.height > 0
        })
        .map(|metrics| {
            (
                metrics.width / metrics.columns,
                metrics.height / metrics.rows,
            )
        })
        .filter(|(width, height)| *width > 0 && *height > 0)
        .unwrap_or((FALLBACK_CELL_WIDTH, FALLBACK_CELL_HEIGHT))
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};

    use ratatui::{buffer::Buffer, layout::Rect};
    use rstest::rstest;

    use crate::{
        glyph::{
            runtime::{GlyphPreviewRuntime, TerminalPixelMetrics, preview_geometry},
            settings::GlyphPreviewSettings,
        },
        graphics::{GraphicsAvailability, GraphicsProtocol, GraphicsUnavailableReason},
        image::kitty::ImageId,
        preview::{GlyphPreviewError, GlyphPreviewStatus, GlyphPreviewUpdate},
        unicode::CodePoint,
    };

    const PLACEHOLDER: Rect = Rect::new(62, 6, 36, 20);

    #[test]
    fn derives_fallback_canvas_pixels_from_placeholder_cells() {
        let geometry = preview_geometry(PLACEHOLDER, None).unwrap();

        assert_eq!(geometry.columns(), 36);
        assert_eq!(geometry.rows(), 20);
        assert_eq!(geometry.canvas().width(), 288);
        assert_eq!(geometry.canvas().height(), 320);
    }

    #[test]
    fn derives_canvas_pixels_from_reported_terminal_pixels() {
        let geometry = preview_geometry(
            PLACEHOLDER,
            Some(TerminalPixelMetrics {
                columns: 100,
                rows: 30,
                width: 1000,
                height: 600,
            }),
        )
        .unwrap();

        assert_eq!(geometry.canvas().width(), 360);
        assert_eq!(geometry.canvas().height(), 400);
    }

    #[test]
    fn bounds_large_terminal_pixels_while_preserving_canvas_aspect_ratio() {
        let geometry = preview_geometry(
            Rect::new(62, 6, 76, 30),
            Some(TerminalPixelMetrics {
                columns: 140,
                rows: 40,
                width: 2800,
                height: 1600,
            }),
        )
        .unwrap();

        assert_eq!(geometry.canvas().width(), 1024);
        assert_eq!(geometry.canvas().height(), 808);
    }

    #[rstest]
    #[case(None)]
    #[case(Some(TerminalPixelMetrics {
        columns: 0,
        rows: 30,
        width: 1000,
        height: 600,
    }))]
    #[case(Some(TerminalPixelMetrics {
        columns: 100,
        rows: 30,
        width: 0,
        height: 0,
    }))]
    #[case(Some(TerminalPixelMetrics {
        columns: 1000,
        rows: 1000,
        width: 100,
        height: 100,
    }))]
    fn uses_fallback_for_missing_or_unusable_pixel_metrics(
        #[case] metrics: Option<TerminalPixelMetrics>,
    ) {
        assert_eq!(
            preview_geometry(PLACEHOLDER, metrics)
                .unwrap()
                .canvas()
                .width(),
            288
        );
    }

    #[test]
    fn rejects_empty_placeholder_geometry() {
        assert_eq!(preview_geometry(Rect::new(0, 0, 0, 20), None), Err(()));
    }

    #[test]
    fn unavailable_runtime_never_writes_graphics() {
        let availability =
            GraphicsAvailability::Unavailable(GraphicsUnavailableReason::UnsupportedTerminal);
        let mut runtime = GlyphPreviewRuntime::new(availability);
        let mut output = Vec::new();

        assert_eq!(
            runtime.synchronize(
                &mut output,
                CodePoint::new(0x41).unwrap(),
                Some(PLACEHOLDER),
                None,
            ),
            GlyphPreviewUpdate::Configure {
                availability,
                image_id: None,
            }
        );
        assert!(output.is_empty());
    }

    #[test]
    fn iterm2_runtime_prepares_once_and_stops_rendering_when_hidden() {
        let mut runtime =
            GlyphPreviewRuntime::new(GraphicsAvailability::Available(GraphicsProtocol::Iterm2));
        assert!(!runtime.uses_kitty_placeholders());
        assert_eq!(
            runtime.configure_update(),
            GlyphPreviewUpdate::Configure {
                availability: GraphicsAvailability::Available(GraphicsProtocol::Iterm2),
                image_id: None,
            }
        );

        let selected = CodePoint::new(0x41).unwrap();
        let mut first = Vec::new();
        let mut unchanged = Vec::new();
        let mut hidden = Vec::new();
        let first_update = runtime.synchronize(&mut first, selected, Some(PLACEHOLDER), None);
        let mut rendered = Buffer::empty(Rect::new(0, 0, 100, 30));
        runtime.render_image(&mut rendered);
        let unchanged_update =
            runtime.synchronize(&mut unchanged, selected, Some(PLACEHOLDER), None);
        let hidden_update = runtime.synchronize(&mut hidden, selected, None, None);
        let mut hidden_render = Buffer::empty(Rect::new(0, 0, 100, 30));
        runtime.render_image(&mut hidden_render);

        assert!(matches!(
            first_update,
            GlyphPreviewUpdate::Prepared {
                image_id: None,
                status: GlyphPreviewStatus::Ready,
                ..
            }
        ));
        assert_eq!(unchanged_update, first_update);
        assert!(first.is_empty());
        assert!(
            rendered
                .cell((PLACEHOLDER.x, PLACEHOLDER.y))
                .unwrap()
                .symbol()
                .contains("]1337;")
        );
        assert!(unchanged.is_empty());
        assert_eq!(hidden_update, GlyphPreviewUpdate::Hidden);
        assert!(hidden.is_empty());
        assert!(
            hidden_render
                .content
                .iter()
                .all(|cell| !cell.symbol().contains("]1337;"))
        );
    }

    #[test]
    fn synchronizes_selection_resize_and_hidden_cleanup() {
        let image_id = ImageId::new(7).unwrap();
        let mut runtime = GlyphPreviewRuntime::with_image_id(
            GraphicsAvailability::Available(GraphicsProtocol::Kitty),
            Some(image_id),
            GlyphPreviewSettings::default(),
        );
        let selected = CodePoint::new(0x41).unwrap();
        let mut first = Vec::new();
        let mut unchanged = Vec::new();
        let mut resized = Vec::new();
        let mut changed_selection = Vec::new();
        let mut hidden = Vec::new();

        let first_update = runtime.synchronize(&mut first, selected, Some(PLACEHOLDER), None);
        let unchanged_update =
            runtime.synchronize(&mut unchanged, selected, Some(PLACEHOLDER), None);
        let resized_update =
            runtime.synchronize(&mut resized, selected, Some(Rect::new(62, 6, 40, 20)), None);
        let changed_selection_update = runtime.synchronize(
            &mut changed_selection,
            CodePoint::new(0x42).unwrap(),
            Some(Rect::new(62, 6, 40, 20)),
            None,
        );
        let hidden_update = runtime.synchronize(&mut hidden, selected, None, None);

        assert!(matches!(
            first_update,
            GlyphPreviewUpdate::Prepared {
                status: GlyphPreviewStatus::Ready,
                ..
            }
        ));
        assert_eq!(first_update, unchanged_update);
        assert_ne!(first_update, resized_update);
        assert!(matches!(
            resized_update,
            GlyphPreviewUpdate::Prepared {
                status: GlyphPreviewStatus::Ready,
                ..
            }
        ));
        assert!(!first.is_empty());
        assert!(unchanged.is_empty());
        assert!(!resized.is_empty());
        assert!(!changed_selection.is_empty());
        assert!(matches!(
            changed_selection_update,
            GlyphPreviewUpdate::Prepared {
                status: GlyphPreviewStatus::Ready,
                ..
            }
        ));
        assert_eq!(hidden_update, GlyphPreviewUpdate::Hidden);
        assert_eq!(hidden, b"\x1b_Ga=d,d=I,i=7,q=2\x1b\\");
    }

    #[rstest]
    #[case(0x20, GlyphPreviewStatus::Blank)]
    #[case(0x10ffff, GlyphPreviewStatus::Missing)]
    #[case(0xd800, GlyphPreviewStatus::NotScalar)]
    #[case(0x0301, GlyphPreviewStatus::CombiningContext)]
    fn maps_renderer_outcomes_to_preview_status(
        #[case] code_point: u32,
        #[case] expected: GlyphPreviewStatus,
    ) {
        let mut runtime = GlyphPreviewRuntime::with_image_id(
            GraphicsAvailability::Available(GraphicsProtocol::Kitty),
            ImageId::new(7),
            GlyphPreviewSettings::default(),
        );

        let update = runtime.synchronize(
            &mut Vec::new(),
            CodePoint::new(code_point).unwrap(),
            Some(PLACEHOLDER),
            None,
        );

        assert!(matches!(
            update,
            GlyphPreviewUpdate::Prepared { status, .. } if status == expected
        ));
    }

    #[test]
    fn reports_invalid_geometry_without_writing_an_image() {
        let mut runtime = GlyphPreviewRuntime::with_image_id(
            GraphicsAvailability::Available(GraphicsProtocol::Kitty),
            ImageId::new(7),
            GlyphPreviewSettings::default(),
        );
        let mut output = Vec::new();

        let update = runtime.synchronize(
            &mut output,
            CodePoint::new(0x41).unwrap(),
            Some(Rect::new(0, 0, 0, 20)),
            None,
        );

        assert_eq!(
            update,
            GlyphPreviewUpdate::Failed {
                geometry: None,
                error: GlyphPreviewError::InvalidGeometry,
            }
        );
        assert!(output.is_empty());
    }

    #[test]
    fn maps_terminal_write_failure_to_a_safe_preview_error() {
        let mut runtime = GlyphPreviewRuntime::with_image_id(
            GraphicsAvailability::Available(GraphicsProtocol::Kitty),
            ImageId::new(7),
            GlyphPreviewSettings::default(),
        );

        let update = runtime.synchronize(
            &mut AlwaysFails,
            CodePoint::new(0x41).unwrap(),
            Some(PLACEHOLDER),
            None,
        );

        assert_eq!(
            update,
            GlyphPreviewUpdate::Failed {
                geometry: Some(preview_geometry(PLACEHOLDER, None).unwrap()),
                error: GlyphPreviewError::Transmission,
            }
        );
    }

    struct AlwaysFails;

    impl Write for AlwaysFails {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("injected write failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("injected flush failure"))
        }
    }
}
