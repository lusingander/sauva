use std::{ffi::CStr, path::PathBuf, ptr};

use fontconfig_sys::{self as ffi, constants};
use fontdb::Database;

use crate::glyph::font::platform::face_for_path;

fn face_from_pattern(database: &Database, pattern: &Pattern<'_>) -> Option<fontdb::ID> {
    let path = PathBuf::from(pattern.get_string(constants::FC_FILE)?.to_str().ok()?);
    let index = pattern.get_integer(constants::FC_INDEX).unwrap_or(0);
    u32::try_from(index)
        .ok()
        .and_then(|index| face_for_path(database, &path, Some(index), None))
}

pub struct Fontconfig {
    library: &'static ffi::Fc,
    config: *mut ffi::FcConfig,
}

impl Fontconfig {
    pub fn new() -> Option<Self> {
        let library = ffi::statics::LIB_RESULT.as_ref().ok()?;
        let config = unsafe { (library.FcInitLoadConfigAndFonts)() };
        (!config.is_null()).then_some(Self { library, config })
    }

    pub fn default_font(&self, database: &Database) -> Option<fontdb::ID> {
        let mut pattern = Pattern::new(self.library)?;
        pattern.add_string(constants::FC_FAMILY, c"sans-serif")?;
        pattern.prepare(self.config);

        let mut result = ffi::FcResultNoMatch;
        let matched =
            unsafe { (self.library.FcFontMatch)(self.config, pattern.pointer, &mut result) };
        if result != ffi::FcResultMatch {
            return None;
        }
        let matched = Pattern::from_owned(self.library, matched)?;
        face_from_pattern(database, &matched)
    }

    pub fn fallback_font(
        &self,
        database: &Database,
        character: char,
        needs_dotted_circle: bool,
    ) -> Option<fontdb::ID> {
        let mut pattern = Pattern::new(self.library)?;
        pattern.add_string(constants::FC_FAMILY, c"sans-serif")?;

        let charset = CharSet::new(self.library)?;
        charset.add(character)?;
        if needs_dotted_circle {
            charset.add('\u{25cc}')?;
        }
        pattern.add_charset(&charset)?;
        pattern.prepare(self.config);

        let mut result = ffi::FcResultNoMatch;
        let set = unsafe {
            (self.library.FcFontSort)(
                self.config,
                pattern.pointer,
                1,
                ptr::null_mut(),
                &mut result,
            )
        };
        if result != ffi::FcResultMatch {
            return None;
        }
        let set = FontSet::new(self.library, set)?;

        set.patterns().find_map(|candidate| {
            candidate
                .supports(character, needs_dotted_circle)
                .then(|| candidate.render_prepare(self.config, &pattern))
                .flatten()
                .and_then(|prepared| face_from_pattern(database, &prepared))
        })
    }
}

impl Drop for Fontconfig {
    fn drop(&mut self) {
        unsafe { (self.library.FcConfigDestroy)(self.config) }
    }
}

struct Pattern<'library> {
    library: &'library ffi::Fc,
    pointer: *mut ffi::FcPattern,
    owned: bool,
}

impl<'library> Pattern<'library> {
    fn new(library: &'library ffi::Fc) -> Option<Self> {
        let pointer = unsafe { (library.FcPatternCreate)() };
        Self::from_owned(library, pointer)
    }

    fn from_owned(library: &'library ffi::Fc, pointer: *mut ffi::FcPattern) -> Option<Self> {
        (!pointer.is_null()).then_some(Self {
            library,
            pointer,
            owned: true,
        })
    }

    fn from_borrowed(library: &'library ffi::Fc, pointer: *mut ffi::FcPattern) -> Option<Self> {
        (!pointer.is_null()).then_some(Self {
            library,
            pointer,
            owned: false,
        })
    }

    fn add_string(&mut self, object: &CStr, value: &CStr) -> Option<()> {
        let added = unsafe {
            (self.library.FcPatternAddString)(self.pointer, object.as_ptr(), value.as_ptr().cast())
        };
        (added != 0).then_some(())
    }

    fn add_charset(&mut self, charset: &CharSet<'_>) -> Option<()> {
        let added = unsafe {
            (self.library.FcPatternAddCharSet)(
                self.pointer,
                constants::FC_CHARSET.as_ptr(),
                charset.pointer,
            )
        };
        (added != 0).then_some(())
    }

