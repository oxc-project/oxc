use schemars::JsonSchema;
use serde::Deserialize;

use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::{
    entrypoint::options::{CompilationMode, CompilerReactTarget, PanicThreshold},
    hir::environment::{EnvironmentConfig, ExhaustiveEffectDepsMode},
};

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

/// The main React Compiler lint rule.
///
/// This rule runs the React Compiler's validation passes on React components
/// and hooks, reporting any issues found. It is the Rust equivalent of
/// `eslint-plugin-react-compiler`'s `ReactCompilerRule`.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct ReactCompilerRule(Box<ReactCompilerConfig>);

impl std::ops::Deref for ReactCompilerRule {
    type Target = ReactCompilerConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Configuration for the `react-compiler/react-compiler` rule.
///
/// Mirrors the ESLint plugin's user-facing options:
/// - `compilationMode`: which functions to compile (`infer` | `all` | `annotation` | `syntax`)
/// - `environment`: overrides for the compiler's `EnvironmentConfig` validation flags
///
/// All `environment` fields default to the ESLint plugin's lint-mode defaults,
/// which are **stricter** than the compiler's own defaults.
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ReactCompilerConfig {
    pub compilation_mode: CompilationModeConfig,
    pub environment: EnvironmentConfigOverrides,
    pub target: TargetConfig,
    pub panic_threshold: PanicThresholdConfig,
    pub ignore_use_no_forget: bool,
    pub custom_opt_out_directives: Option<Vec<String>>,
    pub enable_reanimated_check: EnableReanimatedCheck,
}

/// Which functions the compiler should compile/validate.
#[derive(Debug, Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CompilationModeConfig {
    #[default]
    Infer,
    All,
    Annotation,
    Syntax,
}

impl From<CompilationModeConfig> for CompilationMode {
    fn from(config: CompilationModeConfig) -> Self {
        match config {
            CompilationModeConfig::Infer => CompilationMode::Infer,
            CompilationModeConfig::All => CompilationMode::All,
            CompilationModeConfig::Annotation => CompilationMode::Annotation,
            CompilationModeConfig::Syntax => CompilationMode::Syntax,
        }
    }
}

/// The minimum React version the compiler targets.
#[derive(Debug, Clone, Copy, Default, Deserialize, JsonSchema)]
pub enum TargetConfig {
    #[serde(rename = "17")]
    React17,
    #[serde(rename = "18")]
    React18,
    #[default]
    #[serde(rename = "19")]
    React19,
}

impl From<TargetConfig> for CompilerReactTarget {
    fn from(config: TargetConfig) -> Self {
        match config {
            TargetConfig::React17 => CompilerReactTarget::React17,
            TargetConfig::React18 => CompilerReactTarget::React18,
            TargetConfig::React19 => CompilerReactTarget::React19,
        }
    }
}

/// Controls when compilation errors cause the compiler to panic.
#[derive(Debug, Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PanicThresholdConfig {
    AllErrors,
    CriticalErrors,
    #[default]
    None,
}

impl From<PanicThresholdConfig> for PanicThreshold {
    fn from(config: PanicThresholdConfig) -> Self {
        match config {
            PanicThresholdConfig::AllErrors => PanicThreshold::AllErrors,
            PanicThresholdConfig::CriticalErrors => PanicThreshold::CriticalErrors,
            PanicThresholdConfig::None => PanicThreshold::None,
        }
    }
}

/// Mode for exhaustive effect dependency validation.
#[derive(Debug, Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ExhaustiveEffectDepsConfig {
    #[default]
    Off,
    All,
    MissingOnly,
    ExtraOnly,
}

impl From<ExhaustiveEffectDepsConfig> for ExhaustiveEffectDepsMode {
    fn from(config: ExhaustiveEffectDepsConfig) -> Self {
        match config {
            ExhaustiveEffectDepsConfig::Off => ExhaustiveEffectDepsMode::Off,
            ExhaustiveEffectDepsConfig::All => ExhaustiveEffectDepsMode::All,
            ExhaustiveEffectDepsConfig::MissingOnly => ExhaustiveEffectDepsMode::MissingOnly,
            ExhaustiveEffectDepsConfig::ExtraOnly => ExhaustiveEffectDepsMode::ExtraOnly,
        }
    }
}

