use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

mod arrow_functions;
mod options;

pub use arrow_functions::{ArrowFunctions, ArrowFunctionsOptions};
pub use options::ES2015Options;

pub struct ES2015<'a> {
    options: ES2015Options,

    // Plugins
    arrow_functions: ArrowFunctions<'a>,
}

impl<'a> ES2015<'a> {
    pub fn new(options: ES2015Options) -> Self {
        Self {
            arrow_functions: ArrowFunctions::new(
                options.arrow_function.clone().unwrap_or_default(),
            ),
            options,
        }
    }
}

impl<'a> Traverse<'a> for ES2015<'a> {
    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.exit_program(program, ctx);
        }
    }

    fn enter_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.enter_function(func, ctx);
        }
    }

    fn exit_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.exit_function(func, ctx);
        }
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.enter_expression(expr, ctx);
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.exit_expression(expr, ctx);
        }
    }

    fn enter_static_block(&mut self, block: &mut StaticBlock<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.enter_static_block(block, ctx);
        }
    }

    fn exit_static_block(&mut self, block: &mut StaticBlock<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.exit_static_block(block, ctx);
        }
    }

    fn enter_jsx_element_name(&mut self, node: &mut JSXElementName<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.enter_jsx_element_name(node, ctx);
        }
    }

    fn enter_jsx_member_expression_object(
        &mut self,
        node: &mut JSXMemberExpressionObject<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.enter_jsx_member_expression_object(node, ctx);
        }
    }
}
