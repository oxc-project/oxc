mod logical_assignment_operators;
mod options;

pub use logical_assignment_operators::LogicalAssignmentOperators;
pub use options::ES2021Options;
use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::TraverseCtx;
use std::rc::Rc;

use crate::context::Ctx;

#[allow(dead_code)]
pub struct ES2021<'a> {
    ctx: Ctx<'a>,
    options: ES2021Options,

    // Plugins
    logical_assignment_operators: LogicalAssignmentOperators<'a>,
}

impl<'a> ES2021<'a> {
    pub fn new(options: ES2021Options, ctx: Ctx<'a>) -> Self {
        Self {
            logical_assignment_operators: LogicalAssignmentOperators::new(Rc::clone(&ctx)),
            ctx,
            options,
        }
    }

    pub fn transform_statements(
        &mut self,
        statements: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.logical_assignment_operators {
            self.logical_assignment_operators.transform_statements(statements, ctx);
        }
    }

    pub fn transform_statements_on_exit(
        &mut self,
        statements: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.logical_assignment_operators {
            self.logical_assignment_operators.transform_statements_on_exit(statements, ctx);
        }
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.logical_assignment_operators {
            self.logical_assignment_operators.transform_expression(expr, ctx);
        }
    }
}
