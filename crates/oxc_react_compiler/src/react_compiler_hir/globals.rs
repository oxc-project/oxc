// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Global type registry and built-in shape definitions, ported from Globals.ts.
//!
//! Provides `DEFAULT_SHAPES` (built-in object shapes) and `DEFAULT_GLOBALS`
//! (global variable types including React hooks and JS built-ins).

use std::sync::LazyLock;

use oxc_str::{Ident, IdentHashMap};

use crate::react_compiler_hir::Effect;
use crate::react_compiler_hir::Type;
use crate::react_compiler_hir::environment::is_hook_name;
use crate::react_compiler_hir::object_shape::*;
use crate::react_compiler_hir::type_config::AliasingEffectConfig;
use crate::react_compiler_hir::type_config::AliasingSignatureConfig;
use crate::react_compiler_hir::type_config::ApplyArgConfig;
use crate::react_compiler_hir::type_config::ApplyArgHoleKind;
use crate::react_compiler_hir::type_config::BuiltInTypeRef;
use crate::react_compiler_hir::type_config::TypeConfig;
use crate::react_compiler_hir::type_config::TypeReferenceConfig;
use crate::react_compiler_hir::type_config::ValueKind;
use crate::react_compiler_hir::type_config::ValueReason;

/// Type alias matching TS `Global = BuiltInType | PolyType`.
/// In the Rust port, both map to our `Type` enum.
pub type Global<'a> = Type<'a>;

/// Registry mapping global names to their types.
///
/// Supports two modes:
/// - **Builder mode** (`base=None`): wraps a single map, used during
///   `build_default_globals` to construct the static base.
/// - **Overlay mode** (`base=Some`): holds a `&'static` base map plus a small
///   extras map. Lookups check extras first, then base. Inserts go into extras.
///   Cloning only copies the extras map (the base pointer is shared).
pub struct GlobalRegistry<'a> {
    base: Option<&'static IdentHashMap<'static, Global<'static>>>,
    entries: IdentHashMap<'a, Global<'a>>,
}

impl<'a> GlobalRegistry<'a> {
    /// Create an empty builder-mode registry.
    pub fn new() -> Self {
        Self { base: None, entries: IdentHashMap::default() }
    }

    /// Create an overlay-mode registry backed by a static base.
    pub fn with_base(base: &'static IdentHashMap<'static, Global<'static>>) -> Self {
        Self { base: Some(base), entries: IdentHashMap::default() }
    }

    pub fn get(&self, key: &str) -> Option<&Global<'a>> {
        self.entries.get(key).or_else(|| self.base.and_then(|b| b.get(key).map(shrink_global)))
    }

    pub fn insert(&mut self, key: Ident<'a>, value: Global<'a>) {
        self.entries.insert(key, value);
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key) || self.base.is_some_and(|b| b.contains_key(key))
    }

    /// Iterate over all keys in the registry (base + extras).
    /// Keys in extras that shadow base keys appear only once.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        let base_keys = self
            .base
            .into_iter()
            .flat_map(|b| b.keys())
            .filter(|k| !self.entries.contains_key(k.as_str()))
            .map(Ident::as_str);
        self.entries.keys().map(Ident::as_str).chain(base_keys)
    }

    /// Consume the registry and return the inner map.
    /// Only valid in builder mode (no base).
    pub fn into_inner(self) -> IdentHashMap<'a, Global<'a>> {
        debug_assert!(self.base.is_none(), "into_inner() called on overlay-mode GlobalRegistry");
        self.entries
    }
}

/// Coerce a static global reference to the arena lifetime (covariant).
fn shrink_global<'a, 'b>(global: &'b Global<'static>) -> &'b Global<'a> {
    global
}

impl Default for GlobalRegistry<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for GlobalRegistry<'_> {
    fn clone(&self) -> Self {
        Self { base: self.base, entries: self.entries.clone() }
    }
}

// =============================================================================
// Static base registries (initialized once, shared across all Environments)
// =============================================================================

struct BaseRegistries {
    shapes: IdentHashMap<'static, ObjectShape<'static>>,
    globals: IdentHashMap<'static, Global<'static>>,
}

static BASE: LazyLock<BaseRegistries> = LazyLock::new(|| {
    let mut shapes = build_builtin_shapes();
    let globals = build_default_globals(&mut shapes);
    BaseRegistries { shapes: shapes.into_inner(), globals: globals.into_inner() }
});

/// Get a reference to the static base shapes registry.
pub fn base_shapes() -> &'static IdentHashMap<'static, ObjectShape<'static>> {
    &BASE.shapes
}

/// Get a reference to the static base globals registry.
pub fn base_globals() -> &'static IdentHashMap<'static, Global<'static>> {
    &BASE.globals
}

// =============================================================================
// installTypeConfig — converts TypeConfig to internal Type
// =============================================================================

/// Like `install_type_config` but collects validation errors.
pub fn install_type_config_with_errors<'a>(
    shapes: &mut ShapeRegistry<'a>,
    type_config: &TypeConfig,
    module_name: &str,
    errors: &mut Vec<String>,
) -> Global<'a> {
    install_type_config_inner(shapes, type_config, module_name, &mut Some(errors))
}

