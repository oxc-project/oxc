//! JavaScript code compression through peephole optimizations.
//!
//! The compressor applies various semantic-preserving transformations to reduce code size.
//! It operates in multiple phases:
//!
//! 1. **Normalization**: Converts code to standard forms for easier optimization
//! 2. **Peephole Optimizations**: Local transformations applied in a fixed-point loop
//! 3. **Late Optimizations**: Final cleanup and optimization passes
//!
//! ## Optimization Categories
//!
//! - **Constant Folding**: Evaluates constant expressions at compile time
//! - **Dead Code Elimination**: Removes unreachable or unused code
//! - **Expression Simplification**: Reduces complex expressions
//! - **Control Flow Optimization**: Simplifies conditional statements and loops

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_traverse::ReusableTraverseCtx;

use crate::{
    CompressOptions,
    peephole::{
        DeadCodeElimination, LatePeepholeOptimizations, Normalize, NormalizeOptions,
        PeepholeOptimizations,
    },
    state::MinifierState,
};

/// The main compressor that orchestrates peephole optimizations.
///
/// The compressor applies optimizations in multiple phases:
/// 1. Normalization to prepare code for optimization
/// 2. Iterative peephole optimizations until a fixed point
/// 3. Late optimizations for final cleanup
pub struct Compressor<'a> {
    allocator: &'a Allocator,
}

impl<'a> Compressor<'a> {
    /// Creates a new compressor instance.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Memory allocator for creating new AST nodes during optimization
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { allocator }
    }

    /// Compresses the provided program with default scoping analysis.
    ///
    /// This method first builds semantic information, then applies compression optimizations.
    ///
    /// # Arguments
    ///
    /// * `program` - The JavaScript AST to optimize (modified in-place)
    /// * `options` - Compression options controlling which optimizations to apply
    pub fn build(self, program: &mut Program<'a>, options: CompressOptions) {
        let scoping = SemanticBuilder::new().build(program).semantic.into_scoping();
        self.build_with_scoping(program, scoping, options);
    }

    /// Compresses the program using existing scoping information.
    ///
    /// This is more efficient when scoping information is already available,
    /// as it avoids rebuilding the semantic analysis.
    ///
    /// # Arguments
    ///
    /// * `program` - The JavaScript AST to optimize (modified in-place)  
    /// * `scoping` - Pre-computed scoping and symbol information
    /// * `options` - Compression options controlling which optimizations to apply
    pub fn build_with_scoping(
        self,
        program: &mut Program<'a>,
        scoping: Scoping,
        options: CompressOptions,
    ) {
        let state = MinifierState::new(program.source_type, options);
        let mut ctx = ReusableTraverseCtx::new(state, scoping, self.allocator);
        let normalize_options =
            NormalizeOptions { convert_while_to_fors: true, convert_const_to_let: true };
        Normalize::new(normalize_options).build(program, &mut ctx);
        PeepholeOptimizations::new().run_in_loop(program, &mut ctx);
        LatePeepholeOptimizations::new().build(program, &mut ctx);
    }

    pub fn dead_code_elimination(self, program: &mut Program<'a>, options: CompressOptions) {
        let scoping = SemanticBuilder::new().build(program).semantic.into_scoping();
        self.dead_code_elimination_with_scoping(program, scoping, options);
    }

    pub fn dead_code_elimination_with_scoping(
        self,
        program: &mut Program<'a>,
        scoping: Scoping,
        options: CompressOptions,
    ) {
        let state = MinifierState::new(program.source_type, options);
        let mut ctx = ReusableTraverseCtx::new(state, scoping, self.allocator);
        let normalize_options =
            NormalizeOptions { convert_while_to_fors: false, convert_const_to_let: false };
        Normalize::new(normalize_options).build(program, &mut ctx);
        DeadCodeElimination::new().build(program, &mut ctx);
    }
}
