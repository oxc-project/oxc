#[derive(Clone, Copy, Debug, Default)]
pub struct ParserOptions {
    /// Used to adjust Span positions to fit the global source code.
    pub span_offset: u32,
    /// Unicode mode(`u` or `v` flag) enabled or not.
    pub unicode_mode: bool,
    /// Extended Unicode mode(`v` flag) enabled or not.
    pub unicode_sets_mode: bool,
}

impl ParserOptions {
    #[must_use]
    pub fn with_span_offset(self, span_offset: u32) -> ParserOptions {
        ParserOptions { span_offset, ..self }
    }

    #[must_use]
    pub fn with_unicode_mode(self) -> ParserOptions {
        ParserOptions { unicode_mode: true, ..self }
    }

    #[must_use]
    pub fn with_unicode_sets_mode(self) -> ParserOptions {
        ParserOptions { unicode_mode: true, unicode_sets_mode: true, ..self }
    }
}
