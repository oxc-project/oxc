use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::{
    compiler_error::{CompilerError, CompilerErrorEntry, SourceLocation},
    entrypoint::{
        options::{CompilationMode, OPT_OUT_DIRECTIVES},
        pipeline::run_pipeline,
        program::should_compile_function,
    },
    hir::{
        NonLocalBinding,
        build_hir::{LowerableFunction, collect_import_bindings, lower},
        environment::{CompilerOutputMode, Environment, EnvironmentConfig},
    },
};
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn react_compiler_diagnostic(span: Span, message: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(message.to_string()).with_label(span)
}

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
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ReactCompilerConfig {
    pub compilation_mode: CompilationModeConfig,
    pub environment: EnvironmentConfigOverrides,
}

impl Default for ReactCompilerConfig {
    fn default() -> Self {
        Self {
            compilation_mode: CompilationModeConfig::default(),
            environment: EnvironmentConfigOverrides::default(),
        }
    }
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

/// Overrides for `EnvironmentConfig` validation flags.
///
/// Defaults match the ESLint plugin's `COMPILER_OPTIONS` in lint mode,
/// which enables stricter validation than the compiler's own defaults.
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
        }
    }
}

impl EnvironmentConfigOverrides {
    fn to_environment_config(&self) -> EnvironmentConfig {
        let mut config = EnvironmentConfig::default();
        config.validate_hooks_usage = self.validate_hooks_usage;
        config.validate_ref_access_during_render = self.validate_ref_access_during_render;
        config.validate_no_set_state_in_render = self.validate_no_set_state_in_render;
        config.validate_no_set_state_in_effects = self.validate_no_set_state_in_effects;
        config.validate_no_jsx_in_try_statements = self.validate_no_jsx_in_try_statements;
        config.validate_no_impure_functions_in_render = self.validate_no_impure_functions_in_render;
        config.validate_static_components = self.validate_static_components;
        config.validate_no_derived_computations_in_effects =
            self.validate_no_derived_computations_in_effects;
        config.validate_no_capitalized_calls = self.validate_no_capitalized_calls.clone();
        config.validate_blocklisted_imports = self.validate_blocklisted_imports.clone();
        config.validate_preserve_existing_memoization_guarantees =
            self.validate_preserve_existing_memoization_guarantees;
        config.validate_exhaustive_memoization_dependencies =
            self.validate_exhaustive_memoization_dependencies;
        config
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
        let program = ctx.nodes().program();
        let outer_bindings = collect_import_bindings(&program.body);

        for statement in &program.body {
            lint_statement(statement, &outer_bindings, &self.0, ctx);
        }
    }
}

