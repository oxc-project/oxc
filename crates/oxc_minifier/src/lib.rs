//! ECMAScript Minifier

#![feature(let_chains)]

mod compressor;
mod printer;

use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_ast_lower::AstLower;
use oxc_parser::Parser;

pub use crate::compressor::CompressOptions;
use crate::compressor::Compressor;
use crate::printer::Printer;
pub use crate::printer::PrinterOptions;

#[derive(Debug, Default, Clone, Copy)]
pub struct MinifierOptions {
    pub compress: CompressOptions,
    pub print: PrinterOptions,
}

pub struct Minifier<'a> {
    source_text: &'a str,
    source_type: SourceType,
    options: MinifierOptions,
}

impl<'a> Minifier<'a> {
    pub fn new(source_text: &'a str, source_type: SourceType, options: MinifierOptions) -> Self {
        Self { source_text, source_type, options }
    }

    pub fn build(self) -> String {
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, self.source_text, self.source_type).parse();
        let mut program = AstLower::new(&allocator, self.source_type).build(&ret.program);
        Compressor::new(&allocator, self.options.compress).build(&mut program);
        Printer::new(self.source_text.len(), self.options.print).build(&program)
    }
}
