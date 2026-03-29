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

        let old_len = body.body.len();
        body.body.retain(|element| match element {
            ClassElement::PropertyDefinition(prop) => {
                let PropertyKey::PrivateIdentifier(private_id) = &prop.key else {
                    return true;
                };
                let name: Atom = private_id.name.into();
                if ctx.state.class_symbols_stack.is_private_member_used_in_current_class(&name) {
                    return true;
                }
                prop.value.as_ref().is_some_and(|value| value.may_have_side_effects(ctx))
            }
            ClassElement::MethodDefinition(method) => {
                let PropertyKey::PrivateIdentifier(private_id) = &method.key else {
                    return true;
                };
                let name: Atom = private_id.name.into();
                ctx.state.class_symbols_stack.is_private_member_used_in_current_class(&name)
            }
            ClassElement::AccessorProperty(accessor) => {
                let PropertyKey::PrivateIdentifier(private_id) = &accessor.key else {
                    return true;
                };
                let name: Atom = private_id.name.into();
                if ctx.state.class_symbols_stack.is_private_member_used_in_current_class(&name) {
                    return true;
                }
                accessor.value.as_ref().is_some_and(|value| value.may_have_side_effects(ctx))
            }
            ClassElement::StaticBlock(_) => true,
            ClassElement::TSIndexSignature(_) => {
                unreachable!("TypeScript syntax should be transformed away")
            }
        });
        if body.body.len() != old_len {
            ctx.state.changed = true;
        }
    }

    pub fn get_declared_private_symbols(body: &ClassBody<'a>) -> impl Iterator<Item = Atom<'a>> {
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
