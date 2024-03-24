use oxc_ast::ast::*;

use crate::{
    context::TransformerCtx,
    options::{TransformOptions, TransformTarget},
};

/// ES2021: Numeric Separator
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-numeric-separator>
pub struct NumericSeparator<'a> {
    ctx: TransformerCtx<'a>,
}

impl<'a> NumericSeparator<'a> {
    #![allow(clippy::unused_self)]

    pub fn new(ctx: TransformerCtx<'a>, options: &TransformOptions) -> Option<Self> {
        (options.target < TransformTarget::ES2021 || options.numeric_separator)
            .then_some(Self { ctx })
    }

    pub fn transform_number_literal(&mut self, lit: &mut NumericLiteral<'a>) {
        if !lit.raw.is_empty() {
            lit.raw = self.ctx.ast.new_str(lit.raw.replace('_', "").as_str());
        }
    }

    pub fn transform_bigint_literal(&mut self, lit: &mut BigIntLiteral<'a>) {
        let raw = &lit.raw;
        if !raw.is_empty() {
            lit.raw = raw.replace('_', "").into();
        }
    }
}
