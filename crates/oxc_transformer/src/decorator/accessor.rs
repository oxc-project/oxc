//! Auto-accessor lowering shared by standard and legacy decorators.

use oxc_allocator::{ArenaVec, CloneIn, GetAllocator};
use oxc_ast::ast::*;
use oxc_ast_visit::VisitJs;
use oxc_span::SPAN;
use oxc_str::Str;
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
    utils::{
        ast_builder::{create_accessor_method, take_literal_property_key},
        decorated_class::is_auto_accessor,
    },
};

/// Replace each runtime auto-accessor in place with storage and a getter/setter pair.
///
/// `accessor prop = val` becomes:
/// ```js
/// #_prop_accessor_storage = val;
/// get prop() { return this.#_prop_accessor_storage; }
/// set prop(value) { this.#_prop_accessor_storage = value; }
/// ```
pub(super) fn lower_auto_accessors<'a>(
    body: &mut ClassBody<'a>,
    class_scope_id: ScopeId,
    static_binding: Option<&BoundIdentifier<'a>>,
    native_fields: bool,
    ctx: &mut TraverseCtx<'a>,
) {
    let accessor_count = body.body.iter().filter(|element| is_auto_accessor(element)).count();
    if accessor_count == 0 {
        return;
    }
    // Storage must not capture references to enclosing private names, including those in
    // nested classes. Reserving all private identifiers also covers this class's declarations.
    let mut collector = PrivateNameCollector::default();
    collector.visit_class_body(body);
    let mut used_names = collector.names;
    let mut unique_name = |base: Str<'a>, ctx: &TraverseCtx<'a>| -> Str<'a> {
        let mut name = base;
        let mut n = 2u32;
        while used_names.contains(name.as_str()) {
            name = Str::from_strs_array_in([&base, itoa::Buffer::new().format(n)], ctx);
            n += 1;
        }
        used_names.insert(name.as_str());
        name
    };
    // Each accessor is replaced by storage plus a getter and setter, so the result is
    // `len + 2 * accessor_count` long.
    let mut new_body = ArenaVec::with_capacity_in(body.body.len() + accessor_count * 2, ctx);
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
            // A non-computed literal key stays non-computed unless it has to be converted once.
            key @ (PropertyKey::StaticStringLiteral(_)
            | PropertyKey::StaticNumericLiteral(_)
            | PropertyKey::StaticBigIntLiteral(_))
                if !native_fields =>
            {
                let name = positional_storage_name(&mut counter, ctx);
                let setter_key = key.clone_in(ctx.allocator());
                (name, key, setter_key)
            }
            mut key => {
                let name = positional_storage_name(&mut counter, ctx);
                let mut key = take_literal_property_key(&mut key, ctx)
                    .unwrap_or_else(|| key.into_expression());
                if native_fields {
                    key = helper_call_expr(
                        Helper::ToPropertyKey,
                        ArenaVec::from_value_in(Argument::from(key), ctx),
                        ctx,
                    );
                }
                // Share one key between the generated methods:
                // `accessor [expr]` -> `get [_key = expr]() { ... } set [_key](value) { ... }`.
                // With native fields, convert to a property key once before sharing it.
                let (assignment, reference) = if native_fields {
                    create_computed_key_temp_var(key, ctx)
                } else {
                    duplicate_expression(key, true, ctx)
                };
                (name, PropertyKey::from(assignment), PropertyKey::from(reference))
            }
        };
        let storage_name = unique_name(storage_name, ctx);
        let storage = PropertyDefinition::boxed(
            SPAN,
            PropertyDefinitionType::PropertyDefinition,
            [],
            PropertyKey::new_private_identifier(SPAN, storage_name, ctx),
            None,
            accessor.value.take(),
            accessor.r#static,
            false,
            false,
            false,
            false,
            false,
            None,
            ctx,
        );
        // Use the defining class for static storage so inherited accessors can pass its private
        // brand check: `get prop() { return Class.#_prop_accessor_storage; }`.
        let object = if accessor.r#static { static_binding } else { None };
        // Decorators move to the getter for later processing. Its return type also carries the
        // accessor's annotation so legacy `emitDecoratorMetadata` can derive `design:type`.
        let getter = create_accessor_method(
            accessor.decorators,
            getter_key,
            MethodDefinitionKind::Get,
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
            accessor.r#static,
            storage_name,
            object,
            class_scope_id,
            None,
            ctx,
        );
        new_body.extend([ClassElement::PropertyDefinition(storage), getter, setter]);
    }
    body.body = new_body;
}

/// Storage name for a key without a usable spelling: `_0_accessor_storage`, `_1_accessor_storage`, ...
fn positional_storage_name<'a>(counter: &mut u32, ctx: &TraverseCtx<'a>) -> Str<'a> {
    let name = Str::from_strs_array_in(
        ["_", itoa::Buffer::new().format(*counter), "_accessor_storage"],
        ctx,
    );
    *counter += 1;
    name
}

#[derive(Default)]
struct PrivateNameCollector<'a> {
    names: FxHashSet<&'a str>,
}

impl<'a> VisitJs<'a> for PrivateNameCollector<'a> {
    fn visit_private_identifier(&mut self, ident: &PrivateIdentifier<'a>) {
        self.names.insert(ident.name.as_str());
    }
}
