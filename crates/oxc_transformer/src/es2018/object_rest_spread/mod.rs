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

use serde::Deserialize;

use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::context::TransformCtx;

mod object_rest;
mod object_spread;
use object_rest::ObjectRest;
use object_spread::ObjectSpread;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ObjectRestSpreadOptions {
    #[serde(alias = "loose")]
    pub(crate) set_spread_properties: bool,
    pub(crate) use_built_ins: bool,
}

pub struct ObjectRestSpread<'a, 'ctx> {
    #[allow(dead_code)]
    options: ObjectRestSpreadOptions,

    // Plugins
    object_spread: ObjectSpread<'a, 'ctx>,
    #[allow(dead_code)]
    object_rest: ObjectRest,
}

impl<'a, 'ctx> ObjectRestSpread<'a, 'ctx> {
    pub fn new(options: ObjectRestSpreadOptions, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self {
            object_spread: ObjectSpread::new(options, ctx),
            object_rest: ObjectRest::new(options),
            options,
        }
    }
}

impl<'a, 'ctx> Traverse<'a> for ObjectRestSpread<'a, 'ctx> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.object_spread.enter_expression(expr, ctx);
    }
}
