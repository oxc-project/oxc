use oxc_ast::ast::*;
use oxc_traverse::Traverse;

use crate::{context::TraverseCtx, state::TransformState};

mod exponentiation_operator;
mod options;

pub use exponentiation_operator::ExponentiationOperator;
pub use options::ES2016Options;

pub struct ES2016<'a> {
    options: ES2016Options,

    // Plugins
    exponentiation_operator: ExponentiationOperator<'a>,
}

impl ES2016<'_> {
    pub fn new(options: ES2016Options) -> Self {
        Self { exponentiation_operator: ExponentiationOperator::new(), options }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for ES2016<'a> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.exponentiation_operator {
            self.exponentiation_operator.enter_expression(expr, ctx);
        }
    }
}
