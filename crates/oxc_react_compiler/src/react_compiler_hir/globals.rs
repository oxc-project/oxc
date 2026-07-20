// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Global type registry and built-in shape definitions, ported from Globals.ts.
//!
//! Provides `DEFAULT_SHAPES` (built-in object shapes) and `DEFAULT_GLOBALS`
//! (global variable types including React hooks and JS built-ins).

use std::borrow::Cow;
use std::sync::LazyLock;

use oxc_str::{Ident, IdentHashMap};

use crate::react_compiler_hir::Effect;
use crate::react_compiler_hir::Type;
use crate::react_compiler_hir::environment::is_hook_name;
use crate::react_compiler_hir::object_shape::*;
use crate::react_compiler_hir::type_config::AliasingEffectConfig;
use crate::react_compiler_hir::type_config::AliasingSignatureConfig;
use crate::react_compiler_hir::type_config::ApplyArgConfig;
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
        self.entries.get(key).or_else(|| self.base.and_then(|b| b.get(key)))
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
    fn into_inner(self) -> IdentHashMap<'a, Global<'a>> {
        debug_assert!(self.base.is_none(), "into_inner() called on overlay-mode GlobalRegistry");
        self.entries
    }
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

/// Convert a `TypeConfig` into an internal `Type`, collecting validation errors.
/// Ported from TS `installTypeConfig`.
pub fn install_type_config<'a>(
    shapes: &mut ShapeRegistry<'a>,
    type_config: &TypeConfig,
    module_name: &str,
    errors: &mut Vec<String>,
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
                install_type_config(shapes, &func_config.return_type, module_name, errors);
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
                    aliasing: func_config.aliasing,
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
                install_type_config(shapes, &hook_config.return_type, module_name, errors);
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
                    aliasing: hook_config.aliasing,
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
                            let ty = install_type_config(shapes, value, module_name, errors);
                            // Validate hook-name vs hook-type consistency (matching TS installTypeConfig)
                            let expect_hook = is_hook_name(key);
                            let is_hook = match &ty {
                                Type::Function { shape_id: Some(id), .. } => shapes
                                    .get(id)
                                    .and_then(|shape| shape.function_type.as_ref())
                                    .and_then(|ft| ft.hook_kind.as_ref())
                                    .is_some(),
                                _ => false,
                            };
                            if expect_hook != is_hook {
                                errors.push(format!(
                                    "Expected type for object property '{}' from module '{}' {} based on the property name",
                                    key,
                                    module_name,
                                    if expect_hook { "to be a hook" } else { "not to be a hook" }
                                ));
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
// Const descriptor tables for the static base registries
// =============================================================================
//
// `build_builtin_shapes` / `build_default_globals` run exactly once per process
// (inside the `BASE` LazyLock), so the repetitive `addFunction` / `addObject`
// registrations ported from ObjectShape.ts / Globals.ts are encoded as const
// descriptor tables interpreted by small loops rather than as straight-line
// construction code. This keeps the cold one-time init code small. Table
// entries mirror the upstream declaration order so future re-syncs can diff
// them against Globals.ts / ObjectShape.ts.

/// Compact const form of the `Type`s that appear in built-in signatures and
/// properties: mirrors the TS `PrimitiveType` / `POLY_TYPE` /
/// `{kind: 'Object', shapeId}` literals. Kept minimal (24 bytes vs `Type`'s
/// 56) so the descriptor tables below stay small.
#[derive(Clone, Copy)]
enum TypeDef {
    Primitive,
    Poly,
    Object(Ident<'static>),
}

impl TypeDef {
    fn as_type(self) -> Type<'static> {
        match self {
            TypeDef::Primitive => Type::Primitive,
            TypeDef::Poly => Type::Poly,
            TypeDef::Object(shape_id) => Type::Object { shape_id: Some(shape_id) },
        }
    }
}

/// Const-constructible function signature, mirroring the `addFunction` config
/// objects in ObjectShape.ts / Globals.ts. Interpreted by [`add_method`].
struct MethodDef {
    positional_params: &'static [Effect],
    rest_param: Option<Effect>,
    return_type: TypeDef,
    callee_effect: Effect,
    return_value_kind: ValueKind,
    no_alias: bool,
    mutable_only_if_operands_are_mutable: bool,
    impure: bool,
    canonical_name: Option<&'static str>,
    aliasing: Option<&'static AliasingSignatureConfig>,
}

impl MethodDef {
    /// Field defaults, matching [`FunctionSignatureBuilder::default`].
    const DEFAULT: Self = Self {
        positional_params: &[],
        rest_param: None,
        return_type: TypeDef::Poly,
        callee_effect: Effect::Read,
        return_value_kind: ValueKind::Mutable,
        no_alias: false,
        mutable_only_if_operands_are_mutable: false,
        impure: false,
        canonical_name: None,
        aliasing: None,
    };
}

/// Shorthand for a pure function reading its arguments and returning Primitive.
const PURE_PRIMITIVE_FN: MethodDef = MethodDef {
    rest_param: Some(Effect::Read),
    return_type: TypeDef::Primitive,
    return_value_kind: ValueKind::Primitive,
    ..MethodDef::DEFAULT
};

/// Shorthand for a function freezing its arguments and returning a frozen value.
const FREEZE_ARGS_FN: MethodDef = MethodDef {
    rest_param: Some(Effect::Freeze),
    return_type: TypeDef::Poly,
    return_value_kind: ValueKind::Frozen,
    ..MethodDef::DEFAULT
};

/// One property of an object shape: a method with a function signature, or a
/// plain property like `length: Primitive`.
enum PropDef {
    Method(&'static str, MethodDef),
    Value(&'static str, TypeDef),
}

use PropDef::{Method, Value};

/// A built-in object shape: shape id plus property table.
struct ShapeDef {
    id: Ident<'static>,
    props: &'static [PropDef],
}

/// Register the function shape described by a [`MethodDef`].
#[cold]
#[inline(never)]
fn add_method<'a>(
    shapes: &mut ShapeRegistry<'a>,
    def: &MethodDef,
    id: Option<Ident<'a>>,
    is_constructor: bool,
) -> Type<'a> {
    add_function(
        shapes,
        Vec::new(),
        FunctionSignatureBuilder {
            positional_params: def.positional_params.to_vec(),
            rest_param: def.rest_param,
            return_type: def.return_type.as_type(),
            callee_effect: def.callee_effect,
            return_value_kind: def.return_value_kind,
            no_alias: def.no_alias,
            mutable_only_if_operands_are_mutable: def.mutable_only_if_operands_are_mutable,
            impure: def.impure,
            canonical_name: def.canonical_name.map(Cow::Borrowed),
            aliasing: def.aliasing.copied(),
            ..Default::default()
        },
        id,
        is_constructor,
    )
}

/// Register an object shape from a property table, registering the function
/// shape of every [`PropDef::Method`] entry along the way.
#[cold]
#[inline(never)]
fn add_object_from_def<'a>(
    shapes: &mut ShapeRegistry<'a>,
    id: Option<Ident<'a>>,
    props: &'static [PropDef],
) -> Type<'a> {
    let properties = props
        .iter()
        .map(|prop| match prop {
            Method(name, def) => (Ident::from(*name), add_method(shapes, def, None, false)),
            Value(name, ty) => (Ident::from(*name), ty.as_type()),
        })
        .collect();
    add_object(shapes, id, properties)
}

