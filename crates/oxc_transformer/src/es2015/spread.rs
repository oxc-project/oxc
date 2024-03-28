use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{GetSpan, Span};

use crate::{context::TransformerCtx, options::TransformTarget};

/// ES2015: Spread
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-spread>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-spread>
pub struct Spread<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> Spread<'a> {
    pub fn new(ctx: TransformerCtx<'a>) -> Option<Self> {
        (ctx.options.target < TransformTarget::ES2015 || ctx.options.spread)
            .then_some(Self { ast: ctx.ast })
    }

    pub fn transform_array_expression<'b>(&mut self, expr: &'b mut ArrayExpression<'a>) {
        // Return early if array contains no spread elements
        if !expr.elements.iter().any(|e| e.is_spread_element()) {
            return;
        }

        let iter = expr.elements.iter();

        // SAFETY: we already checked that elements is not empty.
        let first = unsafe { iter.next().unwrap_unchecked() };

        let first = match first {
            ArrayExpressionElement::Elision(..) | ArrayExpressionElement::Expression(..) => {
                return;
            }
            ArrayExpressionElement::SpreadElement(spread)
                if spread.argument.is_array_expression() =>
            {
                let first = match expr.elements.remove(0) {
                    ArrayExpressionElement::SpreadElement(spread) => spread,
                    _ => unreachable!(
                        "We just checked for the element being `SpreadElement` earlier."
                    ),
                };

                match first.unbox().argument {
                    Expression::ArrayExpression(array) => array,
                    _ => unreachable!(
                        "We just checked for the argument being `ArrayExpression` earlier."
                    ),
                }
            }
            ArrayExpressionElement::SpreadElement(..) => self.ast.alloc(ArrayExpression {
                span: Span::default(),
                elements: self.ast.new_vec(),
                trailing_comma: None,
            }),
        };
    }
}
