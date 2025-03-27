#![allow(unused)]

mod generated {
    pub mod format;
}
mod comment;
mod context;
mod formatter;
mod options;
mod write;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;

use crate::{context::JsFormatOptions, formatter::FormatContext};

pub use crate::options::*;

pub struct Formatter<'a> {
    allocator: &'a Allocator,
    source_text: &'a str,
    options: FormatterOptions,
}

impl<'a> Formatter<'a> {
    pub fn new(allocator: &'a Allocator, options: FormatterOptions) -> Self {
        Self { allocator, source_text: "", options }
    }

    pub fn build(&mut self, program: &Program<'a>) -> String {
        let source_text = program.source_text;
        self.source_text = source_text;
        let options = JsFormatOptions::default();
        let context = FormatContext::new(program, options);
        let formatted = formatter::format(
            program,
            context,
            formatter::Arguments::new(&[formatter::Argument::new(program)]),
        )
        .unwrap();
        formatted.print().unwrap().into_code()
    }
}
