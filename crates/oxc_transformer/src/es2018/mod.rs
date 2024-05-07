mod object_rest_spread;
mod object_spread;
mod options;

pub use object_rest_spread::{ObjectRestSpread, ObjectRestSpreadOptions};
pub use options::ES2018Options;

use oxc_ast::ast::*;
use std::rc::Rc;

use crate::{context::Ctx, CompilerAssumptions};

#[allow(dead_code)]
pub struct ES2018<'a> {
    ctx: Ctx<'a>,
    options: ES2018Options,

    // Plugins
    object_rest_spread: ObjectRestSpread<'a>,
}

impl<'a> ES2018<'a> {
    pub fn new(options: ES2018Options, assumptions: CompilerAssumptions, ctx: &Ctx<'a>) -> Self {
        Self {
            ctx: Rc::clone(ctx),
            options,
            object_rest_spread: ObjectRestSpread::new(assumptions, ctx),
        }
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        if self.options.object_rest_spread.is_some() {
            self.object_rest_spread.transform_expression(expr);
        }
    }

    pub fn transform_program_on_exit(&mut self, program: &mut Program<'a>) {
        self.object_rest_spread.transform_program_on_exit(program);
    }
}
