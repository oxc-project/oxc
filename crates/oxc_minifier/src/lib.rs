//! ECMAScript Minifier

#![allow(clippy::literal_string_with_formatting_args, clippy::needless_pass_by_ref_mut)]

mod compressor;
mod context_utils;
mod ctx;
mod keep_var;
mod options;
mod peephole;
mod state;
mod symbol_value;

#[cfg(test)]
mod tester;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_mangler::Mangler;
use oxc_semantic::{Scoping, SemanticBuilder, Stats};

pub use oxc_mangler::{MangleOptions, MangleOptionsKeepNames};

pub use crate::{compressor::Compressor, options::*};

#[derive(Debug, Clone)]
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
        let stats = if let Some(options) = self.options.compress {
            let semantic = SemanticBuilder::new().build(program).semantic;
            let stats = semantic.stats();
            let scoping = semantic.into_scoping();
            Compressor::new(allocator).build_with_scoping(program, scoping, options);
            stats
        } else {
            Stats::default()
        };
        let scoping = self.options.mangle.map(|options| {
            let mut semantic = SemanticBuilder::new()
                .with_stats(stats)
                .with_scope_tree_child_ids(true)
                .build(program)
                .semantic;
            Mangler::default().with_options(options).build_with_semantic(&mut semantic, program);
            semantic.into_scoping()
        });
        MinifierReturn { scoping }
    }
}
