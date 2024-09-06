mod logical_assignment_operators;
mod options;

use std::rc::Rc;

pub use logical_assignment_operators::LogicalAssignmentOperators;
pub use options::ES2021Options;
use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

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
}

impl<'a> Traverse<'a> for ES2021<'a> {
    fn enter_statements(
        &mut self,
        statements: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.logical_assignment_operators {
            self.logical_assignment_operators.enter_statements(statements, ctx);
        }
    }

    fn exit_statements(
        &mut self,
        statements: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.logical_assignment_operators {
            self.logical_assignment_operators.exit_statements(statements, ctx);
        }
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.logical_assignment_operators {
            self.logical_assignment_operators.enter_expression(expr, ctx);
        }
    }
}
