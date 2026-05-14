/// Compiler environment and configuration.
///
/// Port of `HIR/Environment.ts` from the React Compiler.
///
/// The `Environment` holds all compilation context and configuration,
/// including shape registries, global definitions, and feature flags.
/// `EnvironmentConfig` defines all the knobs for controlling compiler behavior.
use std::sync::Arc;

use cow_utils::CowUtils;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::compiler_error::{CompilerError, GENERATED_SOURCE};

use super::{
    globals::{Global, GlobalRegistry},
    hir_types::{
        Effect, HIRFunction, IdentifierName, NonLocalBinding, ReactFunctionType, ValueKind,
    },
    object_shape::{
        BUILT_IN_DEFAULT_MUTATING_HOOK_ID, BUILT_IN_DEFAULT_NONMUTATING_HOOK_ID, FunctionSignature,
        HookKind, ShapeRegistry,
    },
    types::{FunctionType, ObjectType, Type},
};

/// Configuration for an external function reference (source module + import specifier).
#[derive(Debug, Clone)]
pub struct ExternalFunction {
    pub source: String,
    pub import_specifier_name: String,
}

/// Configuration entry for `infer_effect_dependencies`.
///
/// Mirrors the TS shape:
/// ```text
/// { function: ExternalFunction, autodepsIndex: number }
/// ```
///
/// `autodeps_index` is the zero-based argument-array index at which the
/// compiler expects the `AUTODEPS` sentinel (e.g., `1` for
/// `useEffect(fn, AUTODEPS)`). Must be `>= 1` since index 0 is always the
/// callback.
#[derive(Debug, Clone)]
pub struct InferEffectDependenciesEntry {
    pub function: ExternalFunction,
    pub autodeps_index: u32,
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

/// Mode for exhaustive effect dependency validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExhaustiveEffectDepsMode {
    /// No validation.
    Off,
    /// Report both missing and extra dependencies.
    All,
    /// Only report missing dependencies.
    MissingOnly,
    /// Only report extra dependencies.
    ExtraOnly,
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

    /// Enable the new mutation aliasing model.
    ///
    /// `None` means "absent / unset"; `Some(true)` opts in explicitly;
    /// `Some(false)` opts out explicitly. The field is retained as
    /// `Option<bool>` so the pragma parser in `utils/test_utils.rs` can
    /// faithfully accept all three forms (`@enableNewMutationAliasingModel`,
    /// `@enableNewMutationAliasingModel:true`,
    /// `@enableNewMutationAliasingModel:false`) for back-compatible fixtures.
    ///
    /// **At runtime this field is always collapsed to `Some(true)`** by
    /// `EnvironmentContext::build`, regardless of its incoming value. Upstream
    /// v19.2.6 removed this option from `EnvironmentConfig` because the new
    /// mutation-aliasing model became the only code path, and the Rust port
    /// likewise has only that one implementation, so a `Some(false)` opt-out
    /// cannot be honored. Callers wishing to query the resolved value should
    /// use `Environment::enable_new_mutation_aliasing_model()`, which always
    /// returns `true`.
    ///
    /// Corresponds to `enableNewMutationAliasingModel` in older TS versions.
    pub enable_new_mutation_aliasing_model: Option<bool>,

    /// Enable the HIR-based propagate-scope-deps fork.
    ///
    /// In upstream v19.2.6 this is no longer a configurable flag — the HIR
    /// implementation is the only path. We retain the field so the existing
    /// `@enablePropagateDepsInHIR` pragma in fixtures parses without warning.
    ///
    /// Corresponds to `enablePropagateDepsInHIR` in older TS versions.
    pub enable_propagate_deps_in_hir: bool,

    /// Enable the experimental `fire(...)` transform.
    ///
    /// Corresponds to `enableFire` in the TS version (default `false`).
    pub enable_fire: bool,

    /// Enable inference and auto-insertion of effect dependencies.
    ///
    /// When set, the compiler inserts dependency arrays for the configured
    /// hooks. Each entry pairs an external function reference with the 1-based
    /// argument index where the `AUTODEPS` sentinel is expected.
    ///
    /// Corresponds to `inferEffectDependencies` in the TS version (default
    /// `null` / unset).
    pub infer_effect_dependencies: Option<Vec<InferEffectDependenciesEntry>>,

    /// Whether to validate exhaustive memoization dependencies.
    pub validate_exhaustive_memoization_dependencies: bool,

    /// Mode for validating exhaustive effect dependencies.
    ///
    /// - `Off`: no validation
    /// - `All`: report both missing and extra deps
    /// - `MissingOnly`: only report missing deps
    /// - `ExtraOnly`: only report extra deps
    pub validate_exhaustive_effect_dependencies: ExhaustiveEffectDepsMode,

    /// Whether to validate no derived computations in effects.
    pub validate_no_derived_computations_in_effects: bool,

    /// Whether to validate no derived computations in effects (experimental).
    pub validate_no_derived_computations_in_effects_exp: bool,

    /// Whether to validate no setState in effects.
    pub validate_no_set_state_in_effects: bool,

    /// Enable verbose error messages for no-setState-in-effects validation.
    ///
    /// When true, emits a more detailed description with guidance about
    /// common patterns (non-local derived data, derived event, force update).
    ///
    /// Corresponds to `enableVerboseNoSetStateInEffect` in the TS version.
    pub enable_verbose_no_set_state_in_effect: bool,

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

    /// Enable emit freeze. When set, the compiler wraps memoized cache-store
    /// outputs with a runtime call like `__DEV__ ? makeReadOnly(value, "fnName") : value`
    /// to freeze them in development mode.
    ///
    /// Corresponds to `enableEmitFreeze` in the TS version.
    pub enable_emit_freeze: Option<ExternalFunction>,

    /// Enable emitting "change variables" which store the result of whether a particular
    /// reactive scope dependency has changed since the scope was last executed.
    ///
    /// Example with `enable_change_variable_codegen: true`:
    /// ```text
    /// const c_0 = $[0] !== input; // change variable
    /// let output;
    /// if (c_0) ...
    /// ```
    ///
    /// Defaults to `false`, where the comparison is inlined:
    /// ```text
    /// let output;
    /// if ($[0] !== input) ...
    /// ```
    ///
    /// Corresponds to `enableChangeVariableCodegen` in the TS version
    /// (`HIR/Environment.ts`).
    pub enable_change_variable_codegen: bool,

    /// Enable runtime change-detection instrumentation for debugging memoization
    /// correctness. When set, rather than skipping recomputation on cache hits,
    /// the compiler emits a helper call that re-runs the scope body and compares
    /// the cached value against the freshly recomputed value, flagging silent
    /// invalidations caused by rules-of-React violations.
    ///
    /// The configured `ExternalFunction` is imported via
    /// `ProgramContext::add_import_specifier` and invoked with the arguments
    /// `(oldValue, newValue, "name", "fnName", "cached" | "recomputed", "(start:end)")`.
    ///
    /// Upstream rejects pairing this with `disableMemoizationForDebugging`,
    /// but the Rust port does not yet model that knob, so the runtime check
    /// is skipped here. If `disableMemoizationForDebugging` is ever added,
    /// reinstate the invariant in `validate_environment_config` to match
    /// upstream `HIR/Environment.ts` lines 753-763.
    ///
    /// Corresponds to `enableChangeDetectionForDebugging` in the TS version
    /// (`HIR/Environment.ts`).
    pub enable_change_detection_for_debugging: Option<ExternalFunction>,

