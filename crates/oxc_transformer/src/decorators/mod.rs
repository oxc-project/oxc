use std::rc::Rc;

use serde::Deserialize;

use oxc_ast::ast::*;

use crate::context::Ctx;

/// Only `"2023-11"` will be implemented because Babel 8 will only support "2023-11" and "legacy".
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecoratorsOptions;

/// [proposal-decorators](https://babeljs.io/docs/babel-plugin-proposal-decorators)
#[allow(unused)]
pub struct Decorators<'a> {
    options: DecoratorsOptions,
    ctx: Ctx<'a>,
}

impl<'a> Decorators<'a> {
    pub fn new(options: DecoratorsOptions, ctx: &Ctx<'a>) -> Self {
        Self { options, ctx: Rc::clone(ctx) }
    }
}

// Transformers
impl<'a> Decorators<'a> {
    #[allow(clippy::unused_self)]
    pub fn transform_statement(&self, _stmt: &mut Statement<'_>) {}
}