/// Wrapper for `enableReanimatedCheck` that defaults to `true`.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct EnableReanimatedCheck(pub bool);

impl Default for EnableReanimatedCheck {
    fn default() -> Self {
        Self(true)
    }
}

macro_rules! default_true_bool {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
        #[serde(transparent)]
        pub struct $name(pub bool);

        impl Default for $name {
            fn default() -> Self {
                Self(true)
            }
        }
    };
}

default_true_bool!(
    EnableTreatRefLikeIdentifiersAsRefs,
    "Wrapper for `enableTreatRefLikeIdentifiersAsRefs` (default `true`)."
);
default_true_bool!(
    EnableAssumeHooksFollowRulesOfReact,
    "Wrapper for `enableAssumeHooksFollowRulesOfReact` (default `true`)."
);
default_true_bool!(
    EnableOptionalDependencies,
    "Wrapper for `enableOptionalDependencies` (default `true`)."
);
default_true_bool!(
    EnableTransitivelyFreezeFunctionExpressions,
    "Wrapper for `enableTransitivelyFreezeFunctionExpressions` (default `true`)."
);
default_true_bool!(
    EnablePreserveExistingMemoizationGuarantees,
    "Wrapper for `enablePreserveExistingMemoizationGuarantees` (default `true`)."
);

/// Overrides for `EnvironmentConfig` validation flags.
///
/// Defaults match the ESLint plugin's `COMPILER_OPTIONS` in lint mode,
/// which enables stricter validation than the compiler's own defaults.
#[expect(clippy::struct_field_names)]
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct EnvironmentConfigOverrides {
    pub validate_hooks_usage: bool,
    pub validate_ref_access_during_render: bool,
    pub validate_no_set_state_in_render: bool,
    pub validate_no_set_state_in_effects: bool,
    pub validate_no_jsx_in_try_statements: bool,
    pub validate_no_impure_functions_in_render: bool,
    pub validate_static_components: bool,
    pub validate_no_derived_computations_in_effects: bool,
    pub validate_no_capitalized_calls: Option<Vec<String>>,
    pub validate_blocklisted_imports: Option<Vec<String>>,
    pub validate_preserve_existing_memoization_guarantees: bool,
    pub validate_exhaustive_memoization_dependencies: bool,
    pub validate_no_void_use_memo: bool,
    pub validate_exhaustive_effect_dependencies: ExhaustiveEffectDepsConfig,
    pub validate_no_freezing_known_mutable_functions: bool,
    pub validate_no_derived_computations_in_effects_exp: bool,
    pub enable_verbose_no_set_state_in_effect: bool,
    pub enable_use_keyed_state: bool,
    pub enable_treat_ref_like_identifiers_as_refs: EnableTreatRefLikeIdentifiersAsRefs,
    pub enable_treat_set_identifiers_as_state_setters: bool,
    pub enable_assume_hooks_follow_rules_of_react: EnableAssumeHooksFollowRulesOfReact,
    pub enable_optional_dependencies: EnableOptionalDependencies,
    pub enable_transitively_freeze_function_expressions:
        EnableTransitivelyFreezeFunctionExpressions,
    pub enable_preserve_existing_memoization_guarantees:
        EnablePreserveExistingMemoizationGuarantees,
}

