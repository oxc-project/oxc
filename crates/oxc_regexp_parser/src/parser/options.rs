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
}
impl Default for ParserOptions {
    fn default() -> Self {
        Self { strict: false, ecma_version: 2025 }
    }
}

impl ParserOptions {
    #[must_use]
    pub fn new(self, is_strict: bool, ecma_version: u32) -> Self {
        Self { strict: is_strict, ecma_version }
    }
}
