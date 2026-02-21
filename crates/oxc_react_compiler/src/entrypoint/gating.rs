/// Gating support for the React Compiler.
///
/// Port of `Entrypoint/Gating.ts` from the React Compiler.
///
/// Implements feature gating, where compiled and uncompiled versions of a
/// function are both emitted, and a runtime flag determines which to use.
use crate::hir::environment::ExternalFunction;

/// Options for how the gated output should be structured.
#[derive(Debug, Clone)]
pub struct GatingOutput {
    /// The name of the optimized (compiled) function.
    pub optimized_name: String,
    /// The name of the unoptimized (original) function.
    pub unoptimized_name: String,
    /// The gating function to call at runtime.
    pub gating_function: ExternalFunction,
}

/// Generate gating names for a function.
pub fn generate_gating_names(original_name: &str) -> (String, String) {
    let optimized = format!("{original_name}_forget");
    let unoptimized = format!("{original_name}_uncompiled");
    (optimized, unoptimized)
}
