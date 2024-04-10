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
/// * [plugin-transform-react-jsx](https://babeljs.io/docs/babel-plugin-transform-react-jsx)
/// * [plugin-transform-react-display-name](https://babeljs.io/docs/babel-plugin-transform-react-display-name)
#[allow(unused)]
pub struct React<'a> {
    ctx: Ctx<'a>,
    jsx: ReactJsx<'a>,
    jsx_self: ReactJsxSelf<'a>,
    jsx_source: ReactJsxSource<'a>,
    display_name: ReactDisplayName<'a>,
}

// Constructors
impl<'a> React<'a> {
    pub fn new(options: ReactOptions, ctx: &Ctx<'a>) -> Self {
        Self {
            ctx: Rc::clone(ctx),
            jsx: ReactJsx::new(options, ctx),
            jsx_self: ReactJsxSelf::new(ctx),
            jsx_source: ReactJsxSource::new(ctx),
            display_name: ReactDisplayName::new(ctx),
        }
    }
}

// Transformers
impl<'a> React<'a> {
    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        match expr {
            Expression::JSXElement(_e) => {
                // *expr = unimplemented!();
            }
            Expression::JSXFragment(_e) => {
                // *expr = unimplemented!();
            }
            _ => {}
        }
    }
}
