use std::rc::Rc;

use crate::context::Ctx;

pub use super::options::ReactOptions;

/// [plugin-transform-react-jsx](https://babeljs.io/docs/babel-plugin-transform-react-jsx)
///
/// This plugin generates production-ready JS code.
///
/// If you are developing a React app in a development environment,
/// please use @babel/plugin-transform-react-jsx-development for a better debugging experience.
///
/// This plugin is included in `preset-react`.
///
/// References:
///
/// * <https://babeljs.io/docs/babel-plugin-transform-react-jsx>
/// * <https://github.com/babel/babel/tree/main/packages/babel-helper-builder-react-jsx>
#[allow(unused)]
pub struct ReactJsx<'a> {
    options: ReactOptions,
    ctx: Ctx<'a>,
}

impl<'a> ReactJsx<'a> {
    pub fn new(options: ReactOptions, ctx: &Ctx<'a>) -> Self {
        Self { options, ctx: Rc::clone(ctx) }
    }
}