    /// Whether to throw on unknown exceptions (test only).
    pub throw_unknown_exception_testonly: bool,

    /// Enable reset cache on source file changes (HMR support).
    pub enable_reset_cache_on_source_file_changes: Option<bool>,

    /// Enable custom type definitions for react-native-reanimated.
    ///
    /// When true, the compiler treats reanimated shared values as having
    /// specific type signatures to allow correct memoization behavior.
    pub enable_custom_type_definition_for_reanimated: bool,

    /// Enable the shared-runtime module type provider (test only).
    ///
    /// When true, the compiler registers type definitions for the `shared-runtime`
    /// test module, providing correct type information for hooks like `useFragment`
    /// (returns `MixedReadonly`, `noAlias: true`), `useNoAlias`, and typed functions.
    /// This matches the `sharedRuntimeTypeProvider` from the TS test harness.
    pub enable_shared_runtime_type_provider: bool,

    /// Whether to assume hooks follow the rules of React.
    ///
    /// When true (default), custom hooks are treated with `DefaultNonmutatingHook`:
    /// arguments are frozen and return values are frozen.
    /// When false, custom hooks are treated with `DefaultMutatingHook`:
    /// arguments may be conditionally mutated and return values are mutable.
    ///
    /// Corresponds to `enableAssumeHooksFollowRulesOfReact` in the TS version.
    pub enable_assume_hooks_follow_rules_of_react: bool,

    /// Treat identifiers as SetState type if both:
    /// - they are named with a "set-" prefix
    /// - they are called somewhere
    ///
    /// Corresponds to `enableTreatSetIdentifiersAsStateSetters` in the TS version.
    pub enable_treat_set_identifiers_as_state_setters: bool,

    /// List of module names whose imports are blocklisted.
    ///
    /// If set, the compiler will bail out if any import declaration
    /// imports from a module in this list.
    ///
    /// Corresponds to `validateBlocklistedImports` in the TS version.
    pub validate_blocklisted_imports: Option<Vec<String>>,

    /// Enable `useKeyedState` in error messages for setState-during-render validation.
    ///
    /// When true, the "Cannot call setState during render" error suggests using
    /// `useKeyedState(initialState, key)` instead of the default advice about
    /// storing previous values in state.
    ///
    /// Corresponds to `enableUseKeyedState` in the TS version.
    pub enable_use_keyed_state: bool,

    /// Allow setState calls in effects when the value is derived from a ref.
    ///
    /// When true (default), setState calls in effects are exempted from the
    /// `validateNoSetStateInEffects` validation if the first argument is derived
    /// from a ref, or if the block is controlled by a ref-derived conditional.
    ///
    /// Corresponds to `enableAllowSetStateFromRefsInEffects` in the TS version.
    pub enable_allow_set_state_from_refs_in_effects: bool,

    /// Enable optional dependency tracking for optional chain expressions.
    ///
    /// When true (default), the compiler tracks optional chain dependencies
    /// (e.g., `a?.b`) more precisely.
    ///
    /// Corresponds to `enableOptionalDependencies` in the TS version.
    pub enable_optional_dependencies: bool,

    /// Enable transitive freezing of function expression captures.
    ///
    /// When true (default), freezing a function expression also recursively
    /// freezes all of its context captures. This ensures that values captured
    /// by callbacks passed to hooks like useEffect are treated as frozen.
    ///
    /// Corresponds to `enableTransitivelyFreezeFunctionExpressions` in the TS version.
    pub enable_transitively_freeze_function_expressions: bool,

    /// Enable treating ref-like identifiers as refs for type inference.
    ///
    /// When true (default), identifiers whose names end with `Ref` (e.g.,
    /// `myRef`) are inferred as ref types, and their `.current` property
    /// accesses are typed as `BuiltInRefValue`. This prevents mutations to
    /// `.current` from extending mutable ranges.
    ///
    /// Corresponds to `enableTreatRefLikeIdentifiersAsRefs` in the TS version.
    pub enable_treat_ref_like_identifiers_as_refs: bool,

    /// Validate that useMemo/useCallback results are not void (unused or no return value).
    ///
    /// When true, the compiler checks that useMemo callbacks return a value
    /// and that useMemo results are actually used.
    ///
    /// Corresponds to `validateNoVoidUseMemo` in the TS version.
    pub validate_no_void_use_memo: bool,

    /// Validate that known mutable functions are not frozen.
    ///
    /// When true, the compiler checks that functions known to be mutable
    /// (e.g., setState) are not inadvertently frozen by being passed to hooks
    /// or included in memoized values.
    ///
    /// Corresponds to `validateNoFreezingKnownMutableFunctions` in the TS version.
    pub validate_no_freezing_known_mutable_functions: bool,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            custom_hooks: FxHashMap::default(),
            custom_macros: None,
            enable_forest: false,
            enable_function_outlining: true,
            enable_jsx_outlining: false,
            enable_name_anonymous_functions: false,
            validate_hooks_usage: true,
            validate_no_capitalized_calls: None,
            validate_ref_access_during_render: true,
            validate_no_set_state_in_render: true,
            validate_preserve_existing_memoization_guarantees: true,
            // Upstream default: `enablePreserveExistingMemoizationGuarantees: false`.
            // When this is `true`, StartMemoize/FinishMemoize emit Freeze effects that
            // wrongly forbid mutations through transitively-aliased captures (see
            // new-mutability/transitivity-* fixtures). Validation tracking is preserved
            // via `validate_preserve_existing_memoization_guarantees: true` above, which
            // also causes `DropManualMemoization` to insert StartMemoize/FinishMemoize
            // markers without emitting freezing inference effects.
            enable_preserve_existing_memoization_guarantees: false,
            enable_new_mutation_aliasing_model: None,
            enable_propagate_deps_in_hir: true,
            enable_fire: false,
            infer_effect_dependencies: None,
            validate_exhaustive_memoization_dependencies: true,
            validate_exhaustive_effect_dependencies: ExhaustiveEffectDepsMode::Off,
            validate_no_derived_computations_in_effects: false,
            validate_no_derived_computations_in_effects_exp: false,
            validate_no_set_state_in_effects: false,
            enable_verbose_no_set_state_in_effect: false,
            validate_no_jsx_in_try_statements: false,
            validate_no_impure_functions_in_render: false,
            validate_static_components: false,
            validate_source_locations: false,
            assert_valid_mutable_ranges: false,
            enable_emit_instrument_forget: None,
            enable_emit_hook_guards: None,
            enable_emit_freeze: None,
            enable_change_variable_codegen: false,
            enable_change_detection_for_debugging: None,
            throw_unknown_exception_testonly: false,
            enable_reset_cache_on_source_file_changes: None,
            enable_custom_type_definition_for_reanimated: false,
            enable_shared_runtime_type_provider: false,
            enable_assume_hooks_follow_rules_of_react: true,
            enable_treat_set_identifiers_as_state_setters: false,
            validate_blocklisted_imports: None,
            enable_use_keyed_state: false,
            enable_allow_set_state_from_refs_in_effects: true,
            enable_optional_dependencies: true,
            enable_transitively_freeze_function_expressions: true,
            enable_treat_ref_like_identifiers_as_refs: true,
            validate_no_void_use_memo: false,
            validate_no_freezing_known_mutable_functions: true,
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

impl CompilerOutputMode {
    /// True iff this mode emits diagnostics only, no output AST.
    pub const fn is_lint(self) -> bool {
        matches!(self, Self::Lint)
    }

