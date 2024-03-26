use std::rc::Rc;

use oxc_ast::{ast::*, AstBuilder};

use crate::{
    context::TransformerCtx,
    options::{TransformOptions, TransformTarget},
};

/// ES2015: Shorthand Properties
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-literals>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-literals/src/index.ts>
pub struct Literals<'a> {
    _ast: Rc<AstBuilder<'a>>,
}

impl<'a> Literals<'a> {
    #![allow(clippy::unused_self)]

    pub fn new(ctx: TransformerCtx<'a>, options: &TransformOptions) -> Option<Self> {
        (options.target < TransformTarget::ES2015 || options.literals)
            .then_some(Self { _ast: ctx.ast })
    }

    pub fn transform_number_literal(&mut self, lit: &mut NumericLiteral<'a>) {
        // early return if number's raw value is empty or shorter than 2 characters,
        // both `0bxxx` and `0oxxx` need at least 3 characters be defined.
        if lit.raw.len() <= 2 {
            return;
        }

        if let [b'0', b'b' | b'B' | b'o' | b'O'] = lit.raw[0..2].as_bytes() {
            // Set binary and octal raw values to empty, It would force the codegen,
            // go generate them from their value.
            lit.raw = "";
        }
    }

    pub fn transform_string_literal(&mut self, _: &mut StringLiteral<'a>) {
        // TODO: As of today oxc_lexer takes care of this, We have to rework it
        // so it can be controlled via the transformer.
    }
}
