//! ECMAScript Minifier

#![feature(let_chains)]

mod compressor;
mod printer;

use oxc_allocator::Allocator;
use oxc_ast_lower::AstLower;
use oxc_parser::Parser;
use oxc_span::SourceType;

pub use crate::{compressor::CompressOptions, printer::PrinterOptions};
use crate::{compressor::Compressor, printer::Printer};

#[derive(Debug, Clone, Copy)]
pub struct MinifierOptions {
    pub mangle: bool,
    pub compress: CompressOptions,
    pub print: PrinterOptions,
}

impl Default for MinifierOptions {
    fn default() -> Self {
        Self {
            mangle: true,
            compress: CompressOptions::default(),
            print: PrinterOptions::default(),
        }
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
        let ret = AstLower::new(&allocator, self.source_type).build(&ret.program);
        let mut program = ret.program;
        let semantic = ret.semantic;
        let mut semantic =
            Compressor::new(&allocator, semantic, self.options.compress).build(&mut program);
        if self.options.mangle {
            semantic.mangle();
        }
        Printer::new(self.source_text.len(), self.options.print)
            .with_mangle(semantic.symbol_table, self.options.mangle)
            .build(&program)
    }
}
