#![allow(clippy::wildcard_imports)]

//! ECMAScript Minifier

mod ast_passes;
mod compressor;
mod keep_var;
mod node_util;
mod options;
mod plugins;
mod tri;
mod ty;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_mangler::{Mangler, ManglerBuilder};

pub use crate::{
    ast_passes::{CompressorPass, RemoveDeadCode, RemoveSyntax},
    compressor::Compressor,
    options::CompressOptions,
    plugins::*,
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
        let mangler = self.options.mangle.then(|| ManglerBuilder::default().build(program));
        MinifierReturn { mangler }
    }
}
