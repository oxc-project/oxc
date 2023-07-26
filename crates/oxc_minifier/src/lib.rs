//! ECMAScript Minifier

#![feature(let_chains)]

mod compressor;
mod mangler;
mod printer;

use oxc_allocator::Allocator;
use oxc_ast_lower::AstLower;
use oxc_parser::Parser;
use oxc_span::SourceType;

pub use crate::{
    compressor::{CompressOptions, Compressor},
    mangler::ManglerBuilder,
    printer::{Printer, PrinterOptions},
};

#[derive(Debug, Clone, Copy)]
pub struct MinifierOptions {
    pub mangle: bool,
    pub compress: CompressOptions,
    pub print: PrinterOptions,
}

impl Default for MinifierOptions {
    fn default() -> Self {
        Self { mangle: true, compress: CompressOptions::default(), print: PrinterOptions }
    }
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
        let ret = AstLower::new(&allocator, self.source_text, self.source_type).build(&ret.program);
        let program = allocator.alloc(ret.program);
        let semantic = ret.semantic;
        let _semantic = Compressor::new(&allocator, semantic, self.options.compress).build(program);
        let mut printer = Printer::new(self.source_text.len(), self.options.print);
        if self.options.mangle {
            let mangler = ManglerBuilder::new(self.source_text, self.source_type).build(program);
            printer.with_mangler(mangler);
        }
        printer.build(program)
    }
}
