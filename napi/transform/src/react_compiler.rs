//! React Compiler options for the transform binding.
//!
//! A JS-facing mirror of the React Compiler `PluginOptions`, resolved into the concrete
//! options the transformer consumes.
//!
//! `environment` mirrors the option upstream's Babel plugin leads with
//! (`environment: Partial<EnvironmentConfig>`); without it a caller cannot reach the
//! validation and inference passes that are off by default, which the compiler would
//! then carry as unreachable code. The flags there are plain toggles; the five
//! composite entries (`customHooks`, `moduleTypeProvider`, `enableEmitHookGuards`,
//! `enableEmitInstrumentForget`, `validateExhaustiveEffectDependencies`) are not
//! surfaced, nor is `throwUnknownExceptionTestonly`, which exists to make the compiler
//! panic in upstream's own tests.
//!
//! The option types below are deliberately **not** gated on the `react_compiler`
//! feature, for two reasons: `#[napi(object)]` ignores `#[cfg]` on fields (it emits
//! them into the generated `FromNapiValue`/`ToNapiValue` regardless), and keeping the
//! types unconditional makes the generated `index.d.ts` identical in every feature
//! config, so a lean build can never drift from the checked-in one. Only the
//! conversion into the compiler's `PluginOptions` is gated; without the feature,
//! `is_enabled` lets the caller reject the option rather than silently ignore it.

use napi::Either;
use napi_derive::napi;

#[cfg(feature = "react_compiler")]
use std::str::FromStr;

#[cfg(feature = "react_compiler")]
use oxc::react_compiler::{
    CompilerTarget, DynamicGatingConfig, EnvironmentConfig, GatingConfig, PluginOptions,
};

/// Options for the experimental [React Compiler](https://github.com/react/react/tree/main/compiler).
///
/// Mirrors the compiler's `PluginOptions`.
///
/// @see {@link PluginsOptions#reactCompiler}
#[napi(object)]
#[derive(Default, Debug)]
pub struct ReactCompilerOptions {
    /// Which functions to compile.
    ///
    /// @default 'infer'
    #[napi(ts_type = "'infer' | 'syntax' | 'annotation' | 'all'")]
    pub compilation_mode: Option<String>,

    /// What to do when a function cannot be compiled.
    ///
    /// @default 'none'
    #[napi(ts_type = "'none' | 'critical_errors' | 'all_errors'")]
    pub panic_threshold: Option<String>,

    /// React runtime version target. `17` and `18` require the
    /// `react-compiler-runtime` package; `19` ships the runtime in `react`.
    ///
    /// @default '19'
    #[napi(ts_type = "'17' | '18' | '19'")]
    pub target: Option<String>,

    /// Analyze and report diagnostics only; emit no transformed code.
    ///
    /// @default false
    pub no_emit: Option<bool>,

    /// Compiler output mode.
    ///
    /// @default undefined
    #[napi(ts_type = "'client' | 'ssr' | 'lint'")]
    pub output_mode: Option<String>,

    /// Compile even functions marked with the `"use no memo"` / `"use no forget"`
    /// opt-out directives.
    ///
    /// @default false
    pub ignore_use_no_forget: Option<bool>,

    /// Treat Flow suppression comments as opt-outs.
    ///
    /// @default true
    pub flow_suppressions: Option<bool>,

    /// ESLint rules whose suppressions opt a function out of compilation.
    pub eslint_suppression_rules: Option<Vec<String>>,

    /// Extra directives that opt a function out of compilation.
    pub custom_opt_out_directives: Option<Vec<String>>,

    /// Also emit a gated (feature-flagged) version of each compiled function.
    pub gating: Option<ReactCompilerGating>,

    /// Dynamically-gated compilation.
    pub dynamic_gating: Option<ReactCompilerDynamicGating>,

    /// Feature flags and validation toggles for the compilation passes.
    ///
    /// Each field left unset keeps the compiler's own default, matching the
    /// `Partial<EnvironmentConfig>` shape of the upstream Babel plugin's option.
    pub environment: Option<ReactCompilerEnvironmentOptions>,
}

/// Static gating for {@link ReactCompilerOptions#gating}.
#[napi(object)]
#[derive(Debug)]
pub struct ReactCompilerGating {
    /// Module the gating import comes from.
    pub source: String,
    /// Imported specifier used as the gate.
    pub import_specifier_name: String,
}

/// Dynamic gating for {@link ReactCompilerOptions#dynamicGating}.
#[napi(object)]
#[derive(Debug)]
pub struct ReactCompilerDynamicGating {
    /// Module the gating import comes from.
    pub source: String,
}

