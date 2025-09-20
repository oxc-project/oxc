use oxc_ast::ast::*;

use crate::{ToBigInt, ToIntegerIndex, constant_evaluation::DetermineValueType};

use super::super::{MayHaveSideEffects, MayHaveSideEffectsContext, PropertyReadSideEffects};

impl<'a> MayHaveSideEffects<'a> for MemberExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            MemberExpression::ComputedMemberExpression(e) => e.may_have_side_effects(ctx),
            MemberExpression::StaticMemberExpression(e) => e.may_have_side_effects(ctx),
            MemberExpression::PrivateFieldExpression(_) => {
                ctx.property_read_side_effects() != PropertyReadSideEffects::None
            }
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for StaticMemberExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        property_access_may_have_side_effects(&self.object, &self.property.name, ctx)
    }
}

impl<'a> MayHaveSideEffects<'a> for ComputedMemberExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match &self.expression {
            Expression::StringLiteral(s) => {
                property_access_may_have_side_effects(&self.object, &s.value, ctx)
            }
            Expression::TemplateLiteral(t) => t.single_quasi().is_some_and(|quasi| {
                property_access_may_have_side_effects(&self.object, &quasi, ctx)
            }),
            Expression::NumericLiteral(n) => !n.value.to_integer_index().is_some_and(|n| {
                !integer_index_property_access_may_have_side_effects(&self.object, n, ctx)
            }),
            Expression::BigIntLiteral(b) => {
                if b.is_negative() {
                    return true;
                }
                !b.to_big_int(ctx).and_then(ToIntegerIndex::to_integer_index).is_some_and(|b| {
                    !integer_index_property_access_may_have_side_effects(&self.object, b, ctx)
                })
            }
            _ => true,
        }
    }
}

fn property_access_may_have_side_effects<'a>(
    object: &Expression<'a>,
    property: &str,
    ctx: &impl MayHaveSideEffectsContext<'a>,
) -> bool {
    if object.may_have_side_effects(ctx) {
        return true;
    }
    if ctx.property_read_side_effects() == PropertyReadSideEffects::None {
        return false;
    }

    match property {
        "length" => {
            !(matches!(object, Expression::ArrayExpression(_))
                || object.value_type(ctx).is_string())
        }
        _ => true,
    }
}

fn integer_index_property_access_may_have_side_effects<'a>(
    object: &Expression<'a>,
    property: u32,
    ctx: &impl MayHaveSideEffectsContext<'a>,
) -> bool {
    if object.may_have_side_effects(ctx) {
        return true;
    }
    if ctx.property_read_side_effects() == PropertyReadSideEffects::None {
        return false;
    }
    match object {
        Expression::StringLiteral(s) => property as usize >= s.value.encode_utf16().count(),
        Expression::ArrayExpression(arr) => property as usize >= get_array_minimum_length(arr),
        _ => true,
    }
}

fn get_array_minimum_length(arr: &ArrayExpression) -> usize {
    arr.elements
        .iter()
        .map(|e| match e {
            ArrayExpressionElement::SpreadElement(spread) => match &spread.argument {
                Expression::ArrayExpression(arr) => get_array_minimum_length(arr),
                Expression::StringLiteral(str) => str.value.chars().count(),
                _ => 0,
            },
            _ => 1,
        })
        .sum()
}