mod exponentiation_operator;
mod options;

pub use exponentiation_operator::ExponentiationOperator;
pub use options::ES2016Options;
use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::TraverseCtx;
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

    pub fn transform_statements(
        &mut self,
        statements: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.exponentiation_operator {
            self.exponentiation_operator.transform_statements(statements, ctx);
        }
    }

    pub fn transform_statements_on_exit(
        &mut self,
        statements: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.exponentiation_operator {
            self.exponentiation_operator.transform_statements_on_exit(statements, ctx);
        }
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.exponentiation_operator {
            self.exponentiation_operator.transform_expression(expr, ctx);
        }
    }
}
