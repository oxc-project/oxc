use oxc_formatter_core::{FormatContext, SourceText};

use crate::options::TomlFormatOptions;

/// Formatting context for TOML.
pub struct TomlFormatContext<'a> {
    options: TomlFormatOptions,
    source_text: SourceText<'a>,
}

impl<'a> TomlFormatContext<'a> {
    pub fn new(options: TomlFormatOptions, source_code: &'a str) -> Self {
        Self { options, source_text: SourceText::new(source_code) }
    }

    /// Returns the source text with the arena lifetime (vs the trait's borrow-elided `&str`).
    /// Slices taken via this method carry the `'a` lifetime,
    /// so they don't have to be re-allocated for `text(...)`.
    pub fn source_text(&self) -> SourceText<'a> {
        self.source_text
    }
}

impl FormatContext for TomlFormatContext<'_> {
    type Options = TomlFormatOptions;

    fn options(&self) -> &Self::Options {
        &self.options
    }

    fn source_code(&self) -> &str {
        &self.source_text
    }
}
