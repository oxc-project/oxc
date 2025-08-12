//! ECMAScript Minifier

mod compressor;
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
use oxc_semantic::{Scoping, SemanticBuilder};

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

    /// Total number of iterations ran. Useful for debugging performance issues.
    pub iterations: u8,
}

pub struct Minifier {
    options: MinifierOptions,
}

impl<'a> Minifier {
    pub fn new(options: MinifierOptions) -> Self {
        Self { options }
    }

    pub fn minify(self, allocator: &'a Allocator, program: &mut Program<'a>) -> MinifierReturn {
        self.build(false, allocator, program)
    }

    pub fn dce(self, allocator: &'a Allocator, program: &mut Program<'a>) -> MinifierReturn {
        self.build(true, allocator, program)
    }

    fn build(
        self,
        dce: bool,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
    ) -> MinifierReturn {
        let (stats, iterations) = self
            .options
            .compress
            .map(|options| {
                let semantic = SemanticBuilder::new().build(program).semantic;
                let stats = semantic.stats();
                let scoping = semantic.into_scoping();
                let compressor = Compressor::new(allocator);
                let iterations = if dce {
                    let options = CompressOptions {
                        target: options.target,
                        treeshake: options.treeshake,
                        ..CompressOptions::dce()
                    };
                    compressor.dead_code_elimination_with_scoping(program, scoping, options)
                } else {
                    compressor.build_with_scoping(program, scoping, options)
                };
                (stats, iterations)
            })
            .unwrap_or_default();
        let scoping = self.options.mangle.map(|options| {
            let mut semantic = SemanticBuilder::new()
                .with_stats(stats)
                .with_scope_tree_child_ids(true)
                .build(program)
                .semantic;
            Mangler::default().with_options(options).build_with_semantic(&mut semantic, program);
            semantic.into_scoping()
        });
        MinifierReturn { scoping, iterations }
    }
}
