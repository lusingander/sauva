use std::sync::Arc;

use crate::{
    glyph::{CanvasSize, font::FontInfo},
    graphics::{GraphicsAvailability, GraphicsProtocol, GraphicsUnavailableReason},
    image::kitty::ImageId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphPreviewStatus {
    Detecting,
    Hidden,
    Pending,
    Ready,
    CombiningContext,
    Blank,
    Missing,
    NotScalar,
    GraphicsUnavailable(GraphicsUnavailableReason),
    Error(GlyphPreviewError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphPreviewError {
    InvalidGeometry,
    Rendering,
    Transmission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphPreviewGeometry {
    columns: u16,
    rows: u16,
    canvas: CanvasSize,
}

impl GlyphPreviewGeometry {
    pub const fn new(columns: u16, rows: u16, canvas: CanvasSize) -> Self {
        Self {
            columns,
            rows,
            canvas,
        }
    }

    pub const fn columns(self) -> u16 {
        self.columns
    }

    pub const fn rows(self) -> u16 {
        self.rows
    }

    pub const fn canvas(self) -> CanvasSize {
        self.canvas
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphPreviewState {
    availability: Option<GraphicsAvailability>,
    status: GlyphPreviewStatus,
    image_id: Option<ImageId>,
    geometry: Option<GlyphPreviewGeometry>,
    font: Option<Arc<FontInfo>>,
}

impl GlyphPreviewState {
    pub const fn new() -> Self {
        Self {
            availability: None,
            status: GlyphPreviewStatus::Detecting,
            image_id: None,
            geometry: None,
            font: None,
        }
    }

    pub const fn status(&self) -> GlyphPreviewStatus {
        self.status
    }

    pub const fn image_id(&self) -> Option<ImageId> {
        self.image_id
    }

    pub const fn geometry(&self) -> Option<GlyphPreviewGeometry> {
        self.geometry
    }

    pub fn font(&self) -> Option<&FontInfo> {
        self.font.as_deref()
    }

    pub const fn protocol(&self) -> Option<GraphicsProtocol> {
        match self.availability {
            Some(GraphicsAvailability::Available(protocol)) => Some(protocol),
            Some(GraphicsAvailability::Unavailable(_)) | None => None,
        }
    }

    pub fn apply(&mut self, update: GlyphPreviewUpdate) {
        match update {
            GlyphPreviewUpdate::Configure {
                availability,
                image_id,
            } => {
                self.availability = Some(availability);
                self.image_id = image_id;
                self.geometry = None;
                self.font = None;
                self.status = match availability {
                    GraphicsAvailability::Available(_) => GlyphPreviewStatus::Pending,
                    GraphicsAvailability::Unavailable(reason) => {
                        GlyphPreviewStatus::GraphicsUnavailable(reason)
                    }
                };
            }
            GlyphPreviewUpdate::Hidden => {
                self.status = GlyphPreviewStatus::Hidden;
                self.geometry = None;
                self.font = None;
            }
            GlyphPreviewUpdate::Prepared {
                image_id,
                geometry,
                status,
                font,
            } => {
                debug_assert!(matches!(
                    status,
                    GlyphPreviewStatus::Ready
                        | GlyphPreviewStatus::CombiningContext
                        | GlyphPreviewStatus::Blank
                        | GlyphPreviewStatus::Missing
                        | GlyphPreviewStatus::NotScalar
                ));
                self.status = status;
                self.image_id = image_id;
                self.geometry = Some(geometry);
                self.font = font;
            }
            GlyphPreviewUpdate::Failed { geometry, error } => {
                self.status = GlyphPreviewStatus::Error(error);
                self.geometry = geometry;
                self.font = None;
            }
        }
    }

    pub fn selection_changed(&mut self) {
        match self.status {
            GlyphPreviewStatus::Detecting
            | GlyphPreviewStatus::Hidden
            | GlyphPreviewStatus::GraphicsUnavailable(_) => {}
            GlyphPreviewStatus::Pending
            | GlyphPreviewStatus::Ready
            | GlyphPreviewStatus::CombiningContext
            | GlyphPreviewStatus::Blank
            | GlyphPreviewStatus::Missing
            | GlyphPreviewStatus::NotScalar
            | GlyphPreviewStatus::Error(_) => {
                self.status = GlyphPreviewStatus::Pending;
                self.font = None;
            }
        }
    }
}

impl Default for GlyphPreviewState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlyphPreviewUpdate {
    Configure {
        availability: GraphicsAvailability,
        image_id: Option<ImageId>,
    },
    Hidden,
    Prepared {
        image_id: Option<ImageId>,
        geometry: GlyphPreviewGeometry,
        status: GlyphPreviewStatus,
        font: Option<Arc<FontInfo>>,
    },
    Failed {
        geometry: Option<GlyphPreviewGeometry>,
        error: GlyphPreviewError,
    },
}

#[cfg(test)]
mod tests {
    use crate::{
        glyph::{CanvasSize, font::fixture_font_info},
        graphics::{GraphicsAvailability, GraphicsProtocol, GraphicsUnavailableReason},
        image::kitty::ImageId,
        preview::{
            GlyphPreviewGeometry, GlyphPreviewState, GlyphPreviewStatus, GlyphPreviewUpdate,
        },
    };

    #[test]
    fn configures_available_and_unavailable_states() {
        let mut state = GlyphPreviewState::new();
        let image_id = ImageId::new(7).unwrap();

        state.apply(GlyphPreviewUpdate::Configure {
            availability: GraphicsAvailability::Available(GraphicsProtocol::Kitty),
            image_id: Some(image_id),
        });
        assert_eq!(state.status(), GlyphPreviewStatus::Pending);
        assert_eq!(state.image_id(), Some(image_id));

        state.apply(GlyphPreviewUpdate::Configure {
            availability: GraphicsAvailability::Unavailable(
                GraphicsUnavailableReason::UnsupportedTerminal,
            ),
            image_id: None,
        });
        assert_eq!(
            state.status(),
            GlyphPreviewStatus::GraphicsUnavailable(GraphicsUnavailableReason::UnsupportedTerminal)
        );
        assert_eq!(state.image_id(), None);
    }

    #[test]
    fn invalidates_only_a_prepared_selection() {
        let mut state = prepared_state();
        let geometry = state.geometry();
        let image_id = state.image_id();

        state.selection_changed();

        assert_eq!(state.status(), GlyphPreviewStatus::Pending);
        assert_eq!(state.geometry(), geometry);
        assert_eq!(state.image_id(), image_id);

        state.apply(GlyphPreviewUpdate::Hidden);
        state.selection_changed();
        assert_eq!(state.status(), GlyphPreviewStatus::Hidden);
    }

    #[test]
    fn tracks_the_resolved_font_for_the_current_selection() {
        let mut state = GlyphPreviewState::new();
        let font = fixture_font_info();
        state.apply(GlyphPreviewUpdate::Prepared {
            image_id: None,
            geometry: GlyphPreviewGeometry::new(10, 4, CanvasSize::new(80, 64).unwrap()),
            status: GlyphPreviewStatus::Ready,
            font: Some(font),
        });

        assert_eq!(state.font().unwrap().family(), "Test Font");

        state.selection_changed();

        assert!(state.font().is_none());
    }

    fn prepared_state() -> GlyphPreviewState {
        let mut state = GlyphPreviewState::new();
        let image_id = ImageId::new(7).unwrap();
        let canvas = CanvasSize::new(80, 64).unwrap();
        state.apply(GlyphPreviewUpdate::Configure {
            availability: GraphicsAvailability::Available(GraphicsProtocol::Kitty),
            image_id: Some(image_id),
        });
        state.apply(GlyphPreviewUpdate::Prepared {
            image_id: Some(image_id),
            geometry: GlyphPreviewGeometry::new(10, 4, canvas),
            status: GlyphPreviewStatus::Ready,
            font: None,
        });
        state
    }
}
