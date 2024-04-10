use std::rc::Rc;

use crate::context::Ctx;

/// [plugin-transform-react-jsx-source](https://babeljs.io/docs/babel-plugin-transform-react-jsx-source)
#[allow(unused)]
pub struct ReactJsxSource<'a> {
    ctx: Ctx<'a>,
}

impl<'a> ReactJsxSource<'a> {
    pub fn new(ctx: &Ctx<'a>) -> Self {
        Self { ctx: Rc::clone(ctx) }
    }
}