fn install_type_config_inner<'a>(
    shapes: &mut ShapeRegistry<'a>,
    type_config: &TypeConfig,
    module_name: &str,
    errors: &mut Option<&mut Vec<String>>,
) -> Global<'a> {
    match type_config {
        TypeConfig::TypeReference(TypeReferenceConfig { name }) => match name {
            BuiltInTypeRef::Array => Type::Object { shape_id: Some(BUILT_IN_ARRAY_ID) },
            BuiltInTypeRef::MixedReadonly => {
                Type::Object { shape_id: Some(BUILT_IN_MIXED_READONLY_ID) }
            }
            BuiltInTypeRef::Primitive => Type::Primitive,
            BuiltInTypeRef::Ref => Type::Object { shape_id: Some(BUILT_IN_USE_REF_ID) },
            BuiltInTypeRef::Any => Type::Poly,
        },
        TypeConfig::Function(func_config) => {
            // Compute return type first to avoid double-borrow of shapes
            let return_type =
                install_type_config_inner(shapes, &func_config.return_type, module_name, errors);
            add_function(
                shapes,
                Vec::new(),
                FunctionSignatureBuilder {
                    positional_params: func_config.positional_params.clone(),
                    rest_param: func_config.rest_param,
                    callee_effect: func_config.callee_effect,
                    return_type,
                    return_value_kind: func_config.return_value_kind,
                    no_alias: func_config.no_alias.unwrap_or(false),
                    mutable_only_if_operands_are_mutable: func_config
                        .mutable_only_if_operands_are_mutable
                        .unwrap_or(false),
                    impure: func_config.impure.unwrap_or(false),
                    canonical_name: func_config.canonical_name.clone().map(Into::into),
                    aliasing: func_config.aliasing.clone(),
                    known_incompatible: func_config.known_incompatible.clone().map(Into::into),
                    ..Default::default()
                },
                None,
                false,
            )
        }
        TypeConfig::Hook(hook_config) => {
            // Compute return type first to avoid double-borrow of shapes
            let return_type =
                install_type_config_inner(shapes, &hook_config.return_type, module_name, errors);
            add_hook(
                shapes,
                HookSignatureBuilder {
                    hook_kind: HookKind::Custom,
                    positional_params: hook_config.positional_params.clone().unwrap_or_default(),
                    rest_param: hook_config.rest_param.or(Some(Effect::Freeze)),
                    callee_effect: Effect::Read,
                    return_type,
                    return_value_kind: hook_config.return_value_kind.unwrap_or(ValueKind::Frozen),
                    no_alias: hook_config.no_alias.unwrap_or(false),
                    aliasing: hook_config.aliasing.clone(),
                    known_incompatible: hook_config.known_incompatible.clone().map(Into::into),
                    ..Default::default()
                },
                None,
            )
        }
        TypeConfig::Object(obj_config) => {
            let properties: Vec<(Ident<'a>, Type<'a>)> = obj_config
                .properties
                .as_ref()
                .map(|props| {
                    props
                        .iter()
                        .map(|(key, value)| {
                            let ty = install_type_config_inner(
                                shapes,
                                value,
                                module_name,
                                errors,
                            );
                            // Validate hook-name vs hook-type consistency (matching TS installTypeConfig)
                            if let Some(errs) = errors {
                                let expect_hook = is_hook_name(key);
                                let is_hook = match &ty {
                                    Type::Function { shape_id: Some(id), .. } => {
                                        shapes.get(id)
                                            .and_then(|shape| shape.function_type.as_ref())
                                            .and_then(|ft| ft.hook_kind.as_ref())
                                            .is_some()
                                    }
                                    _ => false,
                                };
                                if expect_hook != is_hook {
                                    errs.push(format!(
                                        "Expected type for object property '{}' from module '{}' {} based on the property name",
                                        key,
                                        module_name,
                                        if expect_hook { "to be a hook" } else { "not to be a hook" }
                                    ));
                                }
                            }
                            (shapes.alloc_ident(key), ty)
                        })
                        .collect()
                })
                .unwrap_or_default();
            add_object(shapes, None, properties)
        }
    }
}

// =============================================================================
// Build built-in shapes (BUILTIN_SHAPES from ObjectShape.ts)
// =============================================================================

/// Build the built-in shapes registry. This corresponds to TS `BUILTIN_SHAPES`
/// defined at module level in ObjectShape.ts.
pub fn build_builtin_shapes() -> ShapeRegistry<'static> {
    let mut shapes = ShapeRegistry::new();

    // BuiltInProps: { ref: UseRefType }
    add_object(
        &mut shapes,
        Some(BUILT_IN_PROPS_ID),
        vec![(Ident::from("ref"), Type::Object { shape_id: Some(BUILT_IN_USE_REF_ID) })],
    );

    build_array_shape(&mut shapes);
    build_set_shape(&mut shapes);
    build_map_shape(&mut shapes);
    build_weak_set_shape(&mut shapes);
    build_weak_map_shape(&mut shapes);
    build_object_shape(&mut shapes);
    build_ref_shapes(&mut shapes);
    build_state_shapes(&mut shapes);
    build_hook_shapes(&mut shapes);
    build_misc_shapes(&mut shapes);

    shapes
}

fn simple_function<'a>(
    shapes: &mut ShapeRegistry<'a>,
    positional_params: Vec<Effect>,
    rest_param: Option<Effect>,
    return_type: Type<'a>,
    return_value_kind: ValueKind,
) -> Type<'a> {
    add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params,
            rest_param,
            return_type,
            return_value_kind,
            ..Default::default()
        },
        None,
        false,
    )
}

/// Shorthand for a pure function returning Primitive.
fn pure_primitive_fn<'a>(shapes: &mut ShapeRegistry<'a>) -> Type<'a> {
    simple_function(shapes, Vec::new(), Some(Effect::Read), Type::Primitive, ValueKind::Primitive)
}

