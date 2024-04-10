use std::rc::Rc;

use crate::context::Ctx;

/// [plugin-transform-react-jsx-self](https://babeljs.io/docs/babel-plugin-transform-react-jsx-self)
///
/// This plugin is included in `preset-react`.
///
/// ## Example
///
/// In: `<sometag />`
/// Out: `<sometag __self={this} />`
#[allow(unused)]
pub struct ReactJsxSelf<'a> {
    ctx: Ctx<'a>,
}

impl<'a> ReactJsxSelf<'a> {
    pub fn new(ctx: &Ctx<'a>) -> Self {
        Self { ctx: Rc::clone(ctx) }
    }
}
