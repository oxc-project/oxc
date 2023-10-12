//! Oxc Codegen
//!
//! Supports
//!
//! * whitespace removal
//! * sourcemaps

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct CodegenOptions;

pub struct Codegen<const MINIFY: bool> {
    #[allow(unused)]
    options: CodegenOptions,

    /// Output Code
    code: Vec<u8>,
}

impl<const MINIFY: bool> Codegen<MINIFY> {
    pub fn new(source_len: usize, options: CodegenOptions) -> Self {
        // Initialize the output code buffer to reduce memory reallocation.
        // Minification will reduce by at least half of the original size.
        let capacity = if MINIFY { source_len / 2 } else { source_len };
        Self { options, code: Vec::with_capacity(capacity) }
    }

    pub fn build(self, _program: &Program<'_>) -> String {
        self.into_code()
    }

    fn into_code(self) -> String {
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}
