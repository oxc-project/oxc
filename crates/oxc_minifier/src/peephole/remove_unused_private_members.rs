use oxc_ast::ast::*;
use oxc_ecmascript::side_effects::MayHaveSideEffects;

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
            let keep = match element {
                ClassElement::PropertyDefinition(prop) => {
                    let PropertyKey::PrivateIdentifier(private_id) = &prop.key else {
                        return true;
                    };
                    let name: Str = private_id.name.into();
                    if ctx.state.private_member_usage.is_used(&name) {
                        return true;
                    }
                    prop.value.as_ref().is_some_and(|value| value.may_have_side_effects(ctx))
                }
                ClassElement::MethodDefinition(method) => {
                    let PropertyKey::PrivateIdentifier(private_id) = &method.key else {
                        return true;
                    };
                    let name: Str = private_id.name.into();
                    ctx.state.private_member_usage.is_used(&name)
                }
                ClassElement::AccessorProperty(accessor) => {
                    let PropertyKey::PrivateIdentifier(private_id) = &accessor.key else {
                        return true;
                    };
                    let name: Str = private_id.name.into();
                    if ctx.state.private_member_usage.is_used(&name) {
                        return true;
                    }
                    accessor.value.as_ref().is_some_and(|value| value.may_have_side_effects(ctx))
                }
                ClassElement::StaticBlock(_) => true,
                ClassElement::TSIndexSignature(_) => {
                    unreachable!("TypeScript syntax should be transformed away")
                }
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

    pub fn declared_private_member_names(body: &ClassBody<'a>) -> impl Iterator<Item = Str<'a>> {
        body.body.iter().filter_map(|element| match element {
            ClassElement::PropertyDefinition(prop) => {
                let PropertyKey::PrivateIdentifier(private_id) = &prop.key else {
                    return None;
                };
                Some(private_id.name.into())
            }
            ClassElement::MethodDefinition(method) => {
                let PropertyKey::PrivateIdentifier(private_id) = &method.key else {
                    return None;
                };
                Some(private_id.name.into())
            }
            ClassElement::AccessorProperty(accessor) => {
                let PropertyKey::PrivateIdentifier(private_id) = &accessor.key else {
                    return None;
                };
                Some(private_id.name.into())
            }
            ClassElement::StaticBlock(_) => None,
            ClassElement::TSIndexSignature(_) => {
                unreachable!("TypeScript syntax should be transformed away")
            }
        })
    }
}
