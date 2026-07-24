use oxc_diagnostics::OxcDiagnostic;

/// The category of a React Compiler diagnostic, as an oxc-owned enum.
///
/// Upstream flattens its `ErrorCategory` to a `String` (via `format!("{:?}")`)
/// before it reaches this crate, so [`ReactCompilerCategory::from_compiler_string`]
/// maps those names back to a typed value. Consumers (e.g. the `react-compiler`
/// lint rule) match on this instead of parsing the diagnostic message, which
/// decouples them from the message format.
///
/// `UnexpectedError`/`PipelineError` are synthetic: the upstream throw events
/// they come from carry no category. Anything not enumerated here maps to
/// [`ReactCompilerCategory::Other`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactCompilerCategory {
    Hooks,
    Refs,
    RenderSetState,
    EffectSetState,
    EffectDerivationsOfState,
    ErrorBoundaries,
    Purity,
    StaticComponents,
    VoidUseMemo,
    UseMemo,
    CapitalizedCalls,
    MemoDependencies,
    PreserveManualMemo,
    Immutability,
    /// A compiler-internal assertion failure.
    Invariant,
    /// Synthetic: an unexpected `throw` inside the compiler.
    UnexpectedError,
    /// Synthetic: a compiler pipeline error.
    PipelineError,
    /// Any other category, including ones with no dedicated handling and the
    /// top-level fatal error (which carries no category of its own).
    Other,
}

impl ReactCompilerCategory {
    /// Map an upstream category string (the `Debug` name of its `ErrorCategory`,
    /// e.g. `"Refs"` or `"PreserveManualMemo"`) to a typed category. Unknown
    /// names map to [`ReactCompilerCategory::Other`].
    pub fn from_compiler_string(category: &str) -> Self {
        match category {
            "Hooks" => Self::Hooks,
            "Refs" => Self::Refs,
            "RenderSetState" => Self::RenderSetState,
            "EffectSetState" => Self::EffectSetState,
            "EffectDerivationsOfState" => Self::EffectDerivationsOfState,
            "ErrorBoundaries" => Self::ErrorBoundaries,
            "Purity" => Self::Purity,
            "StaticComponents" => Self::StaticComponents,
            "VoidUseMemo" => Self::VoidUseMemo,
            "UseMemo" => Self::UseMemo,
            "CapitalizedCalls" => Self::CapitalizedCalls,
            "MemoDependencies" => Self::MemoDependencies,
            "PreserveManualMemo" => Self::PreserveManualMemo,
            "Immutability" => Self::Immutability,
            "Invariant" => Self::Invariant,
            _ => Self::Other,
        }
    }
}

/// A React Compiler diagnostic paired with its [`ReactCompilerCategory`].
#[derive(Debug)]
pub struct ReactCompilerDiagnostic {
    pub diagnostic: OxcDiagnostic,
    pub category: ReactCompilerCategory,
}
