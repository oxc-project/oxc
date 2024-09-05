mod diagnostics;
mod display_name;
mod jsx;
mod jsx_self;
mod jsx_source;
mod options;
mod refresh;
mod utils;

use std::rc::Rc;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};
use refresh::ReactRefresh;

pub use self::{
    display_name::ReactDisplayName,
    jsx::ReactJsx,
    options::{ReactJsxRuntime, ReactOptions, ReactRefreshOptions},
};
use crate::context::Ctx;

/// [Preset React](https://babel.dev/docs/babel-preset-react)
///
/// This preset includes the following plugins:
///
/// * [plugin-transform-react-jsx](https://babeljs.io/docs/babel-plugin-transform-react-jsx)
/// * [plugin-transform-react-jsx-self](https://babeljs.io/docs/babel-plugin-transform-react-jsx-self)
/// * [plugin-transform-react-jsx-source](https://babel.dev/docs/babel-plugin-transform-react-jsx-source)
/// * [plugin-transform-react-display-name](https://babeljs.io/docs/babel-plugin-transform-react-display-name)
pub struct React<'a> {
    jsx: ReactJsx<'a>,
    display_name: ReactDisplayName<'a>,
    refresh: ReactRefresh<'a>,
    jsx_plugin: bool,
    display_name_plugin: bool,
    jsx_self_plugin: bool,
    jsx_source_plugin: bool,
    refresh_plugin: bool,
}

// Constructors
impl<'a> React<'a> {
    pub fn new(mut options: ReactOptions, ctx: Ctx<'a>) -> Self {
        if options.jsx_plugin || options.development {
            options.update_with_comments(&ctx);
            options.conform();
        }
        let ReactOptions {
            jsx_plugin,
            display_name_plugin,
            jsx_self_plugin,
            jsx_source_plugin,
            ..
        } = options;
        let refresh = options.refresh.clone();
        Self {
            jsx: ReactJsx::new(options, Rc::clone(&ctx)),
            display_name: ReactDisplayName::new(Rc::clone(&ctx)),
            jsx_plugin,
            display_name_plugin,
            jsx_self_plugin,
            jsx_source_plugin,
            refresh_plugin: refresh.is_some(),
            refresh: ReactRefresh::new(&refresh.unwrap_or_default(), ctx),
        }
    }
}

impl<'a> Traverse<'a> for React<'a> {
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.refresh_plugin {
            self.refresh.enter_program(program, ctx);
        }
    }

    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.refresh_plugin {
            self.refresh.exit_program(program, ctx);
        }
        if self.jsx_plugin {
            self.jsx.exit_program(program, ctx);
        }
    }

    fn enter_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        if self.refresh_plugin {
            self.refresh.enter_statements(stmts, ctx);
        }
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        if self.refresh_plugin {
            self.refresh.exit_statements(stmts, ctx);
        }
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.jsx_plugin {
            self.jsx.enter_expression(expr, ctx);
        }
    }

    fn enter_call_expression(
        &mut self,
        call_expr: &mut CallExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.display_name_plugin {
            self.display_name.enter_call_expression(call_expr, ctx);
        }

        if self.refresh_plugin {
            self.refresh.enter_call_expression(call_expr, ctx);
        }
    }

    fn enter_jsx_opening_element(
        &mut self,
        elem: &mut JSXOpeningElement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.jsx_self_plugin && self.jsx.jsx_self.can_add_self_attribute(ctx) {
            self.jsx.jsx_self.enter_jsx_opening_element(elem, ctx);
        }
        if self.jsx_source_plugin {
            self.jsx.jsx_source.enter_jsx_opening_element(elem, ctx);
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.refresh_plugin {
            self.refresh.exit_expression(expr, ctx);
        }
    }

    fn exit_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.refresh_plugin {
            self.refresh.exit_function(func, ctx);
        }
    }
}
