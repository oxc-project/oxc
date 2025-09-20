use oxc_ast::ast::*;

use super::super::{MayHaveSideEffects, MayHaveSideEffectsContext};

impl<'a> MayHaveSideEffects<'a> for ObjectPropertyKind<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            ObjectPropertyKind::ObjectProperty(o) => o.may_have_side_effects(ctx),
            ObjectPropertyKind::SpreadProperty(e) => match &e.argument {
                Expression::ArrayExpression(arr) => arr.may_have_side_effects(ctx),
                Expression::StringLiteral(_) => false,
                Expression::TemplateLiteral(t) => t.may_have_side_effects(ctx),
                _ => true,
            },
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for ObjectProperty<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        self.key.may_have_side_effects(ctx) || self.value.may_have_side_effects(ctx)
    }
}

impl<'a> MayHaveSideEffects<'a> for PropertyKey<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            PropertyKey::StaticIdentifier(_) | PropertyKey::PrivateIdentifier(_) => false,
            match_expression!(PropertyKey) => {
                // ToPropertyKey(key) throws an error when ToPrimitive(key) throws an Error
                // But we can ignore that by using the assumption.
                self.to_expression().may_have_side_effects(ctx)
            }
        }
    }
}