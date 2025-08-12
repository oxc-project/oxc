use oxc_ast::ast::*;
use oxc_traverse::Traverse;

use crate::{
    state::TransformState, context::TraverseCtx,
};

mod logical_assignment_operators;
mod options;

pub use logical_assignment_operators::LogicalAssignmentOperators;
pub use options::ES2021Options;

pub struct ES2021<'a> {
    options: ES2021Options,

    // Plugins
    logical_assignment_operators: LogicalAssignmentOperators<'a>,
}

impl<'a> ES2021<'a> {
    pub fn new(options: ES2021Options, ) -> Self {
        Self { logical_assignment_operators: LogicalAssignmentOperators::new(), options }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for ES2021<'a, '_> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.logical_assignment_operators {
            self.logical_assignment_operators.enter_expression(expr, ctx);
        }
    }
}
