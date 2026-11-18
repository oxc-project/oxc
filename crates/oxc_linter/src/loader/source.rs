use oxc_span::SourceType;

use crate::frameworks::FrameworkOptions;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct JavaScriptSource {
    pub source_type: SourceType,
    /// Offset of start of the JS/TS code within the file.
    pub start: u32,
    /// Offset of end of the JS/TS code within the file.
    pub end: u32,
    #[expect(dead_code)]
    is_partial: bool,

    // some partial sources can have special options defined, like Vue's `<script setup>`.
    pub framework_options: FrameworkOptions,
}

impl JavaScriptSource {
    /// Create a [`JavaScriptSource`] for a whole file, which is `end` bytes long.
    pub fn new(source_type: SourceType, end: u32) -> Self {
        Self {
            source_type,
            start: 0,
            end,
            is_partial: false,
            framework_options: FrameworkOptions::Default,
        }
    }

    /// Create a [`JavaScriptSource`] for JS/TS code embedded in a file,
    /// from offset `start` up to (not including) offset `end`.
    pub fn partial(source_type: SourceType, start: u32, end: u32) -> Self {
        Self::partial_with_framework_options(source_type, FrameworkOptions::Default, start, end)
    }

    /// Create a [`JavaScriptSource`] for JS/TS code embedded in a file,
    /// from offset `start` up to (not including) offset `end`, with `framework_options`.
    pub fn partial_with_framework_options(
        source_type: SourceType,
        framework_options: FrameworkOptions,
        start: u32,
        end: u32,
    ) -> Self {
        Self { source_type, start, end, is_partial: true, framework_options }
    }

    /// Get the text of the JS/TS code, given the text of the file which contains it.
    #[cfg(test)]
    pub fn source_text<'s>(&self, file_source_text: &'s str) -> &'s str {
        &file_source_text[self.start as usize..self.end as usize]
    }
}
