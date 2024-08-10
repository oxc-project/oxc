mod options;

use std::rc::Rc;

mod exponentiation_operator;
pub use options::ES2016Options;
use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::context::Ctx;

use self::exponentiation_operator::ExponentiationOperator;

#[allow(dead_code)]
pub struct ES2016<'a> {
    ctx: Ctx<'a>,
    options: ES2016Options,
    exponentiation_operator: exponentiation_operator::ExponentiationOperator<'a>,
}

impl<'a> ES2016<'a> {
    pub fn new(options: ES2016Options, ctx: Ctx<'a>) -> Self {
        Self {
            exponentiation_operator: ExponentiationOperator { ctx: Rc::clone(&ctx) },
            ctx,
            options,
        }
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        if self.options.exponentiation_operator {
            self.exponentiation_operator.transform_expression(expr);
        }
    }
}
