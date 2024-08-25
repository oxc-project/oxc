mod arrow_functions;
mod options;

use std::rc::Rc;

pub use arrow_functions::{ArrowFunctions, ArrowFunctionsOptions};
pub use options::ES2015Options;
use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

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
                Rc::clone(&ctx),
            ),
            ctx,
            options,
        }
    }
}

impl<'a> Traverse<'a> for ES2015<'a> {
    fn enter_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.enter_statements(stmts, ctx);
        }
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.exit_statements(stmts, ctx);
        }
    }

    fn enter_jsx_element_name(&mut self, elem: &mut JSXElementName<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.enter_jsx_element_name(elem, ctx);
        }
    }

    fn enter_declaration(&mut self, decl: &mut Declaration<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.enter_declaration(decl, ctx);
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

    fn exit_declaration(&mut self, decl: &mut Declaration<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.exit_declaration(decl, ctx);
        }
    }

    fn enter_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.enter_class(class, ctx);
        }
    }

    fn exit_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.exit_class(class, ctx);
        }
    }

    fn enter_variable_declarator(
        &mut self,
        node: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.enter_variable_declarator(node, ctx);
        }
    }
}