/// Feature flags and validation toggles for the React Compiler's passes.
///
/// Field names mirror the upstream `EnvironmentConfig` one-for-one, so a value that
/// works in a Babel `environment` config works here unchanged. Every field is optional;
/// omitting one keeps the compiler's default. Several of these gate passes that are off
/// by default, so setting them is the only way to reach those diagnostics.
///
/// @see {@link ReactCompilerOptions#environment}
#[napi(object)]
#[derive(Default, Debug)]
pub struct ReactCompilerEnvironmentOptions {
    pub custom_macros: Option<Vec<String>>,
    pub enable_reset_cache_on_source_file_changes: Option<bool>,
    pub enable_preserve_existing_memoization_guarantees: Option<bool>,
    pub validate_preserve_existing_memoization_guarantees: Option<bool>,
    pub validate_exhaustive_memoization_dependencies: Option<bool>,
    pub enable_optional_dependencies: Option<bool>,
    pub enable_name_anonymous_functions: Option<bool>,
    pub validate_hooks_usage: Option<bool>,
    pub validate_ref_access_during_render: Option<bool>,
    pub validate_no_set_state_in_render: Option<bool>,
    pub enable_use_keyed_state: Option<bool>,
    pub validate_no_set_state_in_effects: Option<bool>,
    pub validate_no_derived_computations_in_effects: Option<bool>,
    pub validate_no_derived_computations_in_effects_exp: Option<bool>,
    pub validate_no_jsx_in_try_statements: Option<bool>,
    pub validate_static_components: Option<bool>,
    pub validate_no_capitalized_calls: Option<Vec<String>>,
    pub validate_blocklisted_imports: Option<Vec<String>>,
    pub validate_source_locations: Option<bool>,
    pub validate_no_impure_functions_in_render: Option<bool>,
    pub validate_no_freezing_known_mutable_functions: Option<bool>,
    pub enable_assume_hooks_follow_rules_of_react: Option<bool>,
    pub enable_transitively_freeze_function_expressions: Option<bool>,
    pub enable_function_outlining: Option<bool>,
    pub enable_jsx_outlining: Option<bool>,
    pub assert_valid_mutable_ranges: Option<bool>,
    pub enable_custom_type_definition_for_reanimated: Option<bool>,
    pub enable_treat_ref_like_identifiers_as_refs: Option<bool>,
    pub enable_treat_set_identifiers_as_state_setters: Option<bool>,
    pub validate_no_void_use_memo: Option<bool>,
    pub enable_allow_set_state_from_refs_in_effects: Option<bool>,
    pub enable_verbose_no_set_state_in_effect: Option<bool>,
    pub enable_forest: Option<bool>,
}

#[cfg(feature = "react_compiler")]
impl ReactCompilerEnvironmentOptions {
    /// Overlay the caller's flags onto the compiler's defaults, leaving unset fields alone.
    fn apply_to(self, env: &mut EnvironmentConfig) {
        if self.custom_macros.is_some() {
            env.custom_macros = self.custom_macros;
        }
        if self.enable_reset_cache_on_source_file_changes.is_some() {
            env.enable_reset_cache_on_source_file_changes =
                self.enable_reset_cache_on_source_file_changes;
        }
        if let Some(v) = self.enable_preserve_existing_memoization_guarantees {
            env.enable_preserve_existing_memoization_guarantees = v;
        }
        if let Some(v) = self.validate_preserve_existing_memoization_guarantees {
            env.validate_preserve_existing_memoization_guarantees = v;
        }
        if let Some(v) = self.validate_exhaustive_memoization_dependencies {
            env.validate_exhaustive_memoization_dependencies = v;
        }
        if let Some(v) = self.enable_optional_dependencies {
            env.enable_optional_dependencies = v;
        }
        if let Some(v) = self.enable_name_anonymous_functions {
            env.enable_name_anonymous_functions = v;
        }
        if let Some(v) = self.validate_hooks_usage {
            env.validate_hooks_usage = v;
        }
        if let Some(v) = self.validate_ref_access_during_render {
            env.validate_ref_access_during_render = v;
        }
        if let Some(v) = self.validate_no_set_state_in_render {
            env.validate_no_set_state_in_render = v;
        }
        if let Some(v) = self.enable_use_keyed_state {
            env.enable_use_keyed_state = v;
        }
        if let Some(v) = self.validate_no_set_state_in_effects {
            env.validate_no_set_state_in_effects = v;
        }
        if let Some(v) = self.validate_no_derived_computations_in_effects {
            env.validate_no_derived_computations_in_effects = v;
        }
        if let Some(v) = self.validate_no_derived_computations_in_effects_exp {
            env.validate_no_derived_computations_in_effects_exp = v;
        }
        if let Some(v) = self.validate_no_jsx_in_try_statements {
            env.validate_no_jsx_in_try_statements = v;
        }
        if let Some(v) = self.validate_static_components {
            env.validate_static_components = v;
        }
        if self.validate_no_capitalized_calls.is_some() {
            env.validate_no_capitalized_calls = self.validate_no_capitalized_calls;
        }
        if self.validate_blocklisted_imports.is_some() {
            env.validate_blocklisted_imports = self.validate_blocklisted_imports;
        }
        if let Some(v) = self.validate_source_locations {
            env.validate_source_locations = v;
        }
        if let Some(v) = self.validate_no_impure_functions_in_render {
            env.validate_no_impure_functions_in_render = v;
        }
        if let Some(v) = self.validate_no_freezing_known_mutable_functions {
            env.validate_no_freezing_known_mutable_functions = v;
        }
        if let Some(v) = self.enable_assume_hooks_follow_rules_of_react {
            env.enable_assume_hooks_follow_rules_of_react = v;
        }
        if let Some(v) = self.enable_transitively_freeze_function_expressions {
            env.enable_transitively_freeze_function_expressions = v;
        }
        if let Some(v) = self.enable_function_outlining {
            env.enable_function_outlining = v;
        }
        if let Some(v) = self.enable_jsx_outlining {
            env.enable_jsx_outlining = v;
        }
        if let Some(v) = self.assert_valid_mutable_ranges {
            env.assert_valid_mutable_ranges = v;
        }
        if let Some(v) = self.enable_custom_type_definition_for_reanimated {
            env.enable_custom_type_definition_for_reanimated = v;
        }
        if let Some(v) = self.enable_treat_ref_like_identifiers_as_refs {
            env.enable_treat_ref_like_identifiers_as_refs = v;
        }
        if let Some(v) = self.enable_treat_set_identifiers_as_state_setters {
            env.enable_treat_set_identifiers_as_state_setters = v;
        }
        if let Some(v) = self.validate_no_void_use_memo {
            env.validate_no_void_use_memo = v;
        }
        if let Some(v) = self.enable_allow_set_state_from_refs_in_effects {
            env.enable_allow_set_state_from_refs_in_effects = v;
        }
        if let Some(v) = self.enable_verbose_no_set_state_in_effect {
            env.enable_verbose_no_set_state_in_effect = v;
        }
        if let Some(v) = self.enable_forest {
            env.enable_forest = v;
        }
    }
}

