mod fontdb;

#[cfg(target_os = "macos")]
mod core_text;
#[cfg(target_os = "linux")]
mod fontconfig;

use std::path::Path;

use ::fontdb::{Database, Source};

pub struct PlatformFontResolver {
    #[cfg(target_os = "linux")]
    fontconfig: Option<fontconfig::Fontconfig>,
}

impl PlatformFontResolver {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "linux")]
            fontconfig: fontconfig::Fontconfig::new(),
        }
    }

    pub fn default_font(&self, database: &Database) -> Option<::fontdb::ID> {
        self.native_default_font(database)
            .or_else(|| fontdb::default_font(database))
    }

    pub fn fallback_font(
        &self,
        database: &Database,
        character: char,
        needs_dotted_circle: bool,
    ) -> Option<::fontdb::ID> {
        self.native_fallback_font(database, character, needs_dotted_circle)
            .filter(|id| fontdb::supports(database, *id, character, needs_dotted_circle))
            .or_else(|| fontdb::fallback_font(database, character, needs_dotted_circle))
    }

    #[cfg(target_os = "macos")]
    fn native_default_font(&self, database: &Database) -> Option<::fontdb::ID> {
        core_text::default_font(database)
    }

    #[cfg(target_os = "linux")]
    fn native_default_font(&self, database: &Database) -> Option<::fontdb::ID> {
        self.fontconfig.as_ref()?.default_font(database)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    const fn native_default_font(&self, _: &Database) -> Option<::fontdb::ID> {
        None
    }

    #[cfg(target_os = "macos")]
    fn native_fallback_font(
        &self,
        database: &Database,
        character: char,
        needs_dotted_circle: bool,
    ) -> Option<::fontdb::ID> {
        core_text::fallback_font(database, character, needs_dotted_circle)
    }

    #[cfg(target_os = "linux")]
    fn native_fallback_font(
        &self,
        database: &Database,
        character: char,
        needs_dotted_circle: bool,
    ) -> Option<::fontdb::ID> {
        self.fontconfig
            .as_ref()?
            .fallback_font(database, character, needs_dotted_circle)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    const fn native_fallback_font(&self, _: &Database, _: char, _: bool) -> Option<::fontdb::ID> {
        None
    }
}

fn face_for_path(
    database: &Database,
    path: &Path,
    index: Option<u32>,
    post_script_name: Option<&str>,
) -> Option<::fontdb::ID> {
    database
        .faces()
        .filter(|face| index.is_none_or(|index| face.index == index))
        .filter(|face| {
            matches!(
                &face.source,
                Source::File(source_path) | Source::SharedFile(source_path, _)
                    if source_path == path
            )
        })
        .find(|face| {
            post_script_name.is_none_or(|name| face.post_script_name.eq_ignore_ascii_case(name))
        })
        .map(|face| face.id)
}

#[cfg(test)]
mod tests {
    use ::fontdb::Database;

    use crate::glyph::font::platform::PlatformFontResolver;

    #[test]
    fn resolves_a_system_default_and_fallback_for_ascii() {
        let mut database = Database::new();
        database.load_system_fonts();
        let resolver = PlatformFontResolver::new();

        assert!(resolver.default_font(&database).is_some());
        assert!(resolver.fallback_font(&database, 'A', false).is_some());
    }
}
