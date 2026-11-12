//! Auto-accessor lowering shared by standard and legacy decorators.

use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_str::{JSStr, Str};
use oxc_syntax::scope::ScopeId;
use oxc_traverse::BoundIdentifier;
use rustc_hash::FxHashSet;

use crate::{
    common::{
        computed_key::create_computed_key_temp_var,
        duplicate::duplicate_expression,
        helper_loader::{Helper, helper_call_expr},
    },
    context::TraverseCtx,
    utils::{ast_builder::create_accessor_method, decorated_class::class_body_has_auto_accessors},
};

pub(super) fn static_key_name<'a>(
    key: &PropertyKey<'a>,
    ctx: &TraverseCtx<'a>,
) -> Option<JSStr<'a>> {
    match key {
        PropertyKey::StaticIdentifier(ident) => Some(ident.name.into()),
        PropertyKey::StringLiteral(lit) => Some(lit.value),
        PropertyKey::PrivateIdentifier(ident) => {
            Some(Str::from_strs_array_in(["#", &ident.name], ctx).into())
        }
        _ => None,
    }
}

/// Replace each runtime auto-accessor in place with storage and a getter/setter pair.
/// The callback records mode-specific metadata before traversal visits the generated elements.
pub(super) fn lower_auto_accessors<'a>(
    body: &mut ClassBody<'a>,
    class_scope_id: ScopeId,
    static_binding: Option<&BoundIdentifier<'a>>,
    native_fields: bool,
    mut record: impl FnMut(
        &mut ArenaBox<'a, PropertyDefinition<'a>>,
        &ArenaBox<'a, MethodDefinition<'a>>,
        &PropertyKey<'a>,
        Option<JSStr<'a>>,
        &mut TraverseCtx<'a>,
    ),
    ctx: &mut TraverseCtx<'a>,
) {
    if !class_body_has_auto_accessors(body) {
        return;
    }
    let mut used_names: FxHashSet<&str> = body
        .body
        .iter()
        .filter_map(|element| match element {
            ClassElement::PropertyDefinition(prop) => prop.key.private_name(),
            ClassElement::MethodDefinition(method) => method.key.private_name(),
            ClassElement::AccessorProperty(prop) => prop.key.private_name(),
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
        let original_name = static_key_name(&accessor.key, ctx);
        let mut computed = accessor.computed;
        let (storage_name, getter_key, setter_key) = match accessor.key {
            PropertyKey::StaticIdentifier(ident) => {
                let name = Str::from_strs_array_in(["_", &ident.name, "_accessor_storage"], ctx);
                let setter_key = PropertyKey::new_static_identifier(SPAN, ident.name, ctx);
                (name, PropertyKey::StaticIdentifier(ident), setter_key)
            }
            PropertyKey::PrivateIdentifier(ident) => {
                let name = Str::from_strs_array_in(["_", &ident.name, "_accessor_storage"], ctx);
                let setter_key = PropertyKey::new_private_identifier(SPAN, ident.name, ctx);
                (name, PropertyKey::PrivateIdentifier(ident), setter_key)
            }
            key => {
                let name =
                    Str::from_strs_array_in(["_", &counter.to_string(), "_accessor_storage"], ctx);
                counter += 1;
                let mut key = key.into_expression();
                if native_fields {
                    computed = true;
                    key = helper_call_expr(
                        Helper::ToPropertyKey,
                        ArenaVec::from_value_in(Argument::from(key), ctx),
                        ctx,
                    );
                }
                let (assignment, reference) = if native_fields {
                    create_computed_key_temp_var(key, ctx)
                } else {
                    duplicate_expression(key, true, ctx)
                };
                (name, PropertyKey::from(assignment), PropertyKey::from(reference))
            }
        };
        let storage_name = unique_name(storage_name, ctx);
        let mut storage = PropertyDefinition::boxed(
            SPAN,
            PropertyDefinitionType::PropertyDefinition,
            [],
            PropertyKey::new_private_identifier(SPAN, storage_name, ctx),
            None,
            accessor.value.take(),
            false,
            accessor.r#static,
            false,
            false,
            false,
            false,
            false,
            None,
            ctx,
        );
        let object = if accessor.r#static { static_binding } else { None };
        let getter = create_accessor_method(
            accessor.decorators,
            getter_key,
            MethodDefinitionKind::Get,
            computed,
            accessor.r#static,
            storage_name,
            object,
            class_scope_id,
            accessor.type_annotation.take(),
            ctx,
        );
        let setter = create_accessor_method(
            ArenaVec::new_in(ctx),
            setter_key,
            MethodDefinitionKind::Set,
            computed,
            accessor.r#static,
            storage_name,
            object,
            class_scope_id,
            None,
            ctx,
        );
        let ClassElement::MethodDefinition(getter_method) = &getter else { unreachable!() };
        let ClassElement::MethodDefinition(setter_method) = &setter else { unreachable!() };
        record(&mut storage, getter_method, &setter_method.key, original_name, ctx);
        new_body.extend([ClassElement::PropertyDefinition(storage), getter, setter]);
    }
    body.body = new_body;
}