fn build_array_shape(shapes: &mut ShapeRegistry) {
    let index_of = pure_primitive_fn(shapes);
    let includes = pure_primitive_fn(shapes);
    let pop = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            callee_effect: Effect::Store,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let at = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Read],
            callee_effect: Effect::Capture,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let concat = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Capture),
            return_type: Type::Object { shape_id: Some(BUILT_IN_ARRAY_ID) },
            return_value_kind: ValueKind::Mutable,
            callee_effect: Effect::Capture,
            ..Default::default()
        },
        None,
        false,
    );
    let join = pure_primitive_fn(shapes);
    let slice = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Read),
            callee_effect: Effect::Capture,
            return_type: Type::Object { shape_id: Some(BUILT_IN_ARRAY_ID) },
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let map = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            callee_effect: Effect::ConditionallyMutate,
            return_type: Type::Object { shape_id: Some(BUILT_IN_ARRAY_ID) },
            return_value_kind: ValueKind::Mutable,
            no_alias: true,
            mutable_only_if_operands_are_mutable: true,
            aliasing: Some(AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: vec!["@callback".to_string()],
                rest: None,
                returns: "@returns".to_string(),
                temporaries: vec![
                    "@item".to_string(),
                    "@callbackReturn".to_string(),
                    "@thisArg".to_string(),
                ],
                effects: vec![
                    // Map creates a new mutable array
                    AliasingEffectConfig::Create {
                        into: "@returns".to_string(),
                        value: ValueKind::Mutable,
                        reason: ValueReason::KnownReturnSignature,
                    },
                    // The first arg to the callback is an item extracted from the receiver array
                    AliasingEffectConfig::CreateFrom {
                        from: "@receiver".to_string(),
                        into: "@item".to_string(),
                    },
                    // The undefined this for the callback
                    AliasingEffectConfig::Create {
                        into: "@thisArg".to_string(),
                        value: ValueKind::Primitive,
                        reason: ValueReason::KnownReturnSignature,
                    },
                    // Calls the callback, returning the result into a temporary
                    AliasingEffectConfig::Apply {
                        receiver: "@thisArg".to_string(),
                        function: "@callback".to_string(),
                        mutates_function: false,
                        args: vec![
                            ApplyArgConfig::Place("@item".to_string()),
                            ApplyArgConfig::Hole { kind: ApplyArgHoleKind::Hole },
                            ApplyArgConfig::Place("@receiver".to_string()),
                        ],
                        into: "@callbackReturn".to_string(),
                    },
                    // Captures the result of the callback into the return array
                    AliasingEffectConfig::Capture {
                        from: "@callbackReturn".to_string(),
                        into: "@returns".to_string(),
                    },
                ],
            }),
            ..Default::default()
        },
        None,
        false,
    );
    let filter = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            callee_effect: Effect::ConditionallyMutate,
            return_type: Type::Object { shape_id: Some(BUILT_IN_ARRAY_ID) },
            return_value_kind: ValueKind::Mutable,
            no_alias: true,
            mutable_only_if_operands_are_mutable: true,
            ..Default::default()
        },
        None,
        false,
    );
    let find = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            callee_effect: Effect::ConditionallyMutate,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            no_alias: true,
            mutable_only_if_operands_are_mutable: true,
            ..Default::default()
        },
        None,
        false,
    );
    let find_index = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            callee_effect: Effect::ConditionallyMutate,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            no_alias: true,
            mutable_only_if_operands_are_mutable: true,
            ..Default::default()
        },
        None,
        false,
    );
    let every = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            callee_effect: Effect::ConditionallyMutate,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            no_alias: true,
            mutable_only_if_operands_are_mutable: true,
            ..Default::default()
        },
        None,
        false,
    );
    let some = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            callee_effect: Effect::ConditionallyMutate,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            no_alias: true,
            mutable_only_if_operands_are_mutable: true,
            ..Default::default()
        },
        None,
        false,
    );
    let flat_map = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            callee_effect: Effect::ConditionallyMutate,
            return_type: Type::Object { shape_id: Some(BUILT_IN_ARRAY_ID) },
            return_value_kind: ValueKind::Mutable,
            no_alias: true,
            mutable_only_if_operands_are_mutable: true,
            ..Default::default()
        },
        None,
        false,
    );
    let length = Type::Primitive;
    let push = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Capture),
            callee_effect: Effect::Store,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            aliasing: Some(AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: Vec::new(),
                rest: Some("@rest".to_string()),
                returns: "@returns".to_string(),
                temporaries: Vec::new(),
                effects: vec![
                    // Push directly mutates the array itself
                    AliasingEffectConfig::Mutate { value: "@receiver".to_string() },
                    // The arguments are captured into the array
                    AliasingEffectConfig::Capture {
                        from: "@rest".to_string(),
                        into: "@receiver".to_string(),
                    },
                    // Returns the new length, a primitive
                    AliasingEffectConfig::Create {
                        into: "@returns".to_string(),
                        value: ValueKind::Primitive,
                        reason: ValueReason::KnownReturnSignature,
                    },
                ],
            }),
            ..Default::default()
        },
        None,
        false,
    );

    add_object(
        shapes,
        Some(BUILT_IN_ARRAY_ID),
        vec![
            (Ident::from("indexOf"), index_of),
            (Ident::from("includes"), includes),
            (Ident::from("pop"), pop),
            (Ident::from("at"), at),
            (Ident::from("concat"), concat),
            (Ident::from("length"), length),
            (Ident::from("push"), push),
            (Ident::from("slice"), slice),
            (Ident::from("map"), map),
            (Ident::from("flatMap"), flat_map),
            (Ident::from("filter"), filter),
            (Ident::from("every"), every),
            (Ident::from("some"), some),
            (Ident::from("find"), find),
            (Ident::from("findIndex"), find_index),
            (Ident::from("join"), join),
            // TODO: rest of Array properties
        ],
    );
}

fn build_set_shape(shapes: &mut ShapeRegistry) {
    let has = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Read],
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        None,
        false,
    );
    let add = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Capture],
            callee_effect: Effect::Store,
            return_type: Type::Object { shape_id: Some(BUILT_IN_SET_ID) },
            return_value_kind: ValueKind::Mutable,
            aliasing: Some(AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: Vec::new(),
                rest: Some("@rest".to_string()),
                returns: "@returns".to_string(),
                temporaries: Vec::new(),
                effects: vec![
                    // Set.add returns the receiver Set
                    AliasingEffectConfig::Assign {
                        from: "@receiver".to_string(),
                        into: "@returns".to_string(),
                    },
                    // Set.add mutates the set itself
                    AliasingEffectConfig::Mutate { value: "@receiver".to_string() },
                    // Captures the rest params into the set
                    AliasingEffectConfig::Capture {
                        from: "@rest".to_string(),
                        into: "@receiver".to_string(),
                    },
                ],
            }),
            ..Default::default()
        },
        None,
        false,
    );
    let clear = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            callee_effect: Effect::Store,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        None,
        false,
    );
    let delete = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Read],
            callee_effect: Effect::Store,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        None,
        false,
    );
    let size = Type::Primitive;
    let difference = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Capture],
            callee_effect: Effect::Capture,
            return_type: Type::Object { shape_id: Some(BUILT_IN_SET_ID) },
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let union = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Capture],
            callee_effect: Effect::Capture,
            return_type: Type::Object { shape_id: Some(BUILT_IN_SET_ID) },
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let symmetrical_difference = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Capture],
            callee_effect: Effect::Capture,
            return_type: Type::Object { shape_id: Some(BUILT_IN_SET_ID) },
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let is_subset_of = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Read],
            callee_effect: Effect::Read,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        None,
        false,
    );
    let is_superset_of = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Read],
            callee_effect: Effect::Read,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        None,
        false,
    );
    let for_each = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            callee_effect: Effect::ConditionallyMutate,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            no_alias: true,
            mutable_only_if_operands_are_mutable: true,
            ..Default::default()
        },
        None,
        false,
    );
    let values = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            callee_effect: Effect::Capture,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let keys = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            callee_effect: Effect::Capture,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let entries = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            callee_effect: Effect::Capture,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );

    add_object(
        shapes,
        Some(BUILT_IN_SET_ID),
        vec![
            (Ident::from("add"), add),
            (Ident::from("clear"), clear),
            (Ident::from("delete"), delete),
            (Ident::from("has"), has),
            (Ident::from("size"), size),
            (Ident::from("difference"), difference),
            (Ident::from("union"), union),
            (Ident::from("symmetricalDifference"), symmetrical_difference),
            (Ident::from("isSubsetOf"), is_subset_of),
            (Ident::from("isSupersetOf"), is_superset_of),
            (Ident::from("forEach"), for_each),
            (Ident::from("values"), values),
            (Ident::from("keys"), keys),
            (Ident::from("entries"), entries),
        ],
    );
}

