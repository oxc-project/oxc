//! ECMAScript Minifier

#![allow(clippy::literal_string_with_formatting_args)]

mod compressor;
mod ctx;
mod keep_var;
mod options;
mod peephole;

#[cfg(test)]
mod tester;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_mangler::Mangler;
use oxc_semantic::{Scoping, SemanticBuilder, Stats};

pub use oxc_mangler::{MangleOptions, MangleOptionsKeepNames};

pub use crate::{
    compressor::Compressor, options::CompressOptions, options::CompressOptionsKeepNames,
};

#[derive(Debug, Clone, Copy)]
pub struct MinifierOptions {
    pub mangle: Option<MangleOptions>,
    pub compress: Option<CompressOptions>,
}

impl Default for MinifierOptions {
    fn default() -> Self {
        Self { mangle: Some(MangleOptions::default()), compress: Some(CompressOptions::default()) }
    }
}

pub struct MinifierReturn {
    pub scoping: Option<Scoping>,
}

pub struct Minifier {
    options: MinifierOptions,
}

impl Minifier {
    pub fn new(options: MinifierOptions) -> Self {
        Self { options }
    }

    pub fn build<'a>(self, allocator: &'a Allocator, program: &mut Program<'a>) -> MinifierReturn {
        let stats = if let Some(compress) = self.options.compress {
            let semantic = SemanticBuilder::new().build(program).semantic;
            let stats = semantic.stats();
            let scoping = semantic.into_scoping();
            Compressor::new(allocator, compress).build_with_scoping(scoping, program);
            stats
        } else {
            Stats::default()
        };
        let scoping = self.options.mangle.map(|options| {
            let semantic = SemanticBuilder::new()
                .with_stats(stats)
                .with_scope_tree_child_ids(true)
                .build(program)
                .semantic;
            Mangler::default().with_options(options).build_with_semantic(semantic, program)
        });
        MinifierReturn { scoping }
    }
}
