/// Compiler environment and configuration.
///
/// Port of `HIR/Environment.ts` from the React Compiler.
///
/// The `Environment` holds all compilation context and configuration,
/// including shape registries, global definitions, and feature flags.
/// `EnvironmentConfig` defines all the knobs for controlling compiler behavior.
use rustc_hash::FxHashMap;

use super::{
    hir_types::{Effect, ReactFunctionType, ValueKind},
    object_shape::ShapeRegistry,
};

/// Configuration for an external function reference (source module + import specifier).
#[derive(Debug, Clone)]
pub struct ExternalFunction {
    pub source: String,
    pub import_specifier_name: String,
}

/// Configuration for instrumentation.
#[derive(Debug, Clone)]
pub struct InstrumentationConfig {
    pub func: ExternalFunction,
    pub gating: Option<ExternalFunction>,
    pub global_gating: Option<String>,
}

/// Configuration for a custom hook.
#[derive(Debug, Clone)]
pub struct HookConfig {
    pub effect_kind: Effect,
    pub value_kind: ValueKind,
    pub no_alias: bool,
    pub transitive_mixed_data: bool,
}

/// The full environment configuration — all compiler knobs and settings.
///
/// This corresponds to the Zod-validated `EnvironmentConfig` in the TS version.
#[derive(Debug, Clone)]
pub struct EnvironmentConfig {
    pub custom_hooks: FxHashMap<String, HookConfig>,

    /// A list of functions which the application compiles as macros.
    pub custom_macros: Option<Vec<String>>,

    /// Enable/disable the new type inference based on Flow types.
    pub enable_forest: bool,

    /// Enable function outlining.
    pub enable_function_outlining: bool,

    /// Enable JSX outlining.
    pub enable_jsx_outlining: bool,

    /// Enable naming anonymous functions.
    pub enable_name_anonymous_functions: bool,

    /// Whether to validate hooks usage.
    pub validate_hooks_usage: bool,

    /// Whether to validate no capitalized calls.
    pub validate_no_capitalized_calls: Option<Vec<String>>,

    /// Whether to validate ref access during render.
    pub validate_ref_access_during_render: bool,

    /// Whether to validate no setState in render.
    pub validate_no_set_state_in_render: bool,

    /// Whether to validate preserve existing memoization guarantees.
    pub validate_preserve_existing_memoization_guarantees: bool,

    /// Enable preserve existing memoization guarantees.
    pub enable_preserve_existing_memoization_guarantees: bool,

    /// Whether to validate exhaustive memoization dependencies.
    pub validate_exhaustive_memoization_dependencies: bool,

    /// Whether to validate exhaustive effect dependencies.
    pub validate_exhaustive_effect_dependencies: bool,

    /// Whether to validate no derived computations in effects.
    pub validate_no_derived_computations_in_effects: bool,

    /// Whether to validate no derived computations in effects (experimental).
    pub validate_no_derived_computations_in_effects_exp: bool,

    /// Whether to validate no setState in effects.
    pub validate_no_set_state_in_effects: bool,

    /// Whether to validate no JSX in try statements.
    pub validate_no_jsx_in_try_statements: bool,

    /// Whether to validate no impure functions in render.
    pub validate_no_impure_functions_in_render: bool,

    /// Whether to validate static components.
    pub validate_static_components: bool,

    /// Whether to validate source locations.
    pub validate_source_locations: bool,

    /// Whether to assert valid mutable ranges.
    pub assert_valid_mutable_ranges: bool,

    /// Enable emit instrument forget.
    pub enable_emit_instrument_forget: Option<InstrumentationConfig>,

    /// Enable emit hook guards.
    pub enable_emit_hook_guards: Option<ExternalFunction>,

    /// Whether to throw on unknown exceptions (test only).
    pub throw_unknown_exception_testonly: bool,

    /// Enable reset cache on source file changes (HMR support).
    pub enable_reset_cache_on_source_file_changes: Option<bool>,

