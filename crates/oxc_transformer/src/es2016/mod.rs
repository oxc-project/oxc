use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

mod exponentiation_operator;
mod options;

pub use exponentiation_operator::ExponentiationOperator;
pub use options::ES2016Options;

pub struct ES2016<'a, 'ctx> {
    options: ES2016Options,

    // Plugins
    exponentiation_operator: ExponentiationOperator<'a, 'ctx>,
}

impl<'a, 'ctx> ES2016<'a, 'ctx> {
    pub fn new(options: ES2016Options, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { exponentiation_operator: ExponentiationOperator::new(ctx), options }
    }
}

impl<'a, 'ctx> Traverse<'a> for ES2016<'a, 'ctx> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.exponentiation_operator {
            self.exponentiation_operator.enter_expression(expr, ctx);
        }
    }
}
