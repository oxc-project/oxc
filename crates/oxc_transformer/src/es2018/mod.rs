mod object_rest_spread;
mod options;

pub use object_rest_spread::{ObjectRestSpread, ObjectRestSpreadOptions};
pub use options::ES2018Options;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

pub struct ES2018 {
    options: ES2018Options,

    // Plugins
    object_rest_spread: ObjectRestSpread,
}

impl ES2018 {
    pub fn new(options: ES2018Options) -> Self {
        Self {
            object_rest_spread: ObjectRestSpread::new(
                options.object_rest_spread.unwrap_or_default(),
            ),
            options,
        }
    }
}

impl<'a> Traverse<'a> for ES2018 {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.object_rest_spread.is_some() {
            self.object_rest_spread.enter_expression(expr, ctx);
        }
    }
}
