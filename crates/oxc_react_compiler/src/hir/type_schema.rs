/// Environment config type schema and aliasing signature config parsing.
///
/// Port of `HIR/TypeSchema.ts` from the React Compiler.
///
/// Defines the schema types for configuring custom hooks, module type providers,
/// and aliasing signatures in the environment config.
use crate::hir::hir_types::{ValueKind, ValueReason};

/// Configuration for an aliasing effect (used in type schema definitions).
#[derive(Debug, Clone)]
pub enum AliasingEffectConfig {
    ImmutableCapture { from: String, into: String },
    CreateFrom { from: String, into: String },
    Capture { from: String, into: String },
    Alias { from: String, into: String },
    Assign { from: String, into: String },
    Mutate { value: String },
    MutateTransitiveConditionally { value: String },
    Create { into: String, reason: ValueReason, value: ValueKind },
    Freeze { value: String, reason: ValueReason },
    Impure { place: String },
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

/// A type configuration used in aliasing signatures.
#[derive(Debug, Clone)]
pub struct TypeConfig {
    pub kind: ValueKind,
    pub reason: ValueReason,
}
