use std::str::FromStr;

use napi::Either;
use napi_derive::napi;

use oxc::diagnostics::OxcDiagnostic;
use oxc_react_compiler::{
    CompilerTarget, DynamicGatingConfig, EnvironmentConfig, ExhaustiveEffectDepsMode, GatingConfig,
    PluginOptions,
};

/// Options for compiling a JavaScript or TypeScript React module.
///
/// React Compiler fields mirror `babel-plugin-react-compiler` and
/// `react-compiler-napi`. `lang`, `sourceType`, and `sourcemap` configure the
/// surrounding Oxc parse/codegen pipeline.
#[napi(object)]
#[derive(Default, Debug)]
pub struct TransformOptions {
    /// Treat the source as `js`, `jsx`, `ts`, `tsx`, or `dts`.
    #[napi(ts_type = "'js' | 'jsx' | 'ts' | 'tsx' | 'dts'")]
    pub lang: Option<String>,

    /// Treat the source as script, module, CommonJS, or infer it from syntax.
    #[napi(ts_type = "'script' | 'module' | 'commonjs' | 'unambiguous'")]
    pub source_type: Option<String>,

    /// Generate a source map.
    ///
    /// @default false
    pub sourcemap: Option<bool>,

    /// Which functions the compiler attempts to compile.
    ///
    /// @default 'infer'
    #[napi(ts_type = "'infer' | 'syntax' | 'annotation' | 'all'")]
    pub compilation_mode: Option<String>,

    /// When compiler diagnostics escalate into a hard failure.
    ///
    /// @default 'none'
    #[napi(ts_type = "'none' | 'critical_errors' | 'all_errors'")]
    pub panic_threshold: Option<String>,

    /// React runtime target. React 17 and 18 use `react-compiler-runtime`;
    /// React 19 uses `react/compiler-runtime`.
    ///
    /// @default '19'
    #[napi(ts_type = "'17' | '18' | '19' | ReactCompilerMetaTarget")]
    pub target: Option<Either<String, ReactCompilerMetaTarget>>,

    /// Emit both compiled and original functions behind an imported feature gate.
    pub gating: Option<ReactCompilerGating>,

    /// Enable `"use memo if(...)"` directive-driven gating.
    pub dynamic_gating: Option<ReactCompilerDynamicGating>,

    /// Analyze and report diagnostics without applying compiler output.
    ///
    /// @deprecated Prefer `outputMode: "lint"`.
    /// @default false
    pub no_emit: Option<bool>,

    /// Select client, SSR, or lint output.
    #[napi(ts_type = "'client' | 'ssr' | 'lint'")]
    pub output_mode: Option<String>,

    /// ESLint rule names whose suppressions opt a function out of compilation.
    pub eslint_suppression_rules: Option<Vec<String>>,

    /// Treat Flow suppression comments as opt-outs.
    ///
    /// @default true
    pub flow_suppressions: Option<bool>,

    /// Compile functions carrying `"use no memo"` or `"use no forget"`.
    ///
    /// @default false
    pub ignore_use_no_forget: Option<bool>,

    /// Additional directives that opt a function out of compilation.
    pub custom_opt_out_directives: Option<Vec<String>>,

    /// Only run the React Compiler when the filename contains one of these strings.
    ///
    /// Function-valued `sources` filters from the Babel plugin are intentionally
    /// unsupported across the native boundary.
    pub sources: Option<Vec<String>>,

    /// Feature flags and validation settings for compiler passes.
    pub environment: Option<ReactCompilerEnvironmentOptions>,
}

/// Meta-internal React runtime target.
#[napi(object)]
#[derive(Debug)]
pub struct ReactCompilerMetaTarget {
    #[napi(ts_type = "'donotuse_meta_internal'")]
    pub kind: String,
    pub runtime_module: Option<String>,
}

/// Static feature-gating import.
#[napi(object)]
#[derive(Debug)]
pub struct ReactCompilerGating {
    pub source: String,
    pub import_specifier_name: String,
}

/// Dynamic feature-gating import.
#[napi(object)]
#[derive(Debug)]
pub struct ReactCompilerDynamicGating {
    pub source: String,
}

/// Partial React Compiler environment configuration.
///
/// Unset fields retain compiler defaults. Callback-valued providers and the
/// compiler's test-only panic switch are intentionally not exposed.
#[napi(object)]
#[derive(Default, Debug)]
pub struct ReactCompilerEnvironmentOptions {
    pub custom_macros: Option<Vec<String>>,
    pub enable_reset_cache_on_source_file_changes: Option<bool>,
    pub enable_preserve_existing_memoization_guarantees: Option<bool>,
    pub validate_preserve_existing_memoization_guarantees: Option<bool>,
    pub validate_exhaustive_memoization_dependencies: Option<bool>,
    #[napi(ts_type = "'off' | 'all' | 'missing-only' | 'extra-only'")]
    pub validate_exhaustive_effect_dependencies: Option<String>,
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

impl TransformOptions {
    pub(crate) fn into_transform_options(
        self,
        filename: &str,
    ) -> Result<oxc::transformer::TransformOptions, OxcDiagnostic> {
        let enabled = self
            .sources
            .as_ref()
            .is_none_or(|sources| sources.iter().any(|source| filename.contains(source.as_str())));
        let react_compiler = enabled.then(|| self.into_plugin_options()).transpose()?;

        Ok(oxc::transformer::TransformOptions {
            react_compiler,
            jsx: oxc::transformer::JsxOptions::enable(),
            ..oxc::transformer::TransformOptions::default()
        })
    }

