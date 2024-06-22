#[derive(Clone, Copy, Debug, Default)]
pub struct ParserOptions {
    // Not planning to implement
    // pub strict: bool,
    // pub ecma_version: u32, // or Enum?
    /// Used to adjust Span pos to the global source code.
    pub span_offset: u32,
}

impl ParserOptions {
    #[must_use]
    pub fn with_span_offset(self, span_offset: u32) -> ParserOptions {
        ParserOptions { span_offset, ..self }
    }
}
