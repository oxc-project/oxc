#[derive(Clone, Copy, Debug)]
pub struct ParserOptions {
    /// The flag to disable Annex B syntax.
    /// Default: false
    pub strict: bool,
    /// ECMAScript version.
    /// - `2015` added `u` and `y` flags
    /// - `2018` added `s` flag, Named Capturing Group, Lookbehind Assertion,
    ///   and Unicode Property Escape
    /// - `2019`, `2020`, and `2021` added more valid Unicode Property Escapes
    /// - `2022` added `d` flag
    /// - `2023` added more valid Unicode Property Escapes
    /// - `2024` added `v` flag
    /// - `2025` added duplicate named capturing groups
    /// Default: 2025
    pub ecma_version: u32, // TODO: Enum?

    /// Used to adjust Span pos to the global source code.
    pub span_offset: u32,
}
impl Default for ParserOptions {
    fn default() -> Self {
        Self { strict: false, ecma_version: 2025, span_offset: 0 }
    }
}

impl ParserOptions {
    #[must_use]
    pub fn new(is_strict: bool, ecma_version: u32) -> Self {
        Self { strict: is_strict, ecma_version, span_offset: 0 }
    }

    #[must_use]
    pub fn with_span_offset(self, span_offset: u32) -> ParserOptions {
        ParserOptions { span_offset, ..self }
    }
}
