#![allow(clippy::wildcard_imports, clippy::unused_self)]
//! ECMAScript Minifier

mod ast_passes;
mod compressor;
mod folder;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_mangler::{Mangler, ManglerBuilder};

pub use crate::{
    ast_passes::{RemoveDeadCode, RemoveParens, ReplaceGlobalDefines, ReplaceGlobalDefinesConfig},
    compressor::{CompressOptions, Compressor},
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
        let mangler = self.options.mangle.then(|| ManglerBuilder.build(program));
        MinifierReturn { mangler }
    }
}
