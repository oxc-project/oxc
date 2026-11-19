use oxc_ast::ast::*;
use oxc_ecmascript::side_effects::MayHaveSideEffects;
use oxc_str::Ident;

use crate::TraverseCtx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// Remove unused private fields and methods from class body
    ///
    /// This function uses the private member usage collected during the main traverse
    /// to remove unused private fields and methods from the class body.
    pub fn remove_unused_private_members(body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.current_scope_flags().contains_direct_eval() {
            return;
        }

        body.body.retain(|element| {
            if matches!(element, ClassElement::TSIndexSignature(_)) {
                unreachable!("TypeScript syntax should be transformed away");
            }
            let Some(name) = element.property_key().and_then(PropertyKey::private_name) else {
                return true;
            };
            if ctx.state.private_member_usage.is_used(&name) {
                return true;
            }

            let keep = match element {
                ClassElement::PropertyDefinition(prop) => {
                    prop.value.as_ref().is_some_and(|value| value.may_have_side_effects(ctx))
                }
                ClassElement::AccessorProperty(accessor) => {
                    accessor.value.as_ref().is_some_and(|value| value.may_have_side_effects(ctx))
                }
                _ => false,
            };
            if !keep {
                // The element is being silently dropped from the vector. Walk
                // its subtree so identifier references inside (e.g. a method
                // body) are recorded in `PassChanges::removed_references` and pruned by
                // `flush_pass_changes`. Without this, refs leak across passes
                // and break idempotency. `drop_class_element` also records a
                // mutation for the fixed-point loop driver.
                ctx.drop_class_element(element);
            }
            keep
        });
    }

    pub fn declared_private_member_names(body: &ClassBody<'a>) -> impl Iterator<Item = Ident<'a>> {
        body.body.iter().filter_map(|element| match element {
            ClassElement::TSIndexSignature(_) => {
                unreachable!("TypeScript syntax should be transformed away")
            }
            _ => element.property_key().and_then(PropertyKey::private_name),
        })
    }
}