fn build_map_shape(shapes: &mut ShapeRegistry) {
    let has = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Read],
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        None,
        false,
    );
    let get = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Read],
            callee_effect: Effect::Capture,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let clear = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            callee_effect: Effect::Store,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        None,
        false,
    );
    let set = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Capture, Effect::Capture],
            callee_effect: Effect::Store,
            return_type: Type::Object { shape_id: Some(BUILT_IN_MAP_ID) },
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let delete = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Read],
            callee_effect: Effect::Store,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        None,
        false,
    );
    let size = Type::Primitive;
    let for_each = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            callee_effect: Effect::ConditionallyMutate,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            no_alias: true,
            mutable_only_if_operands_are_mutable: true,
            ..Default::default()
        },
        None,
        false,
    );
    let values = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            callee_effect: Effect::Capture,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let keys = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            callee_effect: Effect::Capture,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let entries = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            callee_effect: Effect::Capture,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );

    add_object(
        shapes,
        Some(BUILT_IN_MAP_ID),
        vec![
            (Ident::from("has"), has),
            (Ident::from("get"), get),
            (Ident::from("set"), set),
            (Ident::from("clear"), clear),
            (Ident::from("delete"), delete),
            (Ident::from("size"), size),
            (Ident::from("forEach"), for_each),
            (Ident::from("values"), values),
            (Ident::from("keys"), keys),
            (Ident::from("entries"), entries),
        ],
    );
}

fn build_weak_set_shape(shapes: &mut ShapeRegistry) {
    let has = pure_primitive_fn(shapes);
    let add = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Capture],
            callee_effect: Effect::Store,
            return_type: Type::Object { shape_id: Some(BUILT_IN_WEAK_SET_ID) },
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let delete = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Read],
            callee_effect: Effect::Store,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        None,
        false,
    );

    add_object(
        shapes,
        Some(BUILT_IN_WEAK_SET_ID),
        vec![(Ident::from("has"), has), (Ident::from("add"), add), (Ident::from("delete"), delete)],
    );
}

fn build_weak_map_shape(shapes: &mut ShapeRegistry) {
    let has = pure_primitive_fn(shapes);
    let get = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Read],
            callee_effect: Effect::Capture,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let set = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Capture, Effect::Capture],
            callee_effect: Effect::Store,
            return_type: Type::Object { shape_id: Some(BUILT_IN_WEAK_MAP_ID) },
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let delete = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Read],
            callee_effect: Effect::Store,
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        None,
        false,
    );

    add_object(
        shapes,
        Some(BUILT_IN_WEAK_MAP_ID),
        vec![
            (Ident::from("has"), has),
            (Ident::from("get"), get),
            (Ident::from("set"), set),
            (Ident::from("delete"), delete),
        ],
    );
}

fn build_object_shape(shapes: &mut ShapeRegistry) {
    // BuiltInObject: has toString() returning Primitive (matches TS BuiltInObjectId shape)
    let to_string = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        None,
        false,
    );
    add_object(shapes, Some(BUILT_IN_OBJECT_ID), vec![(Ident::from("toString"), to_string)]);
    // BuiltInFunction: empty shape
    add_object(shapes, Some(BUILT_IN_FUNCTION_ID), Vec::new());
    // BuiltInJsx: empty shape
    add_object(shapes, Some(BUILT_IN_JSX_ID), Vec::new());
    // BuiltInMixedReadonly: has explicit method types + wildcard returning MixedReadonly
    // (matches TS BuiltInMixedReadonlyId shape)
    let mixed_to_string = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Read),
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        None,
        false,
    );
    let mixed_index_of = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Read),
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        None,
        false,
    );
    let mixed_includes = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Read),
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        None,
        false,
    );
    let mixed_at = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Read],
            return_type: Type::Object { shape_id: Some(BUILT_IN_MIXED_READONLY_ID) },
            callee_effect: Effect::Capture,
            return_value_kind: ValueKind::Frozen,
            ..Default::default()
        },
        None,
        false,
    );
    let mixed_map = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            return_type: Type::Object { shape_id: Some(BUILT_IN_ARRAY_ID) },
            callee_effect: Effect::ConditionallyMutate,
            return_value_kind: ValueKind::Mutable,
            no_alias: true,
            ..Default::default()
        },
        None,
        false,
    );
    let mixed_flat_map = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            return_type: Type::Object { shape_id: Some(BUILT_IN_ARRAY_ID) },
            callee_effect: Effect::ConditionallyMutate,
            return_value_kind: ValueKind::Mutable,
            no_alias: true,
            ..Default::default()
        },
        None,
        false,
    );
    let mixed_filter = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            return_type: Type::Object { shape_id: Some(BUILT_IN_ARRAY_ID) },
            callee_effect: Effect::ConditionallyMutate,
            return_value_kind: ValueKind::Mutable,
            no_alias: true,
            ..Default::default()
        },
        None,
        false,
    );
    let mixed_concat = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Capture),
            return_type: Type::Object { shape_id: Some(BUILT_IN_ARRAY_ID) },
            callee_effect: Effect::Capture,
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let mixed_slice = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Read),
            return_type: Type::Object { shape_id: Some(BUILT_IN_ARRAY_ID) },
            callee_effect: Effect::Capture,
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let mixed_every = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            return_type: Type::Primitive,
            callee_effect: Effect::ConditionallyMutate,
            return_value_kind: ValueKind::Primitive,
            no_alias: true,
            mutable_only_if_operands_are_mutable: true,
            ..Default::default()
        },
        None,
        false,
    );
    let mixed_some = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            return_type: Type::Primitive,
            callee_effect: Effect::ConditionallyMutate,
            return_value_kind: ValueKind::Primitive,
            no_alias: true,
            mutable_only_if_operands_are_mutable: true,
            ..Default::default()
        },
        None,
        false,
    );
    let mixed_find = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            return_type: Type::Object { shape_id: Some(BUILT_IN_MIXED_READONLY_ID) },
            callee_effect: Effect::ConditionallyMutate,
            return_value_kind: ValueKind::Frozen,
            no_alias: true,
            mutable_only_if_operands_are_mutable: true,
            ..Default::default()
        },
        None,
        false,
    );
    let mixed_find_index = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            return_type: Type::Primitive,
            callee_effect: Effect::ConditionallyMutate,
            return_value_kind: ValueKind::Primitive,
            no_alias: true,
            mutable_only_if_operands_are_mutable: true,
            ..Default::default()
        },
        None,
        false,
    );
    let mixed_join = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Read),
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        None,
        false,
    );
    let mut mixed_props = IdentHashMap::default();
    mixed_props.insert(Ident::from("toString"), mixed_to_string);
    mixed_props.insert(Ident::from("indexOf"), mixed_index_of);
    mixed_props.insert(Ident::from("includes"), mixed_includes);
    mixed_props.insert(Ident::from("at"), mixed_at);
    mixed_props.insert(Ident::from("map"), mixed_map);
    mixed_props.insert(Ident::from("flatMap"), mixed_flat_map);
    mixed_props.insert(Ident::from("filter"), mixed_filter);
    mixed_props.insert(Ident::from("concat"), mixed_concat);
    mixed_props.insert(Ident::from("slice"), mixed_slice);
    mixed_props.insert(Ident::from("every"), mixed_every);
    mixed_props.insert(Ident::from("some"), mixed_some);
    mixed_props.insert(Ident::from("find"), mixed_find);
    mixed_props.insert(Ident::from("findIndex"), mixed_find_index);
    mixed_props.insert(Ident::from("join"), mixed_join);
    mixed_props
        .insert(Ident::from("*"), Type::Object { shape_id: Some(BUILT_IN_MIXED_READONLY_ID) });
    shapes.insert(
        BUILT_IN_MIXED_READONLY_ID,
        ObjectShape { properties: mixed_props, function_type: None },
    );
}

