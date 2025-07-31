//! # OXC JavaScript Minifier
//!
//! A high-performance JavaScript minifier that reduces code size through various optimization techniques.
//!
//! ## Overview
//!
//! The minifier consists of three main components:
//! - **Compressor**: Applies semantic-preserving transformations to reduce code size
//! - **Mangler**: Shortens variable and function names while preserving scope semantics  
//! - **Printer**: Removes whitespace and formats the final output
//!
//! ## Quick Start
//!
//! ```rust
//! use oxc_allocator::Allocator;
//! use oxc_ast::ast::Program;
//! use oxc_minifier::{Minifier, MinifierOptions};
//!
//! # fn example(program: &mut Program) {
//! let allocator = Allocator::default();
//! let minifier = Minifier::new(MinifierOptions::default());
//! let result = minifier.build(&allocator, program);
//! # }
//! ```
//!
//! ## Custom Configuration
//!
//! ```rust
//! use oxc_minifier::{CompressOptions, MangleOptions, MinifierOptions};
//!
//! let options = MinifierOptions {
//!     mangle: Some(MangleOptions::default()),
//!     compress: Some(CompressOptions {
//!         drop_console: true,
//!         dead_code_elimination: true,
//!         ..Default::default()
//!     }),
//! };
//! ```
//!
//! ## Architecture
//!
//! The minification process follows this pipeline:
//! 1. **Semantic Analysis**: Build symbol tables and scope information
//! 2. **Compression**: Apply peephole optimizations and dead code elimination  
//! 3. **Mangling**: Shorten identifiers while preserving semantics
//! 4. **Output**: Generate optimized JavaScript code
//!
//! ## Safety
//!
//! The minifier makes certain assumptions about the input code to enable aggressive optimizations.
//! See the README for a complete list of assumptions and limitations.

#![allow(clippy::literal_string_with_formatting_args, clippy::needless_pass_by_ref_mut)]

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
use oxc_semantic::{Scoping, SemanticBuilder, Stats};

pub use oxc_mangler::{MangleOptions, MangleOptionsKeepNames};

pub use crate::{compressor::Compressor, options::*};

/// Configuration options for the JavaScript minifier.
///
/// This struct allows you to control which optimizations are applied during minification:
/// - Set `mangle` to `Some(MangleOptions::default())` to enable identifier shortening
/// - Set `compress` to `Some(CompressOptions::default())` to enable code compression
/// - Set either to `None` to disable that optimization phase
#[derive(Debug, Clone)]
pub struct MinifierOptions {
    /// Identifier mangling options. When `Some`, enables variable name shortening.
    pub mangle: Option<MangleOptions>,
    /// Code compression options. When `Some`, enables peephole optimizations.
    pub compress: Option<CompressOptions>,
}

impl Default for MinifierOptions {
    fn default() -> Self {
        Self { mangle: Some(MangleOptions::default()), compress: Some(CompressOptions::default()) }
    }
}

/// The result of a minification operation.
///
/// Contains the scoping information that may be needed for further processing
/// or debugging of the minified code.
pub struct MinifierReturn {
    /// Scoping information from the mangling phase, if mangling was enabled.
    /// This contains the final symbol table with shortened names.
    pub scoping: Option<Scoping>,
}

/// The main minifier that orchestrates the compression and mangling phases.
///
/// # Example
///
/// ```rust
/// use oxc_allocator::Allocator;
/// use oxc_minifier::{Minifier, MinifierOptions};
/// # use oxc_ast::ast::Program;
///
/// # fn example(program: &mut Program) {
/// let allocator = Allocator::default();
/// let minifier = Minifier::new(MinifierOptions::default());
/// let result = minifier.build(&allocator, program);
/// # }
/// ```
pub struct Minifier {
    options: MinifierOptions,
}

impl Minifier {
    /// Creates a new minifier with the specified options.
    ///
    /// # Arguments
    ///
    /// * `options` - Configuration controlling which optimizations to apply
    pub fn new(options: MinifierOptions) -> Self {
        Self { options }
    }

    /// Performs minification on the provided AST.
    ///
    /// This method applies the configured optimizations in the following order:
    /// 1. **Compression** (if enabled): Applies peephole optimizations and dead code elimination
    /// 2. **Mangling** (if enabled): Shortens identifier names while preserving semantics
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for AST node creation
    /// * `program` - The JavaScript AST to minify (modified in-place)
    ///
    /// # Returns
    ///
    /// A [`MinifierReturn`] containing scoping information from the mangling phase.
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
