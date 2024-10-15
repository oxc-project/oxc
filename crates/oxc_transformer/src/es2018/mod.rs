mod async_generator_functions;
mod object_rest_spread;
mod options;

use async_generator_functions::AsyncGeneratorFunctions;
pub use object_rest_spread::{ObjectRestSpread, ObjectRestSpreadOptions};
pub use options::ES2018Options;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::context::TransformCtx;

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
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.object_rest_spread.is_some() {
            self.object_rest_spread.enter_expression(expr, ctx);
        }
    }
}
