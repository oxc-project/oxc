// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Type configuration types, ported from TypeSchema.ts.
//!
//! These are the JSON-serializable config types used by `moduleTypeProvider`
//! and `installTypeConfig` to describe module/function/hook types.

use crate::react_compiler_utils::FxIndexMap;

use crate::react_compiler_hir::Effect;

/// Mirrors TS `ValueKind` enum for use in config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueKind {
    Mutable,
    Frozen,
    Primitive,
    MaybeFrozen,
    Global,
    Context,
}

/// Mirrors TS `ValueReason` enum for use in config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueReason {
    KnownReturnSignature,
    State,
    ReducerState,
    Context,
    Effect,
    HookCaptured,
    HookReturn,
    Global,
    JsxCaptured,
    StoreLocal,
    ReactiveFunctionArgument,
    Other,
}

// =============================================================================
// Aliasing effect config types (from TypeSchema.ts)
// =============================================================================

#[derive(Debug, Clone, Copy)]
pub enum AliasingEffectConfig {
    Freeze {
        value: &'static str,
        reason: ValueReason,
    },
    Create {
        into: &'static str,
        value: ValueKind,
        reason: ValueReason,
    },
    CreateFrom {
        from: &'static str,
        into: &'static str,
    },
    Assign {
        from: &'static str,
        into: &'static str,
    },
    Alias {
        from: &'static str,
        into: &'static str,
    },
    Capture {
        from: &'static str,
        into: &'static str,
    },
    ImmutableCapture {
        from: &'static str,
        into: &'static str,
    },
    Impure {
        place: &'static str,
    },
    Mutate {
        value: &'static str,
    },
    MutateTransitiveConditionally {
        value: &'static str,
    },
    Apply {
        receiver: &'static str,
        function: &'static str,
        mutates_function: bool,
        args: &'static [ApplyArgConfig],
        into: &'static str,
    },
}

#[derive(Debug, Clone)]
pub enum ApplyArgConfig {
    Place(&'static str),
    Spread {
        #[allow(dead_code)]
        kind: ApplyArgSpreadKind,
        place: &'static str,
    },
    Hole {
        #[allow(dead_code)]
        kind: ApplyArgHoleKind,
    },
}

/// Helper enum for tagged serde of `ApplyArgConfig::Spread`.
#[derive(Debug, Clone)]
pub enum ApplyArgSpreadKind {
    Spread,
}

/// Helper enum for tagged serde of `ApplyArgConfig::Hole`.
#[derive(Debug, Clone)]
pub enum ApplyArgHoleKind {
    Hole,
}

/// Aliasing signature config, the JSON-serializable form.
#[derive(Debug, Clone, Copy)]
pub struct AliasingSignatureConfig {
    pub receiver: &'static str,
    pub params: &'static [&'static str],
    pub rest: Option<&'static str>,
    pub returns: &'static str,
    pub temporaries: &'static [&'static str],
    pub effects: &'static [AliasingEffectConfig],
}

// =============================================================================
// Type config (from TypeSchema.ts)
// =============================================================================

#[derive(Debug, Clone)]
pub enum TypeConfig {
    Object(ObjectTypeConfig),
    Function(FunctionTypeConfig),
    Hook(HookTypeConfig),
    TypeReference(TypeReferenceConfig),
}

#[derive(Debug, Clone)]
pub struct ObjectTypeConfig {
    pub properties: Option<FxIndexMap<String, TypeConfig>>,
}

#[derive(Debug, Clone)]
pub struct FunctionTypeConfig {
    pub positional_params: Vec<Effect>,
    pub rest_param: Option<Effect>,
    pub callee_effect: Effect,
    pub return_type: Box<TypeConfig>,
    pub return_value_kind: ValueKind,
    pub no_alias: Option<bool>,
    pub mutable_only_if_operands_are_mutable: Option<bool>,
    pub impure: Option<bool>,
    pub canonical_name: Option<String>,
    pub aliasing: Option<AliasingSignatureConfig>,
    pub known_incompatible: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HookTypeConfig {
    pub positional_params: Option<Vec<Effect>>,
    pub rest_param: Option<Effect>,
    pub return_type: Box<TypeConfig>,
    pub return_value_kind: Option<ValueKind>,
    pub no_alias: Option<bool>,
    pub aliasing: Option<AliasingSignatureConfig>,
    pub known_incompatible: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltInTypeRef {
    Any,
    Ref,
    Array,
    Primitive,
    MixedReadonly,
}

#[derive(Debug, Clone)]
pub struct TypeReferenceConfig {
    pub name: BuiltInTypeRef,
}
