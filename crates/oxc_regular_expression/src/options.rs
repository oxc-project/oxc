use crate::ast::RegularExpressionFlags;

#[derive(Clone, Copy, Debug, Default)]
pub struct ParserOptions {
    /// Used to adjust Span positions to fit the global source code.
    pub span_offset: u32,
}

impl ParserOptions {
    #[must_use]
    pub fn with_span_offset(self, span_offset: u32) -> Self {
        Self { span_offset }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PatternParserOptions {
    /// Used to adjust Span positions to fit the global source code.
    pub span_offset: u32,
    /// Regular expression flags
    pub flags: RegularExpressionFlags,
}

impl PatternParserOptions {
    #[must_use]
    pub fn with_span_offset(self, span_offset: u32) -> Self {
        Self { span_offset, ..self }
    }

    #[must_use]
    pub fn with_flags(self, flags: RegularExpressionFlags) -> Self {
        Self { flags, ..self }
    }
}
