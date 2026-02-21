/// Aliasing effect types for the React Compiler.
///
/// Port of `Inference/AliasingEffects.ts` from the React Compiler.
///
/// `AliasingEffect` describes a set of "effects" that an instruction/terminal
/// has on one or more values in a program. These effects include mutation of
/// values, freezing values, tracking data flow between values, and other
/// specialized cases.
use crate::{
    compiler_error::{CompilerDiagnostic, SourceLocation},
    hir::{
        FunctionExpressionValue, IdentifierId, ObjectMethodValue, Place, SpreadPattern, ValueKind,
        ValueReason,
    },
};

use super::FunctionSignature;

/// An aliasing effect that describes how an instruction/terminal affects values.
#[derive(Debug, Clone)]
pub enum AliasingEffect {
    /// Marks the given value and its direct aliases as frozen.
    Freeze {
        value: Place,
        reason: ValueReason,
    },
    /// Mutate the value and any direct aliases (not captures).
    /// Errors if the value is not mutable.
    Mutate {
        value: Place,
        reason: Option<MutationReason>,
    },
    /// Mutate the value conditionally (only if known mutable).
    MutateConditionally {
        value: Place,
    },
    /// Mutate the value, any direct aliases, and any transitive captures.
    MutateTransitive {
        value: Place,
    },
    /// Mutates any of the value, its aliases, and captures that are mutable.
    MutateTransitiveConditionally {
        value: Place,
    },
    /// Records information flow from `from` to `into` where local mutation
    /// of `into` will NOT mutate `from`.
    Capture {
        from: Place,
        into: Place,
    },
    /// Records information flow from `from` to `into` where local mutation
    /// of `into` WILL mutate `from`.
    Alias {
        from: Place,
        into: Place,
    },
    /// Indicates the potential for information flow from `from` to `into`.
    /// Mutations flowing through this relationship become conditional.
    MaybeAlias {
        from: Place,
        into: Place,
    },
    /// Records direct assignment: `into = from`.
    Assign {
        from: Place,
        into: Place,
    },
    /// Creates a value of the given type at the given place.
    Create {
        into: Place,
        value: ValueKind,
        reason: ValueReason,
    },
    /// Creates a new value with the same kind as the starting value.
    CreateFrom {
        from: Place,
        into: Place,
    },
    /// Immutable data flow, used for escape analysis.
    /// Does not influence mutable range analysis.
    ImmutableCapture {
        from: Place,
        into: Place,
    },
    /// Calls the function at the given place with the given arguments,
    /// and captures/aliases the result into the given place.
    Apply {
        receiver: Place,
        function: Place,
        mutates_function: bool,
        args: Vec<ApplyArg>,
        into: Place,
        signature: Option<FunctionSignature>,
        loc: SourceLocation,
    },
    /// Constructs a function value with the given captures.
    CreateFunction {
        captures: Vec<Place>,
        function: CreateFunctionKind,
        into: Place,
    },
    /// Mutation of a value known to be immutable.
    MutateFrozen {
        place: Place,
        error: CompilerDiagnostic,
    },
    /// Mutation of a global.
    MutateGlobal {
        place: Place,
        error: CompilerDiagnostic,
    },
    /// Indicates a side-effect that is not safe during render.
    Impure {
        place: Place,
        error: CompilerDiagnostic,
    },
    /// Indicates that a given place is accessed during render.
    Render {
        place: Place,
    },
}

/// An argument to an Apply effect.
#[derive(Debug, Clone)]
pub enum ApplyArg {
    Place(Place),
    Spread(SpreadPattern),
    Hole,
}

/// The kind of function being created in a CreateFunction effect.
#[derive(Debug, Clone)]
pub enum CreateFunctionKind {
    FunctionExpression(FunctionExpressionValue),
    ObjectMethod(ObjectMethodValue),
}

/// A reason for a mutation.
#[derive(Debug, Clone)]
pub enum MutationReason {
    AssignCurrentProperty,
}

/// The aliasing signature of a function — describes how it aliases its parameters.
#[derive(Debug, Clone)]
pub struct AliasingSignature {
    pub receiver: IdentifierId,
    pub params: Vec<IdentifierId>,
    pub rest: Option<IdentifierId>,
    pub returns: IdentifierId,
    pub effects: Vec<AliasingEffect>,
    pub temporaries: Vec<Place>,
}
