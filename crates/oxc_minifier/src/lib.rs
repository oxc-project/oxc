//! ECMAScript Minifier

mod ast_passes;
mod compressor;
mod keep_var;
mod node_util;
mod options;

#[cfg(test)]
mod tester;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_mangler::Mangler;

pub use crate::{ast_passes::CompressorPass, compressor::Compressor, options::CompressOptions};
pub use oxc_mangler::MangleOptions;

#[derive(Debug, Clone, Copy)]
pub struct MinifierOptions {
    pub mangle: Option<MangleOptions>,
    pub compress: CompressOptions,
}

impl Default for MinifierOptions {
    fn default() -> Self {
        Self { mangle: Some(MangleOptions::default()), compress: CompressOptions::default() }
    }
}

pub struct MinifierReturn {
    pub mangler: Option<Mangler>,
}

pub struct Minifier {
    options: MinifierOptions,
}

impl Minifier {
    pub fn new(options: MinifierOptions) -> Self {
        Self { options }
    }

    pub fn build<'a>(self, allocator: &'a Allocator, program: &mut Program<'a>) -> MinifierReturn {
        Compressor::new(allocator, self.options.compress).build(program);
        let mangler = self
            .options
            .mangle
            .map(|options| Mangler::default().with_options(options).build(program));
        MinifierReturn { mangler }
    }
}