    fn prepare(&mut self, config: *mut ffi::FcConfig) {
        unsafe {
            (self.library.FcConfigSubstitute)(config, self.pointer, ffi::FcMatchPattern);
            (self.library.FcDefaultSubstitute)(self.pointer);
        }
    }

    fn render_prepare(
        &self,
        config: *mut ffi::FcConfig,
        requested: &Pattern<'_>,
    ) -> Option<Pattern<'library>> {
        let pointer =
            unsafe { (self.library.FcFontRenderPrepare)(config, requested.pointer, self.pointer) };
        Pattern::from_owned(self.library, pointer)
    }

    fn get_string(&self, object: &CStr) -> Option<&CStr> {
        let mut value = ptr::null_mut();
        let result = unsafe {
            (self.library.FcPatternGetString)(self.pointer, object.as_ptr(), 0, &mut value)
        };
        if result != ffi::FcResultMatch || value.is_null() {
            return None;
        }
        Some(unsafe { CStr::from_ptr(value.cast()) })
    }

    fn get_integer(&self, object: &CStr) -> Option<i32> {
        let mut value = 0;
        let result = unsafe {
            (self.library.FcPatternGetInteger)(self.pointer, object.as_ptr(), 0, &mut value)
        };
        (result == ffi::FcResultMatch).then_some(value)
    }

    fn supports(&self, character: char, needs_dotted_circle: bool) -> bool {
        let mut charset = ptr::null_mut();
        let result = unsafe {
            (self.library.FcPatternGetCharSet)(
                self.pointer,
                constants::FC_CHARSET.as_ptr(),
                0,
                &mut charset,
            )
        };
        if result != ffi::FcResultMatch || charset.is_null() {
            return false;
        }
        unsafe {
            (self.library.FcCharSetHasChar)(charset, character as u32) != 0
                && (!needs_dotted_circle
                    || (self.library.FcCharSetHasChar)(charset, '\u{25cc}' as u32) != 0)
        }
    }
}

impl Drop for Pattern<'_> {
    fn drop(&mut self) {
        if self.owned {
            unsafe { (self.library.FcPatternDestroy)(self.pointer) }
        }
    }
}

struct CharSet<'library> {
    library: &'library ffi::Fc,
    pointer: *mut ffi::FcCharSet,
}

impl<'library> CharSet<'library> {
    fn new(library: &'library ffi::Fc) -> Option<Self> {
        let pointer = unsafe { (library.FcCharSetCreate)() };
        (!pointer.is_null()).then_some(Self { library, pointer })
    }

    fn add(&self, character: char) -> Option<()> {
        let added = unsafe { (self.library.FcCharSetAddChar)(self.pointer, character as u32) };
        (added != 0).then_some(())
    }
}

impl Drop for CharSet<'_> {
    fn drop(&mut self) {
        unsafe { (self.library.FcCharSetDestroy)(self.pointer) }
    }
}

struct FontSet<'library> {
    library: &'library ffi::Fc,
    pointer: *mut ffi::FcFontSet,
}

impl<'library> FontSet<'library> {
    fn new(library: &'library ffi::Fc, pointer: *mut ffi::FcFontSet) -> Option<Self> {
        (!pointer.is_null()).then_some(Self { library, pointer })
    }

    fn patterns(&self) -> impl Iterator<Item = Pattern<'library>> + '_ {
        let count = unsafe { (*self.pointer).nfont.max(0) as usize };
        (0..count).filter_map(|index| {
            let pointer = unsafe { *(*self.pointer).fonts.add(index) };
            Pattern::from_borrowed(self.library, pointer)
        })
    }
}

impl Drop for FontSet<'_> {
    fn drop(&mut self) {
        unsafe { (self.library.FcFontSetDestroy)(self.pointer) }
    }
}

#[cfg(test)]
mod tests {
    use fontdb::Database;

    use crate::glyph::font::platform::fontconfig;

    #[test]
    fn resolves_the_system_default_and_an_ascii_fallback() {
        let mut database = Database::new();
        database.load_system_fonts();

        let resolver = fontconfig::Fontconfig::new().unwrap();

        assert!(resolver.default_font(&database).is_some());
        assert!(resolver.fallback_font(&database, 'A', false).is_some());
    }
}
