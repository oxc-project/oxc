//! ES2021: Numeric Separator
//!
//! This plugin remove underscore(_) in number (100_0_0).
//!
//! > This plugin is included in `preset-env`, in ES2021
//!
//! ## Example
//!
//! Input:
//! ```js
//! Decimal Literals
//! let budget = 1_000_000_000_000;
//!
//! Binary Literals
//! let nibbles = 0b1010_0001_1000_0101;
//!
//! Hex Literal
//! let message = 0xa0_b0_c0;
//! ```
//!
//! Output:
//! ```js
//! Decimal Literals
//! let budget = 1000000000000;
//!
//! Binary Literals
//! let nibbles = 0b1010000110000101;
//!
//! Hex Literal
//! let message = 0xa0b0c0;
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-numeric-separator](https://babel.dev/docs/babel-plugin-transform-numeric-separator).
//!
//! ## References:
//! * Babel plugin implementation: <https://github.com/babel/babel/blob/v7.26.2/packages/babel-plugin-transform-numeric-separator/src/index.ts>
//! * Numeric Separator TC39 proposal: <https://github.com/tc39/proposal-numeric-separator?tab=readme-ov-file>

use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

pub struct NumericSeparator<'a, 'ctx> {
    _ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> NumericSeparator<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { _ctx: ctx }
    }
}

const SEPARATOR: &str = "_";

impl<'a, 'ctx> Traverse<'a> for NumericSeparator<'a, 'ctx> {
    fn enter_numeric_literal(&mut self, node: &mut NumericLiteral<'a>, ctx: &mut TraverseCtx<'a>) {
        let raw = node.raw;
        if !raw.contains(SEPARATOR) {
            return;
        }
        let new_raw = raw.replace(SEPARATOR, "");
        let new_node = ctx.ast.numeric_literal(SPAN, node.value, new_raw, node.base);
        *node = new_node;
    }

    fn enter_big_int_literal(&mut self, node: &mut BigIntLiteral<'a>, ctx: &mut TraverseCtx<'a>) {
        let raw = &node.raw;
        if !raw.contains(SEPARATOR) {
            return;
        }
        let new_raw = raw.replace(SEPARATOR, "");
        let new_node = ctx.ast.big_int_literal(SPAN, new_raw, node.base);
        *node = new_node;
    }
}
