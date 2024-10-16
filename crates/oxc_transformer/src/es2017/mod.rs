use oxc_ast::ast::{ArrowFunctionExpression, Expression, Statement};
use oxc_traverse::{Traverse, TraverseCtx};

use crate::es2017::async_to_generator::AsyncToGenerator;
use crate::es2017::options::ES2017Options;

mod async_to_generator;
pub mod options;

#[allow(dead_code)]
pub struct ES2017 {
    options: ES2017Options,

    // Plugins
    async_to_generator: AsyncToGenerator,
}

impl ES2017 {
    pub fn new(options: ES2017Options) -> ES2017 {
        ES2017 { async_to_generator: AsyncToGenerator, options }
    }
}

impl<'a> Traverse<'a> for ES2017 {
    fn exit_expression(&mut self, node: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.async_to_generator {
            self.async_to_generator.exit_expression(node, ctx);
        }
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.async_to_generator {
            self.async_to_generator.exit_statement(stmt, ctx);
        }
    }

    fn exit_arrow_function_expression(
        &mut self,
        node: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.async_to_generator {
            self.async_to_generator.exit_arrow_function_expression(node, ctx);
        }
    }
}
