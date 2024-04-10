use std::rc::Rc;

use crate::context::Ctx;

/// [plugin-transform-react-display-name](https://babeljs.io/docs/babel-plugin-transform-react-display-name)
///
/// This plugin is included in `preset-react`.
///
/// ## Example
///
/// In: `var bar = createReactClass({});`
/// Out: `var bar = createReactClass({ displayName: "bar" });`
#[allow(unused)]
pub struct ReactDisplayName<'a> {
    ctx: Ctx<'a>,
}

impl<'a> ReactDisplayName<'a> {
    pub fn new(ctx: &Ctx<'a>) -> Self {
        Self { ctx: Rc::clone(ctx) }
    }
}