    /// True iff this mode emits SSR-flavored output.
    pub const fn is_ssr(self) -> bool {
        matches!(self, Self::Ssr)
    }

    /// True iff this mode is the full client mode with auto-memoization enabled.
    ///
    /// Returns `false` for `ClientNoMemo`, `Ssr`, and `Lint`.
    pub const fn is_client(self) -> bool {
        matches!(self, Self::Client)
    }
}

/// Routes a validation outcome to one of two error buffers inside an
/// [`Environment`].
///
/// The two channels have different semantics:
///
/// | Channel | Buffer | Blocks codegen? |
/// |---------|--------|-----------------|
/// | `Diagnostic` | `env.state.diagnostics` | No — emitted as diagnostics only |
/// | `RecordedFatal` | `env.state.recorded_errors` | Yes — pipeline continues but output is discarded |
///
/// Pass a channel to [`Environment::report`] to route a validation outcome
/// explicitly.  Use [`Environment::report_lint_aware`] for the common
/// "lint mode → diagnostic, otherwise → recorded-fatal" policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorChannel {
    /// The error goes to `env.state.diagnostics` and never blocks codegen.
    Diagnostic,
    /// The error goes to `env.state.recorded_errors`.  The pipeline continues
    /// but the final codegen output is discarded if any recorded errors exist.
    RecordedFatal,
}

/// Immutable per-Environment context shared across all clones via `Arc`.
///
/// Holds the read-only configuration that does not change after the
/// `Environment` is constructed: feature flags, the shape/global registries,
/// the source/file metadata, and the pre-resolved `module_types` cache.
///
/// `Environment::clone` is now an O(1) Arc bump on this struct plus a small
/// clone of the per-function `EnvironmentState`.
#[derive(Debug, Clone)]
pub struct EnvironmentContext {
    /// The kind of React function being compiled (Component / Hook / Other).
    pub fn_type: ReactFunctionType,

    /// The compiler output mode (Client / Ssr / ClientNoMemo / Lint).
    pub output_mode: CompilerOutputMode,

    /// The validated environment configuration (all compiler knobs).
    pub config: Arc<EnvironmentConfig>,

    /// Per-`Environment` shape registry, derived from the cached defaults
    /// plus any custom hooks / type-provider extensions applied at build time.
    pub shapes: Arc<ShapeRegistry>,

    /// Per-`Environment` global registry, derived from the cached defaults
    /// plus any custom hooks installed at build time.
    pub globals: Arc<GlobalRegistry>,

    /// The source code of the file being compiled.
    /// Used for HMR/Fast Refresh cache invalidation.
    pub code: Option<Arc<str>>,

    /// The filename of the file being compiled.
    /// Used for instrumentation (instrument forget) emission.
    pub filename: Option<Arc<str>>,

    /// Module type registry — maps module names to their type definitions.
    ///
    /// Port of `#moduleTypes` from the TS `Environment` class.
    ///
    /// Pre-resolved at context-build time: contains the config-gated
    /// test/runtime modules (`react-native-reanimated` and the shared-runtime
    /// suite) plus every module known to `DefaultModuleTypeProvider`. As a
    /// result, `resolve_module_type` is a pure lookup with no shape mutation
    /// on the hot path.
    pub module_types: Arc<FxHashMap<String, super::types::Type>>,

    /// Whether validations should be enabled.
    pub enable_validations: bool,

    /// Whether memoization should be enabled.
    pub enable_memoization: bool,

    /// Whether manual memoization dropping is enabled.
    pub enable_drop_manual_memoization: bool,
}

/// The compilation environment, holding all context needed during compilation.
///
/// This is the central context object threaded through all compiler passes.
/// It is a thin `{ ctx, state }` façade: read-only configuration lives in
/// `ctx` (shared by `Arc`), and per-function mutable state lives in `state`.
///
/// `ctx` is `pub` so call sites that need an `Arc` clone of the inner
/// registries can do `Arc::clone(&env.ctx.shapes)` without breaking the
/// Arc-bump optimization. `state` stays private; access goes through
/// `Environment`'s inherent methods (`next_block_id`, `record_errors`,
/// `take_diagnostics`, etc.).
#[derive(Debug, Clone)]
pub struct Environment {
    /// Immutable per-Environment context, shared across clones via `Arc`.
    pub ctx: Arc<EnvironmentContext>,

    /// Per-function mutable state — counters, diagnostics, recorded errors,
    /// outlined functions, and known referenced names.
    state: EnvironmentState,
}

/// Per-function mutable state owned by an `Environment`.
///
/// Holds ID counters, accumulated diagnostics and recorded errors, the
/// outlined-functions buffer, and the set of known referenced identifier
/// names. These fields are reset (or refreshed) on each compilation; the
/// surrounding `Environment` keeps the read-only configuration that
/// outlives a single function.
#[derive(Debug, Clone, Default)]
pub struct EnvironmentState {
    /// ID counter for fresh `BlockId`s.
    pub next_block_id: u32,

    /// ID counter for fresh `ScopeId`s.
    pub next_scope_id: u32,

    /// ID counter for fresh `IdentifierId`s.
    pub next_identifier_id: u32,

    /// Counter for generating globally unique identifier names.
    pub next_uid: u32,

    /// Collected diagnostics from lint-mode validation passes (TS `logErrors` equivalent).
    /// These are only logged/reported but do NOT prevent compilation.
    pub diagnostics: Vec<CompilerError>,

    /// Accumulated non-fatal validation errors (TS `recordError` equivalent).
    /// These allow the pipeline to continue through codegen, but are checked
    /// at the end — if any are present, the compiled output is discarded and
    /// the errors are returned.
    pub recorded_errors: Vec<CompilerError>,

    /// Outlined functions extracted from this function.
    ///
    /// Each entry is a tuple of the outlined HIR function and an optional
    /// `ReactFunctionType` string (e.g., `Some("Component")` or `None`).
    pub outlined_functions: Vec<OutlinedFunctionEntry>,

    /// Known referenced names — used for generating globally unique identifiers.
    ///
    /// Corresponds to `ProgramContext.knownReferencedNames` in the TS version.
    pub known_referenced_names: FxHashSet<String>,

    /// `true` once `infer_effect_dependencies` has rewritten at least one
    /// `useEffect(..., AUTODEPS)` call in this function.
    ///
    /// Used by the entrypoint retry-compilation path (`enable_fire` /
    /// `infer_effect_dependencies`) to detect whether a retry actually
    /// changed the function. Corresponds to `Environment.hasInferredEffect`
    /// in the TS reference.
    pub has_inferred_effect: bool,
}

/// An entry in the outlined functions list.
#[derive(Debug, Clone)]
pub struct OutlinedFunctionEntry {
    pub func: HIRFunction,
    pub fn_type: Option<ReactFunctionType>,
}

