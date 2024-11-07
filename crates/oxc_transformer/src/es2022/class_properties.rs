//! ES2022: Class Properties
//!
//! This plugin transforms class properties to initializers inside class constructor.
//!
//! > This plugin is included in `preset-env`, in ES2022
//!
//! ## Example
//!
//! Input:
//! ```js
//! class C {
//!   foo = 123;
//!   #bar = 456;
//! }
//!
//! let x = 123;
//! class D extends S {
//!   foo = x;
//!   constructor(x) {
//!     if (x) {
//!       let s = super(x);
//!     } else {
//!       super(x);
//!     }
//!   }
//! }
//! ```
//!
//! Output:
//! ```js
//! var _bar = /*#__PURE__*/ new WeakMap();
//! class C {
//!   constructor() {
//!     babelHelpers.defineProperty(this, "foo", 123);
//!     babelHelpers.classPrivateFieldInitSpec(this, _bar, 456);
//!   }
//! }
//!
//! let x = 123;
//! class D extends S {
//!   constructor(_x) {
//!     if (_x) {
//!       let s = (super(_x), babelHelpers.defineProperty(this, "foo", x));
//!     } else {
//!       super(_x);
//!       babelHelpers.defineProperty(this, "foo", x);
//!     }
//!   }
//! }
//! ```
//!
//! ## Implementation
//!
//! WORK IN PROGRESS. INCOMPLETE.
//!
//! Implementation based on [@babel/plugin-transform-class-properties](https://babel.dev/docs/babel-plugin-transform-class-properties).
//!
//! ## References:
//! * Babel plugin implementation:
//!   * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-class-properties>
//!   * <https://github.com/babel/babel/blob/main/packages/babel-helper-create-class-features-plugin/src/index.ts>
//!   * <https://github.com/babel/babel/blob/main/packages/babel-helper-create-class-features-plugin/src/fields.ts>
//! * Class properties TC39 proposal: <https://github.com/tc39/proposal-class-fields>

use serde::Deserialize;

use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ClassPropertiesOptions {
    #[serde(alias = "loose")]
    pub(crate) set_public_class_fields: bool,
}

pub struct ClassProperties<'a, 'ctx> {
    #[expect(dead_code)]
    options: ClassPropertiesOptions,
    #[expect(dead_code)]
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    pub fn new(options: ClassPropertiesOptions, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { options, ctx }
    }
}

impl<'a, 'ctx> Traverse<'a> for ClassProperties<'a, 'ctx> {
    fn enter_class_body(&mut self, _body: &mut ClassBody<'a>, _ctx: &mut TraverseCtx<'a>) {}
}
