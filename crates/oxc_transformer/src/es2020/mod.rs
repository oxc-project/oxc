mod nullish_coalescing_operator;
mod options;

pub use nullish_coalescing_operator::NullishCoalescingOperator;
pub use options::ES2020Options;
use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};
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
}

impl<'a> Traverse<'a> for ES2020<'a> {
    fn enter_statements(
        &mut self,
        statements: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.nullish_coalescing_operator {
            self.nullish_coalescing_operator.enter_statements(statements, ctx);
        }
    }

    fn exit_statements(
        &mut self,
        statements: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.nullish_coalescing_operator {
            self.nullish_coalescing_operator.exit_statements(statements, ctx);
        }
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.nullish_coalescing_operator {
            self.nullish_coalescing_operator.enter_expression(expr, ctx);
        }
    }
}