impl EnvironmentState {
    /// Advance all ID counters to be at least as high as the given state's
    /// counters. This is the state-to-state counterpart of
    /// `Environment::advance_counters_past`; see `Environment::advance_counters_past`
    /// for the full rationale.
    pub(super) fn advance_counters_past(&mut self, other: &EnvironmentState) {
        self.next_block_id = self.next_block_id.max(other.next_block_id);
        self.next_scope_id = self.next_scope_id.max(other.next_scope_id);
        self.next_identifier_id = self.next_identifier_id.max(other.next_identifier_id);

        debug_assert!(
            self.next_block_id >= other.next_block_id
                && self.next_scope_id >= other.next_scope_id
                && self.next_identifier_id >= other.next_identifier_id,
            "advance_counters_past must leave destination at >= source counters \
             (block: {} >= {}, scope: {} >= {}, id: {} >= {})",
            self.next_block_id,
            other.next_block_id,
            self.next_scope_id,
            other.next_scope_id,
            self.next_identifier_id,
            other.next_identifier_id,
        );
    }
}

impl EnvironmentContext {
    /// Construct the immutable per-Environment context.
    ///
    /// Applies all per-`Environment` customizations exactly once: type
    /// providers, custom hooks, and `DefaultModuleTypeProvider` pre-resolution.
    /// After this returns, the `EnvironmentContext` is final and is wrapped in
    /// an `Arc` so subsequent `Environment::clone` calls are O(1).
    ///
    /// # Errors
    /// Returns a `CompilerError` if the configuration is invalid (e.g., a
    /// custom hook name conflicts with a built-in global definition).
    pub fn build(
        fn_type: ReactFunctionType,
        output_mode: CompilerOutputMode,
        mut config: EnvironmentConfig,
    ) -> Result<Self, CompilerError> {
        let enable_memoization =
            !matches!(output_mode, CompilerOutputMode::Ssr | CompilerOutputMode::ClientNoMemo);
        let enable_validations = true;
        let enable_drop_manual_memoization =
            !matches!(output_mode, CompilerOutputMode::ClientNoMemo);

        // Resolve the `enableNewMutationAliasingModel` flag to its only supported
        // runtime value, `Some(true)`.
        //
        // Upstream v19.2.6 REMOVED this option from `EnvironmentConfig` entirely —
        // the new mutation-aliasing model is now the only code path. We retain the
        // field as `Option<bool>` so the pragma parser in `test_utils.rs` can keep
        // accepting `@enableNewMutationAliasingModel`, `:true`, and `:false` for
        // back-compatible fixture sources, but at construction time we collapse
        // *all three* states (`None`, `Some(true)`, `Some(false)`) to `Some(true)`.
        //
        // Rationale: the Rust port has only ONE mutation/effects inference path —
        // the new-aliasing model in `crates/oxc_react_compiler/src/inference/` —
        // so a fixture written for the legacy path cannot be honored by routing
        // through a different implementation. The 3 fixtures in `compiler/` that
        // carry `@enableNewMutationAliasingModel:false`
        // (`allow-assigning-to-global-in-function-spread-as-jsx.js`,
        // `bug-capturing-func-maybealias-captured-mutate.ts`,
        // `bug-separate-memoization-due-to-callback-capturing.js`) already pass
        // conformance under the new model, so collapsing `:false` is harmless.
        //
        // Collapsing here also keeps `Environment::enable_new_mutation_aliasing_model()`
        // total and never-false, which is required by the `debug_assert!` in
        // `run_pipeline` and avoids panicking on debug builds when an external API
        // caller (or the fixture pragma parser) sets `Some(false)` directly.
        config.enable_new_mutation_aliasing_model = Some(true);

        // Initialize shapes, globals, and module_types from the pristine cached
        // defaults via Arc. The defaults are built once per process in
        // `default_registries` and already include every module known to
        // `DefaultModuleTypeProvider`. Per-`Environment` customization (custom
        // hooks + config-gated test/runtime type providers) applies below via
        // `Arc::make_mut`, which copies the underlying registry/map exactly
        // once on the first mutation.
        let mut shapes = super::default_registries::default_shapes_arc();
        let mut globals = super::default_registries::default_globals_arc();
        let mut module_types = super::default_registries::default_module_types_arc();

        // Determine whether any per-Environment customization will mutate the
        // shape registry. If not, we can keep the Arc bump path pure (no deep
        // clone). The `DefaultModuleTypeProvider` modules are no longer
        // considered here because they are pre-resolved into the shared
        // LazyLock snapshot.
        let needs_shape_mutation = config.enable_custom_type_definition_for_reanimated
            || config.enable_shared_runtime_type_provider
            || !config.custom_hooks.is_empty();
        let needs_globals_mutation = !config.custom_hooks.is_empty();

        // Register module types for configured (non-default) type providers.
        // Mutates the `module_types` map via copy-on-write through Arc.
        if config.enable_custom_type_definition_for_reanimated
            || config.enable_shared_runtime_type_provider
        {
            // Both branches need a shape registry to install types into, and a
            // `module_types` map to publish the resolved types into. Both are
            // covered by `needs_shape_mutation` above.
            debug_assert!(needs_shape_mutation);
            let shapes_mut = Arc::make_mut(&mut shapes);
            let module_types_mut = Arc::make_mut(&mut module_types);
            if config.enable_custom_type_definition_for_reanimated {
                let reanimated_type = super::globals::get_reanimated_module_type(shapes_mut);
                module_types_mut.insert("react-native-reanimated".to_string(), reanimated_type);
            }
            if config.enable_shared_runtime_type_provider {
                let shared_runtime_type =
                    super::globals::get_shared_runtime_module_type(shapes_mut);
                module_types_mut.insert("shared-runtime".to_string(), shared_runtime_type);

                let known_incompatible_type =
                    super::globals::get_known_incompatible_test_module_type(shapes_mut);
                module_types_mut.insert(
                    "ReactCompilerKnownIncompatibleTest".to_string(),
                    known_incompatible_type,
                );

                let react_compiler_test_type =
                    super::globals::get_react_compiler_test_module_type(shapes_mut);
                module_types_mut.insert("ReactCompilerTest".to_string(), react_compiler_test_type);

                let use_default_not_hook_type =
                    super::globals::get_use_default_export_not_typed_as_hook_module_type(
                        shapes_mut,
                    );
                module_types_mut.insert(
                    "useDefaultExportNotTypedAsHook".to_string(),
                    use_default_not_hook_type,
                );
            }
        }

        // Register custom hooks from config into the globals registry.
        // Port of Environment.ts constructor lines 582-601.
        if needs_globals_mutation {
            // `Arc::make_mut(&mut shapes)` is a no-op clone if the prior
            // type-provider block already deep-cloned the shape registry;
            // otherwise it clones it now exactly once for the custom hooks.
            let shapes_mut = Arc::make_mut(&mut shapes);
            let globals_mut = Arc::make_mut(&mut globals);
            for (hook_name, hook) in &config.custom_hooks {
                CompilerError::invariant_result(
                    !globals_mut.contains_key(hook_name),
                    &format!(
                        "[Globals] Found existing definition in global registry for custom hook {hook_name}"
                    ),
                    None,
                    GENERATED_SOURCE,
                )?;
                let return_type = if hook.transitive_mixed_data {
                    Type::Object(ObjectType {
                        shape_id: Some(super::object_shape::BUILT_IN_MIXED_READONLY_ID.to_string()),
                    })
                } else {
                    Type::Poly
                };
                let shape_id = super::object_shape::add_hook(
                    shapes_mut,
                    None,
                    FunctionSignature {
                        rest_param: Some(hook.effect_kind),
                        return_type: return_type.clone(),
                        return_value_kind: hook.value_kind,
                        callee_effect: Effect::Read,
                        hook_kind: Some(HookKind::Custom),
                        no_alias: hook.no_alias,
                        ..FunctionSignature::default()
                    },
                );
                globals_mut.insert(
                    hook_name.clone(),
                    Global::Typed(Type::Function(FunctionType {
                        shape_id: Some(shape_id),
                        return_type: Box::new(return_type),
                        is_constructor: false,
                    })),
                );
            }
        }

        Ok(Self {
            fn_type,
            output_mode,
            config: Arc::new(config),
            shapes,
            globals,
            code: None,
            filename: None,
            module_types,
            enable_validations,
            enable_memoization,
            enable_drop_manual_memoization,
        })
    }
}

