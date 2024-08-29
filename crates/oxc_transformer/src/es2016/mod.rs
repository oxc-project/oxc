mod exponentiation_operator;
mod options;

pub use exponentiation_operator::ExponentiationOperator;
pub use options::ES2016Options;
use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};
use std::rc::Rc;

use crate::context::Ctx;

#[allow(dead_code)]
pub struct ES2016<'a> {
    ctx: Ctx<'a>,
    options: ES2016Options,

    // Plugins
    exponentiation_operator: ExponentiationOperator<'a>,
}

impl<'a> ES2016<'a> {
    pub fn new(options: ES2016Options, ctx: Ctx<'a>) -> Self {
        Self { exponentiation_operator: ExponentiationOperator::new(Rc::clone(&ctx)), ctx, options }
    }
}

impl<'a> Traverse<'a> for ES2016<'a> {
    fn enter_statements(
        &mut self,
        statements: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.exponentiation_operator {
            self.exponentiation_operator.enter_statements(statements, ctx);
        }
    }

    fn exit_statements(
        &mut self,
        statements: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.exponentiation_operator {
            self.exponentiation_operator.exit_statements(statements, ctx);
        }
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.exponentiation_operator {
            self.exponentiation_operator.enter_expression(expr, ctx);
        }
    }
}
