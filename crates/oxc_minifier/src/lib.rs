//! ECMAScript Minifier

mod compressor;
mod mangler;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;

pub use crate::{
    compressor::{CompressOptions, Compressor},
    mangler::ManglerBuilder,
};

#[derive(Debug, Clone, Copy)]
pub struct MinifierOptions {
    pub mangle: bool,
    pub compress: CompressOptions,
}

impl Default for MinifierOptions {
    fn default() -> Self {
        Self { mangle: true, compress: CompressOptions::default() }
    }
}

pub struct Minifier {
    options: MinifierOptions,
}

impl Minifier {
    pub fn new(options: MinifierOptions) -> Self {
        Self { options }
    }

    pub fn build<'a>(self, allocator: &'a Allocator, program: &mut Program<'a>) {
        Compressor::new(allocator, self.options.compress).build(program);
        // if self.options.mangle {
        // let mangler = ManglerBuilder.build(program);
        // printer.with_mangler(mangler);
        // }
    }
}
