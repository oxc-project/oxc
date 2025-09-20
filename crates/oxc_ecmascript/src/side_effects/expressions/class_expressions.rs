use oxc_ast::ast::*;

use super::super::{MayHaveSideEffects, MayHaveSideEffectsContext};

impl<'a> MayHaveSideEffects<'a> for Class<'a> {
    /// Based on <https://github.com/evanw/esbuild/blob/v0.25.0/internal/js_ast/js_ast_helpers.go#L2320>
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        if !self.decorators.is_empty() {
            return true;
        }

        // NOTE: extending a value that is neither constructors nor null, throws an error
        // but that error is ignored here (it is included in the assumption)
        // Example cases: `class A extends 0 {}`, `class A extends (async function() {}) {}`
        // Considering these cases is difficult and requires to de-opt most classes with a super class.
        // To allow classes with a super class to be removed, we ignore this side effect.
        if self.super_class.as_ref().is_some_and(|sup| {
            // `(class C extends (() => {}))` is TypeError.
            matches!(sup.without_parentheses(), Expression::ArrowFunctionExpression(_))
                || sup.may_have_side_effects(ctx)
        }) {
            return true;
        }

        self.body.body.iter().any(|element| element.may_have_side_effects(ctx))
    }
}

impl<'a> MayHaveSideEffects<'a> for ClassElement<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            ClassElement::StaticBlock(block) => {
                block.body.iter().any(|stmt| stmt.may_have_side_effects(ctx))
            }
            ClassElement::MethodDefinition(e) => {
                !e.decorators.is_empty() || e.key.may_have_side_effects(ctx)
            }
            ClassElement::PropertyDefinition(e) => {
                !e.decorators.is_empty()
                    || e.key.may_have_side_effects(ctx)
                    || (e.r#static
                        && e.value.as_ref().is_some_and(|v| v.may_have_side_effects(ctx)))
            }
            ClassElement::AccessorProperty(e) => {
                !e.decorators.is_empty() || e.key.may_have_side_effects(ctx)
            }
            ClassElement::TSIndexSignature(_) => false,
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for ChainElement<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            ChainElement::CallExpression(e) => e.may_have_side_effects(ctx),
            ChainElement::TSNonNullExpression(e) => e.expression.may_have_side_effects(ctx),
            match_member_expression!(ChainElement) => {
                self.to_member_expression().may_have_side_effects(ctx)
            }
        }
    }
}
