mod async_to_generator;
pub mod options;

use crate::context::Ctx;
use crate::es2017::async_to_generator::AsyncToGenerator;
use crate::es2017::options::ES2017Options;
use oxc_ast::ast::{ArrowFunctionExpression, Expression, Function};
use oxc_traverse::{Traverse, TraverseCtx};
use std::rc::Rc;

#[allow(dead_code)]
pub struct ES2017<'a> {
    ctx: Ctx<'a>,
    options: ES2017Options,

    // Plugins
    async_to_generator: AsyncToGenerator<'a>,
}

impl ES2017<'_> {
    pub fn new(options: ES2017Options, ctx: Ctx) -> ES2017 {
        ES2017 { async_to_generator: AsyncToGenerator::new(Rc::clone(&ctx)), ctx, options }
    }
}

impl<'a> Traverse<'a> for ES2017<'a> {
    fn exit_expression(&mut self, node: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.async_to_generator {
            self.async_to_generator.exit_expression(node, ctx);
        }
    }

    fn exit_function(&mut self, node: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.async_to_generator {
            self.async_to_generator.exit_function(node, ctx);
        }
    }

    fn exit_arrow_function_expression(&mut self, node: &mut ArrowFunctionExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.async_to_generator {
            self.async_to_generator.exit_arrow_function_expression(node, ctx);
        }
    }
}
