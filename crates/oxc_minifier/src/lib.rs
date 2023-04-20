//! ECMAScript Minifier

#![feature(let_chains)]

mod compressor;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;

pub use crate::compressor::CompressOptions;
use crate::compressor::Compressor;

#[derive(Debug, Default, Clone, Copy)]
pub struct MinifierOptions {
    pub compress: CompressOptions,
}

pub struct Minifier<'a> {
    compressor: Compressor<'a>,
}

impl<'a> Minifier<'a> {
    pub fn new(allocator: &'a Allocator, options: MinifierOptions) -> Self {
        Self { compressor: Compressor::new(allocator, options.compress) }
    }

    pub fn build<'b>(self, program: &'b mut Program<'a>) {
        self.compressor.build(program);
    }
}
