//! Lowering of decorated `accessor` properties.

use std::mem;

use rustc_hash::FxHashSet;

use oxc_allocator::{ArenaVec, GetAddress};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_str::Str;
use oxc_syntax::scope::ScopeId;
use oxc_traverse::BoundIdentifier;

use crate::{
    common::duplicate::duplicate_expression, context::TraverseCtx,
    utils::ast_builder::create_accessor_method,
};

use super::StandardDecoratorState;

impl<'a> StandardDecoratorState<'a> {
    /// Entry phase: lower `accessor` properties to private storage + getter/setter pair.
    ///
    /// `@dec accessor x = 1` -> `#_x_accessor_storage = 1; @dec get x() { return this.#_x_accessor_storage; } set x(value) { ... }`
    ///
    /// Decorators move to the getter, which is recorded so it's decorated with `accessor` kind.
    pub(crate) fn lower_auto_accessors(
        &mut self,
        body: &mut ClassBody<'a>,
        class_scope_id: ScopeId,
        static_binding: Option<&BoundIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !body
            .body
            .iter()
            .any(|e| matches!(e, ClassElement::AccessorProperty(p) if !p.r#type.is_abstract()))
        {
            return;
        }

        // Private names are matched by name, so generated storage names must not clash
        // with existing private names, or with each other (`accessor x` + `static accessor x`).
        let mut used_names: FxHashSet<&str> = body
            .body
            .iter()
            .filter_map(|element| match element {
                ClassElement::PropertyDefinition(p) => p.key.private_name(),
                ClassElement::MethodDefinition(m) => m.key.private_name(),
                ClassElement::AccessorProperty(p) => p.key.private_name(),
                _ => None,
            })
            .map(|name| name.as_str())
            .collect();
        let mut unique_name = |base: Str<'a>, ctx: &TraverseCtx<'a>| -> Str<'a> {
            let mut name = base;
            let mut n = 2u32;
            while used_names.contains(name.as_str()) {
                name = Str::from_strs_array_in([&base, &n.to_string()], ctx);
                n += 1;
            }
            used_names.insert(name.as_str());
            name
        };

        let mut new_body = ArenaVec::with_capacity_in(body.body.len() + 4, ctx);
        let mut counter = 0u32;
        for element in body.body.drain(..) {
            let ClassElement::AccessorProperty(accessor) = element else {
                new_body.push(element);
                continue;
            };
            if accessor.r#type.is_abstract() {
                new_body.push(ClassElement::AccessorProperty(accessor));
                continue;
            }

            let mut accessor = accessor.unbox();
            let is_static = accessor.r#static;
            let computed = accessor.computed;
            let decorators = mem::replace(&mut accessor.decorators, ArenaVec::new_in(ctx));
            let is_decorated = !decorators.is_empty();

            let original_name = Self::static_key_name(&accessor.key, ctx);
            let (storage_name, getter_key, setter_key) = match accessor.key {
                PropertyKey::StaticIdentifier(ident) => {
                    let name =
                        Str::from_strs_array_in(["_", &ident.name, "_accessor_storage"], ctx);
                    let key = PropertyKey::StaticIdentifier(ident);
                    let key2 = PropertyKey::new_static_identifier(
                        SPAN,
                        match &key {
                            PropertyKey::StaticIdentifier(ident) => ident.name,
                            _ => unreachable!(),
                        },
                        ctx,
                    );
                    (name, key, key2)
                }
                PropertyKey::PrivateIdentifier(ident) => {
                    let name =
                        Str::from_strs_array_in(["_", &ident.name, "_accessor_storage"], ctx);
                    let key2 = PropertyKey::new_private_identifier(SPAN, ident.name, ctx);
                    (name, PropertyKey::PrivateIdentifier(ident), key2)
                }
                key => {
                    let counter_str = counter.to_string();
                    counter += 1;
                    let name =
                        Str::from_strs_array_in(["_", &counter_str, "_accessor_storage"], ctx);
                    let key_expr = key.into_expression();
                    let (assignment, reference) = duplicate_expression(key_expr, true, ctx);
                    (name, PropertyKey::from(assignment), PropertyKey::from(reference))
                }
            };

            let storage_name = unique_name(storage_name, ctx);

            // `#_x_accessor_storage = value`
            let storage = ClassElement::new_property_definition(
                SPAN,
                PropertyDefinitionType::PropertyDefinition,
                [],
                PropertyKey::new_private_identifier(SPAN, storage_name, ctx),
                None,
                accessor.value.take(),
                false,
                is_static,
                false,
                false,
                false,
                false,
                false,
                None,
                ctx,
            );
            if let ClassElement::PropertyDefinition(prop) = &storage {
                if is_decorated {
                    self.accessor_storage.insert(prop.address());
                }
                if let Some(name) = &original_name {
                    self.accessor_names.insert(prop.address(), *name);
                }
            }
            new_body.push(storage);

            let getter = create_accessor_method(
                decorators,
                getter_key,
                MethodDefinitionKind::Get,
                computed,
                is_static,
                storage_name,
                if is_static { static_binding } else { None },
                class_scope_id,
                None,
                ctx,
            );
            if is_decorated && let ClassElement::MethodDefinition(method) = &getter {
                self.accessor_getters.insert(method.address());
            }
            new_body.push(getter);

            new_body.push(create_accessor_method(
                ArenaVec::new_in(ctx),
                setter_key,
                MethodDefinitionKind::Set,
                computed,
                is_static,
                storage_name,
                if is_static { static_binding } else { None },
                class_scope_id,
                None,
                ctx,
            ));
        }
        body.body = new_body;
    }
}
