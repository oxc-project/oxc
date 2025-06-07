#![allow(
    unused,
    clippy::inline_always,
    clippy::missing_panics_doc,
    clippy::needless_pass_by_ref_mut,
    clippy::todo,
    clippy::unused_self,
    clippy::enum_variant_names,
    clippy::struct_field_names
)] // FIXME: all these needs to be fixed.

mod generated {
    pub mod format;
}
mod formatter;
mod options;
mod parentheses;
mod utils;
mod write;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;

use crate::formatter::FormatContext;
pub use crate::options::*;

pub struct Formatter<'a> {
    allocator: &'a Allocator,
    source_text: &'a str,
    options: FormatOptions,
}

impl<'a> Formatter<'a> {
    pub fn new(allocator: &'a Allocator, options: FormatOptions) -> Self {
        Self { allocator, source_text: "", options }
    }

    pub fn build(mut self, program: &Program<'a>) -> String {
        let source_text = program.source_text;
        self.source_text = source_text;
        let context = FormatContext::new(program, self.options);
        let formatted = formatter::format(
            program,
            context,
            formatter::Arguments::new(&[formatter::Argument::new(program)]),
        )
        .unwrap();
        formatted.print().unwrap().into_code()
    }
}
