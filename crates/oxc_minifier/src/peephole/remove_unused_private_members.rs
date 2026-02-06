use oxc_ast::ast::*;
use oxc_ecmascript::side_effects::MayHaveSideEffects;
use oxc_span::Ident;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// Remove unused private fields and methods from class body
    ///
    /// This function uses the private member usage collected during the main traverse
    /// to remove unused private fields and methods from the class body.
    pub fn remove_unused_private_members(body: &mut ClassBody<'a>, ctx: &mut Ctx<'a, '_>) {
        if ctx.current_scope_flags().contains_direct_eval() {
            return;
        }

        let old_len = body.body.len();
        body.body.retain(|element| match element {
            ClassElement::PropertyDefinition(prop) => {
                let PropertyKey::PrivateIdentifier(private_id) = &prop.key else {
                    return true;
                };
                let name = private_id.name;
                if ctx.state.class_symbols_stack.is_private_member_used_in_current_class(&name) {
                    return true;
                }
                prop.value.as_ref().is_some_and(|value| value.may_have_side_effects(ctx))
            }
            ClassElement::MethodDefinition(method) => {
                let PropertyKey::PrivateIdentifier(private_id) = &method.key else {
                    return true;
                };
                let name = private_id.name;
                ctx.state.class_symbols_stack.is_private_member_used_in_current_class(&name)
            }
            ClassElement::AccessorProperty(accessor) => {
                let PropertyKey::PrivateIdentifier(private_id) = &accessor.key else {
                    return true;
                };
                let name = private_id.name;
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

    pub fn get_declared_private_symbols(body: &ClassBody<'a>) -> impl Iterator<Item = Ident<'a>> {
        body.body.iter().filter_map(|element| match element {
            ClassElement::PropertyDefinition(prop) => {
                let PropertyKey::PrivateIdentifier(private_id) = &prop.key else {
                    return None;
                };
                Some(private_id.name)
            }
            ClassElement::MethodDefinition(method) => {
                let PropertyKey::PrivateIdentifier(private_id) = &method.key else {
                    return None;
                };
                Some(private_id.name)
            }
            ClassElement::AccessorProperty(accessor) => {
                let PropertyKey::PrivateIdentifier(private_id) = &accessor.key else {
                    return None;
                };
                Some(private_id.name)
            }
            ClassElement::StaticBlock(_) => None,
            ClassElement::TSIndexSignature(_) => {
                unreachable!("TypeScript syntax should be transformed away")
            }
        })
    }
}