impl Environment {
    /// Create a new environment with the given configuration.
    ///
    /// # Errors
    /// Returns a `CompilerError` if the configuration is invalid (e.g., a custom hook
    /// name conflicts with a built-in global definition).
    pub fn new(
        fn_type: ReactFunctionType,
        output_mode: CompilerOutputMode,
        config: EnvironmentConfig,
    ) -> Result<Self, CompilerError> {
        let ctx = Arc::new(EnvironmentContext::build(fn_type, output_mode, config)?);
        Ok(Self { ctx, state: EnvironmentState::default() })
    }

    /// Set the source code on the environment. Used by codegen passes that need
    /// to convert byte offsets in `SourceLocation::Source` into 1-indexed line
    /// numbers (e.g. `enableChangeDetectionForDebugging` emits
    /// `"(start_line:end_line)"` labels). Mirrors the way upstream's
    /// `Environment` is constructed with `code` from the Babel pipeline.
    ///
    /// The mutation is performed via `Arc::make_mut`, which clones the inner
    /// context once if there are other strong references (none in the standard
    /// construction path).
    pub fn set_source_code(&mut self, code: Arc<str>) {
        Arc::make_mut(&mut self.ctx).code = Some(code);
    }

    // =========================================================================
    // Context accessors
    //
    // Read-only views into the shared `EnvironmentContext`. Call sites should
    // prefer these to avoid reaching into `env.ctx.X` directly.
    // =========================================================================

    /// The per-Environment shape registry.
    #[inline]
    pub fn shapes(&self) -> &ShapeRegistry {
        &self.ctx.shapes
    }

    /// The per-Environment global registry.
    #[inline]
    pub fn globals(&self) -> &GlobalRegistry {
        &self.ctx.globals
    }

    /// The validated environment configuration.
    #[inline]
    pub fn config(&self) -> &EnvironmentConfig {
        &self.ctx.config
    }

    /// The kind of React function being compiled.
    #[inline]
    pub fn fn_type(&self) -> ReactFunctionType {
        self.ctx.fn_type
    }

    /// The compiler output mode.
    #[inline]
    pub fn output_mode(&self) -> CompilerOutputMode {
        self.ctx.output_mode
    }

    /// The source code of the file being compiled, if any.
    #[inline]
    pub fn code(&self) -> Option<&str> {
        self.ctx.code.as_deref()
    }

    /// The filename of the file being compiled, if any.
    #[inline]
    pub fn filename(&self) -> Option<&str> {
        self.ctx.filename.as_deref()
    }

    /// Pre-resolved module type registry.
    #[inline]
    pub fn module_types(&self) -> &FxHashMap<String, Type> {
        &self.ctx.module_types
    }

    /// Whether validations should be enabled.
    #[inline]
    pub fn enable_validations(&self) -> bool {
        self.ctx.enable_validations
    }

    /// Whether memoization should be enabled.
    #[inline]
    pub fn enable_memoization(&self) -> bool {
        self.ctx.enable_memoization
    }

    /// Whether the new mutation-aliasing model is enabled.
    ///
    /// In upstream v19.2.6 this option was removed from `EnvironmentConfig`
    /// because the new mutation-aliasing model became the only code path.
    /// `EnvironmentContext::build` collapses every possible `Option<bool>`
    /// state (`None`, `Some(true)`, `Some(false)`) to `Some(true)` because
    /// the Rust port likewise has only one inference implementation. This
    /// accessor therefore always returns `true`.
    ///
    /// It is retained as an accessor (rather than inlined as `true`) so the
    /// pipeline call site can document the gate explicitly. If a future
    /// change reintroduces a legacy inference path, the accessor — and the
    /// `build()` collapse above — are the two places to update.
    #[inline]
    pub fn enable_new_mutation_aliasing_model(&self) -> bool {
        // `build()` collapses to `Some(true)`. The `unwrap_or(true)` is a
        // defensive total fallback in case some future direct-construction
        // path bypasses `build()`.
        self.ctx.config.enable_new_mutation_aliasing_model.unwrap_or(true)
    }

    /// Whether manual memoization dropping is enabled.
    #[inline]
    pub fn enable_drop_manual_memoization(&self) -> bool {
        self.ctx.enable_drop_manual_memoization
    }

    /// Get the next block ID value without incrementing.
    pub fn next_block_id_value(&self) -> u32 {
        self.state.next_block_id
    }

    /// Generate a fresh block ID.
    pub fn next_block_id(&mut self) -> super::hir_types::BlockId {
        let id = self.state.next_block_id;
        self.state.next_block_id += 1;
        super::hir_types::BlockId(id)
    }

    /// Generate a fresh scope ID.
    pub fn next_scope_id(&mut self) -> super::hir_types::ScopeId {
        let id = self.state.next_scope_id;
        self.state.next_scope_id += 1;
        super::hir_types::ScopeId(id)
    }

    /// Generate a fresh identifier ID.
    pub fn next_identifier_id(&mut self) -> super::hir_types::IdentifierId {
        let id = self.state.next_identifier_id;
        self.state.next_identifier_id += 1;
        super::hir_types::IdentifierId(id)
    }

    /// Advance all ID counters to be at least as high as the given environment's
    /// counters. This simulates the TS behavior where nested function lowering
    /// shares the same Environment object (by reference), so the outer function's
    /// counters automatically advance past all inner function's allocations.
    pub fn advance_counters_past(&mut self, other: &Environment) {
        self.state.advance_counters_past(&other.state);
    }

    /// Mark this function as having had an effect dependency inferred.
    ///
    /// Corresponds to setting `env.hasInferredEffect = true` in the TS
    /// reference. Used by the entrypoint retry-compilation path to detect
    /// whether re-running the pipeline with `no_inferred_memo` produced any
    /// useful rewrite.
    pub fn mark_has_inferred_effect(&mut self) {
        self.state.has_inferred_effect = true;
    }

