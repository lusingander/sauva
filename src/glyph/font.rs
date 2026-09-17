pub mod platform;

use std::{collections::HashSet, sync::Arc};

use fontdb::{Database, Family, Query};
use swash::{FontRef, StringId};

use crate::glyph::{
    GlyphRenderError, font::platform::PlatformFontResolver, settings::GlyphPreviewSettings,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FontHandle(fontdb::ID);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontInfo {
    family: String,
    style: String,
    version: Option<String>,
}

impl FontInfo {
    pub fn family(&self) -> &str {
        &self.family
    }

    pub fn style(&self) -> &str {
        &self.style
    }

    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedFont {
    handle: FontHandle,
    info: Arc<FontInfo>,
}

impl ResolvedFont {
    pub fn info(&self) -> &Arc<FontInfo> {
        &self.info
    }
}

pub struct FontCollection {
    database: Database,
    preferred: Vec<ResolvedFont>,
    system_default: Option<ResolvedFont>,
    emoji: Vec<ResolvedFont>,
    platform: PlatformFontResolver,
}

impl FontCollection {
    pub fn new(settings: &GlyphPreviewSettings) -> Result<Self, GlyphRenderError> {
        let mut database = Database::new();
        database.load_system_fonts();

        let preferred = query_families(&database, &settings.font_families);
        let emoji = query_families(&database, &settings.emoji_font_families);
        let platform = PlatformFontResolver::new();
        let system_default = platform
            .default_font(&database)
            .map(|id| resolved_font(&database, id));

        Ok(Self {
            database,
            preferred,
            system_default,
            emoji,
            platform,
        })
    }

    pub fn resolve(&self, character: char, needs_dotted_circle: bool) -> Option<ResolvedFont> {
        let configured = self
            .preferred
            .iter()
            .chain(self.system_default.iter())
            .chain(&self.emoji)
            .find(|font| self.supports(font, character, needs_dotted_circle))
            .cloned();
        if configured.is_some() {
            return configured;
        }

        self.platform
            .fallback_font(&self.database, character, needs_dotted_circle)
            .map(|id| resolved_font(&self.database, id))
            .filter(|font| self.supports(font, character, needs_dotted_circle))
    }

    pub fn with_font<T>(
        &self,
        font: &ResolvedFont,
        operation: impl FnOnce(FontRef<'_>) -> T,
    ) -> Option<T> {
        self.database.with_face_data(font.handle.0, |data, index| {
            FontRef::from_index(data, index as usize).map(operation)
        })?
    }

    fn supports(&self, font: &ResolvedFont, character: char, needs_dotted_circle: bool) -> bool {
        self.with_font(font, |font| {
            font.charmap().map(character) != 0
                && (!needs_dotted_circle || font.charmap().map('\u{25cc}') != 0)
        })
        .unwrap_or(false)
    }
}

fn query_families(database: &Database, families: &[String]) -> Vec<ResolvedFont> {
    let mut seen = HashSet::new();
    families
        .iter()
        .filter_map(|family| {
            database.query(&Query {
                families: &[Family::Name(family)],
                ..Query::default()
            })
        })
        .filter(|id| seen.insert(*id))
        .map(|id| resolved_font(database, id))
        .collect()
}

fn resolved_font(database: &Database, id: fontdb::ID) -> ResolvedFont {
    ResolvedFont {
        handle: FontHandle(id),
        info: font_info(database, id),
    }
}

fn font_info(database: &Database, id: fontdb::ID) -> Arc<FontInfo> {
    let face = database.face(id).expect("a queried font face exists");
    let family = face
        .families
        .first()
        .map(|(family, _)| family.clone())
        .unwrap_or_else(|| face.post_script_name.clone());
    let style = match face.style {
        fontdb::Style::Normal => "Regular",
        fontdb::Style::Italic => "Italic",
        fontdb::Style::Oblique => "Oblique",
    }
    .to_owned();
    let version = database
        .with_face_data(id, |data, index| {
            FontRef::from_index(data, index as usize).and_then(|font| {
                font.localized_strings()
                    .find_by_id(StringId::Version, None)
                    .map(|version| version.chars().collect::<String>())
            })
        })
        .flatten();

    Arc::new(FontInfo {
        family,
        style,
        version,
    })
}

pub fn fixture_font_info() -> Arc<FontInfo> {
    Arc::new(FontInfo {
        family: "Test Font".to_owned(),
        style: "Regular".to_owned(),
        version: None,
    })
}

#[cfg(test)]
mod tests {
    use crate::glyph::{font::FontCollection, settings::GlyphPreviewSettings};

    #[test]
    fn missing_configured_families_are_skipped() {
        let settings = GlyphPreviewSettings {
            font_families: vec!["A font family that does not exist".to_owned()],
            emoji_font_families: Vec::new(),
            ..GlyphPreviewSettings::default()
        };
        let fonts = FontCollection::new(&settings).unwrap();

        assert!(fonts.resolve('A', false).is_some());
    }

    #[test]
    fn system_default_resolves_ordinary_text_before_later_fallbacks() {
        let fonts = FontCollection::new(&GlyphPreviewSettings::default()).unwrap();
        let resolved = fonts.resolve('1', false).unwrap();
        let system_default = fonts.system_default.as_ref().unwrap();

        assert_eq!(resolved.handle, system_default.handle);
    }

    #[test]
    fn configured_family_is_checked_before_the_system_default() {
        let system = FontCollection::new(&GlyphPreviewSettings::default()).unwrap();
        let family = system.resolve('A', false).unwrap().info.family.clone();
        let configured = FontCollection::new(&GlyphPreviewSettings {
            font_families: vec![family],
            ..GlyphPreviewSettings::default()
        })
        .unwrap();

        assert_eq!(
            configured.resolve('A', false).unwrap().handle,
            configured.preferred[0].handle
        );
    }
}
