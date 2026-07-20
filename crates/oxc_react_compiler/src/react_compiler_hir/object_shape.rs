// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Object shapes and function signatures, ported from ObjectShape.ts.
//!
//! Defines the shape registry used by Environment to resolve property types
//! and function call signatures for built-in objects, hooks, and user-defined types.

use std::borrow::Cow;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::sync::atomic::{AtomicU32, Ordering};

use oxc_allocator::Allocator;
use oxc_str::{Ident, IdentHashMap, format_ident, static_ident};

use crate::react_compiler_hir::Effect;
use crate::react_compiler_hir::Type;
use crate::react_compiler_hir::type_config::{
    AliasingEffectConfig, AliasingSignatureConfig, ValueKind, ValueReason,
};

// =============================================================================
// Shape ID constants (matching TS ObjectShape.ts)
// =============================================================================

pub const BUILT_IN_PROPS_ID: Ident<'static> = static_ident!("BuiltInProps");
pub const BUILT_IN_ARRAY_ID: Ident<'static> = static_ident!("BuiltInArray");
pub const BUILT_IN_SET_ID: Ident<'static> = static_ident!("BuiltInSet");
pub const BUILT_IN_MAP_ID: Ident<'static> = static_ident!("BuiltInMap");
pub const BUILT_IN_WEAK_SET_ID: Ident<'static> = static_ident!("BuiltInWeakSet");
pub const BUILT_IN_WEAK_MAP_ID: Ident<'static> = static_ident!("BuiltInWeakMap");
pub const BUILT_IN_FUNCTION_ID: Ident<'static> = static_ident!("BuiltInFunction");
pub const BUILT_IN_JSX_ID: Ident<'static> = static_ident!("BuiltInJsx");
pub const BUILT_IN_OBJECT_ID: Ident<'static> = static_ident!("BuiltInObject");
pub const BUILT_IN_USE_STATE_ID: Ident<'static> = static_ident!("BuiltInUseState");
pub const BUILT_IN_SET_STATE_ID: Ident<'static> = static_ident!("BuiltInSetState");
pub const BUILT_IN_USE_ACTION_STATE_ID: Ident<'static> = static_ident!("BuiltInUseActionState");
pub const BUILT_IN_SET_ACTION_STATE_ID: Ident<'static> = static_ident!("BuiltInSetActionState");
pub const BUILT_IN_USE_REF_ID: Ident<'static> = static_ident!("BuiltInUseRefId");
pub const BUILT_IN_REF_VALUE_ID: Ident<'static> = static_ident!("BuiltInRefValue");
pub const BUILT_IN_MIXED_READONLY_ID: Ident<'static> = static_ident!("BuiltInMixedReadonly");
pub const BUILT_IN_USE_EFFECT_HOOK_ID: Ident<'static> = static_ident!("BuiltInUseEffectHook");
pub const BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID: Ident<'static> =
    static_ident!("BuiltInUseLayoutEffectHook");
pub const BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID: Ident<'static> =
    static_ident!("BuiltInUseInsertionEffectHook");
pub const BUILT_IN_USE_OPERATOR_ID: Ident<'static> = static_ident!("BuiltInUseOperator");
pub const BUILT_IN_USE_REDUCER_ID: Ident<'static> = static_ident!("BuiltInUseReducer");
pub const BUILT_IN_DISPATCH_ID: Ident<'static> = static_ident!("BuiltInDispatch");
pub const BUILT_IN_USE_CONTEXT_HOOK_ID: Ident<'static> = static_ident!("BuiltInUseContextHook");
pub const BUILT_IN_USE_TRANSITION_ID: Ident<'static> = static_ident!("BuiltInUseTransition");
pub const BUILT_IN_USE_OPTIMISTIC_ID: Ident<'static> = static_ident!("BuiltInUseOptimistic");
pub const BUILT_IN_SET_OPTIMISTIC_ID: Ident<'static> = static_ident!("BuiltInSetOptimistic");
pub const BUILT_IN_START_TRANSITION_ID: Ident<'static> = static_ident!("BuiltInStartTransition");
pub const BUILT_IN_USE_EFFECT_EVENT_ID: Ident<'static> = static_ident!("BuiltInUseEffectEvent");
pub const BUILT_IN_EFFECT_EVENT_ID: Ident<'static> = static_ident!("BuiltInEffectEventFunction");
pub const REANIMATED_SHARED_VALUE_ID: Ident<'static> = static_ident!("ReanimatedSharedValueId");

