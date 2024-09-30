//! Utility transforms which are in common between other transforms.

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

mod var_declarations;

use var_declarations::VarDeclarations;
pub use var_declarations::VarDeclarationsStore;

pub struct Common<'a, 'ctx> {
    var_declarations: VarDeclarations<'a, 'ctx>,
}

impl<'a, 'ctx> Common<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { var_declarations: VarDeclarations::new(ctx) }
    }
}

impl<'a, 'ctx> Traverse<'a> for Common<'a, 'ctx> {
    #[inline] // Inline because it's no-op in release mode
    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.var_declarations.exit_program(program, ctx);
    }

    fn enter_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.var_declarations.enter_statements(stmts, ctx);
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.var_declarations.exit_statements(stmts, ctx);
    }
}