/// Lint-mode defaults — stricter than `EnvironmentConfig::default()`.
impl Default for EnvironmentConfigOverrides {
    fn default() -> Self {
        Self {
            validate_hooks_usage: true,
            validate_ref_access_during_render: true,
            validate_no_set_state_in_render: true,
            validate_no_set_state_in_effects: true,
            validate_no_jsx_in_try_statements: true,
            validate_no_impure_functions_in_render: true,
            validate_static_components: true,
            validate_no_derived_computations_in_effects: true,
            validate_no_capitalized_calls: Some(vec![]),
            validate_blocklisted_imports: None,
            validate_preserve_existing_memoization_guarantees: true,
            validate_exhaustive_memoization_dependencies: true,
            validate_no_void_use_memo: true,
            validate_exhaustive_effect_dependencies: ExhaustiveEffectDepsConfig::Off,
            validate_no_freezing_known_mutable_functions: true,
            validate_no_derived_computations_in_effects_exp: false,
            enable_verbose_no_set_state_in_effect: false,
            enable_use_keyed_state: false,
            enable_treat_ref_like_identifiers_as_refs: EnableTreatRefLikeIdentifiersAsRefs(true),
            enable_treat_set_identifiers_as_state_setters: false,
            enable_assume_hooks_follow_rules_of_react: EnableAssumeHooksFollowRulesOfReact(true),
            enable_optional_dependencies: EnableOptionalDependencies(true),
            enable_transitively_freeze_function_expressions:
                EnableTransitivelyFreezeFunctionExpressions(true),
            enable_preserve_existing_memoization_guarantees:
                EnablePreserveExistingMemoizationGuarantees(true),
        }
    }
}

