use std::rc::Rc;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_traverse::ReusableTraverseCtx;

use crate::{
    CompressOptions,
    ctx::MinifierState,
    peephole::{
        DeadCodeElimination, LatePeepholeOptimizations, Normalize, NormalizeOptions,
        PeepholeOptimizations,
    },
};

pub struct Compressor<'a> {
    allocator: &'a Allocator,
    options: Rc<CompressOptions>,
}

impl<'a> Compressor<'a> {
    pub fn new(allocator: &'a Allocator, options: CompressOptions) -> Self {
        Self { allocator, options: Rc::new(options) }
    }

    pub fn build(self, program: &mut Program<'a>) {
        let scoping = SemanticBuilder::new().build(program).semantic.into_scoping();
        self.build_with_scoping(scoping, program);
    }

    pub fn build_with_scoping(self, scoping: Scoping, program: &mut Program<'a>) {
        let state = MinifierState::new(Rc::clone(&self.options));
        let mut ctx = ReusableTraverseCtx::new(state, scoping, self.allocator);
        let normalize_options =
            NormalizeOptions { convert_while_to_fors: true, convert_const_to_let: true };
        Normalize::new(normalize_options).build(program, &mut ctx);
        PeepholeOptimizations::new(self.options.target, self.options.keep_names)
            .run_in_loop(program, &mut ctx);
        LatePeepholeOptimizations::new(self.options.target).build(program, &mut ctx);
    }

    pub fn dead_code_elimination(self, program: &mut Program<'a>) {
        let scoping = SemanticBuilder::new().build(program).semantic.into_scoping();
        self.dead_code_elimination_with_scoping(scoping, program);
    }

    pub fn dead_code_elimination_with_scoping(self, scoping: Scoping, program: &mut Program<'a>) {
        let state = MinifierState::new(Rc::clone(&self.options));
        let mut ctx = ReusableTraverseCtx::new(state, scoping, self.allocator);
        let normalize_options =
            NormalizeOptions { convert_while_to_fors: false, convert_const_to_let: false };
        Normalize::new(normalize_options).build(program, &mut ctx);
        DeadCodeElimination::new().build(program, &mut ctx);
    }
}
