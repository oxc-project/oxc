mod arrow_functions;
mod options;

pub use arrow_functions::{ArrowFunctions, ArrowFunctionsOptions};
pub use options::ES2015Options;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use std::rc::Rc;

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

    pub fn enter_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_statements(stmts);
        }
    }

    pub fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_statements_on_exit(stmts);
        }
    }

    pub fn transform_jsx_element_name(&mut self, elem: &mut JSXElementName<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_jsx_element_name(elem);
        }
    }

    pub fn transform_declaration(&mut self, decl: &mut Declaration<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_declaration(decl);
        }
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_expression(expr);
        }
    }

    pub fn transform_expression_on_exit(&mut self, expr: &mut Expression<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_expression_on_exit(expr);
        }
    }

    pub fn transform_declaration_on_exit(&mut self, decl: &mut Declaration<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_declaration_on_exit(decl);
        }
    }

    pub fn transform_class(&mut self, class: &mut Class<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_class(class);
        }
    }

    pub fn transform_class_on_exit(&mut self, class: &mut Class<'a>) {
        if self.options.arrow_function.is_some() {
            self.arrow_functions.transform_class_on_exit(class);
        }
    }
}
