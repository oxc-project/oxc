use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_semantic::{Scoping, SemanticBuilder};

use crate::{
    CompressOptions, ReusableTraverseCtx,
    peephole::{Normalize, NormalizeOptions, PeepholeOptimizations},
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
        let max_iterations = options.max_iterations;
        let state = MinifierState::new(program.source_type, options, /* dce */ false);
        let mut ctx = ReusableTraverseCtx::new(state, scoping, self.allocator);
        let normalize_options = NormalizeOptions {
            convert_while_to_fors: true,
            convert_const_to_let: true,
            remove_unnecessary_use_strict: true,
        };
        Normalize::new(normalize_options).build(program, &mut ctx);
        Self::run_in_loop(max_iterations, program, &mut ctx)
    }

    pub fn dead_code_elimination(self, program: &mut Program<'a>, options: CompressOptions) -> u8 {
        let scoping = SemanticBuilder::new().build(program).semantic.into_scoping();
        self.dead_code_elimination_with_scoping(program, scoping, options)
    }

    /// Returns total number of iterations ran.
    pub fn dead_code_elimination_with_scoping(
        self,
        program: &mut Program<'a>,
        scoping: Scoping,
        options: CompressOptions,
    ) -> u8 {
        let max_iterations = options.max_iterations;
        let state = MinifierState::new(program.source_type, options, /* dce */ true);
        let mut ctx = ReusableTraverseCtx::new(state, scoping, self.allocator);
        let normalize_options = NormalizeOptions {
            convert_while_to_fors: false,
            convert_const_to_let: false,
            remove_unnecessary_use_strict: false,
        };
        Normalize::new(normalize_options).build(program, &mut ctx);
        Self::run_in_loop(max_iterations, program, &mut ctx)
    }

    /// Fixed-point iteration loop for peephole optimizations.
    fn run_in_loop(
        max_iterations: Option<u8>,
        program: &mut Program<'a>,
        ctx: &mut ReusableTraverseCtx<'a>,
    ) -> u8 {
        let mut iteration = 0u8;
        loop {
            PeepholeOptimizations.run_once(program, ctx);
            if !ctx.state().changed {
                break;
            }
            if let Some(max) = max_iterations {
                if iteration >= max {
                    break;
                }
            } else if iteration > 10 {
                debug_assert!(false, "Ran loop more than 10 times.");
                break;
            }
            iteration += 1;
        }
        iteration
    }
}
