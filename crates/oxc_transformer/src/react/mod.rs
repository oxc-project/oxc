use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder};
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

mod comments;
mod diagnostics;
mod display_name;
mod jsx;
mod jsx_self;
mod jsx_source;
mod options;
mod refresh;
mod utils;
use refresh::ReactRefresh;

pub use display_name::ReactDisplayName;
pub use jsx::ReactJsx;
pub use options::{JsxOptions, JsxRuntime, ReactRefreshOptions};

pub(crate) use comments::update_options_with_comments;

/// [Preset React](https://babel.dev/docs/babel-preset-react)
///
/// This preset includes the following plugins:
///
/// * [plugin-transform-react-jsx](https://babeljs.io/docs/babel-plugin-transform-react-jsx)
/// * [plugin-transform-react-jsx-self](https://babeljs.io/docs/babel-plugin-transform-react-jsx-self)
/// * [plugin-transform-react-jsx-source](https://babel.dev/docs/babel-plugin-transform-react-jsx-source)
/// * [plugin-transform-react-display-name](https://babeljs.io/docs/babel-plugin-transform-react-display-name)
pub struct React<'a, 'ctx> {
    jsx: ReactJsx<'a, 'ctx>,
    display_name: ReactDisplayName<'a, 'ctx>,
    refresh: ReactRefresh<'a, 'ctx>,
    jsx_plugin: bool,
    display_name_plugin: bool,
    jsx_self_plugin: bool,
    jsx_source_plugin: bool,
    refresh_plugin: bool,
}

// Constructors
impl<'a, 'ctx> React<'a, 'ctx> {
    pub fn new(mut options: JsxOptions, ast: AstBuilder<'a>, ctx: &'ctx TransformCtx<'a>) -> Self {
        if options.jsx_plugin || options.development {
            options.conform();
        }
        let JsxOptions {
            jsx_plugin, display_name_plugin, jsx_self_plugin, jsx_source_plugin, ..
        } = options;
        let refresh = options.refresh.clone();
        Self {
            jsx: ReactJsx::new(options, ast, ctx),
            display_name: ReactDisplayName::new(ctx),
            jsx_plugin,
            display_name_plugin,
            jsx_self_plugin,
            jsx_source_plugin,
            refresh_plugin: refresh.is_some(),
            refresh: ReactRefresh::new(&refresh.unwrap_or_default(), ast, ctx),
        }
    }
}

impl<'a, 'ctx> Traverse<'a> for React<'a, 'ctx> {
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.jsx_plugin {
            program.source_type = program.source_type.with_standard(true);
        }
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
        } else if self.jsx_source_plugin {
            self.jsx.jsx_source.exit_program(program, ctx);
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
        if !self.jsx_plugin {
            if self.jsx_self_plugin && self.jsx.jsx_self.can_add_self_attribute(ctx) {
                self.jsx.jsx_self.enter_jsx_opening_element(elem, ctx);
            }
            if self.jsx_source_plugin {
                self.jsx.jsx_source.enter_jsx_opening_element(elem, ctx);
            }
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.jsx_plugin {
            self.jsx.exit_expression(expr, ctx);
        }
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
