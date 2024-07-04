#[derive(Clone, Copy, Debug, Default)]
pub struct ParserOptions {
    /// Used to adjust Span pos to the global source code.
    pub span_offset: u32,
    /// The same as `u` flag.
    pub unicode_flag: bool,
    /// The same as `v` flag, it extends `u` flag behavior.
    pub unicode_sets_flag: bool,
    // Not planning to implement
    // pub strict: bool,
    // pub ecma_version: u32, // or Enum?
}

impl ParserOptions {
    #[must_use]
    pub fn with_span_offset(self, span_offset: u32) -> ParserOptions {
        ParserOptions { span_offset, ..self }
    }

    #[must_use]
    /// Only for using `PattenrParser` alone usage.
    /// `FlagParser` does not use, (Literal)`Parser` internally updates these value with parsed flags.
    pub fn with_unicode_flags(self, unicode_flag: bool, unicode_sets_flag: bool) -> ParserOptions {
        ParserOptions { unicode_flag, unicode_sets_flag, ..self }
    }
}
