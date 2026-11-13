//! Auto-accessor lowering shared by decorator transforms.

use std::borrow::Cow;

use oxc_allocator::{ArenaVec, CloneIn, GetAllocator};
use oxc_ast::{StaticPropertyName, ast::*};
use oxc_span::SPAN;
use oxc_str::Str;
use oxc_syntax::scope::ScopeId;
use oxc_traverse::{BoundIdentifier, ast_operations::get_var_name_from_node};

use crate::{
    common::duplicate::duplicate_expression, context::TraverseCtx,
    utils::ast_builder::create_accessor_method,
};

/// Replace runtime auto-accessors with private backing fields and getter/setter pairs.
/// Decorators and type annotations move to the getter for legacy metadata emission.
pub(super) fn lower_auto_accessors<'a>(
    body: &mut ClassBody<'a>,
    class_scope_id: ScopeId,
    static_binding: Option<&BoundIdentifier<'a>>,
    ctx: &mut TraverseCtx<'a>,
) {
    let mut new_body = ArenaVec::with_capacity_in(body.body.len() * 3, ctx);

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

        // Transfer decorators to the getter so legacy decorator transform can process them.
        let decorators = std::mem::replace(&mut accessor.decorators, ArenaVec::new_in(ctx));
        // Transfer the type annotation to the getter's return type so that
        // `emitDecoratorMetadata` can derive `design:type` from it. Without this,
        // the lowered getter is untyped and `design:type` falls through to `Object`.
        let type_annotation = accessor.type_annotation.take();

        // For computed keys, duplicate the key expression so getter and setter
        // each get their own reference:
        //   `accessor [expr] = val` →
        //   `get [_expr = expr]() { ... } set [expr](value) { ... }`
        // Note: Legacy decorators do not support private identifiers,
        // so we only handle static identifiers and computed keys.
        let (storage_name, getter_key, setter_key) = if accessor.computed {
            let key_expr = accessor.key.into_expression();
            let (assignment, reference) = duplicate_expression(key_expr, true, ctx);

            // Use raw identifier name when possible to avoid collisions from
            // `get_var_name_from_node` stripping leading underscores.
            let key_name: Cow<'_, str> = match &reference {
                Expression::Identifier(ident) => Cow::Borrowed(ident.name.as_str()),
                _ => Cow::Owned(get_var_name_from_node(&reference)),
            };
            let getter_key = PropertyKey::from(assignment);
            let setter_key = PropertyKey::from(reference);
            let storage_name =
                Str::from_strs_array_in(["_", &key_name, "_computed_accessor_storage"], ctx);
            (storage_name, getter_key, setter_key)
        } else {
            // Use `name()` to get the raw property name, avoiding `get_var_name_from_node`
            // which strips leading underscores (e.g. `prop` and `_prop` both become "prop").
            let key_name = accessor
                .key
                .name()
                .and_then(StaticPropertyName::into_cow_str)
                .unwrap_or_else(|| Cow::Owned(get_var_name_from_node(&accessor.key)));
            let storage_name = Str::from_strs_array_in(["_", &key_name, "_accessor_storage"], ctx);
            let getter_key = accessor.key.clone_in(ctx.ast.allocator());
            let setter_key = accessor.key.clone_in(ctx.ast.allocator());
            (storage_name, getter_key, setter_key)
        };

        // For static accessors, use class name reference; for instance accessors, use `this`.
        let object_binding = if is_static { static_binding } else { None };

        // 1. Private backing field: `#<name>_accessor_storage = <value>`
        new_body.push(ClassElement::new_property_definition(
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
        ));

        // 2. Getter: `get <name>() { return <object>.#<name>_accessor_storage; }`
        new_body.push(create_accessor_method(
            decorators,
            getter_key,
            MethodDefinitionKind::Get,
            computed,
            is_static,
            storage_name,
            object_binding,
            class_scope_id,
            type_annotation,
            ctx,
        ));

        // 3. Setter: `set <name>(value) { <object>.#<name>_accessor_storage = value; }`
        new_body.push(create_accessor_method(
            ArenaVec::new_in(ctx),
            setter_key,
            MethodDefinitionKind::Set,
            computed,
            is_static,
            storage_name,
            object_binding,
            class_scope_id,
            None,
            ctx,
        ));
    }

    body.body = new_body;
}
