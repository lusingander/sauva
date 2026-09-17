use core_foundation::{
    base::{CFRange, TCFType},
    string::{CFString, CFStringRef},
};
use core_text::font::{CTFont, CTFontRef, kCTFontSystemFontType, new_ui_font_for_language};
use fontdb::Database;

use crate::glyph::font::platform::face_for_path;

pub fn default_font(database: &Database) -> Option<fontdb::ID> {
    let font = system_font();
    face_for_font(database, &font)
}

pub fn fallback_font(
    database: &Database,
    character: char,
    needs_dotted_circle: bool,
) -> Option<fontdb::ID> {
    let base = system_font();
    let mut text = character.to_string();
    if needs_dotted_circle {
        text.push('\u{25cc}');
    }
    let string = CFString::new(&text);
    let range = CFRange {
        location: 0,
        length: text.encode_utf16().count() as isize,
    };
    let fallback = unsafe {
        let font = CTFontCreateForString(
            base.as_concrete_TypeRef(),
            string.as_concrete_TypeRef(),
            range,
        );
        if font.is_null() {
            return None;
        }
        CTFont::wrap_under_create_rule(font)
    };

    face_for_font(database, &fallback)
}

fn system_font() -> CTFont {
    new_ui_font_for_language(kCTFontSystemFontType, 0.0, None)
}

fn face_for_font(database: &Database, font: &CTFont) -> Option<fontdb::ID> {
    let path = font.url()?.to_path()?;
    let post_script_name = font.postscript_name();
    face_for_path(database, &path, None, Some(&post_script_name))
}

#[link(name = "CoreText", kind = "framework")]
unsafe extern "C" {
    fn CTFontCreateForString(
        current_font: CTFontRef,
        string: CFStringRef,
        range: CFRange,
    ) -> CTFontRef;
}

#[cfg(test)]
mod tests {
    use fontdb::Database;

    use crate::glyph::font::platform::core_text;

    #[test]
    fn resolves_the_system_text_face_and_a_japanese_fallback() {
        let mut database = Database::new();
        database.load_system_fonts();

        assert!(core_text::default_font(&database).is_some());
        assert!(core_text::fallback_font(&database, 'あ', false).is_some());
    }
}