fn lint_statement<'a>(
    statement: &'a Statement<'a>,
    outer_bindings: &FxHashMap<String, NonLocalBinding>,
    config: &ReactCompilerConfig,
    ctx: &LintContext<'a>,
) {
    match statement {
        Statement::FunctionDeclaration(function) => {
            let directives = function_directives(function);
            let lowerable_function = LowerableFunction::Function(function);
            lint_function(
                &lowerable_function,
                function.id.as_ref().map(|id| id.name.as_str()),
                &directives,
                function.span,
                outer_bindings,
                config,
                ctx,
            );
        }
        Statement::VariableDeclaration(declaration) => {
            lint_variable_declaration(declaration, outer_bindings, config, ctx);
        }
        Statement::ExportDefaultDeclaration(export_default) => match &export_default.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(function)
            | ExportDefaultDeclarationKind::FunctionExpression(function) => {
                let directives = function_directives(function);
                let lowerable_function = LowerableFunction::Function(function);
                lint_function(
                    &lowerable_function,
                    function.id.as_ref().map(|id| id.name.as_str()),
                    &directives,
                    function.span,
                    outer_bindings,
                    config,
                    ctx,
                );
            }
            ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
                let directives = arrow_directives(arrow);
                let lowerable_function = LowerableFunction::ArrowFunction(arrow);
                lint_function(
                    &lowerable_function,
                    None,
                    &directives,
                    arrow.span,
                    outer_bindings,
                    config,
                    ctx,
                );
            }
            _ => {}
        },
        Statement::ExportNamedDeclaration(export_named) => {
            if let Some(declaration) = &export_named.declaration {
                match declaration {
                    Declaration::FunctionDeclaration(function) => {
                        let directives = function_directives(function);
                        let lowerable_function = LowerableFunction::Function(function);
                        lint_function(
                            &lowerable_function,
                            function.id.as_ref().map(|id| id.name.as_str()),
                            &directives,
                            function.span,
                            outer_bindings,
                            config,
                            ctx,
                        );
                    }
                    Declaration::VariableDeclaration(declaration) => {
                        lint_variable_declaration(declaration, outer_bindings, config, ctx);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn lint_variable_declaration<'a>(
    declaration: &'a VariableDeclaration<'a>,
    outer_bindings: &FxHashMap<String, NonLocalBinding>,
    config: &ReactCompilerConfig,
    ctx: &LintContext<'a>,
) {
    for declarator in &declaration.declarations {
        let binding_name = match &declarator.id {
            BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
            _ => None,
        };

        let Some(initializer) = &declarator.init else {
            continue;
        };

        match initializer {
            Expression::FunctionExpression(function) => {
                let directives = function_directives(function);
                let function_name =
                    function.id.as_ref().map(|id| id.name.as_str()).or(binding_name);
                let lowerable_function = LowerableFunction::Function(function);
                lint_function(
                    &lowerable_function,
                    function_name,
                    &directives,
                    function.span,
                    outer_bindings,
                    config,
                    ctx,
                );
            }
            Expression::ArrowFunctionExpression(arrow) => {
                let directives = arrow_directives(arrow);
                let lowerable_function = LowerableFunction::ArrowFunction(arrow);
                lint_function(
                    &lowerable_function,
                    binding_name,
                    &directives,
                    arrow.span,
                    outer_bindings,
                    config,
                    ctx,
                );
            }
            _ => {}
        }
    }
}

fn lint_function<'a>(
    function: &LowerableFunction<'a>,
    name: Option<&str>,
    directives: &[String],
    fallback_span: Span,
    outer_bindings: &FxHashMap<String, NonLocalBinding>,
    config: &ReactCompilerConfig,
    ctx: &LintContext<'a>,
) {
    // In lint mode, ignore opt-out directives ('use no forget', 'use no memo')
    // so that validation still runs on opted-out functions.
    // This matches the ESLint plugin's behavior where the lint rule always
    // validates, even if the compiler won't transform the function.
    let lint_directives: Vec<String> = directives
        .iter()
        .filter(|d| !OPT_OUT_DIRECTIVES.contains(&d.as_str()))
        .cloned()
        .collect();
    let Some(fn_type) =
        should_compile_function(name, &lint_directives, config.compilation_mode.into(), false)
    else {
        return;
    };

    let env_config = config.environment.to_environment_config();
    let environment = Environment::new(fn_type, CompilerOutputMode::Lint, env_config);

    let mut hir_function = match lower(&environment, fn_type, function, outer_bindings.clone()) {
        Ok(hir_function) => hir_function,
        Err(error) => {
            report_compiler_error(&error, fallback_span, ctx);
            return;
        }
    };

    if let Err(error) = run_pipeline(&mut hir_function, &environment) {
        report_compiler_error(&error, fallback_span, ctx);
    }

    for diagnostic in hir_function.env.take_diagnostics() {
        report_compiler_error(&diagnostic, fallback_span, ctx);
    }
}

fn report_compiler_error(error: &CompilerError, fallback_span: Span, ctx: &LintContext<'_>) {
    for entry in &error.details {
        let span = compiler_error_entry_span(entry).unwrap_or(fallback_span);
        ctx.diagnostic(react_compiler_diagnostic(span, &entry.to_string()));
    }
}

fn function_directives(function: &Function<'_>) -> Vec<String> {
    function.body.as_ref().map_or_else(Vec::new, |body| {
        body.directives.iter().map(|directive| directive.directive.to_string()).collect()
    })
}

fn arrow_directives(function: &ArrowFunctionExpression<'_>) -> Vec<String> {
    function.body.directives.iter().map(|directive| directive.directive.to_string()).collect()
}

fn compiler_error_entry_span(entry: &CompilerErrorEntry) -> Option<Span> {
    let location = match entry {
        CompilerErrorEntry::Diagnostic(diagnostic) => diagnostic.primary_location(),
        CompilerErrorEntry::Detail(detail) => detail.primary_location(),
    };

    match location {
        Some(SourceLocation::Source(span)) => Some(span),
        _ => None,
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
        // 'use no forget' does not disable eslint rule
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
