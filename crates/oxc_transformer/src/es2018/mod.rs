mod object_rest;
mod object_rest_spread;
mod object_spread;
mod options;

pub use object_rest_spread::{ObjectRestSpread, ObjectRestSpreadOptions};
pub use options::ES2018Options;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};
use std::rc::Rc;

use crate::context::Ctx;

#[allow(dead_code)]
pub struct ES2018<'a> {
    ctx: Ctx<'a>,
    options: ES2018Options,

    // Plugins
    object_rest_spread: Option<ObjectRestSpread<'a>>,
}

impl<'a> ES2018<'a> {
    pub fn new(options: ES2018Options, ctx: Ctx<'a>) -> Self {
        Self {
            object_rest_spread: options
                .object_rest_spread
                .map(|options| ObjectRestSpread::new(options, Rc::clone(&ctx))),
            ctx,
            options,
        }
    }
}

impl<'a> Traverse<'a> for ES2018<'a> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(object_rest_spread) = &mut self.object_rest_spread {
            object_rest_spread.enter_expression(expr, ctx);
        }
    }
}