// =============================================================================
// Core types
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookKind {
    UseContext,
    UseState,
    UseActionState,
    UseReducer,
    UseRef,
    UseEffect,
    UseLayoutEffect,
    UseInsertionEffect,
    UseMemo,
    UseCallback,
    UseTransition,
    UseImperativeHandle,
    UseEffectEvent,
    UseOptimistic,
    Custom,
}

impl Display for HookKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            HookKind::UseContext => write!(f, "useContext"),
            HookKind::UseState => write!(f, "useState"),
            HookKind::UseActionState => write!(f, "useActionState"),
            HookKind::UseReducer => write!(f, "useReducer"),
            HookKind::UseRef => write!(f, "useRef"),
            HookKind::UseEffect => write!(f, "useEffect"),
            HookKind::UseLayoutEffect => write!(f, "useLayoutEffect"),
            HookKind::UseInsertionEffect => write!(f, "useInsertionEffect"),
            HookKind::UseMemo => write!(f, "useMemo"),
            HookKind::UseCallback => write!(f, "useCallback"),
            HookKind::UseTransition => write!(f, "useTransition"),
            HookKind::UseImperativeHandle => write!(f, "useImperativeHandle"),
            HookKind::UseEffectEvent => write!(f, "useEffectEvent"),
            HookKind::UseOptimistic => write!(f, "useOptimistic"),
            HookKind::Custom => write!(f, "Custom"),
        }
    }
}

/// Call signature of a function, used for type and effect inference.
/// Ported from TS `FunctionSignature`.
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub positional_params: Vec<Effect>,
    pub rest_param: Option<Effect>,
    pub return_value_kind: ValueKind,
    pub return_value_reason: Option<ValueReason>,
    pub callee_effect: Effect,
    pub hook_kind: Option<HookKind>,
    pub no_alias: bool,
    pub mutable_only_if_operands_are_mutable: bool,
    pub impure: bool,
    pub known_incompatible: Option<Cow<'static, str>>,
    pub canonical_name: Option<Cow<'static, str>>,
    /// Aliasing signature in config form. Full parsing into AliasingSignature
    /// with Place values is deferred until the aliasing effects system is ported.
    pub aliasing: Option<AliasingSignatureConfig>,
}

/// Shape of an object or function type.
/// Ported from TS `ObjectShape`.
#[derive(Debug, Clone)]
pub struct ObjectShape<'a> {
    pub properties: IdentHashMap<'a, Type<'a>>,
    pub function_type: Option<FunctionSignature>,
}

/// Registry mapping shape IDs to their ObjectShape definitions.
///
/// Supports two modes:
/// - **Builder mode** (`base=None`): wraps a single map, used during
///   `build_builtin_shapes` / `build_default_globals` to construct the static base.
///   Anonymous shape ids minted here are leaked; they become part of the
///   process-lifetime static tables anyway, and the build runs once.
/// - **Overlay mode** (`base=Some`): holds a `&'static` base map plus a small
///   extras map. Lookups check extras first, then base. Inserts go into extras,
///   with anonymous ids allocated in the compilation arena.
///   Cloning only copies the extras map (the base pointer is shared).
pub struct ShapeRegistry<'a> {
    base: Option<&'static IdentHashMap<'static, ObjectShape<'static>>>,
    entries: IdentHashMap<'a, ObjectShape<'a>>,
    allocator: Option<&'a Allocator>,
}

impl<'a> ShapeRegistry<'a> {
    /// Create an empty builder-mode registry.
    pub fn new() -> Self {
        Self { base: None, entries: IdentHashMap::default(), allocator: None }
    }

    /// Create an overlay-mode registry backed by a static base.
    pub fn with_base(
        base: &'static IdentHashMap<'static, ObjectShape<'static>>,
        allocator: &'a Allocator,
    ) -> Self {
        Self { base: Some(base), entries: IdentHashMap::default(), allocator: Some(allocator) }
    }

