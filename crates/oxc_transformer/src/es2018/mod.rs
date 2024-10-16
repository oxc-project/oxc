use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::context::TransformCtx;

mod object_rest_spread;
mod options;

pub use object_rest_spread::{ObjectRestSpread, ObjectRestSpreadOptions};
pub use options::ES2018Options;

pub struct ES2018<'a, 'ctx> {
    options: ES2018Options,

    // Plugins
    object_rest_spread: ObjectRestSpread<'a, 'ctx>,
}

impl<'a, 'ctx> ES2018<'a, 'ctx> {
    pub fn new(options: ES2018Options, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self {
            object_rest_spread: ObjectRestSpread::new(
                options.object_rest_spread.unwrap_or_default(),
                ctx,
            ),
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
