use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

mod arrow_functions;
mod options;

pub use arrow_functions::{ArrowFunctions, ArrowFunctionsOptions};
pub use options::ES2015Options;

use crate::context::TransformCtx;

pub struct ES2015<'a, 'ctx> {
    options: ES2015Options,

    // Plugins
    arrow_functions: ArrowFunctions<'a, 'ctx>,
}

impl<'a, 'ctx> ES2015<'a, 'ctx> {
    pub fn new(options: ES2015Options, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self {
            arrow_functions: ArrowFunctions::new(
                options.arrow_function.clone().unwrap_or_default(),
                ctx,
            ),
            options,
        }
    }
}

impl<'a, 'ctx> Traverse<'a> for ES2015<'a, 'ctx> {
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.enter_program(program, ctx);
        }
    }
}
