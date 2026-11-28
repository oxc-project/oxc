//! Auto-accessor lowering shared by standard and legacy decorators.

use oxc_allocator::{ArenaBox, ArenaVec, CloneIn, GetAllocator};
use oxc_ast::ast::*;
use oxc_ast_visit::VisitJs;
use oxc_span::{GetSpan, SPAN};
use oxc_str::{JSStr, Str};
use oxc_syntax::scope::ScopeId;
use oxc_traverse::BoundIdentifier;

use crate::{
    common::{computed_key::create_computed_key_temp_var, duplicate::duplicate_expression},
    context::TraverseCtx,
    utils::{
        ast_builder::{create_accessor_method, create_to_property_key, take_literal_property_key},
        decorated_class::{PrivateNames, is_auto_accessor},
    },
};

/// Name of a property key for named evaluation: `x`, `"x"`, `1`, `#x` and a computed `["x"]`.
pub(super) fn static_key_name<'a>(
    key: &PropertyKey<'a>,
    ctx: &TraverseCtx<'a>,
) -> Option<JSStr<'a>> {
    match key {
        PropertyKey::StringLiteral(lit) => Some(lit.value),
        PropertyKey::PrivateIdentifier(ident) => {
            Some(Str::from_strs_array_in(["#", &ident.name], ctx).into())
        }
        _ => key.literal_name().map(|name| name.into_js_str_in(ctx)),
    }
}

/// How [`lower_auto_accessors`] converts computed keys shared by a getter and setter.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum AccessorKeys {
    /// Class fields stay native. Every non-identifier key is converted with `toPropertyKey` once
    /// and stored in a temp var before the class.
    Native,
    /// Class-properties lowers the class. Keys other than string and numeric literals are
    /// converted with `toPropertyKey` once, inline in the getter key.
    Lowered,
    /// Legacy decorators keep TypeScript's output: keys are shared without conversion, because
    /// wrapping them would reorder the `decorate` calls.
    Legacy,
}

/// Elements generated for one auto-accessor, passed to the caller's `record` callback before
/// traversal visits them.
pub(super) struct GeneratedAccessor<'r, 'a> {
    pub storage: &'r mut ArenaBox<'a, PropertyDefinition<'a>>,
    pub setter: &'r ArenaBox<'a, MethodDefinition<'a>>,
}

/// Replace each runtime auto-accessor in place with storage and a getter/setter pair.
///
/// `accessor prop = val` becomes:
/// ```js
/// #_prop_accessor_storage = val;
/// get prop() { return this.#_prop_accessor_storage; }
/// set prop(value) { this.#_prop_accessor_storage = value; }
/// ```
///
/// `record` receives the generated elements of each accessor, for mode-specific metadata.
pub(super) fn lower_auto_accessors<'a>(
    body: &mut ClassBody<'a>,
    class_scope_id: ScopeId,
    static_binding: Option<&BoundIdentifier<'a>>,
    keys: AccessorKeys,
    mut record: impl FnMut(GeneratedAccessor<'_, 'a>, &mut TraverseCtx<'a>),
    ctx: &mut TraverseCtx<'a>,
) {
    let native_fields = keys == AccessorKeys::Native;
    let accessor_count = body.body.iter().filter(|element| is_auto_accessor(element)).count();
    if accessor_count == 0 {
        return;
    }
    // Storage must not capture references to enclosing private names, including those in
    // nested classes. Reserving all private identifiers also covers this class's declarations.
    let mut private_names = PrivateNames::default();
    private_names.visit_class_body(body);
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
        // The storage, getter, setter and their bodies map to the accessor, so a debugger
        // stepping into them lands on its declaration. Statements in the bodies map to the key.
        let span = accessor.span;
        let key_span = accessor.key.span();
        let (storage_name, getter_key, setter_key) = match accessor.key {
            PropertyKey::StaticIdentifier(ident) => {
                let name = Str::from_strs_array_in(["_", &ident.name, "_accessor_storage"], ctx);
                let setter_key = PropertyKey::new_static_identifier(ident.span, ident.name, ctx);
                (name, PropertyKey::StaticIdentifier(ident), setter_key)
            }
            PropertyKey::PrivateIdentifier(ident) => {
                let name = Str::from_strs_array_in(["_", &ident.name, "_accessor_storage"], ctx);
                let setter_key = PropertyKey::new_private_identifier(ident.span, ident.name, ctx);
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
                // Convert to a property key once before sharing it. Otherwise an object key's
                // `toString` runs for each method, and they can be defined on different keys.
                let convert = match keys {
                    AccessorKeys::Native => true,
                    AccessorKeys::Lowered => {
                        !matches!(key, Expression::StringLiteral(_) | Expression::NumericLiteral(_))
                    }
                    AccessorKeys::Legacy => false,
                };
                if convert {
                    key = create_to_property_key(key, ctx);
                }
                // Share one key between the generated methods:
                // `accessor [expr]` -> `get [_key = expr]() { ... } set [_key](value) { ... }`.
                let (assignment, reference) = if native_fields {
                    create_computed_key_temp_var(key, ctx)
                } else {
                    duplicate_expression(key, true, ctx)
                };
                (name, PropertyKey::from(assignment), PropertyKey::from(reference))
            }
        };
        let storage_name = private_names.generate(storage_name, ctx);
        let mut storage = PropertyDefinition::boxed(
            span,
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
            span,
            key_span,
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
            span,
            key_span,
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
        record(GeneratedAccessor { storage: &mut storage, setter: &setter }, ctx);
        new_body.extend([
            ClassElement::PropertyDefinition(storage),
            ClassElement::MethodDefinition(getter),
            ClassElement::MethodDefinition(setter),
        ]);
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
