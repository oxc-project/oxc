use oxc_ast::ast::*;

use crate::to_primitive::ToPrimitive;

use super::super::{MayHaveSideEffects, MayHaveSideEffectsContext};

impl<'a> MayHaveSideEffects<'a> for IdentifierReference<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self.name.as_str() {
            "NaN" | "Infinity" | "undefined" => false,
            // Reading global variables may have a side effect.
            // NOTE: It should also return true when the reference might refer to a reference value created by a with statement
            // NOTE: we ignore TDZ errors
            _ => ctx.unknown_global_side_effects() && ctx.is_global_reference(self),
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for TemplateLiteral<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        self.expressions.iter().any(|e| {
            // ToString is called for each expression.
            // If the expression is a Symbol or ToPrimitive returns a Symbol, an error is thrown.
            // ToPrimitive returns the value as-is for non-Object values, so we can use it instead of ToString here.
            e.to_primitive(ctx).is_symbol() != Some(false) || e.may_have_side_effects(ctx)
        })
    }
}