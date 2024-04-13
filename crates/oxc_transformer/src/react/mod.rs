mod display_name;
mod jsx;
mod jsx_self;
mod jsx_source;
mod options;

use std::rc::Rc;

use oxc_ast::ast::*;

use crate::context::Ctx;

pub use self::{
    display_name::ReactDisplayName, jsx::ReactJsx, jsx_self::ReactJsxSelf,
    jsx_source::ReactJsxSource, options::ReactOptions,
};

/// [Preset React](https://babel.dev/docs/babel-preset-react)
///
/// This preset includes the following plugins:
///
/// * [plugin-transform-react-jsx](https://babeljs.io/docs/babel-plugin-transform-react-jsx)
/// * [plugin-transform-react-jsx-self](https://babeljs.io/docs/babel-plugin-transform-react-jsx-self)
/// * [plugin-transform-react-jsx-source](https://babel.dev/docs/babel-plugin-transform-react-jsx-source)
/// * [plugin-transform-react-display-name](https://babeljs.io/docs/babel-plugin-transform-react-display-name)
#[allow(unused)]
pub struct React<'a> {
    options: Rc<ReactOptions>,
    ctx: Ctx<'a>,
    jsx: ReactJsx<'a>,
    jsx_self: ReactJsxSelf<'a>,
    jsx_source: ReactJsxSource<'a>,
    display_name: ReactDisplayName<'a>,
    development: bool,
}

// Constructors
impl<'a> React<'a> {
    pub fn new(options: ReactOptions, ctx: &Ctx<'a>) -> Self {
        let development = options.development;
        let options = Rc::new(options);
        Self {
            options: Rc::clone(&options),
            ctx: Rc::clone(ctx),
            jsx: ReactJsx::new(&options, ctx),
            jsx_self: ReactJsxSelf::new(ctx),
            jsx_source: ReactJsxSource::new(ctx),
            display_name: ReactDisplayName::new(ctx),
            development,
        }
    }
}

// Transforms
impl<'a> React<'a> {
    pub fn transform_expression(&self, expr: &mut Expression<'a>) {
        match expr {
            Expression::AssignmentExpression(e) => {
                if self.options.display_name_plugin {
                    self.display_name.transform_assignment_expression(e);
                }
            }
            Expression::JSXElement(_e) => {
                // *expr = unimplemented!();
            }
            Expression::JSXFragment(_e) => {
                // *expr = unimplemented!();
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

    pub fn transform_jsx_opening_element(&self, elem: &mut JSXOpeningElement<'a>) {
        if self.development {
            if self.options.jsx_self_plugin {
                self.jsx_self.transform_jsx_opening_element(elem);
            }
            if self.options.jsx_source_plugin {
                self.jsx_source.transform_jsx_opening_element(elem);
            }
        }
    }
}