/// Whether the `reactCompiler` option asks for the compiler to run. `false` and an
/// absent option both mean "disabled", and must not be treated as a request.
#[cfg(not(feature = "react_compiler"))]
pub fn is_enabled(option: Option<&Either<bool, ReactCompilerOptions>>) -> bool {
    matches!(option, Some(Either::A(true) | Either::B(_)))
}

/// Resolve the JS `reactCompiler` option into the compiler's `PluginOptions`:
/// `true` → default options, an object → those options, `false`/absent → disabled.
///
/// # Errors
///
/// Returns an error if a string-valued option is not one of its documented variants.
#[cfg(feature = "react_compiler")]
pub fn resolve(
    option: Option<Either<bool, ReactCompilerOptions>>,
) -> Result<Option<PluginOptions>, String> {
    match option {
        Some(Either::A(true)) => Ok(Some(PluginOptions::default())),
        Some(Either::B(options)) => options.into_plugin_options().map(Some),
        Some(Either::A(false)) | None => Ok(None),
    }
}

/// Parse a string-valued option into the enum the compiler takes. The `ts_type`
/// annotations constrain these at the type level only, so a plain-JS caller can still
/// reach this with anything.
#[cfg(feature = "react_compiler")]
fn parse<T: FromStr>(value: &str, option: &str) -> Result<T, String> {
    T::from_str(value)
        .map_err(|_| format!("Invalid plugins.reactCompiler.{option} option: `{value}`."))
}

#[cfg(feature = "react_compiler")]
impl ReactCompilerOptions {
    fn into_plugin_options(self) -> Result<PluginOptions, String> {
        let mut options = PluginOptions::default();
        if let Some(compilation_mode) = self.compilation_mode {
            options.compilation_mode = parse(&compilation_mode, "compilationMode")?;
        }
        if let Some(panic_threshold) = self.panic_threshold {
            options.panic_threshold = parse(&panic_threshold, "panicThreshold")?;
        }
        if let Some(target) = self.target {
            // `CompilerTarget::Version` takes any string and silently falls back to the
            // React 19 runtime for unrecognized ones, so reject typos here instead.
            if !matches!(target.as_str(), "17" | "18" | "19") {
                return Err(format!("Invalid plugins.reactCompiler.target option: `{target}`."));
            }
            options.target = CompilerTarget::Version(target);
        }
        if let Some(no_emit) = self.no_emit {
            options.no_emit = no_emit;
        }
        if let Some(output_mode) = self.output_mode {
            options.output_mode = Some(parse(&output_mode, "outputMode")?);
        }
        if let Some(ignore_use_no_forget) = self.ignore_use_no_forget {
            options.ignore_use_no_forget = ignore_use_no_forget;
        }
        if let Some(flow_suppressions) = self.flow_suppressions {
            options.flow_suppressions = flow_suppressions;
        }
        if self.eslint_suppression_rules.is_some() {
            options.eslint_suppression_rules = self.eslint_suppression_rules;
        }
        if self.custom_opt_out_directives.is_some() {
            options.custom_opt_out_directives = self.custom_opt_out_directives;
        }
        if let Some(gating) = self.gating {
            options.gating = Some(GatingConfig {
                source: gating.source,
                import_specifier_name: gating.import_specifier_name,
            });
        }
        if let Some(dynamic_gating) = self.dynamic_gating {
            options.dynamic_gating = Some(DynamicGatingConfig { source: dynamic_gating.source });
        }
        if let Some(environment) = self.environment {
            environment.apply_to(&mut options.environment);
        }
        Ok(options)
    }
}