    /// Returns whether this function had at least one effect dependency
    /// inferred during the pass.
    pub fn has_inferred_effect(&self) -> bool {
        self.state.has_inferred_effect
    }

    /// Log errors from a validation pass result. If the result is Err, the errors
    /// are collected into the environment's diagnostics list rather than propagated.
    ///
    /// This matches the TS `env.logErrors(result)` pattern used for lint-mode
    /// validation passes.
    pub fn log_errors(&mut self, result: Result<(), crate::compiler_error::CompilerError>) {
        if let Err(error) = result {
            self.state.diagnostics.push(error);
        }
    }

    /// Retrieve and clear all collected diagnostics.
    pub fn take_diagnostics(&mut self) -> Vec<crate::compiler_error::CompilerError> {
        std::mem::take(&mut self.state.diagnostics)
    }

    /// Record errors from a validation pass result. If the result is Err, the errors
    /// are accumulated on the environment so the pipeline can continue through codegen.
    /// These are checked at the end of the pipeline via `take_recorded_errors()`.
    ///
    /// This matches the TS `env.recordError(detail)` pattern used by validation passes
    /// like `validateNoRefAccessInRender` and `validatePreservedManualMemoization`.
    pub fn record_errors(&mut self, result: Result<(), crate::compiler_error::CompilerError>) {
        if let Err(error) = result {
            self.state.recorded_errors.push(error);
        }
    }

    /// Routes a validation outcome to one of two error buffers.
    ///
    /// ```text
    /// ErrorChannel::Diagnostic   → env.state.diagnostics      (never blocks codegen)
    /// ErrorChannel::RecordedFatal → env.state.recorded_errors  (pipeline continues,
    ///                               but final codegen output is discarded if any exist)
    /// ```
    ///
    /// Use the typed channel when the routing decision is made outside the call
    /// site (e.g. passed from a caller) or when you want to be explicit about the
    /// policy.  For the common "lint-mode → diagnostic, otherwise → recorded-fatal"
    /// pattern use `report_lint_aware` instead.
    pub fn report(
        &mut self,
        channel: ErrorChannel,
        result: Result<(), crate::compiler_error::CompilerError>,
    ) {
        match channel {
            ErrorChannel::Diagnostic => self.log_errors(result),
            ErrorChannel::RecordedFatal => self.record_errors(result),
        }
    }

    /// Demotes a validation result to a diagnostic in Lint output mode; records
    /// it as a fatal (pipeline-blocking) error otherwise.
    ///
    /// This is the standard routing for validators that the TS reference
    /// intentionally runs in all modes but treats as non-fatal in lint mode.
    /// Equivalent to:
    /// ```rust,ignore
    /// if env.output_mode().is_lint() {
    ///     env.log_errors(result);
    /// } else {
    ///     env.record_errors(result);
    /// }
    /// ```
    pub fn report_lint_aware(&mut self, result: Result<(), crate::compiler_error::CompilerError>) {
        let channel = if self.output_mode().is_lint() {
            ErrorChannel::Diagnostic
        } else {
            ErrorChannel::RecordedFatal
        };
        self.report(channel, result);
    }

    /// Returns true if any errors have been recorded via `record_errors`.
    pub fn has_recorded_errors(&self) -> bool {
        !self.state.recorded_errors.is_empty()
    }

    /// Retrieve and clear all recorded errors, aggregated into a single `CompilerError`.
    pub fn take_recorded_errors(&mut self) -> Option<crate::compiler_error::CompilerError> {
        if self.state.recorded_errors.is_empty() {
            return None;
        }
        let errors = std::mem::take(&mut self.state.recorded_errors);
        let mut combined = crate::compiler_error::CompilerError::new();
        for error in errors {
            combined.merge(error);
        }
        Some(combined)
    }

    /// Register a function to be outlined (extracted) from this function.
    ///
    /// Corresponds to `Environment.outlineFunction()` in the TS version.
    pub fn outline_function(&mut self, func: HIRFunction, fn_type: Option<ReactFunctionType>) {
        self.state.outlined_functions.push(OutlinedFunctionEntry { func, fn_type });
    }

    /// Get the list of outlined functions.
    ///
    /// Corresponds to `Environment.getOutlinedFunctions()` in the TS version.
    pub fn get_outlined_functions(&self) -> &[OutlinedFunctionEntry] {
        &self.state.outlined_functions
    }

    /// Generate a globally unique identifier name.
    ///
    /// Corresponds to `Environment.generateGloballyUniqueIdentifierName()` in
    /// the TS version. In the TS version this delegates to Babel's
    /// `scope.generateUidIdentifier()` which produces names like `_name`,
    /// `_name2`, etc.
    ///
    /// Babel's `generateUid` applies `toIdentifier()` to sanitize the hint
    /// into a valid JS identifier (replacing brackets, dots, angle brackets,
    /// spaces, etc. with camelCase), then strips leading underscores and
    /// trailing digits before prefixing with `_`.
    pub fn generate_globally_unique_identifier_name(
        &mut self,
        hint: Option<&str>,
    ) -> IdentifierName {
        let base = hint.unwrap_or("temp");
        // Mimic Babel's generateUid: toIdentifier(name).replace(/^_+/, "").replace(/\d+$/g, "")
        let sanitized = to_identifier(base);
        let stripped = sanitized.trim_start_matches('_');
        let stripped = stripped.trim_end_matches(|c: char| c.is_ascii_digit());
        let stripped = if stripped.is_empty() { "temp" } else { stripped };
        let prefix = format!("_{stripped}");
        let mut candidate = prefix.clone();
        loop {
            if !self.state.known_referenced_names.contains(&candidate) {
                break;
            }
            self.state.next_uid += 1;
            candidate = format!("{prefix}{}", self.state.next_uid);
        }
        self.state.known_referenced_names.insert(candidate.clone());
        IdentifierName::Named(candidate)
    }

    /// Register a name as known/referenced, preventing it from being
    /// used by `generate_globally_unique_identifier_name`.
    ///
    /// Corresponds to `ProgramContext.addNewReference()` in the TS version.
    pub fn add_new_reference(&mut self, name: &str) {
        self.state.known_referenced_names.insert(name.to_string());
    }

    /// Seed known referenced names from an external source (e.g., `ProgramContext`).
    ///
    /// This ensures that identifiers generated by previous function compilations
    /// are known, preventing duplicate names when multiple functions are compiled
    /// in the same file.
    pub fn seed_known_referenced_names(&mut self, names: &FxHashSet<String>) {
        self.state.known_referenced_names.extend(names.iter().cloned());
    }

    /// Get the known referenced names (for propagating back to `ProgramContext`).
    pub fn known_referenced_names(&self) -> &FxHashSet<String> {
        &self.state.known_referenced_names
    }

    // =========================================================================
    // Global and type lookup methods
    // =========================================================================

