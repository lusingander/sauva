use std::{collections::VecDeque, sync::Arc};

use crate::{
    glyph::{
        CanvasSize, DEFAULT_CACHE_MAX_BYTES, DEFAULT_CACHE_MAX_ENTRIES, GlyphPreview,
        GlyphRenderContext, GlyphRenderError, GlyphRenderer, render_context,
        settings::GlyphPreviewSettings,
    },
    unicode::CodePoint,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphCacheKey {
    code_point: CodePoint,
    size: CanvasSize,
    context: GlyphRenderContext,
}

#[derive(Debug)]
struct GlyphCacheEntry {
    key: GlyphCacheKey,
    preview: Arc<GlyphPreview>,
    byte_len: usize,
}

pub struct GlyphCache {
    renderer: GlyphRenderer,
    entries: VecDeque<GlyphCacheEntry>,
    total_bytes: usize,
    max_entries: usize,
    max_bytes: usize,
}

impl GlyphCache {
    pub fn with_settings(settings: GlyphPreviewSettings) -> Result<Self, GlyphRenderError> {
        Self::with_limits(settings, DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_MAX_BYTES)
    }

    pub(super) fn with_limits(
        settings: GlyphPreviewSettings,
        max_entries: usize,
        max_bytes: usize,
    ) -> Result<Self, GlyphRenderError> {
        Ok(Self {
            renderer: GlyphRenderer::new(settings)?,
            entries: VecDeque::new(),
            total_bytes: 0,
            max_entries,
            max_bytes,
        })
    }

    pub fn get_or_render(
        &mut self,
        code_point: CodePoint,
        size: CanvasSize,
    ) -> Result<Arc<GlyphPreview>, GlyphRenderError> {
        let key = GlyphCacheKey {
            code_point,
            size,
            context: render_context(code_point),
        };
        if let Some(position) = self.entries.iter().position(|entry| entry.key == key) {
            let entry = self
                .entries
                .remove(position)
                .expect("the located glyph cache entry exists");
            let preview = Arc::clone(&entry.preview);
            self.entries.push_back(entry);
            return Ok(preview);
        }

        let preview = Arc::new(self.renderer.render(code_point, size)?);
        let byte_len = preview.byte_len();
        if self.max_entries == 0 || byte_len > self.max_bytes {
            return Ok(preview);
        }

        while self.entries.len() >= self.max_entries
            || self.total_bytes.saturating_add(byte_len) > self.max_bytes
        {
            let Some(evicted) = self.entries.pop_front() else {
                break;
            };
            self.total_bytes -= evicted.byte_len;
        }
        self.total_bytes += byte_len;
        self.entries.push_back(GlyphCacheEntry {
            key,
            preview: Arc::clone(&preview),
            byte_len,
        });
        Ok(preview)
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    #[cfg(test)]
    pub(super) const fn total_bytes(&self) -> usize {
        self.total_bytes
    }
}