impl EnvironmentConfigOverrides {
    pub(crate) fn to_environment_config(&self) -> EnvironmentConfig {
        EnvironmentConfig {
            validate_hooks_usage: self.validate_hooks_usage,
            validate_ref_access_during_render: self.validate_ref_access_during_render,
            validate_no_set_state_in_render: self.validate_no_set_state_in_render,
            validate_no_set_state_in_effects: self.validate_no_set_state_in_effects,
            validate_no_jsx_in_try_statements: self.validate_no_jsx_in_try_statements,
            validate_no_impure_functions_in_render: self.validate_no_impure_functions_in_render,
            validate_static_components: self.validate_static_components,
            validate_no_derived_computations_in_effects: self
                .validate_no_derived_computations_in_effects,
            validate_no_capitalized_calls: self.validate_no_capitalized_calls.clone(),
            validate_blocklisted_imports: self.validate_blocklisted_imports.clone(),
            validate_preserve_existing_memoization_guarantees: self
                .validate_preserve_existing_memoization_guarantees,
            validate_exhaustive_memoization_dependencies: self
                .validate_exhaustive_memoization_dependencies,
            validate_no_void_use_memo: self.validate_no_void_use_memo,
            validate_exhaustive_effect_dependencies: self
                .validate_exhaustive_effect_dependencies
                .into(),
            validate_no_freezing_known_mutable_functions: self
                .validate_no_freezing_known_mutable_functions,
            validate_no_derived_computations_in_effects_exp: self
                .validate_no_derived_computations_in_effects_exp,
            enable_verbose_no_set_state_in_effect: self.enable_verbose_no_set_state_in_effect,
            enable_use_keyed_state: self.enable_use_keyed_state,
            enable_treat_ref_like_identifiers_as_refs: self
                .enable_treat_ref_like_identifiers_as_refs
                .0,
            enable_treat_set_identifiers_as_state_setters: self
                .enable_treat_set_identifiers_as_state_setters,
            enable_assume_hooks_follow_rules_of_react: self
                .enable_assume_hooks_follow_rules_of_react
                .0,
            enable_optional_dependencies: self.enable_optional_dependencies.0,
            enable_transitively_freeze_function_expressions: self
                .enable_transitively_freeze_function_expressions
                .0,
            enable_preserve_existing_memoization_guarantees: self
                .enable_preserve_existing_memoization_guarantees
                .0,
            ..EnvironmentConfig::default()
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Runs React Compiler validation passes on components and hooks to detect
    /// code patterns that would prevent automatic memoization or indicate
    /// violations of React's rules.
    ///
    /// ### Why is this bad?
    ///
    /// Code that violates React's rules (mutating props, calling hooks
    /// conditionally, reading refs during render, etc.) can cause bugs
    /// and prevents the React Compiler from optimizing the code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function Component(props) {
    ///   props.value = 1; // Mutating props
    ///   return <div>{props.value}</div>;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Component(props) {
    ///   const value = props.value + 1;
    ///   return <div>{value}</div>;
    /// }
    /// ```
    ReactCompilerRule,
    react_compiler,
    correctness,
    config = ReactCompilerConfig,
);

impl Rule for ReactCompilerRule {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(ctx, &self.0);
        super::cache::report_all(ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // === Basic valid cases ===
        (r"function Component(props) { return <div>{props.value}</div>; }", None),
        (r"function useMyHook() { const [state, setState] = useState(0); return state; }", None),
        (r"function helper() { return 42; }", None),
        (r"const Component = (props) => { return <div>{props.value}</div>; };", None),
        // Named exports
        (r"export function Component(props) { return <div>{props.value}</div>; }", None),
        (r"export const Component = (props) => { return <div>{props.value}</div>; };", None),
        // 'use no forget' opts out of compilation (matching ESLint plugin behavior)
        (
            r#"
            let count = 0;
            function Component() {
              'use no forget';
              return cond ?? useConditionalHook();
            }
            "#,
            None,
        ),
        // === Ported from eslint-plugin-react-compiler PluginTest ===
        // Invariants are only for the compiler team — not surfaced as lint errors
        (
            r#"
            function Component(props) {
              let y = function () {
                m(x);
              };
              let x = { a };
              m(x);
              return y;
            }
            "#,
            None,
        ),
        // Classes don't throw
        (
            r#"
            class Foo {
              #bar() {}
            }
            "#,
            None,
        ),
        // === Ported from InvalidHooksRule ===
        (
            r#"
            function Component() {
              useHook();
              return <div>Hello world</div>;
            }
            "#,
            None,
        ),
        // === Ported from ReactCompilerRuleTypescript ===
        (
            r#"
            function Button(props) {
              return null;
            }
            "#,
            None,
        ),
    ];

    let fail = vec![
        // === Ported from eslint-plugin-react-compiler PluginTest ===
        // Conditional hook call
        (
            r#"
            function Component() {
              const result = cond ?? useConditionalHook();
              return <>{result}</>;
            }
            "#,
            None,
        ),
        // Multiple conditional hooks in same file
        (
            r#"
            function useConditional1() {
              'use memo';
              return cond ?? useConditionalHook();
            }
            function useConditional2(props) {
              'use memo';
              return props.cond && useConditionalHook();
            }
            "#,
            None,
        ),
        // void useMemo + setState in useMemo
        (
            r#"
            import {useMemo, useState} from 'react';

            function Component({item, cond}) {
              const [prevItem, setPrevItem] = useState(item);
              const [state, setState] = useState(0);

              useMemo(() => {
                if (cond) {
                  setPrevItem(item);
                  setState(0);
                }
              }, [cond, item, init]);

              return <Child x={state} />;
            }
            "#,
            None,
        ),
        // === Ported from InvalidHooksRule ===
        // Simple conditional hook violation
        (
            r#"
            function useConditional() {
              if (cond) {
                useConditionalHook();
              }
            }
            "#,
            None,
        ),
        // Multiple conditional hooks in same function
        (
            r#"
            function useConditional() {
              cond ?? useConditionalHook();
              props.cond && useConditionalHook();
              return <div>Hello world</div>;
            }
            "#,
            None,
        ),
        // === Ported from NoCapitalizedCallsRule ===
        // Direct capitalized call
        (
            r#"
            import Child from './Child';
            function Component() {
              return <>
                {Child()}
              </>;
            }
            "#,
            None,
        ),
        // Method call with capitalized name
        (
            r#"
            import myModule from './MyModule';
            function Component() {
              return <>
                {myModule.Child()}
              </>;
            }
            "#,
            None,
        ),
        // === Ported from ImpureFunctionCallsRule ===
        // Known impure function calls
        (
            r#"
            function Component() {
              const date = Date.now();
              const now = performance.now();
              const rand = Math.random();
              return <Foo date={date} now={now} rand={rand} />;
            }
            "#,
            None,
        ),
        // === Ported from NoAmbiguousJsxRule ===
        // JSX in try blocks
        (
            r#"
            function Component(props) {
              let el;
              try {
                el = <Child />;
              } catch {
                return null;
              }
              return el;
            }
            "#,
            None,
        ),
        // === Ported from NoRefAccessInRender ===
        // Ref access during render
        (
            r#"
            function Component(props) {
              const ref = useRef(null);
              const value = ref.current;
              return value;
            }
            "#,
            None,
        ),
        // === Ported from ReactCompilerRuleTypescript ===
        // Mutating useState value
        (
            r#"
            import { useState } from 'react';
            function Component(props) {
              const x: `foo${1}` = 'foo1';
              const [state, setState] = useState({a: 0});
              state.a = 1;
              return <div>{props.foo}</div>;
            }
            "#,
            None,
        ),
    ];

    Tester::new(ReactCompilerRule::NAME, ReactCompilerRule::PLUGIN, pass, fail).test_and_snapshot();
}

#[test]
fn test_config_deserialization() {
    let config: ReactCompilerConfig = serde_json::from_value(serde_json::json!({
        "compilationMode": "all",
        "environment": {
            "validateStaticComponents": false
        }
    }))
    .unwrap();

    assert!(matches!(config.compilation_mode, CompilationModeConfig::All));
    assert!(!config.environment.validate_static_components);
    // Other fields keep lint-mode defaults
    assert!(config.environment.validate_no_set_state_in_effects);
    assert!(config.environment.validate_no_jsx_in_try_statements);
}

#[test]
fn test_new_config_fields_deserialization() {
    let config: ReactCompilerConfig = serde_json::from_value(serde_json::json!({
        "target": "17",
        "panicThreshold": "all_errors",
        "ignoreUseNoForget": true,
        "customOptOutDirectives": ["use skip"],
        "enableReanimatedCheck": false,
        "environment": {
            "validateNoVoidUseMemo": false,
            "validateExhaustiveEffectDependencies": "all",
            "validateNoFreezingKnownMutableFunctions": false,
            "enableAssumeHooksFollowRulesOfReact": false,
            "enableTreatRefLikeIdentifiersAsRefs": false,
            "enableOptionalDependencies": false,
            "enableTransitivelyFreezeFunctionExpressions": false,
            "enablePreserveExistingMemoizationGuarantees": false
        }
    }))
    .unwrap();

    assert!(matches!(config.target, TargetConfig::React17));
    assert!(matches!(config.panic_threshold, PanicThresholdConfig::AllErrors));
    assert!(config.ignore_use_no_forget);
    assert_eq!(config.custom_opt_out_directives, Some(vec!["use skip".to_string()]));
    assert!(!config.enable_reanimated_check.0);
    assert!(!config.environment.validate_no_void_use_memo);
    assert!(matches!(
        config.environment.validate_exhaustive_effect_dependencies,
        ExhaustiveEffectDepsConfig::All
    ));
    assert!(!config.environment.validate_no_freezing_known_mutable_functions);
    assert!(!config.environment.enable_assume_hooks_follow_rules_of_react.0);
    assert!(!config.environment.enable_treat_ref_like_identifiers_as_refs.0);
    assert!(!config.environment.enable_optional_dependencies.0);
    assert!(!config.environment.enable_transitively_freeze_function_expressions.0);
    assert!(!config.environment.enable_preserve_existing_memoization_guarantees.0);

    // Verify defaults for new fields when not specified
    let default_config: ReactCompilerConfig =
        serde_json::from_value(serde_json::json!({})).unwrap();
    assert!(matches!(default_config.target, TargetConfig::React19));
    assert!(matches!(default_config.panic_threshold, PanicThresholdConfig::None));
    assert!(!default_config.ignore_use_no_forget);
    assert!(default_config.custom_opt_out_directives.is_none());
    assert!(default_config.enable_reanimated_check.0);
    assert!(default_config.environment.validate_no_void_use_memo);
    assert!(default_config.environment.validate_no_freezing_known_mutable_functions);
    assert!(default_config.environment.enable_assume_hooks_follow_rules_of_react.0);
    assert!(default_config.environment.enable_treat_ref_like_identifiers_as_refs.0);
    assert!(default_config.environment.enable_optional_dependencies.0);
    assert!(default_config.environment.enable_transitively_freeze_function_expressions.0);
    assert!(default_config.environment.enable_preserve_existing_memoization_guarantees.0);
}
