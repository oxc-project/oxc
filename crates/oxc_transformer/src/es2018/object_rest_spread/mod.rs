//! ES2018 object spread transformation.
//!
//! This plugin transforms rest properties for object destructuring assignment and spread properties for object literals.
//!
//! > This plugin is included in `preset-env`, in ES2018
//!
//! ## Example
//!
//! Input:
//! ```js
//! var x = { a: 1, b: 2 };
//! var y = { ...x, c: 3 };
//! ```
//!
//! Output:
//! ```js
//! var x = { a: 1, b: 2 };
//! var y = _objectSpread({}, x, { c: 3 });
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-object-rest-spread](https://babeljs.io/docs/babel-plugin-transform-object-rest-spread).
//!
//! ## References:
//! * Babel plugin implementation: <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-object-rest-spread>
//! * Object rest/spread TC39 proposal: <https://github.com/tc39/proposal-object-rest-spread>

use crate::context::Ctx;

use object_rest::ObjectRest;
use object_spread::ObjectSpread;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};
use serde::Deserialize;
use std::rc::Rc;
mod object_rest;
mod object_spread;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ObjectRestSpreadOptions {
    #[serde(alias = "loose")]
    pub(crate) set_spread_properties: bool,
    pub(crate) use_built_ins: bool,
}

#[allow(dead_code)]
pub struct ObjectRestSpread<'a> {
    ctx: Ctx<'a>,
    options: ObjectRestSpreadOptions,

    // Plugins
    object_spread: ObjectSpread<'a>,
    object_rest: ObjectRest<'a>,
}

impl<'a> ObjectRestSpread<'a> {
    pub fn new(options: ObjectRestSpreadOptions, ctx: Ctx<'a>) -> Self {
        Self {
            object_spread: ObjectSpread::new(options, Rc::clone(&ctx)),
            object_rest: ObjectRest::new(options, Rc::clone(&ctx)),
            ctx,
            options,
        }
    }
}

impl<'a> Traverse<'a> for ObjectRestSpread<'a> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.object_spread.enter_expression(expr, ctx);
    }
}
