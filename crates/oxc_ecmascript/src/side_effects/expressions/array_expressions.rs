use oxc_ast::ast::*;

use super::super::{MayHaveSideEffects, MayHaveSideEffectsContext};

impl<'a> MayHaveSideEffects<'a> for ArrayExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        self.elements.iter().any(|element| element.may_have_side_effects(ctx))
    }
}

impl<'a> MayHaveSideEffects<'a> for ArrayExpressionElement<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            ArrayExpressionElement::SpreadElement(e) => match &e.argument {
                Expression::ArrayExpression(arr) => arr.may_have_side_effects(ctx),
                Expression::StringLiteral(_) => false,
                Expression::TemplateLiteral(t) => t.may_have_side_effects(ctx),
                Expression::Identifier(ident) => {
                    // TODO: Treat `arguments` outside a function scope as having side effects.
                    // Currently assumes global `arguments` is side-effect-free, but this may be incorrect
                    // in some contexts (e.g., if `arguments` is redefined globally).
                    !(ident.name == "arguments" && ctx.is_global_reference(ident))
                }
                _ => true,
            },
            match_expression!(ArrayExpressionElement) => {
                self.to_expression().may_have_side_effects(ctx)
            }
            ArrayExpressionElement::Elision(_) => false,
        }
    }
}

/// Get the minimum guaranteed length of an array expression
pub fn get_array_minimum_length(arr: &ArrayExpression) -> usize {
    let mut length = 0;
    for element in &arr.elements {
        match element {
            ArrayExpressionElement::SpreadElement(_) => break,
            _ => length += 1,
        }
    }
    length
}