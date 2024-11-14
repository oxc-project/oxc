use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

mod logical_assignment_operators;
mod numeric_separator;
mod options;

pub use logical_assignment_operators::LogicalAssignmentOperators;
pub use numeric_separator::NumericSeparator;
pub use options::ES2021Options;

pub struct ES2021<'a, 'ctx> {
    options: ES2021Options,

    // Plugins
    logical_assignment_operators: LogicalAssignmentOperators<'a, 'ctx>,
    numeric_separator: NumericSeparator<'a, 'ctx>,
}

impl<'a, 'ctx> ES2021<'a, 'ctx> {
    pub fn new(options: ES2021Options, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self {
            logical_assignment_operators: LogicalAssignmentOperators::new(ctx),
            numeric_separator: NumericSeparator::new(ctx),
            options,
        }
    }
}

impl<'a, 'ctx> Traverse<'a> for ES2021<'a, 'ctx> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.logical_assignment_operators {
            self.logical_assignment_operators.enter_expression(expr, ctx);
        }
        if self.options.numeric_separator {
            self.numeric_separator.enter_expression(expr, ctx);
        }
    }
}