    /// Resolve a non-local binding to its global type.
    ///
    /// Port of `Environment.getGlobalDeclaration()` from `HIR/Environment.ts`.
    ///
    /// Takes `&self` because `module_types` is pre-resolved at context-build
    /// time; resolving a module never mutates the shape registry on the hot
    /// path.
    ///
    /// # Errors
    /// Returns a `CompilerError` if a type provider gives a type that is inconsistent
    /// with the hook naming convention (e.g., a hook name mapped to a non-hook type).
    pub fn get_global_declaration(
        &self,
        binding: &NonLocalBinding,
        loc: crate::compiler_error::SourceLocation,
    ) -> Result<Option<Global>, crate::compiler_error::CompilerError> {
        match binding {
            NonLocalBinding::ModuleLocal { name } => {
                if is_hook_name(name) {
                    Ok(Some(self.get_custom_hook_type()))
                } else {
                    Ok(None)
                }
            }
            NonLocalBinding::Global { name } => {
                if let Some(g) = self.globals().get(name) {
                    Ok(Some(g.clone()))
                } else if is_hook_name(name) {
                    Ok(Some(self.get_custom_hook_type()))
                } else {
                    Ok(None)
                }
            }
            NonLocalBinding::ImportSpecifier { name, module, imported } => {
                if is_known_react_module(module) {
                    // For React modules, look up by imported name
                    if let Some(g) = self.globals().get(imported) {
                        Ok(Some(g.clone()))
                    } else if is_hook_name(imported) || is_hook_name(name) {
                        Ok(Some(self.get_custom_hook_type()))
                    } else {
                        Ok(None)
                    }
                } else {
                    // Check module type registry (e.g., react-native-reanimated)
                    if let Some(module_type) = self.resolve_module_type(module) {
                        // Validate all properties of the module for hook name/type consistency.
                        // Port of the validation in TS `installTypeConfig` for "object" kind.
                        self.validate_module_type_properties(&module_type, module, loc)?;

                        if let Some(imported_type) = self.get_property_type(&module_type, imported)
                        {
                            return Ok(Some(Global::Typed(imported_type)));
                        }
                    }

                    // Fall back to hook name pattern
                    if is_hook_name(imported) || is_hook_name(name) {
                        Ok(Some(self.get_custom_hook_type()))
                    } else {
                        Ok(None)
                    }
                }
            }
            NonLocalBinding::ImportDefault { name, module } => {
                if is_known_react_module(module) {
                    if let Some(g) = self.globals().get(name) {
                        Ok(Some(g.clone()))
                    } else if is_hook_name(name) {
                        Ok(Some(self.get_custom_hook_type()))
                    } else {
                        Ok(None)
                    }
                } else {
                    // Check module type registry for default export
                    if let Some(module_type) = self.resolve_module_type(module) {
                        // Validate all properties of the module for hook name/type consistency.
                        self.validate_module_type_properties(&module_type, module, loc)?;

                        if let Some(default_type) = self.get_property_type(&module_type, "default")
                        {
                            // Check that hook-like module names have hook types, and vice versa.
                            let expect_hook = is_hook_name(module);
                            let is_hook = get_hook_kind_for_type(self, &default_type).is_some();
                            if expect_hook != is_hook {
                                return Err(crate::compiler_error::CompilerError::invalid_config(
                                    "Invalid type configuration for module",
                                    Some(&format!(
                                        "Expected type for `import ... from '{module}'` {} based on the module name",
                                        if expect_hook {
                                            "to be a hook"
                                        } else {
                                            "not to be a hook"
                                        }
                                    )),
                                    Some(loc),
                                ));
                            }
                            return Ok(Some(Global::Typed(default_type)));
                        }
                    }
                    if is_hook_name(name) {
                        Ok(Some(self.get_custom_hook_type()))
                    } else {
                        Ok(None)
                    }
                }
            }
            NonLocalBinding::ImportNamespace { name, module } => {
                if is_known_react_module(module) {
                    if let Some(g) = self.globals().get(name) {
                        Ok(Some(g.clone()))
                    } else if is_hook_name(name) {
                        Ok(Some(self.get_custom_hook_type()))
                    } else {
                        Ok(None)
                    }
                } else {
                    // Check module type registry for namespace import
                    if let Some(module_type) = self.resolve_module_type(module) {
                        // Validate all properties of the module for hook name/type consistency.
                        self.validate_module_type_properties(&module_type, module, loc)?;

                        // Check that hook-like module names have hook types, and vice versa.
                        let expect_hook = is_hook_name(module);
                        let is_hook = get_hook_kind_for_type(self, &module_type).is_some();
                        if expect_hook != is_hook {
                            return Err(crate::compiler_error::CompilerError::invalid_config(
                                "Invalid type configuration for module",
                                Some(&format!(
                                    "Expected type for `import ... from '{module}'` {} based on the module name",
                                    if expect_hook { "to be a hook" } else { "not to be a hook" }
                                )),
                                Some(loc),
                            ));
                        }
                        return Ok(Some(Global::Typed(module_type)));
                    }
                    if is_hook_name(name) {
                        Ok(Some(self.get_custom_hook_type()))
                    } else {
                        Ok(None)
                    }
                }
            }
        }
    }

    /// Validate that all properties of a module object type have consistent
    /// hook name / hook type pairings.
    ///
    /// Port of the validation in TS `installTypeConfig` for "object" kind:
    /// iterates over all properties and checks that hook-named properties
    /// have hook types and non-hook-named properties don't.
    fn validate_module_type_properties(
        &self,
        module_type: &Type,
        module_name: &str,
        loc: crate::compiler_error::SourceLocation,
    ) -> Result<(), crate::compiler_error::CompilerError> {
        let shape_id = match module_type {
            Type::Object(ObjectType { shape_id: Some(id) }) => id.as_str(),
            _ => return Ok(()),
        };

        let Some(shape) = self.shapes().get(shape_id) else {
            return Ok(());
        };

        // Collect keys and sort to get deterministic iteration order.
        // The TS code uses Object.entries which preserves insertion order;
        // we sort to ensure consistent behavior.
        let mut keys: Vec<&String> = shape.properties.keys().collect();
        keys.sort();

        for key in keys {
            let Some(prop_type) = shape.properties.get(key) else {
                continue;
            };
            let expect_hook = is_hook_name(key);
            let is_hook = get_hook_kind_for_type(self, prop_type).is_some();
            if expect_hook != is_hook {
                return Err(crate::compiler_error::CompilerError::invalid_config(
                    "Invalid type configuration for module",
                    Some(&format!(
                        "Expected type for object property '{key}' from module '{module_name}' {} based on the property name",
                        if expect_hook { "to be a hook" } else { "not to be a hook" }
                    )),
                    Some(loc),
                ));
            }
        }

        Ok(())
    }

    /// Resolve a module name to its type definition, if one is registered.
    ///
    /// Port of `#resolveModuleType()` from `HIR/Environment.ts`.
    ///
    /// Pure lookup: every module known to the configured type providers
    /// (config-gated test/runtime modules + the `DefaultModuleTypeProvider`
    /// catalogue) is pre-resolved into `module_types` at context-build time,
    /// so this method never mutates the shape registry.
    fn resolve_module_type(&self, module_name: &str) -> Option<Type> {
        self.module_types().get(module_name).cloned()
    }

