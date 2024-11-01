mod async_generator_functions;
mod object_rest_spread;
mod options;

use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::context::TransformCtx;
use async_generator_functions::AsyncGeneratorFunctions;
pub use object_rest_spread::{ObjectRestSpread, ObjectRestSpreadOptions};
pub use options::ES2018Options;

pub struct ES2018<'a, 'ctx> {
    options: ES2018Options,

    // Plugins
    object_rest_spread: ObjectRestSpread<'a, 'ctx>,
    async_generator_functions: AsyncGeneratorFunctions<'a, 'ctx>,
}

impl<'a, 'ctx> ES2018<'a, 'ctx> {
    pub fn new(options: ES2018Options, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self {
            object_rest_spread: ObjectRestSpread::new(
                options.object_rest_spread.unwrap_or_default(),
                ctx,
            ),
            async_generator_functions: AsyncGeneratorFunctions::new(ctx),
            options,
        }
    }
}

impl<'a, 'ctx> Traverse<'a> for ES2018<'a, 'ctx> {
    fn enter_program(&mut self, node: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.async_generator_functions {
            self.async_generator_functions.enter_program(node, ctx);
        }
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.object_rest_spread.is_some() {
            self.object_rest_spread.enter_expression(expr, ctx);
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.async_generator_functions {
            self.async_generator_functions.exit_expression(expr, ctx);
        }
    }

    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.async_generator_functions {
            self.async_generator_functions.enter_statement(stmt, ctx);
        }
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.async_generator_functions {
            self.async_generator_functions.exit_statement(stmt, ctx);
        }
    }

    fn enter_for_of_statement(&mut self, node: &mut ForOfStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.async_generator_functions {
            self.async_generator_functions.enter_for_of_statement(node, ctx);
        }
    }

    fn enter_function(&mut self, node: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.async_generator_functions {
            self.async_generator_functions.enter_function(node, ctx);
        }
    }

    fn exit_function(&mut self, node: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.async_generator_functions {
            self.async_generator_functions.exit_function(node, ctx);
        }
    }
}
