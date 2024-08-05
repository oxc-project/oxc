#[derive(Clone, Copy, Debug, Default)]
pub struct ParserOptions {
    /// Used to adjust Span positions to fit the global source code.
    pub span_offset: u32,
    /// The same as `u` flag, enable Unicode mode.
    pub unicode_flag: bool,
    /// The same as `v` flag, enable extended Unicode mode.
    pub unicode_sets_flag: bool,
}

impl ParserOptions {
    #[must_use]
    pub fn with_span_offset(self, span_offset: u32) -> ParserOptions {
        ParserOptions { span_offset, ..self }
    }

    #[must_use]
    /// Only for `PatternParser` alone usage API.
    /// `FlagParser` does not use, `(Literal)Parser` internally updates with parsed flags.
    pub fn with_unicode_flags(self, unicode_flag: bool, unicode_sets_flag: bool) -> ParserOptions {
        ParserOptions { unicode_flag, unicode_sets_flag, ..self }
    }
}
