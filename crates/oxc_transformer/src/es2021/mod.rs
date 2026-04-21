use oxc_ast::ast::*;
use oxc_traverse::Traverse;

use crate::{context::TraverseCtx, state::TransformState};

mod logical_assignment_operators;
mod options;

pub use logical_assignment_operators::LogicalAssignmentOperators;
pub use options::ES2021Options;

pub struct ES2021 {
    options: ES2021Options,

    // Plugins
    logical_assignment_operators: LogicalAssignmentOperators,
}

impl ES2021 {
    pub fn new(options: ES2021Options) -> Self {
        Self { logical_assignment_operators: LogicalAssignmentOperators::new(), options }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for ES2021 {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.logical_assignment_operators {
            self.logical_assignment_operators.enter_expression(expr, ctx);
        }
    }
}
