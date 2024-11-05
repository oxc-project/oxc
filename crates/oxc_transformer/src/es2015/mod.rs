use oxc_traverse::Traverse;

mod arrow_functions;
mod options;

pub use arrow_functions::{ArrowFunctions, ArrowFunctionsOptions};
pub use options::ES2015Options;

use crate::context::TransformCtx;

pub struct ES2015<'a, 'ctx> {
    #[expect(unused)]
    options: ES2015Options,

    // Plugins
    #[expect(unused)]
    arrow_functions: ArrowFunctions<'a, 'ctx>,
}

impl<'a, 'ctx> ES2015<'a, 'ctx> {
    pub fn new(options: ES2015Options, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self {
            arrow_functions: ArrowFunctions::new(options.arrow_function.unwrap_or_default(), ctx),
            options,
        }
    }
}

impl<'a, 'ctx> Traverse<'a> for ES2015<'a, 'ctx> {}
