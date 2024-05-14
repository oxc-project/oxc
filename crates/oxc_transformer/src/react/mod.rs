mod display_name;
mod jsx;
mod jsx_self;
mod jsx_source;
mod options;
mod utils;

use std::rc::Rc;

use oxc_ast::ast::*;
use oxc_traverse::{Ancestor, FinderRet, TraverseCtx};

use crate::context::Ctx;

pub use self::{display_name::ReactDisplayName, jsx::ReactJsx, options::ReactOptions};

/// [Preset React](https://babel.dev/docs/babel-preset-react)
///
/// This preset includes the following plugins:
///
/// * [plugin-transform-react-jsx](https://babeljs.io/docs/babel-plugin-transform-react-jsx)
/// * [plugin-transform-react-jsx-self](https://babeljs.io/docs/babel-plugin-transform-react-jsx-self)
/// * [plugin-transform-react-jsx-source](https://babel.dev/docs/babel-plugin-transform-react-jsx-source)
/// * [plugin-transform-react-display-name](https://babeljs.io/docs/babel-plugin-transform-react-display-name)
pub struct React<'a> {
    options: Rc<ReactOptions>,
    jsx: ReactJsx<'a>,
    display_name: ReactDisplayName<'a>,
}

// Constructors
impl<'a> React<'a> {
    pub fn new(options: ReactOptions, ctx: &Ctx<'a>) -> Self {
        let mut options = options;
        if options.is_jsx_plugin_enabled() {
            options.update_with_comments(ctx);
        }
        let options = Rc::new(options);
        Self {
            options: Rc::clone(&options),
            jsx: ReactJsx::new(&options, ctx),
            display_name: ReactDisplayName::new(ctx),
        }
    }
}

// Transforms
impl<'a> React<'a> {
    pub fn transform_program_on_exit(&mut self, program: &mut Program<'a>) {
        if self.options.is_jsx_plugin_enabled() {
            self.jsx.transform_program_on_exit(program);
        }
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        match expr {
            Expression::JSXElement(e) => {
                if self.options.is_jsx_plugin_enabled() {
                    *expr = self.jsx.transform_jsx_element(e);
                }
            }
            Expression::JSXFragment(e) => {
                if self.options.is_jsx_plugin_enabled() {
                    *expr = self.jsx.transform_jsx_fragment(e);
                }
            }
            _ => {}
        }
    }

    pub fn transform_call_expression(
        &self,
        call_expr: &mut CallExpression<'a>,
        ctx: &TraverseCtx<'a>,
    ) {
        if self.options.display_name_plugin {
            self.display_name.transform_call_expression(call_expr, ctx);
        }
    }

    pub fn transform_jsx_opening_element(
        &mut self,
        elem: &mut JSXOpeningElement<'a>,
        ctx: &TraverseCtx<'a>,
    ) {
        if self.options.is_jsx_self_plugin_enabled() {
            let is_constructor = ctx
                .find_scope(|scope| {
                    if scope.is_block() || scope.is_arrow() {
                        return FinderRet::Continue;
                    }
                    FinderRet::Found(scope.is_constructor())
                })
                .unwrap_or(false);
            if !is_constructor
                // no super class
                || ctx
                    .find_ancestor(|ancestor| match ancestor {
                        Ancestor::ClassBody(class) => FinderRet::Found(class.super_class().is_none()),
                        _ => FinderRet::Continue,
                    })
                    .unwrap_or(true)
            {
                self.jsx.jsx_self.transform_jsx_opening_element(elem);
            }
        }
        if self.options.is_jsx_source_plugin_enabled() {
            self.jsx.jsx_source.transform_jsx_opening_element(elem);
        }
    }
}
