use oxc_span::SourceType;
use oxc_str::CompactStr;

use crate::frameworks::FrameworkOptions;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct JavaScriptSource<'a> {
    pub source_text: &'a str,
    pub source_type: SourceType,
    /// The javascript source could be embedded in some file,
    /// use `start` to record start offset of js block in the original file.
    pub start: u32,
    #[expect(dead_code)]
    is_partial: bool,

    // some partial sources can have special options defined, like Vue's `<script setup>`.
    pub framework_options: FrameworkOptions,

    /// Variables that the framework declares for this block, like the keys of Astro's
    /// `<script define:vars={{ ... }}>`.
    pub globals: Vec<CompactStr>,
}

impl<'a> JavaScriptSource<'a> {
    pub fn new(source_text: &'a str, source_type: SourceType) -> Self {
        Self {
            source_text,
            source_type,
            start: 0,
            is_partial: false,
            framework_options: FrameworkOptions::Default,
            globals: Vec::new(),
        }
    }

    pub fn partial(source_text: &'a str, source_type: SourceType, start: u32) -> Self {
        Self::partial_with_framework_options(
            source_text,
            source_type,
            FrameworkOptions::Default,
            start,
        )
    }

    pub fn partial_with_framework_options(
        source_text: &'a str,
        source_type: SourceType,
        framework_options: FrameworkOptions,
        start: u32,
    ) -> Self {
        Self {
            source_text,
            source_type,
            start,
            is_partial: true,
            framework_options,
            globals: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_globals(mut self, globals: Vec<CompactStr>) -> Self {
        self.globals = globals;
        self
    }

    pub fn as_str(&self) -> &'a str {
        &self.source_text[(self.start as usize)..]
    }
}

impl AsRef<str> for JavaScriptSource<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
