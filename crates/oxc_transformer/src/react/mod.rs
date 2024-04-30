mod display_name;
mod jsx;
mod jsx_self;
mod jsx_source;
mod options;

use std::rc::Rc;

use oxc_ast::ast::*;

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
        // TODO: PERF: These two transforms reallocathe program.statements,
        // they should be combined so that allocation is computed only once for program.statements.
        if self.options.is_jsx_plugin_enabled() {
            self.jsx.transform_program_on_exit(program);
        }
        if self.options.is_jsx_source_plugin_enabled() {
            self.jsx.jsx_source.transform_program_on_exit(program);
        }
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        match expr {
            Expression::AssignmentExpression(e) => {
                if self.options.display_name_plugin {
                    self.display_name.transform_assignment_expression(e);
                }
            }
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

    pub fn transform_variable_declarator(&self, declarator: &mut VariableDeclarator<'a>) {
        if self.options.display_name_plugin {
            self.display_name.transform_variable_declarator(declarator);
        }
    }

    pub fn transform_object_property(&self, prop: &mut ObjectProperty<'a>) {
        if self.options.display_name_plugin {
            self.display_name.transform_object_property(prop);
        }
    }

    pub fn transform_export_default_declaration(&self, decl: &mut ExportDefaultDeclaration<'a>) {
        if self.options.display_name_plugin {
            self.display_name.transform_export_default_declaration(decl);
        }
    }

    pub fn transform_jsx_opening_element(&mut self, elem: &mut JSXOpeningElement<'a>) {
        if self.options.is_jsx_self_plugin_enabled() {
            self.jsx.jsx_self.transform_jsx_opening_element(elem);
        }
        if self.options.is_jsx_source_plugin_enabled() {
            self.jsx.jsx_source.transform_jsx_opening_element(elem);
        }
    }
}