    pub fn get(&self, key: &str) -> Option<&ObjectShape<'a>> {
        self.entries.get(key).or_else(|| self.base.and_then(|b| b.get(key)))
    }

    pub fn insert(&mut self, key: Ident<'a>, value: ObjectShape<'a>) {
        self.entries.insert(key, value);
    }

    /// Consume the registry and return the inner map.
    /// Only valid in builder mode (no base).
    pub(crate) fn into_inner(self) -> IdentHashMap<'a, ObjectShape<'a>> {
        debug_assert!(self.base.is_none(), "into_inner() called on overlay-mode ShapeRegistry");
        self.entries
    }

    /// Allocate an identifier with the registry's lifetime: in the compilation
    /// arena in overlay mode, leaked in (once-per-process) builder mode.
    pub fn alloc_ident(&self, s: &str) -> Ident<'a> {
        match self.allocator {
            Some(allocator) => Ident::from_str_in(s, &allocator),
            None => Ident::from(&*s.to_string().leak()),
        }
    }

    /// Mint a unique anonymous shape id. Mirrors TS `nextAnonId` in ObjectShape.ts.
    /// The counter is process-global so ids stay unique across the static base
    /// build and every per-compilation registry.
    fn next_anon_id(&self) -> Ident<'a> {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        match self.allocator {
            Some(allocator) => format_ident!(allocator, "<generated_{id}>"),
            // Builder mode runs once per process to construct the static base
            // tables; its handful of generated ids are static data.
            None => Ident::from(&*format!("<generated_{id}>").leak()),
        }
    }
}

impl Default for ShapeRegistry<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ShapeRegistry<'_> {
    fn clone(&self) -> Self {
        Self { base: self.base, entries: self.entries.clone(), allocator: self.allocator }
    }
}

// =============================================================================
// Builder functions (matching TS addFunction, addHook, addObject)
// =============================================================================

/// Add a non-hook function to a ShapeRegistry.
/// Returns a `Type::Function` representing the added function.
pub fn add_function<'a>(
    registry: &mut ShapeRegistry<'a>,
    properties: Vec<(Ident<'a>, Type<'a>)>,
    sig: FunctionSignatureBuilder<'a>,
    id: Option<Ident<'a>>,
    is_constructor: bool,
) -> Type<'a> {
    let shape_id = id.unwrap_or_else(|| registry.next_anon_id());
    let return_type = sig.return_type.clone();
    add_shape(
        registry,
        shape_id,
        properties,
        Some(FunctionSignature {
            positional_params: sig.positional_params,
            rest_param: sig.rest_param,
            return_value_kind: sig.return_value_kind,
            return_value_reason: sig.return_value_reason,
            callee_effect: sig.callee_effect,
            hook_kind: None,
            no_alias: sig.no_alias,
            mutable_only_if_operands_are_mutable: sig.mutable_only_if_operands_are_mutable,
            impure: sig.impure,
            known_incompatible: sig.known_incompatible,
            canonical_name: sig.canonical_name,
            aliasing: sig.aliasing,
        }),
    );
    Type::Function { shape_id: Some(shape_id), return_type: Box::new(return_type), is_constructor }
}

/// Add a hook to a ShapeRegistry.
/// Returns a `Type::Function` representing the added hook.
pub fn add_hook<'a>(
    registry: &mut ShapeRegistry<'a>,
    sig: HookSignatureBuilder<'a>,
    id: Option<Ident<'a>>,
) -> Type<'a> {
    let shape_id = id.unwrap_or_else(|| registry.next_anon_id());
    let return_type = sig.return_type.clone();
    add_shape(
        registry,
        shape_id,
        Vec::new(),
        Some(FunctionSignature {
            positional_params: sig.positional_params,
            rest_param: sig.rest_param,
            return_value_kind: sig.return_value_kind,
            return_value_reason: sig.return_value_reason,
            callee_effect: sig.callee_effect,
            hook_kind: Some(sig.hook_kind),
            no_alias: sig.no_alias,
            mutable_only_if_operands_are_mutable: false,
            impure: false,
            known_incompatible: sig.known_incompatible,
            canonical_name: None,
            aliasing: sig.aliasing,
        }),
    );
    Type::Function {
        shape_id: Some(shape_id),
        return_type: Box::new(return_type),
        is_constructor: false,
    }
}

/// Add an object to a ShapeRegistry.
/// Returns a `Type::Object` representing the added object.
pub fn add_object<'a>(
    registry: &mut ShapeRegistry<'a>,
    id: Option<Ident<'a>>,
    properties: Vec<(Ident<'a>, Type<'a>)>,
) -> Type<'a> {
    let shape_id = id.unwrap_or_else(|| registry.next_anon_id());
    add_shape(registry, shape_id, properties, None);
    Type::Object { shape_id: Some(shape_id) }
}