    /// Look up a property type from the shape registry.
    ///
    /// Port of `Environment.getPropertyType()` from `HIR/Environment.ts`.
    ///
    /// Lookup order for string properties: exact name → wildcard ('*') → hook pattern.
    pub fn get_property_type(&self, receiver: &Type, property: &str) -> Option<Type> {
        let shape_id = match receiver {
            Type::Object(ObjectType { shape_id: Some(id) })
            | Type::Function(FunctionType { shape_id: Some(id), .. }) => Some(id.as_str()),
            _ => None,
        };

        if let Some(shape_id) = shape_id
            && let Some(shape) = self.shapes().get(shape_id)
        {
            // Try exact property name, then wildcard, then hook pattern
            if let Some(t) = shape.properties.get(property) {
                return Some(t.clone());
            }
            if let Some(t) = shape.properties.get("*") {
                return Some(t.clone());
            }
            if is_hook_name(property) {
                return Some(Global::to_type(&self.get_custom_hook_type()));
            }
            return None;
        }

        // No shape: only check hook pattern for string properties
        if is_hook_name(property) {
            return Some(Global::to_type(&self.get_custom_hook_type()));
        }
        None
    }

    /// Get the fallthrough (wildcard) property type for a computed property access.
    ///
    /// Port of `Environment.getFallthroughPropertyType()` from `HIR/Environment.ts`.
    /// For computed property accesses like `obj[idx]`, only looks up the `*` wildcard
    /// property on the receiver's shape (ignoring the specific property value).
    pub fn get_fallthrough_property_type(&self, receiver: &Type) -> Option<Type> {
        let shape_id = match receiver {
            Type::Object(ObjectType { shape_id: Some(id) })
            | Type::Function(FunctionType { shape_id: Some(id), .. }) => Some(id.as_str()),
            _ => None,
        };

        if let Some(shape_id) = shape_id
            && let Some(shape) = self.shapes().get(shape_id)
        {
            return shape.properties.get("*").cloned();
        }
        None
    }

    /// Get the function signature from a function type's shape.
    ///
    /// Port of `Environment.getFunctionSignature()` from `HIR/Environment.ts`.
    pub fn get_function_signature(&self, type_: &Type) -> Option<&FunctionSignature> {
        let shape_id = match type_ {
            Type::Function(FunctionType { shape_id: Some(id), .. }) => Some(id.as_str()),
            _ => None,
        };

        if let Some(shape_id) = shape_id
            && let Some(shape) = self.shapes().get(shape_id)
        {
            return shape.function_type.as_ref();
        }
        None
    }

    /// Get the default hook type for unrecognized hooks.
    ///
    /// Corresponds to `#getCustomHookType()` in the TS version.
    /// When `enableAssumeHooksFollowRulesOfReact` is true (default), returns
    /// `DefaultNonmutatingHook` (arguments frozen, return frozen).
    /// When false, returns `DefaultMutatingHook` (arguments conditionally mutated,
    /// return mutable).
    fn get_custom_hook_type(&self) -> Global {
        let shape_id = if self.config().enable_assume_hooks_follow_rules_of_react {
            BUILT_IN_DEFAULT_NONMUTATING_HOOK_ID
        } else {
            BUILT_IN_DEFAULT_MUTATING_HOOK_ID
        };
        Global::Typed(Type::Function(FunctionType {
            shape_id: Some(shape_id.to_string()),
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        }))
    }
}

/// Check if a name matches the React hook naming convention: `use[A-Z0-9]`.
pub fn is_hook_name(name: &str) -> bool {
    if let Some(rest) = name.strip_prefix("use") {
        rest.starts_with(|c: char| c.is_ascii_uppercase() || c.is_ascii_digit())
    } else {
        false
    }
}

/// Convert a string to a valid JavaScript identifier.
///
/// Port of Babel's `toIdentifier()` from `@babel/types/src/converters/toIdentifier.ts`.
///
/// 1. Replaces all non-identifier characters with `-`
/// 2. Removes leading dashes and digits
/// 3. CamelCases dash-separated segments (and collapses whitespace/dashes)
/// 4. Prefixes with `_` if result is not a valid identifier start
fn to_identifier(input: &str) -> String {
    // Step 1: Replace all non-identifier characters with `-`
    let mut dashed = String::with_capacity(input.len());
    for c in input.chars() {
        if is_identifier_char(c) {
            dashed.push(c);
        } else {
            dashed.push('-');
        }
    }

    // Step 2: Remove leading dashes and digits
    let trimmed = dashed.trim_start_matches(|c: char| c == '-' || c.is_ascii_digit());

    // Step 3: CamelCase — replace /[-\s]+(.)?/g with uppercased capture
    let mut result = String::with_capacity(trimmed.len());
    let mut capitalize_next = false;
    for c in trimmed.chars() {
        if c == '-' || c.is_whitespace() {
            capitalize_next = true;
        } else if capitalize_next {
            // Uppercase the first char after a dash/space sequence
            for uc in c.to_uppercase() {
                result.push(uc);
            }
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    // Step 4: If still not valid identifier start, prefix with _
    if result.is_empty() {
        return "_".to_string();
    }
    let first = result.chars().next().unwrap();
    if !is_identifier_start(first) {
        return format!("_{result}");
    }

    result
}

/// Check if a character is a valid JS identifier continuation character.
///
/// Matches Unicode ID_Continue plus `$` and `_` (like Babel's `isIdentifierChar`).
fn is_identifier_char(c: char) -> bool {
    // Simple check: ASCII alphanumeric, _, $, or Unicode letter/digit
    c == '_' || c == '$' || c.is_alphanumeric()
}

/// Check if a character is a valid JS identifier start character.
fn is_identifier_start(c: char) -> bool {
    c == '_' || c == '$' || c.is_alphabetic()
}

/// Check if a module name is a known React module.
fn is_known_react_module(module: &str) -> bool {
    let lower = module.cow_to_lowercase();
    lower == "react" || lower == "react-dom"
}

impl Global {
    /// Convert a Global to a Type (extracting the inner type if Typed).
    pub fn to_type(global: &Global) -> Type {
        match global {
            Global::Typed(t) => t.clone(),
            Global::Untyped => Type::Poly,
        }
    }
}

/// Get the hook kind for a given type, if it is a hook.
pub fn get_hook_kind_for_type(env: &Environment, type_: &Type) -> Option<HookKind> {
    env.get_function_signature(type_).and_then(|sig| sig.hook_kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: every `Option<bool>` state of
    /// `enable_new_mutation_aliasing_model` must resolve to `true` after
    /// `Environment::new`, because the Rust port has only the new-aliasing
    /// model implemented and `run_pipeline` `debug_assert!`s the resolved
    /// value. Without this invariant, fixtures carrying
    /// `@enableNewMutationAliasingModel:false` (which the pragma parser
    /// translates to `Some(false)`) would panic in debug builds.
    #[test]
    fn new_mutation_aliasing_model_normalizes_to_true() {
        for initial in [None, Some(true), Some(false)] {
            let config = EnvironmentConfig {
                enable_new_mutation_aliasing_model: initial,
                ..EnvironmentConfig::default()
            };
            let env =
                Environment::new(ReactFunctionType::Component, CompilerOutputMode::Client, config)
                    .expect("default-shaped EnvironmentConfig must build");
            assert!(
                env.enable_new_mutation_aliasing_model(),
                "expected accessor to return true for initial = {initial:?}"
            );
            assert_eq!(
                env.config().enable_new_mutation_aliasing_model,
                Some(true),
                "expected config field to be collapsed to Some(true) for initial = {initial:?}",
            );
        }
    }
}
