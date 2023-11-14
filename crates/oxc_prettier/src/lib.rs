//! Prettier
//!
//! A port of <https://github.com/prettier/prettier>

mod doc;
mod format;
mod macros;
mod printer;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;

use crate::{format::Format, printer::Printer};

pub struct PrettierOptions {
    /// Print semicolons at the ends of statements.
    /// Default: true
    semi: bool,
}

impl Default for PrettierOptions {
    fn default() -> Self {
        Self { semi: true }
    }
}

pub struct Prettier<'a> {
    allocator: &'a Allocator,

    options: PrettierOptions,
}

impl<'a> Prettier<'a> {
    pub fn new(allocator: &'a Allocator, _options: PrettierOptions) -> Self {
        Self { allocator, options: _options }
    }

    pub fn build(mut self, program: &Program<'a>) -> String {
        let doc = program.format(&mut self);
        Printer::new(doc).build()
    }
}