fn add_shape<'a>(
    registry: &mut ShapeRegistry<'a>,
    id: Ident<'a>,
    properties: Vec<(Ident<'a>, Type<'a>)>,
    function_type: Option<FunctionSignature>,
) {
    let shape = ObjectShape { properties: properties.into_iter().collect(), function_type };
    // Note: TS has an invariant that the id doesn't already exist. We use
    // insert which overwrites. In practice duplicates don't occur for built-in
    // shapes, and for user configs we want last-write-wins behavior.
    registry.insert(id, shape);
}

// =============================================================================
// Builder structs (to avoid large parameter lists)
// =============================================================================

/// Builder for non-hook function signatures.
pub struct FunctionSignatureBuilder<'a> {
    pub positional_params: Vec<Effect>,
    pub rest_param: Option<Effect>,
    pub return_type: Type<'a>,
    pub return_value_kind: ValueKind,
    pub return_value_reason: Option<ValueReason>,
    pub callee_effect: Effect,
    pub no_alias: bool,
    pub mutable_only_if_operands_are_mutable: bool,
    pub impure: bool,
    pub known_incompatible: Option<Cow<'static, str>>,
    pub canonical_name: Option<Cow<'static, str>>,
    pub aliasing: Option<AliasingSignatureConfig>,
}

impl Default for FunctionSignatureBuilder<'_> {
    fn default() -> Self {
        Self {
            positional_params: Vec::new(),
            rest_param: None,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            return_value_reason: None,
            callee_effect: Effect::Read,
            no_alias: false,
            mutable_only_if_operands_are_mutable: false,
            impure: false,
            known_incompatible: None,
            canonical_name: None,
            aliasing: None,
        }
    }
}

/// Builder for hook signatures.
pub struct HookSignatureBuilder<'a> {
    pub positional_params: Vec<Effect>,
    pub rest_param: Option<Effect>,
    pub return_type: Type<'a>,
    pub return_value_kind: ValueKind,
    pub return_value_reason: Option<ValueReason>,
    pub callee_effect: Effect,
    pub hook_kind: HookKind,
    pub no_alias: bool,
    pub known_incompatible: Option<Cow<'static, str>>,
    pub aliasing: Option<AliasingSignatureConfig>,
}

impl Default for HookSignatureBuilder<'_> {
    fn default() -> Self {
        Self {
            positional_params: Vec::new(),
            rest_param: None,
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            return_value_reason: None,
            callee_effect: Effect::Read,
            hook_kind: HookKind::Custom,
            no_alias: false,
            known_incompatible: None,
            aliasing: None,
        }
    }
}

// =============================================================================
// Default hook types used for unknown hooks
// =============================================================================

/// Default type for hooks when enableAssumeHooksFollowRulesOfReact is true.
/// Matches TS `DefaultNonmutatingHook`.
pub fn default_nonmutating_hook<'a>(registry: &mut ShapeRegistry<'a>) -> Type<'a> {
    add_hook(
        registry,
        HookSignatureBuilder {
            rest_param: Some(Effect::Freeze),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Frozen,
            hook_kind: HookKind::Custom,
            aliasing: Some(AliasingSignatureConfig {
                receiver: "@receiver",
                params: &[],
                rest: Some("@rest"),
                returns: "@returns",
                temporaries: &[],
                effects: &[
                    // Freeze the arguments
                    AliasingEffectConfig::Freeze {
                        value: "@rest",
                        reason: ValueReason::HookCaptured,
                    },
                    // Returns a frozen value
                    AliasingEffectConfig::Create {
                        into: "@returns",
                        value: ValueKind::Frozen,
                        reason: ValueReason::HookReturn,
                    },
                    // May alias any arguments into the return
                    AliasingEffectConfig::Alias { from: "@rest", into: "@returns" },
                ],
            }),
            ..Default::default()
        },
        Some(static_ident!("DefaultNonmutatingHook")),
    )
}

/// Default type for hooks when enableAssumeHooksFollowRulesOfReact is false.
/// Matches TS `DefaultMutatingHook`.
pub fn default_mutating_hook<'a>(registry: &mut ShapeRegistry<'a>) -> Type<'a> {
    add_hook(
        registry,
        HookSignatureBuilder {
            rest_param: Some(Effect::ConditionallyMutate),
            return_type: Type::Poly,
            return_value_kind: ValueKind::Mutable,
            hook_kind: HookKind::Custom,
            ..Default::default()
        },
        Some(static_ident!("DefaultMutatingHook")),
    )
}