    fn into_plugin_options(self) -> Result<PluginOptions, OxcDiagnostic> {
        let mut options = PluginOptions::default();

        if let Some(value) = self.compilation_mode {
            options.compilation_mode = parse(&value, "compilationMode")?;
        }
        if let Some(value) = self.panic_threshold {
            options.panic_threshold = parse(&value, "panicThreshold")?;
        }
        if let Some(target) = self.target {
            options.target = match target {
                Either::A(version) => {
                    if !matches!(version.as_str(), "17" | "18" | "19") {
                        return Err(invalid_option("target", &version));
                    }
                    CompilerTarget::Version(version)
                }
                Either::B(target) => {
                    if target.kind != "donotuse_meta_internal" {
                        return Err(invalid_option("target.kind", &target.kind));
                    }
                    CompilerTarget::MetaInternal {
                        kind: target.kind,
                        runtime_module: target
                            .runtime_module
                            .unwrap_or_else(|| "react".to_string()),
                    }
                }
            };
        }
        if let Some(value) = self.no_emit {
            options.no_emit = value;
        }
        if let Some(value) = self.output_mode {
            options.output_mode = Some(parse(&value, "outputMode")?);
        }
        if let Some(value) = self.flow_suppressions {
            options.flow_suppressions = value;
        }
        if let Some(value) = self.ignore_use_no_forget {
            options.ignore_use_no_forget = value;
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
            environment.apply_to(&mut options.environment)?;
        }

        Ok(options)
    }
}

impl ReactCompilerEnvironmentOptions {
    fn apply_to(self, environment: &mut EnvironmentConfig) -> Result<(), OxcDiagnostic> {
        macro_rules! apply_bool_options {
            ($($field:ident),+ $(,)?) => {
                $(
                    if let Some(value) = self.$field {
                        environment.$field = value;
                    }
                )+
            };
        }

        if self.custom_macros.is_some() {
            environment.custom_macros = self.custom_macros;
        }
        if self.enable_reset_cache_on_source_file_changes.is_some() {
            environment.enable_reset_cache_on_source_file_changes =
                self.enable_reset_cache_on_source_file_changes;
        }
        if let Some(value) = self.validate_exhaustive_effect_dependencies {
            environment.validate_exhaustive_effect_dependencies = match value.as_str() {
                "off" => ExhaustiveEffectDepsMode::Off,
                "all" => ExhaustiveEffectDepsMode::All,
                "missing-only" => ExhaustiveEffectDepsMode::MissingOnly,
                "extra-only" => ExhaustiveEffectDepsMode::ExtraOnly,
                _ => {
                    return Err(invalid_option(
                        "environment.validateExhaustiveEffectDependencies",
                        &value,
                    ));
                }
            };
        }
        if self.validate_no_capitalized_calls.is_some() {
            environment.validate_no_capitalized_calls = self.validate_no_capitalized_calls;
        }
        if self.validate_blocklisted_imports.is_some() {
            environment.validate_blocklisted_imports = self.validate_blocklisted_imports;
        }

        apply_bool_options!(
            enable_preserve_existing_memoization_guarantees,
            validate_preserve_existing_memoization_guarantees,
            validate_exhaustive_memoization_dependencies,
            enable_optional_dependencies,
            enable_name_anonymous_functions,
            validate_hooks_usage,
            validate_ref_access_during_render,
            validate_no_set_state_in_render,
            enable_use_keyed_state,
            validate_no_set_state_in_effects,
            validate_no_derived_computations_in_effects,
            validate_no_derived_computations_in_effects_exp,
            validate_no_jsx_in_try_statements,
            validate_static_components,
            validate_source_locations,
            validate_no_impure_functions_in_render,
            validate_no_freezing_known_mutable_functions,
            enable_assume_hooks_follow_rules_of_react,
            enable_transitively_freeze_function_expressions,
            enable_function_outlining,
            enable_jsx_outlining,
            assert_valid_mutable_ranges,
            enable_custom_type_definition_for_reanimated,
            enable_treat_ref_like_identifiers_as_refs,
            enable_treat_set_identifiers_as_state_setters,
            validate_no_void_use_memo,
            enable_allow_set_state_from_refs_in_effects,
            enable_verbose_no_set_state_in_effect,
            enable_forest,
        );

        Ok(())
    }
}

fn parse<T: FromStr>(value: &str, option: &str) -> Result<T, OxcDiagnostic> {
    T::from_str(value).map_err(|_| invalid_option(option, value))
}

fn invalid_option(option: &str, value: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Invalid React Compiler `{option}` option: `{value}`."))
}