fn build_ref_shapes(shapes: &mut ShapeRegistry) {
    // BuiltInUseRefId: { current: Object { shapeId: BuiltInRefValue } }
    add_object(
        shapes,
        Some(BUILT_IN_USE_REF_ID),
        vec![(Ident::from("current"), Type::Object { shape_id: Some(BUILT_IN_REF_VALUE_ID) })],
    );
    // BuiltInRefValue: { *: Object { shapeId: BuiltInRefValue } } (self-referencing)
    add_object(
        shapes,
        Some(BUILT_IN_REF_VALUE_ID),
        vec![(Ident::from("*"), Type::Object { shape_id: Some(BUILT_IN_REF_VALUE_ID) })],
    );
}

fn build_state_shapes(shapes: &mut ShapeRegistry) {
    // BuiltInSetState: function that freezes its argument
    let set_state = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        Some(BUILT_IN_SET_STATE_ID),
        false,
    );

    // BuiltInUseState: object with [0] = Poly (state), [1] = setState function
    add_object(
        shapes,
        Some(BUILT_IN_USE_STATE_ID),
        vec![(Ident::from("0"), Type::Poly), (Ident::from("1"), set_state)],
    );

    // BuiltInSetActionState
    let set_action_state = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        Some(BUILT_IN_SET_ACTION_STATE_ID),
        false,
    );

    // BuiltInUseActionState: [0] = Poly, [1] = setActionState function
    add_object(
        shapes,
        Some(BUILT_IN_USE_ACTION_STATE_ID),
        vec![(Ident::from("0"), Type::Poly), (Ident::from("1"), set_action_state)],
    );

    // BuiltInDispatch
    let dispatch = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        Some(BUILT_IN_DISPATCH_ID),
        false,
    );

    // BuiltInUseReducer: [0] = Poly, [1] = dispatch function
    add_object(
        shapes,
        Some(BUILT_IN_USE_REDUCER_ID),
        vec![(Ident::from("0"), Type::Poly), (Ident::from("1"), dispatch)],
    );

    // BuiltInStartTransition
    let start_transition = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            // Note: TS uses restParam: null for startTransition
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        Some(BUILT_IN_START_TRANSITION_ID),
        false,
    );

    // BuiltInUseTransition: [0] = Primitive (isPending), [1] = startTransition function
    add_object(
        shapes,
        Some(BUILT_IN_USE_TRANSITION_ID),
        vec![(Ident::from("0"), Type::Primitive), (Ident::from("1"), start_transition)],
    );

    // BuiltInSetOptimistic
    let set_optimistic = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        Some(BUILT_IN_SET_OPTIMISTIC_ID),
        false,
    );

    // BuiltInUseOptimistic: [0] = Poly, [1] = setOptimistic function
    add_object(
        shapes,
        Some(BUILT_IN_USE_OPTIMISTIC_ID),
        vec![(Ident::from("0"), Type::Poly), (Ident::from("1"), set_optimistic)],
    );
}

fn build_hook_shapes(shapes: &mut ShapeRegistry) {
    // BuiltInEffectEvent function shape (the return value of useEffectEvent)
    add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            callee_effect: Effect::ConditionallyMutate,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        Some(BUILT_IN_EFFECT_EVENT_ID),
        false,
    );
}

fn build_misc_shapes(shapes: &mut ShapeRegistry) {
    // ReanimatedSharedValue: empty properties (matching TS)
    add_object(shapes, Some(REANIMATED_SHARED_VALUE_ID), Vec::new());
}

/// Build the reanimated module type. Ported from TS `getReanimatedModuleType`.
pub fn get_reanimated_module_type<'a>(shapes: &mut ShapeRegistry<'a>) -> Type<'a> {
    let mut reanimated_type: Vec<(Ident, Type)> = Vec::new();

    // hooks that freeze args and return frozen value
    let frozen_hooks = [
        "useFrameCallback",
        "useAnimatedStyle",
        "useAnimatedProps",
        "useAnimatedScrollHandler",
        "useAnimatedReaction",
        "useWorkletCallback",
    ];
    for hook in &frozen_hooks {
        let hook_type = add_hook(
            shapes,
            HookSignatureBuilder {
                rest_param: Some(Effect::Freeze),
                return_type: Type::Poly,
                return_value_kind: ValueKind::Frozen,
                no_alias: true,
                hook_kind: HookKind::Custom,
                ..Default::default()
            },
            None,
        );
        reanimated_type.push((Ident::from(*hook), hook_type));
    }

    // hooks that return a mutable value (modelled as shared value)
    let mutable_hooks = ["useSharedValue", "useDerivedValue"];
    for hook in &mutable_hooks {
        let hook_type = add_hook(
            shapes,
            HookSignatureBuilder {
                rest_param: Some(Effect::Freeze),
                return_type: Type::Object { shape_id: Some(REANIMATED_SHARED_VALUE_ID) },
                return_value_kind: ValueKind::Mutable,
                no_alias: true,
                hook_kind: HookKind::Custom,
                ..Default::default()
            },
            None,
        );
        reanimated_type.push((Ident::from(*hook), hook_type));
    }

    // functions that return mutable value
    let funcs = [
        "withTiming",
        "withSpring",
        "createAnimatedPropAdapter",
        "withDecay",
        "withRepeat",
        "runOnUI",
        "executeOnUIRuntimeSync",
    ];
    for func_name in &funcs {
        let func_type = add_function(
            shapes,
            Vec::new(),
            FunctionSignatureBuilder {
                rest_param: Some(Effect::Read),
                return_type: Type::Poly,
                return_value_kind: ValueKind::Mutable,
                no_alias: true,
                ..Default::default()
            },
            None,
            false,
        );
        reanimated_type.push((Ident::from(*func_name), func_type));
    }

    add_object(shapes, None, reanimated_type)
}

