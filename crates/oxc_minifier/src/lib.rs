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

mod compression_pass;
mod compressor;
pub(crate) mod generated;
mod keep_var;
mod minifier_traverse;
mod options;
mod peephole;
pub mod property;
mod state;
mod symbol_liveness;
mod symbol_metadata;
mod symbol_state;
mod symbol_value;
mod traverse_context;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_index::IndexVec;
use oxc_mangler::Mangler;
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_str::CompactStr;
use oxc_syntax::class::ClassId;
use rustc_hash::FxHashMap;

use crate::state::CompressionMode;

pub use oxc_mangler::{MangleOptions, MangleOptionsKeepNames};
pub use property::{
    InvalidManglePropertyCacheTarget, ManglePropertiesOptions, ManglePropertyCache,
    PropertyKeyOrigin, PropertyKeyProvenance, PropertyMangler, PropertyMapping,
    is_valid_property_mangle_cache_target,
};

pub(crate) use crate::generated::traverse::Traverse;
#[doc(hidden)]
pub(crate) use crate::traverse_context::MinifierTraverseCtx as TraverseCtx;
pub(crate) use crate::traverse_context::ReusableMinifierTraverseCtx as ReusableTraverseCtx;
pub use crate::{compressor::Compressor, options::*};

#[derive(Debug, Clone)]
pub struct MinifierOptions {
    pub mangle: Option<MangleOptions>,
    /// Property-name mangling for the current [`Program`].
    ///
    /// This option does not coordinate mappings across programs. Bundlers that need cross-file
    /// consistency must collect all programs with [`PropertyMangler`], assign once, and rewrite
    /// each program exactly once before emitting chunks.
    pub mangle_properties: Option<ManglePropertiesOptions>,
    pub compress: Option<CompressOptions>,
}

impl Default for MinifierOptions {
    fn default() -> Self {
        Self {
            mangle: Some(MangleOptions::default()),
            mangle_properties: None,
            compress: Some(CompressOptions::default()),
        }
    }
}

pub struct MinifierReturn {
    pub scoping: Option<Scoping>,

    /// A vector where each element corresponds to a class in declaration order.
    /// Each element is a mapping from original private member names to their mangled names.
    pub class_private_mappings: Option<IndexVec<ClassId, FxHashMap<String, CompactStr>>>,

    /// Updated property-name cache when property mangling ran.
    pub property_mangle_cache: Option<ManglePropertyCache>,

    /// Total number of iterations ran. Useful for debugging performance issues.
    pub iterations: u8,
}

pub struct Minifier {
    options: MinifierOptions,
    property_key_provenance: Option<PropertyKeyProvenance>,
}

impl<'a> Minifier {
    pub fn new(options: MinifierOptions) -> Self {
        Self { options, property_key_provenance: None }
    }

    /// Attach property-key provenance produced by a transformer for this exact `Program`.
    #[must_use]
    pub fn with_property_key_provenance(mut self, provenance: PropertyKeyProvenance) -> Self {
        self.property_key_provenance = Some(provenance);
        self
    }

    pub fn minify(self, allocator: &'a Allocator, program: &mut Program<'a>) -> MinifierReturn {
        self.build(CompressionMode::Full, allocator, program)
    }

    pub fn dce(self, allocator: &'a Allocator, program: &mut Program<'a>) -> MinifierReturn {
        self.build(CompressionMode::TreeShakeOnly, allocator, program)
    }

    fn build(
        self,
        mode: CompressionMode,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
    ) -> MinifierReturn {
        let Self { options, property_key_provenance } = self;
        let MinifierOptions { mangle, mangle_properties, compress } = options;

        // Rewrite property names before compression. Compression can erase quote boundaries and
        // fold annotated literals, so collecting before and rewriting after it is not sound.
        let property_mangle_cache = mangle_properties.map(|options| {
            let mut mangler = PropertyMangler::new(options);
            mangler.collect(program, property_key_provenance.as_ref());
            mangler.assign();
            mangler.rewrite(program, allocator, property_key_provenance.as_ref());
            mangler.into_cache()
        });

        let (stats, iterations) = compress
            .map(|options| {
                let semantic = SemanticBuilder::new().build(program).semantic;
                let stats = semantic.stats();
                let scoping = semantic.into_scoping();
                let compressor = Compressor::new(allocator);
                let iterations = if matches!(mode, CompressionMode::TreeShakeOnly) {
                    let options = CompressOptions {
                        target: options.target,
                        treeshake: options.treeshake,
                        ..CompressOptions::dce()
                    };
                    compressor.dead_code_elimination_with_scoping(program, scoping, options)
                } else {
                    compressor.build_with_scoping(program, scoping, options)
                };
                (Some(stats), iterations)
            })
            .unwrap_or_default();
        let (scoping, class_private_mappings) = mangle
            .map(|options| {
                let mut builder =
                    SemanticBuilder::new().with_build_nodes(true).with_class_table(true);
                if let Some(stats) = stats {
                    builder = builder.with_stats(stats);
                }
                let mut semantic = builder.build(program).semantic;
                let class_private_mappings = Mangler::default()
                    .with_options(options)
                    .build_with_semantic(&mut semantic, program);
                (semantic.into_scoping(), class_private_mappings)
            })
            .map_or((None, None), |(scoping, mappings)| (Some(scoping), Some(mappings)));
        MinifierReturn { scoping, class_private_mappings, property_mangle_cache, iterations }
    }
}
