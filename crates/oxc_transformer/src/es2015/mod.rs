mod arrow_functions;
mod options;

pub use arrow_functions::{ArrowFunctions, ArrowFunctionsOptions};
pub use options::ES2015Options;

use oxc_ast::ast::*;
use oxc_traverse::TraverseCtx;

use crate::context::Ctx;

#[allow(dead_code)]
pub struct ES2015<'a> {
    ctx: Ctx<'a>,
    options: ES2015Options,

    // Plugins
    arrow_functions: ArrowFunctions<'a>,
}

impl<'a> ES2015<'a> {
    pub fn new(options: ES2015Options, ctx: Ctx<'a>) -> Self {
        Self {
            arrow_functions: ArrowFunctions::new(
                options.arrow_function.clone().unwrap_or_default(),
            ),
            ctx,
            options,
        }
    }

    pub fn transform_program(&mut self, program: &Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_program(program, ctx);
        }
    }

    pub fn transform_program_on_exit(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_program_on_exit(program, ctx);
        }
    }

    pub fn transform_function(&mut self, func: &Function<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_function(func, ctx);
        }
    }

    pub fn transform_function_on_exit(
        &mut self,
        func: &mut Function<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_function_on_exit(func, ctx);
        }
    }

    pub fn transform_static_block(&mut self, block: &StaticBlock<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_static_block(block, ctx);
        }
    }

    pub fn transform_static_block_on_exit(
        &mut self,
        block: &mut StaticBlock<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_static_block_on_exit(block, ctx);
        }
    }

    pub fn transform_ts_module_block(
        &mut self,
        block: &mut TSModuleBlock<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_ts_module_block(block, ctx);
        }
    }

    pub fn transform_ts_module_block_on_exit(
        &mut self,
        block: &mut TSModuleBlock<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_ts_module_block_on_exit(block, ctx);
        }
    }

    pub fn transform_arrow_function_expression(
        &mut self,
        arrow_function_expr: &ArrowFunctionExpression<'a>,
    ) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_arrow_function_expression(arrow_function_expr);
        }
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_expression(expr, ctx);
        }
    }

    pub fn transform_expression_on_exit(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_expression_on_exit(expr, ctx);
        }
    }

    pub fn transform_jsx_element_name(
        &mut self,
        elem: &mut JSXElementName<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_jsx_element_name(elem, ctx);
        }
    }
}
