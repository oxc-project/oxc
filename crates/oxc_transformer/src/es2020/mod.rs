use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

mod nullish_coalescing_operator;
mod options;

pub use nullish_coalescing_operator::NullishCoalescingOperator;
pub use options::ES2020Options;

pub struct ES2020<'a, 'ctx> {
    options: ES2020Options,

    // Plugins
    nullish_coalescing_operator: NullishCoalescingOperator<'a, 'ctx>,
}

impl<'a, 'ctx> ES2020<'a, 'ctx> {
    pub fn new(options: ES2020Options, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { nullish_coalescing_operator: NullishCoalescingOperator::new(ctx), options }
    }
}

impl<'a, 'ctx> Traverse<'a> for ES2020<'a, 'ctx> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.nullish_coalescing_operator {
            self.nullish_coalescing_operator.enter_expression(expr, ctx);
        }
    }
}