// =============================================================================
// Build default globals (DEFAULT_GLOBALS from Globals.ts)
// =============================================================================

/// Build the default globals registry. This corresponds to TS `DEFAULT_GLOBALS`.
///
/// Requires a mutable reference to the shapes registry because some globals
/// (like Object.keys, Array.isArray) register new shapes.
pub fn build_default_globals(shapes: &mut ShapeRegistry<'static>) -> GlobalRegistry<'static> {
    let mut globals = GlobalRegistry::new();

    // React APIs — returns the list so we can reuse them for the React namespace
    let react_apis = build_react_apis(shapes, &mut globals);

    // Untyped globals (treated as Poly) — must come before typed globals
    // so typed definitions take priority (matching TS ordering)
    for name in UNTYPED_GLOBALS {
        globals.insert(Ident::from(*name), Type::Poly);
    }

    // Typed JS globals (overwrites Poly entries from UNTYPED_GLOBALS).
    // Returns the list of typed globals for use as globalThis/global properties.
    let typed_globals = build_typed_globals(shapes, &mut globals, react_apis);

    // globalThis and global — populated with all typed globals as properties
    // (matching TS: `addObject(DEFAULT_SHAPES, 'globalThis', TYPED_GLOBALS)`)
    globals.insert(
        Ident::from("globalThis"),
        add_object(shapes, Some(Ident::from("globalThis")), typed_globals.clone()),
    );
    globals.insert(
        Ident::from("global"),
        add_object(shapes, Some(Ident::from("global")), typed_globals),
    );

    globals
}

const UNTYPED_GLOBALS: &[&str] = &[
    "Object",
    "Function",
    "RegExp",
    "Date",
    "Error",
    "TypeError",
    "RangeError",
    "ReferenceError",
    "SyntaxError",
    "URIError",
    "EvalError",
    "DataView",
    "Float32Array",
    "Float64Array",
    "Int8Array",
    "Int16Array",
    "Int32Array",
    "WeakMap",
    "Uint8Array",
    "Uint8ClampedArray",
    "Uint16Array",
    "Uint32Array",
    "ArrayBuffer",
    "JSON",
    "console",
    "eval",
];

/// Build the React API types (REACT_APIS from TS). Returns the list of (name, type) pairs
/// so they can be reused as properties of the React namespace object (matching TS behavior
/// where the SAME type objects are used in both DEFAULT_GLOBALS and the React namespace).
fn build_react_apis<'a>(
    shapes: &mut ShapeRegistry<'a>,
    globals: &mut GlobalRegistry<'a>,
) -> Vec<(Ident<'a>, Type<'a>)> {
    let mut react_apis: Vec<(Ident, Type)> = Vec::new();

    // useContext
    let use_context = add_hook(
        shapes,
        HookSignatureBuilder {
            rest_param: Some(Effect::Read),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            return_value_reason: Some(ValueReason::Context),
            hook_kind: HookKind::UseContext,
            ..Default::default()
        },
        Some(BUILT_IN_USE_CONTEXT_HOOK_ID),
    );
    react_apis.push((Ident::from("useContext"), use_context));

    // useState
    let use_state = add_hook(
        shapes,
        HookSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Object { shape_id: Some(BUILT_IN_USE_STATE_ID) },
            return_value_kind: ValueKind::Frozen,
            return_value_reason: Some(ValueReason::State),
            hook_kind: HookKind::UseState,
            ..Default::default()
        },
        None,
    );
    react_apis.push((Ident::from("useState"), use_state));

    // useActionState
    let use_action_state = add_hook(
        shapes,
        HookSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Object { shape_id: Some(BUILT_IN_USE_ACTION_STATE_ID) },
            return_value_kind: ValueKind::Frozen,
            return_value_reason: Some(ValueReason::State),
            hook_kind: HookKind::UseActionState,
            ..Default::default()
        },
        None,
    );
    react_apis.push((Ident::from("useActionState"), use_action_state));

    // useReducer
    let use_reducer = add_hook(
        shapes,
        HookSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Object { shape_id: Some(BUILT_IN_USE_REDUCER_ID) },
            return_value_kind: ValueKind::Frozen,
            return_value_reason: Some(ValueReason::ReducerState),
            hook_kind: HookKind::UseReducer,
            ..Default::default()
        },
        None,
    );
    react_apis.push((Ident::from("useReducer"), use_reducer));

    // useRef
    let use_ref = add_hook(
        shapes,
        HookSignatureBuilder {
            rest_param: Some(Effect::Capture),
            return_type: Type::Object { shape_id: Some(BUILT_IN_USE_REF_ID) },
            return_value_kind: ValueKind::Mutable,
            hook_kind: HookKind::UseRef,
            ..Default::default()
        },
        None,
    );
    react_apis.push((Ident::from("useRef"), use_ref));

    // useImperativeHandle
    let use_imperative_handle = add_hook(
        shapes,
        HookSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Frozen,
            hook_kind: HookKind::UseImperativeHandle,
            ..Default::default()
        },
        None,
    );
    react_apis.push((Ident::from("useImperativeHandle"), use_imperative_handle));

    // useMemo
    let use_memo = add_hook(
        shapes,
        HookSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            hook_kind: HookKind::UseMemo,
            ..Default::default()
        },
        None,
    );
    react_apis.push((Ident::from("useMemo"), use_memo));

    // useCallback
    let use_callback = add_hook(
        shapes,
        HookSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            hook_kind: HookKind::UseCallback,
            ..Default::default()
        },
        None,
    );
    react_apis.push((Ident::from("useCallback"), use_callback));

    // useEffect (with aliasing signature)
    let use_effect = add_hook(
        shapes,
        HookSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Frozen,
            hook_kind: HookKind::UseEffect,
            aliasing: Some(AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: Vec::new(),
                rest: Some("@rest".to_string()),
                returns: "@returns".to_string(),
                temporaries: vec!["@effect".to_string()],
                effects: vec![
                    AliasingEffectConfig::Freeze {
                        value: "@rest".to_string(),
                        reason: ValueReason::Effect,
                    },
                    AliasingEffectConfig::Create {
                        into: "@effect".to_string(),
                        value: ValueKind::Frozen,
                        reason: ValueReason::KnownReturnSignature,
                    },
                    AliasingEffectConfig::Capture {
                        from: "@rest".to_string(),
                        into: "@effect".to_string(),
                    },
                    AliasingEffectConfig::Create {
                        into: "@returns".to_string(),
                        value: ValueKind::Primitive,
                        reason: ValueReason::KnownReturnSignature,
                    },
                ],
            }),
            ..Default::default()
        },
        Some(BUILT_IN_USE_EFFECT_HOOK_ID),
    );
    react_apis.push((Ident::from("useEffect"), use_effect));

    // useLayoutEffect
    let use_layout_effect = add_hook(
        shapes,
        HookSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            hook_kind: HookKind::UseLayoutEffect,
            ..Default::default()
        },
        Some(BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID),
    );
    react_apis.push((Ident::from("useLayoutEffect"), use_layout_effect));

    // useInsertionEffect
    let use_insertion_effect = add_hook(
        shapes,
        HookSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            hook_kind: HookKind::UseInsertionEffect,
            ..Default::default()
        },
        Some(BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID),
    );
    react_apis.push((Ident::from("useInsertionEffect"), use_insertion_effect));

    // useTransition
    let use_transition = add_hook(
        shapes,
        HookSignatureBuilder {
            rest_param: None,
            return_type: Type::Object { shape_id: Some(BUILT_IN_USE_TRANSITION_ID) },
            return_value_kind: ValueKind::Frozen,
            hook_kind: HookKind::UseTransition,
            ..Default::default()
        },
        None,
    );
    react_apis.push((Ident::from("useTransition"), use_transition));

    // useOptimistic
    let use_optimistic = add_hook(
        shapes,
        HookSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Object { shape_id: Some(BUILT_IN_USE_OPTIMISTIC_ID) },
            return_value_kind: ValueKind::Frozen,
            return_value_reason: Some(ValueReason::State),
            hook_kind: HookKind::UseOptimistic,
            ..Default::default()
        },
        None,
    );
    react_apis.push((Ident::from("useOptimistic"), use_optimistic));

    // use (not a hook, it's a function)
    let use_fn = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            ..Default::default()
        },
        Some(BUILT_IN_USE_OPERATOR_ID),
        false,
    );
    react_apis.push((Ident::from("use"), use_fn));

    // useEffectEvent
    let use_effect_event = add_hook(
        shapes,
        HookSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Function {
                shape_id: Some(BUILT_IN_EFFECT_EVENT_ID),
                return_type: Box::new(Type::Poly),
                is_constructor: false,
            },
            return_value_kind: ValueKind::Frozen,
            hook_kind: HookKind::UseEffectEvent,
            ..Default::default()
        },
        Some(BUILT_IN_USE_EFFECT_EVENT_ID),
    );
    react_apis.push((Ident::from("useEffectEvent"), use_effect_event));

    // Insert all React APIs as standalone globals
    for (name, ty) in &react_apis {
        globals.insert(*name, ty.clone());
    }

    react_apis
}