// =============================================================================
// Build built-in shapes (BUILTIN_SHAPES from ObjectShape.ts)
// =============================================================================

/// Build the built-in shapes registry. This corresponds to TS `BUILTIN_SHAPES`
/// defined at module level in ObjectShape.ts.
#[cold]
#[inline(never)]
fn build_builtin_shapes() -> ShapeRegistry<'static> {
    let mut shapes = ShapeRegistry::new();
    for def in BUILTIN_SHAPE_DEFS {
        add_object_from_def(&mut shapes, Some(def.id), def.props);
    }
    build_state_shapes(&mut shapes);
    build_hook_shapes(&mut shapes);
    build_misc_shapes(&mut shapes);
    shapes
}

/// The built-in object shapes, in upstream ObjectShape.ts declaration order.
const BUILTIN_SHAPE_DEFS: &[ShapeDef] = &[
    // BuiltInProps: { ref: UseRefType }
    ShapeDef {
        id: BUILT_IN_PROPS_ID,
        props: &[Value("ref", TypeDef::Object(BUILT_IN_USE_REF_ID))],
    },
    // BuiltInArray
    ShapeDef {
        id: BUILT_IN_ARRAY_ID,
        props: &[
            Method("indexOf", PURE_PRIMITIVE_FN),
            Method("includes", PURE_PRIMITIVE_FN),
            Method(
                "pop",
                MethodDef {
                    callee_effect: Effect::Store,
                    return_type: TypeDef::Poly,
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "at",
                MethodDef {
                    positional_params: &[Effect::Read],
                    callee_effect: Effect::Capture,
                    return_type: TypeDef::Poly,
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "concat",
                MethodDef {
                    rest_param: Some(Effect::Capture),
                    return_type: TypeDef::Object(BUILT_IN_ARRAY_ID),
                    return_value_kind: ValueKind::Mutable,
                    callee_effect: Effect::Capture,
                    ..MethodDef::DEFAULT
                },
            ),
            Value("length", TypeDef::Primitive),
            Method(
                "push",
                MethodDef {
                    rest_param: Some(Effect::Capture),
                    callee_effect: Effect::Store,
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    aliasing: Some(&AliasingSignatureConfig {
                        receiver: "@receiver",
                        params: &[],
                        rest: Some("@rest"),
                        returns: "@returns",
                        temporaries: &[],
                        effects: &[
                            // Push directly mutates the array itself
                            AliasingEffectConfig::Mutate { value: "@receiver" },
                            // The arguments are captured into the array
                            AliasingEffectConfig::Capture { from: "@rest", into: "@receiver" },
                            // Returns the new length, a primitive
                            AliasingEffectConfig::Create {
                                into: "@returns",
                                value: ValueKind::Primitive,
                                reason: ValueReason::KnownReturnSignature,
                            },
                        ],
                    }),
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "slice",
                MethodDef {
                    rest_param: Some(Effect::Read),
                    callee_effect: Effect::Capture,
                    return_type: TypeDef::Object(BUILT_IN_ARRAY_ID),
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "map",
                MethodDef {
                    rest_param: Some(Effect::ConditionallyMutate),
                    callee_effect: Effect::ConditionallyMutate,
                    return_type: TypeDef::Object(BUILT_IN_ARRAY_ID),
                    return_value_kind: ValueKind::Mutable,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    aliasing: Some(&AliasingSignatureConfig {
                        receiver: "@receiver",
                        params: &["@callback"],
                        rest: None,
                        returns: "@returns",
                        temporaries: &["@item", "@callbackReturn", "@thisArg"],
                        effects: &[
                            // Map creates a new mutable array
                            AliasingEffectConfig::Create {
                                into: "@returns",
                                value: ValueKind::Mutable,
                                reason: ValueReason::KnownReturnSignature,
                            },
                            // The first arg to the callback is an item extracted from the receiver array
                            AliasingEffectConfig::CreateFrom { from: "@receiver", into: "@item" },
                            // The undefined this for the callback
                            AliasingEffectConfig::Create {
                                into: "@thisArg",
                                value: ValueKind::Primitive,
                                reason: ValueReason::KnownReturnSignature,
                            },
                            // Calls the callback, returning the result into a temporary
                            AliasingEffectConfig::Apply {
                                receiver: "@thisArg",
                                function: "@callback",
                                mutates_function: false,
                                args: &[
                                    ApplyArgConfig::Place("@item"),
                                    ApplyArgConfig::Hole,
                                    ApplyArgConfig::Place("@receiver"),
                                ],
                                into: "@callbackReturn",
                            },
                            // Captures the result of the callback into the return array
                            AliasingEffectConfig::Capture {
                                from: "@callbackReturn",
                                into: "@returns",
                            },
                        ],
                    }),
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "flatMap",
                MethodDef {
                    rest_param: Some(Effect::ConditionallyMutate),
                    callee_effect: Effect::ConditionallyMutate,
                    return_type: TypeDef::Object(BUILT_IN_ARRAY_ID),
                    return_value_kind: ValueKind::Mutable,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "filter",
                MethodDef {
                    rest_param: Some(Effect::ConditionallyMutate),
                    callee_effect: Effect::ConditionallyMutate,
                    return_type: TypeDef::Object(BUILT_IN_ARRAY_ID),
                    return_value_kind: ValueKind::Mutable,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "every",
                MethodDef {
                    rest_param: Some(Effect::ConditionallyMutate),
                    callee_effect: Effect::ConditionallyMutate,
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "some",
                MethodDef {
                    rest_param: Some(Effect::ConditionallyMutate),
                    callee_effect: Effect::ConditionallyMutate,
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "find",
                MethodDef {
                    rest_param: Some(Effect::ConditionallyMutate),
                    callee_effect: Effect::ConditionallyMutate,
                    return_type: TypeDef::Poly,
                    return_value_kind: ValueKind::Mutable,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "findIndex",
                MethodDef {
                    rest_param: Some(Effect::ConditionallyMutate),
                    callee_effect: Effect::ConditionallyMutate,
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..MethodDef::DEFAULT
                },
            ),
            Method("join", PURE_PRIMITIVE_FN),
            // TODO: rest of Array properties
        ],
    },
    // BuiltInSet
    ShapeDef {
        id: BUILT_IN_SET_ID,
        props: &[
            Method(
                "add",
                MethodDef {
                    positional_params: &[Effect::Capture],
                    callee_effect: Effect::Store,
                    return_type: TypeDef::Object(BUILT_IN_SET_ID),
                    return_value_kind: ValueKind::Mutable,
                    aliasing: Some(&AliasingSignatureConfig {
                        receiver: "@receiver",
                        params: &[],
                        rest: Some("@rest"),
                        returns: "@returns",
                        temporaries: &[],
                        effects: &[
                            // Set.add returns the receiver Set
                            AliasingEffectConfig::Assign { from: "@receiver", into: "@returns" },
                            // Set.add mutates the set itself
                            AliasingEffectConfig::Mutate { value: "@receiver" },
                            // Captures the rest params into the set
                            AliasingEffectConfig::Capture { from: "@rest", into: "@receiver" },
                        ],
                    }),
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "clear",
                MethodDef {
                    callee_effect: Effect::Store,
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "delete",
                MethodDef {
                    positional_params: &[Effect::Read],
                    callee_effect: Effect::Store,
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "has",
                MethodDef {
                    positional_params: &[Effect::Read],
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    ..MethodDef::DEFAULT
                },
            ),
            Value("size", TypeDef::Primitive),
            Method(
                "difference",
                MethodDef {
                    positional_params: &[Effect::Capture],
                    callee_effect: Effect::Capture,
                    return_type: TypeDef::Object(BUILT_IN_SET_ID),
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "union",
                MethodDef {
                    positional_params: &[Effect::Capture],
                    callee_effect: Effect::Capture,
                    return_type: TypeDef::Object(BUILT_IN_SET_ID),
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "symmetricalDifference",
                MethodDef {
                    positional_params: &[Effect::Capture],
                    callee_effect: Effect::Capture,
                    return_type: TypeDef::Object(BUILT_IN_SET_ID),
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "isSubsetOf",
                MethodDef {
                    positional_params: &[Effect::Read],
                    callee_effect: Effect::Read,
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "isSupersetOf",
                MethodDef {
                    positional_params: &[Effect::Read],
                    callee_effect: Effect::Read,
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "forEach",
                MethodDef {
                    rest_param: Some(Effect::ConditionallyMutate),
                    callee_effect: Effect::ConditionallyMutate,
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "values",
                MethodDef {
                    callee_effect: Effect::Capture,
                    return_type: TypeDef::Poly,
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "keys",
                MethodDef {
                    callee_effect: Effect::Capture,
                    return_type: TypeDef::Poly,
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "entries",
                MethodDef {
                    callee_effect: Effect::Capture,
                    return_type: TypeDef::Poly,
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
        ],
    },
    // BuiltInMap
    ShapeDef {
        id: BUILT_IN_MAP_ID,
        props: &[
            Method(
                "has",
                MethodDef {
                    positional_params: &[Effect::Read],
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "get",
                MethodDef {
                    positional_params: &[Effect::Read],
                    callee_effect: Effect::Capture,
                    return_type: TypeDef::Poly,
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "set",
                MethodDef {
                    positional_params: &[Effect::Capture, Effect::Capture],
                    callee_effect: Effect::Store,
                    return_type: TypeDef::Object(BUILT_IN_MAP_ID),
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "clear",
                MethodDef {
                    callee_effect: Effect::Store,
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "delete",
                MethodDef {
                    positional_params: &[Effect::Read],
                    callee_effect: Effect::Store,
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    ..MethodDef::DEFAULT
                },
            ),
            Value("size", TypeDef::Primitive),
            Method(
                "forEach",
                MethodDef {
                    rest_param: Some(Effect::ConditionallyMutate),
                    callee_effect: Effect::ConditionallyMutate,
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "values",
                MethodDef {
                    callee_effect: Effect::Capture,
                    return_type: TypeDef::Poly,
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "keys",
                MethodDef {
                    callee_effect: Effect::Capture,
                    return_type: TypeDef::Poly,
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "entries",
                MethodDef {
                    callee_effect: Effect::Capture,
                    return_type: TypeDef::Poly,
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
        ],
    },
    // BuiltInWeakSet
    ShapeDef {
        id: BUILT_IN_WEAK_SET_ID,
        props: &[
            Method("has", PURE_PRIMITIVE_FN),
            Method(
                "add",
                MethodDef {
                    positional_params: &[Effect::Capture],
                    callee_effect: Effect::Store,
                    return_type: TypeDef::Object(BUILT_IN_WEAK_SET_ID),
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "delete",
                MethodDef {
                    positional_params: &[Effect::Read],
                    callee_effect: Effect::Store,
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    ..MethodDef::DEFAULT
                },
            ),
        ],
    },
    // BuiltInWeakMap
    ShapeDef {
        id: BUILT_IN_WEAK_MAP_ID,
        props: &[
            Method("has", PURE_PRIMITIVE_FN),
            Method(
                "get",
                MethodDef {
                    positional_params: &[Effect::Read],
                    callee_effect: Effect::Capture,
                    return_type: TypeDef::Poly,
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "set",
                MethodDef {
                    positional_params: &[Effect::Capture, Effect::Capture],
                    callee_effect: Effect::Store,
                    return_type: TypeDef::Object(BUILT_IN_WEAK_MAP_ID),
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "delete",
                MethodDef {
                    positional_params: &[Effect::Read],
                    callee_effect: Effect::Store,
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    ..MethodDef::DEFAULT
                },
            ),
        ],
    },
    // BuiltInObject: has toString() returning Primitive (matches TS BuiltInObjectId shape)
    ShapeDef {
        id: BUILT_IN_OBJECT_ID,
        props: &[Method(
            "toString",
            MethodDef {
                return_type: TypeDef::Primitive,
                return_value_kind: ValueKind::Primitive,
                ..MethodDef::DEFAULT
            },
        )],
    },
    // BuiltInFunction: empty shape
    ShapeDef { id: BUILT_IN_FUNCTION_ID, props: &[] },
    // BuiltInJsx: empty shape
    ShapeDef { id: BUILT_IN_JSX_ID, props: &[] },
    // BuiltInMixedReadonly: has explicit method types + wildcard returning MixedReadonly
    // (matches TS BuiltInMixedReadonlyId shape)
    ShapeDef {
        id: BUILT_IN_MIXED_READONLY_ID,
        props: &[
            Method("toString", PURE_PRIMITIVE_FN),
            Method("indexOf", PURE_PRIMITIVE_FN),
            Method("includes", PURE_PRIMITIVE_FN),
            Method(
                "at",
                MethodDef {
                    positional_params: &[Effect::Read],
                    return_type: TypeDef::Object(BUILT_IN_MIXED_READONLY_ID),
                    callee_effect: Effect::Capture,
                    return_value_kind: ValueKind::Frozen,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "map",
                MethodDef {
                    rest_param: Some(Effect::ConditionallyMutate),
                    return_type: TypeDef::Object(BUILT_IN_ARRAY_ID),
                    callee_effect: Effect::ConditionallyMutate,
                    return_value_kind: ValueKind::Mutable,
                    no_alias: true,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "flatMap",
                MethodDef {
                    rest_param: Some(Effect::ConditionallyMutate),
                    return_type: TypeDef::Object(BUILT_IN_ARRAY_ID),
                    callee_effect: Effect::ConditionallyMutate,
                    return_value_kind: ValueKind::Mutable,
                    no_alias: true,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "filter",
                MethodDef {
                    rest_param: Some(Effect::ConditionallyMutate),
                    return_type: TypeDef::Object(BUILT_IN_ARRAY_ID),
                    callee_effect: Effect::ConditionallyMutate,
                    return_value_kind: ValueKind::Mutable,
                    no_alias: true,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "concat",
                MethodDef {
                    rest_param: Some(Effect::Capture),
                    return_type: TypeDef::Object(BUILT_IN_ARRAY_ID),
                    callee_effect: Effect::Capture,
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "slice",
                MethodDef {
                    rest_param: Some(Effect::Read),
                    return_type: TypeDef::Object(BUILT_IN_ARRAY_ID),
                    callee_effect: Effect::Capture,
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "every",
                MethodDef {
                    rest_param: Some(Effect::ConditionallyMutate),
                    return_type: TypeDef::Primitive,
                    callee_effect: Effect::ConditionallyMutate,
                    return_value_kind: ValueKind::Primitive,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "some",
                MethodDef {
                    rest_param: Some(Effect::ConditionallyMutate),
                    return_type: TypeDef::Primitive,
                    callee_effect: Effect::ConditionallyMutate,
                    return_value_kind: ValueKind::Primitive,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "find",
                MethodDef {
                    rest_param: Some(Effect::ConditionallyMutate),
                    return_type: TypeDef::Object(BUILT_IN_MIXED_READONLY_ID),
                    callee_effect: Effect::ConditionallyMutate,
                    return_value_kind: ValueKind::Frozen,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "findIndex",
                MethodDef {
                    rest_param: Some(Effect::ConditionallyMutate),
                    return_type: TypeDef::Primitive,
                    callee_effect: Effect::ConditionallyMutate,
                    return_value_kind: ValueKind::Primitive,
                    no_alias: true,
                    mutable_only_if_operands_are_mutable: true,
                    ..MethodDef::DEFAULT
                },
            ),
            Method("join", PURE_PRIMITIVE_FN),
            Value("*", TypeDef::Object(BUILT_IN_MIXED_READONLY_ID)),
        ],
    },
    // BuiltInUseRefId: { current: Object { shapeId: BuiltInRefValue } }
    ShapeDef {
        id: BUILT_IN_USE_REF_ID,
        props: &[Value("current", TypeDef::Object(BUILT_IN_REF_VALUE_ID))],
    },
    // BuiltInRefValue: { *: Object { shapeId: BuiltInRefValue } } (self-referencing)
    ShapeDef {
        id: BUILT_IN_REF_VALUE_ID,
        props: &[Value("*", TypeDef::Object(BUILT_IN_REF_VALUE_ID))],
    },
];

/// React state-flavored built-in shapes: each is a named setter function shape
/// plus a `[value, setter]` tuple object shape (e.g. BuiltInSetState +
/// BuiltInUseState).
struct StateShapeDef {
    /// Shape id of the setter function (element `1` of the tuple).
    setter_id: Ident<'static>,
    /// Shape id of the `[value, setter]` tuple object.
    tuple_id: Ident<'static>,
    /// Rest param effect of the setter function.
    setter_rest_param: Option<Effect>,
    /// Type of the state value (element `0` of the tuple).
    value_type: TypeDef,
}

const STATE_SHAPE_DEFS: &[StateShapeDef] = &[
    // BuiltInSetState (a function that freezes its argument) / BuiltInUseState
    StateShapeDef {
        setter_id: BUILT_IN_SET_STATE_ID,
        tuple_id: BUILT_IN_USE_STATE_ID,
        setter_rest_param: Some(Effect::Freeze),
        value_type: TypeDef::Poly,
    },
    // BuiltInSetActionState / BuiltInUseActionState
    StateShapeDef {
        setter_id: BUILT_IN_SET_ACTION_STATE_ID,
        tuple_id: BUILT_IN_USE_ACTION_STATE_ID,
        setter_rest_param: Some(Effect::Freeze),
        value_type: TypeDef::Poly,
    },
    // BuiltInDispatch / BuiltInUseReducer
    StateShapeDef {
        setter_id: BUILT_IN_DISPATCH_ID,
        tuple_id: BUILT_IN_USE_REDUCER_ID,
        setter_rest_param: Some(Effect::Freeze),
        value_type: TypeDef::Poly,
    },
    // BuiltInStartTransition / BuiltInUseTransition ([0] is the isPending flag)
    // Note: TS uses restParam: null for startTransition
    StateShapeDef {
        setter_id: BUILT_IN_START_TRANSITION_ID,
        tuple_id: BUILT_IN_USE_TRANSITION_ID,
        setter_rest_param: None,
        value_type: TypeDef::Primitive,
    },
    // BuiltInSetOptimistic / BuiltInUseOptimistic
    StateShapeDef {
        setter_id: BUILT_IN_SET_OPTIMISTIC_ID,
        tuple_id: BUILT_IN_USE_OPTIMISTIC_ID,
        setter_rest_param: Some(Effect::Freeze),
        value_type: TypeDef::Poly,
    },
];

#[cold]
#[inline(never)]
fn build_state_shapes(shapes: &mut ShapeRegistry<'static>) {
    for def in STATE_SHAPE_DEFS {
        let setter = add_method(
            shapes,
            &MethodDef {
                rest_param: def.setter_rest_param,
                return_type: TypeDef::Primitive,
                return_value_kind: ValueKind::Primitive,
                ..MethodDef::DEFAULT
            },
            Some(def.setter_id),
            false,
        );
        add_object(
            shapes,
            Some(def.tuple_id),
            vec![(Ident::from("0"), def.value_type.as_type()), (Ident::from("1"), setter)],
        );
    }
}

#[cold]
#[inline(never)]
fn build_hook_shapes(shapes: &mut ShapeRegistry<'static>) {
    // BuiltInEffectEvent function shape (the return value of useEffectEvent)
    add_method(
        shapes,
        &MethodDef {
            rest_param: Some(Effect::ConditionallyMutate),
            callee_effect: Effect::ConditionallyMutate,
            return_type: TypeDef::Poly,
            return_value_kind: ValueKind::Mutable,
            ..MethodDef::DEFAULT
        },
        Some(BUILT_IN_EFFECT_EVENT_ID),
        false,
    );
}

#[cold]
#[inline(never)]
fn build_misc_shapes(shapes: &mut ShapeRegistry<'static>) {
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
#[cold]
#[inline(never)]
fn build_default_globals(shapes: &mut ShapeRegistry<'static>) -> GlobalRegistry<'static> {
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

/// A React hook global (an `addHook` entry of TS Globals.ts `REACT_APIS`).
/// Interpreted by [`build_react_apis`].
struct HookDef {
    name: &'static str,
    hook_kind: HookKind,
    rest_param: Option<Effect>,
    return_type: TypeDef,
    return_value_kind: ValueKind,
    return_value_reason: Option<ValueReason>,
    shape_id: Option<Ident<'static>>,
    aliasing: Option<&'static AliasingSignatureConfig>,
}

impl HookDef {
    /// Field defaults shared by most React hooks: freeze the arguments and
    /// return a frozen value.
    const DEFAULT: Self = Self {
        name: "",
        hook_kind: HookKind::Custom,
        rest_param: Some(Effect::Freeze),
        return_type: TypeDef::Poly,
        return_value_kind: ValueKind::Frozen,
        return_value_reason: None,
        shape_id: None,
        aliasing: None,
    };
}

/// The React hook APIs, in upstream Globals.ts `REACT_APIS` declaration order.
/// `use` and `useEffectEvent` are registered imperatively in
/// [`build_react_apis`] and are not part of this table.
const REACT_HOOK_DEFS: &[HookDef] = &[
    HookDef {
        name: "useContext",
        hook_kind: HookKind::UseContext,
        rest_param: Some(Effect::Read),
        return_value_reason: Some(ValueReason::Context),
        shape_id: Some(BUILT_IN_USE_CONTEXT_HOOK_ID),
        ..HookDef::DEFAULT
    },
    HookDef {
        name: "useState",
        hook_kind: HookKind::UseState,
        return_type: TypeDef::Object(BUILT_IN_USE_STATE_ID),
        return_value_reason: Some(ValueReason::State),
        ..HookDef::DEFAULT
    },
    HookDef {
        name: "useActionState",
        hook_kind: HookKind::UseActionState,
        return_type: TypeDef::Object(BUILT_IN_USE_ACTION_STATE_ID),
        return_value_reason: Some(ValueReason::State),
        ..HookDef::DEFAULT
    },
    HookDef {
        name: "useReducer",
        hook_kind: HookKind::UseReducer,
        return_type: TypeDef::Object(BUILT_IN_USE_REDUCER_ID),
        return_value_reason: Some(ValueReason::ReducerState),
        ..HookDef::DEFAULT
    },
    HookDef {
        name: "useRef",
        hook_kind: HookKind::UseRef,
        rest_param: Some(Effect::Capture),
        return_type: TypeDef::Object(BUILT_IN_USE_REF_ID),
        return_value_kind: ValueKind::Mutable,
        ..HookDef::DEFAULT
    },
    HookDef {
        name: "useImperativeHandle",
        hook_kind: HookKind::UseImperativeHandle,
        return_type: TypeDef::Primitive,
        ..HookDef::DEFAULT
    },
    HookDef { name: "useMemo", hook_kind: HookKind::UseMemo, ..HookDef::DEFAULT },
    HookDef { name: "useCallback", hook_kind: HookKind::UseCallback, ..HookDef::DEFAULT },
    HookDef {
        name: "useEffect",
        hook_kind: HookKind::UseEffect,
        return_type: TypeDef::Primitive,
        shape_id: Some(BUILT_IN_USE_EFFECT_HOOK_ID),
        aliasing: Some(&AliasingSignatureConfig {
            receiver: "@receiver",
            params: &[],
            rest: Some("@rest"),
            returns: "@returns",
            temporaries: &["@effect"],
            effects: &[
                AliasingEffectConfig::Freeze { value: "@rest", reason: ValueReason::Effect },
                AliasingEffectConfig::Create {
                    into: "@effect",
                    value: ValueKind::Frozen,
                    reason: ValueReason::KnownReturnSignature,
                },
                AliasingEffectConfig::Capture { from: "@rest", into: "@effect" },
                AliasingEffectConfig::Create {
                    into: "@returns",
                    value: ValueKind::Primitive,
                    reason: ValueReason::KnownReturnSignature,
                },
            ],
        }),
        ..HookDef::DEFAULT
    },
    HookDef {
        name: "useLayoutEffect",
        hook_kind: HookKind::UseLayoutEffect,
        shape_id: Some(BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID),
        ..HookDef::DEFAULT
    },
    HookDef {
        name: "useInsertionEffect",
        hook_kind: HookKind::UseInsertionEffect,
        shape_id: Some(BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID),
        ..HookDef::DEFAULT
    },
    HookDef {
        name: "useTransition",
        hook_kind: HookKind::UseTransition,
        rest_param: None,
        return_type: TypeDef::Object(BUILT_IN_USE_TRANSITION_ID),
        ..HookDef::DEFAULT
    },
    HookDef {
        name: "useOptimistic",
        hook_kind: HookKind::UseOptimistic,
        return_type: TypeDef::Object(BUILT_IN_USE_OPTIMISTIC_ID),
        return_value_reason: Some(ValueReason::State),
        ..HookDef::DEFAULT
    },
];

/// Build the React API types (REACT_APIS from TS). Returns the list of (name, type) pairs
/// so they can be reused as properties of the React namespace object (matching TS behavior
/// where the SAME type objects are used in both DEFAULT_GLOBALS and the React namespace).
#[cold]
#[inline(never)]
fn build_react_apis(
    shapes: &mut ShapeRegistry<'static>,
    globals: &mut GlobalRegistry<'static>,
) -> Vec<(Ident<'static>, Type<'static>)> {
    let mut react_apis: Vec<(Ident, Type)> = Vec::new();

    for def in REACT_HOOK_DEFS {
        let hook = add_hook(
            shapes,
            HookSignatureBuilder {
                rest_param: def.rest_param,
                return_type: def.return_type.as_type(),
                return_value_kind: def.return_value_kind,
                return_value_reason: def.return_value_reason,
                hook_kind: def.hook_kind.clone(),
                aliasing: def.aliasing.copied(),
                ..Default::default()
            },
            def.shape_id,
        );
        react_apis.push((Ident::from(def.name), hook));
    }

    // use (not a hook, it's a function)
    let use_fn = add_method(shapes, &FREEZE_ARGS_FN, Some(BUILT_IN_USE_OPERATOR_ID), false);
    react_apis.push((Ident::from("use"), use_fn));

    // useEffectEvent — its return type is a function type, which cannot be
    // described in a const table, so it is registered imperatively.
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

/// A typed global namespace object: global name (also used as the shape id)
/// plus property table. Interpreted by [`build_typed_globals`].
struct GlobalObjectDef {
    name: &'static str,
    props: &'static [PropDef],
}

/// The typed global namespace objects, in upstream Globals.ts `TYPED_GLOBALS`
/// declaration order.
const TYPED_GLOBAL_OBJECTS: &[GlobalObjectDef] = &[
    // Object
    GlobalObjectDef {
        name: "Object",
        props: &[
            Method(
                "keys",
                MethodDef {
                    positional_params: &[Effect::Read],
                    return_type: TypeDef::Object(BUILT_IN_ARRAY_ID),
                    return_value_kind: ValueKind::Mutable,
                    aliasing: Some(&AliasingSignatureConfig {
                        receiver: "@receiver",
                        params: &["@object"],
                        rest: None,
                        returns: "@returns",
                        temporaries: &[],
                        effects: &[
                            AliasingEffectConfig::Create {
                                into: "@returns",
                                value: ValueKind::Mutable,
                                reason: ValueReason::KnownReturnSignature,
                            },
                            // Only keys are captured, and keys are immutable
                            AliasingEffectConfig::ImmutableCapture {
                                from: "@object",
                                into: "@returns",
                            },
                        ],
                    }),
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "fromEntries",
                MethodDef {
                    positional_params: &[Effect::ConditionallyMutate],
                    return_type: TypeDef::Object(BUILT_IN_OBJECT_ID),
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "entries",
                MethodDef {
                    positional_params: &[Effect::Capture],
                    return_type: TypeDef::Object(BUILT_IN_ARRAY_ID),
                    return_value_kind: ValueKind::Mutable,
                    aliasing: Some(&AliasingSignatureConfig {
                        receiver: "@receiver",
                        params: &["@object"],
                        rest: None,
                        returns: "@returns",
                        temporaries: &[],
                        effects: &[
                            AliasingEffectConfig::Create {
                                into: "@returns",
                                value: ValueKind::Mutable,
                                reason: ValueReason::KnownReturnSignature,
                            },
                            // Object values are captured into the return
                            AliasingEffectConfig::Capture { from: "@object", into: "@returns" },
                        ],
                    }),
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "values",
                MethodDef {
                    positional_params: &[Effect::Capture],
                    return_type: TypeDef::Object(BUILT_IN_ARRAY_ID),
                    return_value_kind: ValueKind::Mutable,
                    aliasing: Some(&AliasingSignatureConfig {
                        receiver: "@receiver",
                        params: &["@object"],
                        rest: None,
                        returns: "@returns",
                        temporaries: &[],
                        effects: &[
                            AliasingEffectConfig::Create {
                                into: "@returns",
                                value: ValueKind::Mutable,
                                reason: ValueReason::KnownReturnSignature,
                            },
                            // Object values are captured into the return
                            AliasingEffectConfig::Capture { from: "@object", into: "@returns" },
                        ],
                    }),
                    ..MethodDef::DEFAULT
                },
            ),
        ],
    },
    // Array
    GlobalObjectDef {
        name: "Array",
        props: &[
            Method(
                "isArray",
                MethodDef {
                    positional_params: &[Effect::Read],
                    return_type: TypeDef::Primitive,
                    return_value_kind: ValueKind::Primitive,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "from",
                MethodDef {
                    positional_params: &[
                        Effect::ConditionallyMutateIterator,
                        Effect::ConditionallyMutate,
                        Effect::ConditionallyMutate,
                    ],
                    rest_param: Some(Effect::Read),
                    return_type: TypeDef::Object(BUILT_IN_ARRAY_ID),
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
            Method(
                "of",
                MethodDef {
                    rest_param: Some(Effect::Read),
                    return_type: TypeDef::Object(BUILT_IN_ARRAY_ID),
                    return_value_kind: ValueKind::Mutable,
                    ..MethodDef::DEFAULT
                },
            ),
        ],
    },
    // Math
    GlobalObjectDef {
        name: "Math",
        props: &[
            Method("max", PURE_PRIMITIVE_FN),
            Method("min", PURE_PRIMITIVE_FN),
            Method("trunc", PURE_PRIMITIVE_FN),
            Method("ceil", PURE_PRIMITIVE_FN),
            Method("floor", PURE_PRIMITIVE_FN),
            Method("pow", PURE_PRIMITIVE_FN),
            Value("PI", TypeDef::Primitive),
            // Math.random is impure
            Method(
                "random",
                MethodDef {
                    return_type: TypeDef::Poly,
                    return_value_kind: ValueKind::Mutable,
                    impure: true,
                    canonical_name: Some("Math.random"),
                    ..MethodDef::DEFAULT
                },
            ),
        ],
    },
    // performance
    GlobalObjectDef {
        name: "performance",
        props: &[Method(
            "now",
            MethodDef {
                rest_param: Some(Effect::Read),
                return_type: TypeDef::Poly,
                return_value_kind: ValueKind::Mutable,
                impure: true,
                canonical_name: Some("performance.now"),
                ..MethodDef::DEFAULT
            },
        )],
    },
    // Date
    GlobalObjectDef {
        name: "Date",
        props: &[Method(
            "now",
            MethodDef {
                rest_param: Some(Effect::Read),
                return_type: TypeDef::Poly,
                return_value_kind: ValueKind::Mutable,
                impure: true,
                canonical_name: Some("Date.now"),
                ..MethodDef::DEFAULT
            },
        )],
    },
    // console
    GlobalObjectDef {
        name: "console",
        props: &[
            Method("error", PURE_PRIMITIVE_FN),
            Method("info", PURE_PRIMITIVE_FN),
            Method("log", PURE_PRIMITIVE_FN),
            Method("table", PURE_PRIMITIVE_FN),
            Method("trace", PURE_PRIMITIVE_FN),
            Method("warn", PURE_PRIMITIVE_FN),
        ],
    },
];

/// Simple global functions returning Primitive.
const PRIMITIVE_GLOBAL_FNS: &[&str] = &[
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
];

/// Map, Set, WeakMap, WeakSet constructors.
const COLLECTION_CTORS: &[(&str, Ident<'static>)] = &[
    ("Map", BUILT_IN_MAP_ID),
    ("Set", BUILT_IN_SET_ID),
    ("WeakMap", BUILT_IN_WEAK_MAP_ID),
    ("WeakSet", BUILT_IN_WEAK_SET_ID),
];

/// Build typed globals and return them as a list for use as globalThis/global properties.
#[cold]
#[inline(never)]
fn build_typed_globals(
    shapes: &mut ShapeRegistry<'static>,
    globals: &mut GlobalRegistry<'static>,
    react_apis: Vec<(Ident<'static>, Type<'static>)>,
) -> Vec<(Ident<'static>, Type<'static>)> {
    let mut typed_globals: Vec<(Ident, Type)> = Vec::new();

    // Object, Array, Math, performance, Date, console
    for def in TYPED_GLOBAL_OBJECTS {
        let global = add_object_from_def(shapes, Some(Ident::from(def.name)), def.props);
        typed_globals.push((Ident::from(def.name), global.clone()));
        globals.insert(Ident::from(def.name), global);
    }

    // Simple global functions returning Primitive
    for name in PRIMITIVE_GLOBAL_FNS {
        let f = add_method(shapes, &PURE_PRIMITIVE_FN, None, false);
        typed_globals.push((Ident::from(*name), f.clone()));
        globals.insert(Ident::from(*name), f);
    }

    // Primitive globals
    typed_globals.push((Ident::from("Infinity"), Type::Primitive));
    globals.insert(Ident::from("Infinity"), Type::Primitive);
    typed_globals.push((Ident::from("NaN"), Type::Primitive));
    globals.insert(Ident::from("NaN"), Type::Primitive);

    // Map, Set, WeakMap, WeakSet constructors
    for (name, shape_id) in COLLECTION_CTORS {
        let ctor = add_method(
            shapes,
            &MethodDef {
                positional_params: &[Effect::ConditionallyMutateIterator],
                return_type: TypeDef::Object(*shape_id),
                return_value_kind: ValueKind::Mutable,
                ..MethodDef::DEFAULT
            },
            None,
            true,
        );
        typed_globals.push((Ident::from(*name), ctor.clone()));
        globals.insert(Ident::from(*name), ctor);
    }

    // React global object — reuses the same REACT_APIS types (matching TS behavior
    // where the same type objects are used as both standalone globals and React.* properties)
    let react_create_element = add_method(shapes, &FREEZE_ARGS_FN, None, false);
    let react_clone_element = add_method(shapes, &FREEZE_ARGS_FN, None, false);
    let react_create_ref = add_method(
        shapes,
        &MethodDef {
            rest_param: Some(Effect::Capture),
            return_type: TypeDef::Object(BUILT_IN_USE_REF_ID),
            return_value_kind: ValueKind::Mutable,
            ..MethodDef::DEFAULT
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
    let jsx_fn = add_method(shapes, &FREEZE_ARGS_FN, None, false);
    typed_globals.push((Ident::from("_jsx"), jsx_fn.clone()));
    globals.insert(Ident::from("_jsx"), jsx_fn);

    typed_globals
}
