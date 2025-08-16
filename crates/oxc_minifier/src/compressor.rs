use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_traverse::ReusableTraverseCtx;

use crate::{
    CompressOptions,
    peephole::{DeadCodeElimination, Normalize, NormalizeOptions, PeepholeOptimizations},
    state::MinifierState,
};

pub struct Compressor<'a> {
    allocator: &'a Allocator,
}

impl<'a> Compressor<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { allocator }
    }

    pub fn build(self, program: &mut Program<'a>, options: CompressOptions) {
        let scoping = SemanticBuilder::new().build(program).semantic.into_scoping();
        self.build_with_scoping(program, scoping, options);
    }

    /// Returns total number of iterations ran.
    pub fn build_with_scoping(
        self,
        program: &mut Program<'a>,
        scoping: Scoping,
        options: CompressOptions,
    ) -> u8 {
        let state = MinifierState::new(program.source_type, options);
        let mut ctx = ReusableTraverseCtx::new(state, scoping, self.allocator);
        let normalize_options =
            NormalizeOptions { convert_while_to_fors: true, convert_const_to_let: true };
        Normalize::new(normalize_options).build(program, &mut ctx);
        PeepholeOptimizations::new().run_in_loop(program, &mut ctx)
    }

    pub fn dead_code_elimination(self, program: &mut Program<'a>, options: CompressOptions) -> u8 {
        let scoping = SemanticBuilder::new().build(program).semantic.into_scoping();
        self.dead_code_elimination_with_scoping(program, scoping, options)
    }

    pub fn dead_code_elimination_with_scoping(
        self,
        program: &mut Program<'a>,
        scoping: Scoping,
        options: CompressOptions,
    ) -> u8 {
        let state = MinifierState::new(program.source_type, options);
        let mut ctx = ReusableTraverseCtx::new(state, scoping, self.allocator);
        let normalize_options =
            NormalizeOptions { convert_while_to_fors: false, convert_const_to_let: false };
        Normalize::new(normalize_options).build(program, &mut ctx);
        DeadCodeElimination::new().run_in_loop(program, &mut ctx);
        1
    }
}
