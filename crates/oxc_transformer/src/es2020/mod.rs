mod nullish_coalescing_operator;
mod options;

pub use nullish_coalescing_operator::NullishCoalescingOperator;
pub use options::ES2020Options;
use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::TraverseCtx;
use std::rc::Rc;

use crate::context::Ctx;

#[allow(dead_code)]
pub struct ES2020<'a> {
    ctx: Ctx<'a>,
    options: ES2020Options,

    // Plugins
    nullish_coalescing_operator: NullishCoalescingOperator<'a>,
}

impl<'a> ES2020<'a> {
    pub fn new(options: ES2020Options, ctx: Ctx<'a>) -> Self {
        Self {
            nullish_coalescing_operator: NullishCoalescingOperator::new(Rc::clone(&ctx)),
            ctx,
            options,
        }
    }

    pub fn transform_statements(
        &mut self,
        statements: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.nullish_coalescing_operator {
            self.nullish_coalescing_operator.transform_statements(statements, ctx);
        }
    }

    pub fn transform_statements_on_exit(
        &mut self,
        statements: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.nullish_coalescing_operator {
            self.nullish_coalescing_operator.transform_statements_on_exit(statements, ctx);
        }
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.nullish_coalescing_operator {
            self.nullish_coalescing_operator.transform_expression(expr, ctx);
        }
    }
}
