/// Environment config type schema and aliasing signature config parsing.
///
/// Port of `HIR/TypeSchema.ts` from the React Compiler.
///
/// Defines the schema types for configuring custom hooks, module type providers,
/// and aliasing signatures in the environment config.
use rustc_hash::FxHashMap;

use crate::hir::hir_types::{Effect, ValueKind, ValueReason};

/// Configuration for an aliasing effect (used in type schema definitions).
#[derive(Debug, Clone)]
pub enum AliasingEffectConfig {
    ImmutableCapture {
        from: String,
        into: String,
    },
    CreateFrom {
        from: String,
        into: String,
    },
    Capture {
        from: String,
        into: String,
    },
    Alias {
        from: String,
        into: String,
    },
    Assign {
        from: String,
        into: String,
    },
    Mutate {
        value: String,
    },
    MutateTransitiveConditionally {
        value: String,
    },
    Create {
        into: String,
        reason: ValueReason,
        value: ValueKind,
    },
    Freeze {
        value: String,
        reason: ValueReason,
    },
    Impure {
        place: String,
    },
    Apply {
        receiver: String,
        function: String,
        mutates_function: bool,
        args: Vec<AliasingEffectArgConfig>,
        into: String,
    },
}

/// An argument in an Apply aliasing effect config.
#[derive(Debug, Clone)]
pub enum AliasingEffectArgConfig {
    Place(String),
    Spread { place: String },
    Hole,
}

/// Configuration for an aliasing signature.
#[derive(Debug, Clone)]
pub struct AliasingSignatureConfig {
    pub receiver: String,
    pub params: Vec<String>,
    pub rest: Option<String>,
    pub returns: String,
    pub temporaries: Vec<TypeConfig>,
    pub effects: Vec<AliasingEffectConfig>,
}

/// A type configuration used in aliasing signatures (for temporaries).
#[derive(Debug, Clone)]
pub struct TypeConfig {
    pub kind: ValueKind,
    pub reason: ValueReason,
}

// =====================================================================================
// Module type configuration — TypeConfig union from TypeSchema.ts
// =====================================================================================

/// A built-in type name used in type references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltInTypeName {
    Any,
    Ref,
    Array,
    Primitive,
    MixedReadonly,
}

/// Type configuration for module type providers.
///
/// Port of the `TypeConfig` union type from `TypeSchema.ts`.
/// This is used by `DefaultModuleTypeProvider` to describe the shapes
/// of known module exports.
#[derive(Debug, Clone)]
pub enum ModuleTypeConfig {
    /// An object type with named properties.
    Object { properties: FxHashMap<String, ModuleTypeConfig> },
    /// A function type with call signature information.
    Function {
        positional_params: Vec<Effect>,
        rest_param: Option<Effect>,
        callee_effect: Effect,
        return_type: Box<ModuleTypeConfig>,
        return_value_kind: ValueKind,
        no_alias: bool,
        mutable_only_if_operands_are_mutable: bool,
        impure: bool,
        canonical_name: Option<String>,
        aliasing: Option<AliasingSignatureConfig>,
        known_incompatible: Option<String>,
    },
    /// A hook type (like a function but with hook-specific semantics).
    Hook {
        positional_params: Option<Vec<Effect>>,
        rest_param: Option<Effect>,
        return_type: Box<ModuleTypeConfig>,
        return_value_kind: Option<ValueKind>,
        no_alias: bool,
        aliasing: Option<AliasingSignatureConfig>,
        known_incompatible: Option<String>,
    },
    /// A reference to a built-in type.
    TypeReference { name: BuiltInTypeName },
}