/// Build typed globals and return them as a list for use as globalThis/global properties.
fn build_typed_globals<'a>(
    shapes: &mut ShapeRegistry<'a>,
    globals: &mut GlobalRegistry<'a>,
    react_apis: Vec<(Ident<'a>, Type<'a>)>,
) -> Vec<(Ident<'a>, Type<'a>)> {
    let mut typed_globals: Vec<(Ident, Type)> = Vec::new();
    // Object
    let obj_keys = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Read],
            return_type: Type::Object { shape_id: Some(BUILT_IN_ARRAY_ID) },
            return_value_kind: ValueKind::Mutable,
            aliasing: Some(AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: vec!["@object".to_string()],
                rest: None,
                returns: "@returns".to_string(),
                temporaries: Vec::new(),
                effects: vec![
                    AliasingEffectConfig::Create {
                        into: "@returns".to_string(),
                        value: ValueKind::Mutable,
                        reason: ValueReason::KnownReturnSignature,
                    },
                    // Only keys are captured, and keys are immutable
                    AliasingEffectConfig::ImmutableCapture {
                        from: "@object".to_string(),
                        into: "@returns".to_string(),
                    },
                ],
            }),
            ..Default::default()
        },
        None,
        false,
    );
    let obj_from_entries = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::ConditionallyMutate],
            return_type: Type::Object { shape_id: Some(BUILT_IN_OBJECT_ID) },
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let obj_entries = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Capture],
            return_type: Type::Object { shape_id: Some(BUILT_IN_ARRAY_ID) },
            return_value_kind: ValueKind::Mutable,
            aliasing: Some(AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: vec!["@object".to_string()],
                rest: None,
                returns: "@returns".to_string(),
                temporaries: Vec::new(),
                effects: vec![
                    AliasingEffectConfig::Create {
                        into: "@returns".to_string(),
                        value: ValueKind::Mutable,
                        reason: ValueReason::KnownReturnSignature,
                    },
                    // Object values are captured into the return
                    AliasingEffectConfig::Capture {
                        from: "@object".to_string(),
                        into: "@returns".to_string(),
                    },
                ],
            }),
            ..Default::default()
        },
        None,
        false,
    );
    let obj_values = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Capture],
            return_type: Type::Object { shape_id: Some(BUILT_IN_ARRAY_ID) },
            return_value_kind: ValueKind::Mutable,
            aliasing: Some(AliasingSignatureConfig {
                receiver: "@receiver".to_string(),
                params: vec!["@object".to_string()],
                rest: None,
                returns: "@returns".to_string(),
                temporaries: Vec::new(),
                effects: vec![
                    AliasingEffectConfig::Create {
                        into: "@returns".to_string(),
                        value: ValueKind::Mutable,
                        reason: ValueReason::KnownReturnSignature,
                    },
                    // Object values are captured into the return
                    AliasingEffectConfig::Capture {
                        from: "@object".to_string(),
                        into: "@returns".to_string(),
                    },
                ],
            }),
            ..Default::default()
        },
        None,
        false,
    );
    let object_global = add_object(
        shapes,
        Some(Ident::from("Object")),
        vec![
            (Ident::from("keys"), obj_keys),
            (Ident::from("fromEntries"), obj_from_entries),
            (Ident::from("entries"), obj_entries),
            (Ident::from("values"), obj_values),
        ],
    );
    typed_globals.push((Ident::from("Object"), object_global.clone()));
    globals.insert(Ident::from("Object"), object_global);

    // Array
    let array_is_array = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::Read],
            return_type: Type::Primitive,
            return_value_kind: ValueKind::Primitive,
            ..Default::default()
        },
        None,
        false,
    );
    let array_from = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![
                Effect::ConditionallyMutateIterator,
                Effect::ConditionallyMutate,
                Effect::ConditionallyMutate,
            ],
            rest_param: Some(Effect::Read),
            return_type: Type::Object { shape_id: Some(BUILT_IN_ARRAY_ID) },
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let array_of = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Read),
            return_type: Type::Object { shape_id: Some(BUILT_IN_ARRAY_ID) },
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );
    let array_global = add_object(
        shapes,
        Some(Ident::from("Array")),
        vec![
            (Ident::from("isArray"), array_is_array),
            (Ident::from("from"), array_from),
            (Ident::from("of"), array_of),
        ],
    );
    typed_globals.push((Ident::from("Array"), array_global.clone()));
    globals.insert(Ident::from("Array"), array_global);

    // Math
    let math_fns: Vec<(Ident, Type)> = ["max", "min", "trunc", "ceil", "floor", "pow"]
        .iter()
        .map(|name| (Ident::from(*name), pure_primitive_fn(shapes)))
        .collect();
    let mut math_props = math_fns;
    math_props.push((Ident::from("PI"), Type::Primitive));
    // Math.random is impure
    let math_random = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            impure: true,
            canonical_name: Some("Math.random".into()),
            ..Default::default()
        },
        None,
        false,
    );
    math_props.push((Ident::from("random"), math_random));
    let math_global = add_object(shapes, Some(Ident::from("Math")), math_props);
    typed_globals.push((Ident::from("Math"), math_global.clone()));
    globals.insert(Ident::from("Math"), math_global);

    // performance
    let perf_now = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Read),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            impure: true,
            canonical_name: Some("performance.now".into()),
            ..Default::default()
        },
        None,
        false,
    );
    let perf_global =
        add_object(shapes, Some(Ident::from("performance")), vec![(Ident::from("now"), perf_now)]);
    typed_globals.push((Ident::from("performance"), perf_global.clone()));
    globals.insert(Ident::from("performance"), perf_global);

    // Date
    let date_now = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Read),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            impure: true,
            canonical_name: Some("Date.now".into()),
            ..Default::default()
        },
        None,
        false,
    );
    let date_global =
        add_object(shapes, Some(Ident::from("Date")), vec![(Ident::from("now"), date_now)]);
    typed_globals.push((Ident::from("Date"), date_global.clone()));
    globals.insert(Ident::from("Date"), date_global);

    // console
    let console_methods: Vec<(Ident, Type)> = ["error", "info", "log", "table", "trace", "warn"]
        .iter()
        .map(|name| (Ident::from(*name), pure_primitive_fn(shapes)))
        .collect();
    let console_global = add_object(shapes, Some(Ident::from("console")), console_methods);
    typed_globals.push((Ident::from("console"), console_global.clone()));
    globals.insert(Ident::from("console"), console_global);

    // Simple global functions returning Primitive
    for name in &[
        "Boolean",
        "Number",
        "String",
        "parseInt",
        "parseFloat",
        "isNaN",
        "isFinite",
        "encodeURI",
        "encodeURIComponent",
        "decodeURI",
        "decodeURIComponent",
    ] {
        let f = pure_primitive_fn(shapes);
        typed_globals.push((Ident::from(*name), f.clone()));
        globals.insert(Ident::from(*name), f);
    }

    // Primitive globals
    typed_globals.push((Ident::from("Infinity"), Type::Primitive));
    globals.insert(Ident::from("Infinity"), Type::Primitive);
    typed_globals.push((Ident::from("NaN"), Type::Primitive));
    globals.insert(Ident::from("NaN"), Type::Primitive);

    // Map, Set, WeakMap, WeakSet constructors
    let map_ctor = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::ConditionallyMutateIterator],
            return_type: Type::Object { shape_id: Some(BUILT_IN_MAP_ID) },
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        true,
    );
    typed_globals.push((Ident::from("Map"), map_ctor.clone()));
    globals.insert(Ident::from("Map"), map_ctor);

    let set_ctor = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::ConditionallyMutateIterator],
            return_type: Type::Object { shape_id: Some(BUILT_IN_SET_ID) },
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        true,
    );
    typed_globals.push((Ident::from("Set"), set_ctor.clone()));
    globals.insert(Ident::from("Set"), set_ctor);

    let weak_map_ctor = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::ConditionallyMutateIterator],
            return_type: Type::Object { shape_id: Some(BUILT_IN_WEAK_MAP_ID) },
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        true,
    );
    typed_globals.push((Ident::from("WeakMap"), weak_map_ctor.clone()));
    globals.insert(Ident::from("WeakMap"), weak_map_ctor);

    let weak_set_ctor = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: vec![Effect::ConditionallyMutateIterator],
            return_type: Type::Object { shape_id: Some(BUILT_IN_WEAK_SET_ID) },
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        true,
    );
    typed_globals.push((Ident::from("WeakSet"), weak_set_ctor.clone()));
    globals.insert(Ident::from("WeakSet"), weak_set_ctor);

    // React global object — reuses the same REACT_APIS types (matching TS behavior
    // where the same type objects are used as both standalone globals and React.* properties)
    let react_create_element = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            ..Default::default()
        },
        None,
        false,
    );
    let react_clone_element = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            ..Default::default()
        },
        None,
        false,
    );
    let react_create_ref = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Capture),
            return_type: Type::Object { shape_id: Some(BUILT_IN_USE_REF_ID) },
            return_value_kind: ValueKind::Mutable,
            ..Default::default()
        },
        None,
        false,
    );

    // Build React namespace properties from react_apis + React-specific functions
    let mut react_props: Vec<(Ident, Type)> = react_apis;
    react_props.push((Ident::from("createElement"), react_create_element));
    react_props.push((Ident::from("cloneElement"), react_clone_element));
    react_props.push((Ident::from("createRef"), react_create_ref));

    let react_global = add_object(shapes, None, react_props);
    typed_globals.push((Ident::from("React"), react_global.clone()));
    globals.insert(Ident::from("React"), react_global);

    // _jsx (used by JSX transform)
    let jsx_fn = add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            ..Default::default()
        },
        None,
        false,
    );
    typed_globals.push((Ident::from("_jsx"), jsx_fn.clone()));
    globals.insert(Ident::from("_jsx"), jsx_fn);

    typed_globals
}
