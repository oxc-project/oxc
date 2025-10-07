//! # Oxc Minifier
//!
//! High-performance JavaScript/TypeScript minifier focused on maximum compression.
//!
//! ## Overview
//!
//! The Oxc minifier applies a comprehensive set of optimizations to JavaScript code
//! to achieve the smallest possible output while maintaining correctness. It draws
//! inspiration from industry-leading minifiers like Closure Compiler, Terser, esbuild,
//! and SWC.
//!
//! ## Features
//!
//! - **Maximum Compression**: Fixed-point iteration ensures all optimization opportunities are found
//! - **Comprehensive Optimizations**: 17+ transformation passes and growing
//! - **100% Correct**: Extensive testing with test262, Babel, and TypeScript test suites
//! - **Fast**: Efficient algorithms and arena allocation for performance
//!
//! ## Example
//!
//! ```rust
//! use oxc_minifier::{Minifier, MinifierOptions};
//! use oxc_allocator::Allocator;
//! use oxc_parser::Parser;
//! use oxc_span::SourceType;
//!
//! let allocator = Allocator::default();
//! let source_text = "const x = 1 + 1; console.log(x);";
//! let source_type = SourceType::mjs();
//! let ret = Parser::new(&allocator, source_text, source_type).parse();
//! let mut program = ret.program;
//!
//! let options = MinifierOptions::default();
//! let minifier = Minifier::new(options);
//! let result = minifier.minify(&allocator, &mut program);
//! ```
//!
//! ## Architecture
//!
//! The minifier consists of:
//! - **Compressor**: Orchestrates the optimization pipeline
//! - **Peephole Optimizations**: Individual transformation passes
//! - **Mangler**: Variable renaming for size reduction
//!
//! See the [crate documentation](https://github.com/oxc-project/oxc/tree/main/crates/oxc_minifier) for more details.

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
use oxc_index::IndexVec;
use oxc_mangler::Mangler;
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_span::CompactStr;
use oxc_syntax::class::ClassId;
use rustc_hash::FxHashMap;

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

    /// A vector where each element corresponds to a class in declaration order.
    /// Each element is a mapping from original private member names to their mangled names.
    pub class_private_mappings: Option<IndexVec<ClassId, FxHashMap<String, CompactStr>>>,

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
        let (scoping, class_private_mappings) = self
            .options
            .mangle
            .map(|options| {
                let mut semantic = SemanticBuilder::new()
                    .with_stats(stats)
                    .with_scope_tree_child_ids(true)
                    .build(program)
                    .semantic;
                let class_private_mappings = Mangler::default()
                    .with_options(options)
                    .build_with_semantic(&mut semantic, program);
                (semantic.into_scoping(), class_private_mappings)
            })
            .map_or((None, None), |(scoping, mappings)| (Some(scoping), Some(mappings)));
        MinifierReturn { scoping, class_private_mappings, iterations }
    }
}