    /// Enable custom type definitions for react-native-reanimated.
    ///
    /// When true, the compiler treats reanimated shared values as having
    /// specific type signatures to allow correct memoization behavior.
    pub enable_custom_type_definition_for_reanimated: bool,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            custom_hooks: FxHashMap::default(),
            custom_macros: None,
            enable_forest: false,
            enable_function_outlining: false,
            enable_jsx_outlining: false,
            enable_name_anonymous_functions: false,
            validate_hooks_usage: true,
            validate_no_capitalized_calls: None,
            validate_ref_access_during_render: true,
            validate_no_set_state_in_render: true,
            validate_preserve_existing_memoization_guarantees: false,
            enable_preserve_existing_memoization_guarantees: false,
            validate_exhaustive_memoization_dependencies: false,
            validate_exhaustive_effect_dependencies: false,
            validate_no_derived_computations_in_effects: false,
            validate_no_derived_computations_in_effects_exp: false,
            validate_no_set_state_in_effects: false,
            validate_no_jsx_in_try_statements: false,
            validate_no_impure_functions_in_render: false,
            validate_static_components: false,
            validate_source_locations: false,
            assert_valid_mutable_ranges: false,
            enable_emit_instrument_forget: None,
            enable_emit_hook_guards: None,
            throw_unknown_exception_testonly: false,
            enable_reset_cache_on_source_file_changes: None,
            enable_custom_type_definition_for_reanimated: false,
        }
    }
}

/// Validate an environment config.
///
/// In the TS version, this uses Zod schema validation.
/// In Rust, we rely on the type system for basic validation
/// and add runtime checks as needed.
///
/// # Errors
/// Returns a `CompilerError` if the config is invalid.
pub fn validate_environment_config(
    config: EnvironmentConfig,
) -> Result<EnvironmentConfig, crate::compiler_error::CompilerError> {
    // Validate that enable_emit_instrument_forget has at least one gating method
    if let Some(ref instrumentation) = config.enable_emit_instrument_forget
        && instrumentation.gating.is_none()
        && instrumentation.global_gating.is_none()
    {
        return Err(crate::compiler_error::CompilerError::invalid_config(
            "Expected at least one of gating or globalGating in instrumentation config",
            None,
            None,
        ));
    }
    Ok(config)
}

/// The output mode of the compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerOutputMode {
    /// Build optimized for SSR, with client features removed.
    Ssr,
    /// Build optimized for the client, with auto memoization.
    Client,
    /// Build optimized for the client without auto memo.
    ClientNoMemo,
    /// Lint mode: output is unused but validations run.
    Lint,
}

/// The compilation environment, holding all context needed during compilation.
///
/// This is the central context object threaded through all compiler passes.
#[derive(Debug, Clone)]
pub struct Environment {
    pub fn_type: ReactFunctionType,
    pub output_mode: CompilerOutputMode,
    pub config: EnvironmentConfig,
    pub shapes: ShapeRegistry,

    /// Whether validations should be enabled.
    pub enable_validations: bool,

    /// Whether memoization should be enabled.
    pub enable_memoization: bool,

    /// Whether manual memoization dropping is enabled.
    pub enable_drop_manual_memoization: bool,

    // ID counters
    next_block_id: u32,
    next_scope_id: u32,
    next_identifier_id: u32,
}

impl Environment {
    /// Create a new environment with the given configuration.
    pub fn new(
        fn_type: ReactFunctionType,
        output_mode: CompilerOutputMode,
        config: EnvironmentConfig,
    ) -> Self {
        let enable_memoization =
            !matches!(output_mode, CompilerOutputMode::Lint | CompilerOutputMode::ClientNoMemo);
        let enable_validations = true;
        let enable_drop_manual_memoization = enable_memoization;

        Self {
            fn_type,
            output_mode,
            config,
            shapes: FxHashMap::default(),
            enable_validations,
            enable_memoization,
            enable_drop_manual_memoization,
            next_block_id: 0,
            next_scope_id: 0,
            next_identifier_id: 0,
        }
    }

    /// Get the next block ID value without incrementing.
    pub fn next_block_id_value(&self) -> u32 {
        self.next_block_id
    }

    /// Generate a fresh block ID.
    pub fn next_block_id(&mut self) -> super::hir_types::BlockId {
        let id = self.next_block_id;
        self.next_block_id += 1;
        super::hir_types::BlockId(id)
    }

    /// Generate a fresh scope ID.
    pub fn next_scope_id(&mut self) -> super::hir_types::ScopeId {
        let id = self.next_scope_id;
        self.next_scope_id += 1;
        super::hir_types::ScopeId(id)
    }

    /// Generate a fresh identifier ID.
    pub fn next_identifier_id(&mut self) -> super::hir_types::IdentifierId {
        let id = self.next_identifier_id;
        self.next_identifier_id += 1;
        super::hir_types::IdentifierId(id)
    }
}
