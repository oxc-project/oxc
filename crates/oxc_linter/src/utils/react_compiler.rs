use std::borrow::Cow;

use lazy_regex::Regex;
use oxc_ast::ast::{
    BindingPattern, Declaration, ExportDefaultDeclarationKind, Expression, FunctionType, Program,
    Statement,
};
use oxc_diagnostics::OxcCode;
use oxc_react_compiler::{
    CompilerOutputMode, EnvironmentConfig, ErrorCategory, ExhaustiveEffectDepsMode, LintDiagnostic,
    PluginOptions,
};

use crate::{
    context::{ContextHost, LintContext},
    loader::LINT_PARTIAL_LOADER_EXTENSIONS,
};

/// Environment additions to the shared React Compiler run that come from rule
/// configuration, extracted once per resolved config in `Linter::run` (from the
/// `react/capitalized-calls` rule's options and whether `react/todo` or
/// `react/invariant` is enabled). Config-derived rather than per-run rule state,
/// so the single shared run stays valid for any combination of enabled rules.
#[derive(Debug, Default, Clone)]
pub struct ReactCompilerEnvOptions {
    /// Extra allowed names for `validateNoCapitalizedCalls`.
    pub capitalized_calls_allow: Vec<String>,
    /// Allowed-name pattern for `validateNoCapitalizedCalls`.
    pub capitalized_calls_allow_pattern: Option<Regex>,
    /// Whether `react/todo` or `react/invariant` is enabled. Otherwise lint skips
    /// the compiler passes that only report those categories.
    pub report_todo_and_invariant: bool,
}

/// The compiler options the React Compiler family of rules lints with — `lint`
/// output mode plus validations that are off by default in the compiler.
/// Mirrors `COMPILER_OPTIONS` in `eslint-plugin-react-hooks`'s
/// `src/shared/RunReactCompiler.ts`.
///
/// The options are a superset shared by every rule in the family: which rules
/// are enabled only routes categories to reporters, and the only per-config
/// variation is [`ReactCompilerEnvOptions`]. That keeps the single shared run
/// valid for any combination of enabled rules.
fn react_compiler_plugin_options(env_options: &ReactCompilerEnvOptions) -> PluginOptions {
    PluginOptions {
        output_mode: Some(CompilerOutputMode::Lint),
        // Oxlint does not parse Flow files.
        flow_suppressions: false,
        // The upstream defaults plus the oxlint spellings of the same rules,
        // so `oxlint-disable`/`eslint-disable` comments for either plugin
        // prefix make the compiler skip the function and report it under
        // `react/rule-suppression`.
        eslint_suppression_rules: Some(vec![
            "react-hooks/exhaustive-deps".to_string(),
            "react-hooks/rules-of-hooks".to_string(),
            "react/exhaustive-deps".to_string(),
            "react/rules-of-hooks".to_string(),
        ]),
        environment: EnvironmentConfig {
            validate_ref_access_during_render: true,
            validate_no_set_state_in_render: true,
            validate_no_set_state_in_effects: true,
            validate_no_jsx_in_try_statements: true,
            validate_no_impure_functions_in_render: true,
            validate_static_components: true,
            validate_no_freezing_known_mutable_functions: true,
            validate_no_void_use_memo: true,
            validate_no_capitalized_calls: Some(env_options.capitalized_calls_allow.clone()),
            validate_no_capitalized_calls_pattern: env_options
                .capitalized_calls_allow_pattern
                .clone(),
            validate_hooks_usage: true,
            validate_no_derived_computations_in_effects: true,
            validate_exhaustive_memoization_dependencies: true,
            validate_exhaustive_effect_dependencies: ExhaustiveEffectDepsMode::All,
            // The Babel plugin enables this when it detects `react-native-reanimated`, which
            // Oxlint cannot do without a plugin list or resolver. The type model only applies
            // to values imported from `react-native-reanimated`, so enabling it is safe.
            enable_custom_type_definition_for_reanimated: true,
            ..EnvironmentConfig::default()
        },
        lint_skip_reactive_scope_passes: !env_options.report_todo_and_invariant,
        ..PluginOptions::default()
    }
}

/// Shared per-file output of the React Compiler, computed at most once per
/// file and reused by every enabled rule in the family — the counterpart of
/// the shared compile cache in `eslint-plugin-react-hooks`.
/// `--timing` note: the whole compile is attributed to whichever family rule
/// happens to run first on a file, same as the upstream ESLint plugin.
#[derive(Debug, Default)]
pub struct ReactCompilerResults {
    diagnostics: Vec<LintDiagnostic>,
}

impl ReactCompilerResults {
    /// Findings for one category, in the order the compiler reported them.
    fn diagnostics_for(&self, category: ErrorCategory) -> impl Iterator<Item = &LintDiagnostic> {
        self.diagnostics.iter().filter(move |d| d.category == category)
    }
}

/// `LintResult::fatal` is deliberately ignored: Oxlint uses fixed compiler
/// options, and category routing reports diagnostics independently of transform
/// fatality.
pub fn build_react_compiler_results(host: &ContextHost) -> ReactCompilerResults {
    let semantic = host.semantic();
    let program = semantic.nodes().program();
    let result = oxc_react_compiler::lint(
        program,
        semantic,
        host.allocator(),
        react_compiler_plugin_options(host.react_compiler_env_options()),
    );

    ReactCompilerResults { diagnostics: result.diagnostics }
}

/// Shared `should_run` for the React Compiler family: match the upstream
/// `node_modules` source filter and [`may_contain_react_code`] gate, and skip files
/// with multiple `<script>` sections (vue/astro/svelte). The latter is required for
/// correctness, not just speed — the cache on [`ContextHost`] is per file, so it
/// would replay the first script section's findings for every later section.
pub fn should_run_react_compiler(ctx: &ContextHost) -> bool {
    !ctx.file_path().to_string_lossy().contains("node_modules")
        && !ctx
            .file_extension()
            .is_some_and(|ext| LINT_PARTIAL_LOADER_EXTENSIONS.iter().any(|e| e == &ext))
        && may_contain_react_code(ctx.semantic().nodes().program())
}

/// Port of `mayContainReactCode` in `eslint-plugin-react-hooks`'s
/// `src/shared/RunReactCompiler.ts`: only compile files with a top-level function
/// named like a component or hook, or an anonymous default-exported function.
fn may_contain_react_code(program: &Program) -> bool {
    program.body.iter().any(is_top_level_react_code)
}

fn is_top_level_react_code(statement: &Statement) -> bool {
    match statement {
        Statement::ExportDeclaration(export) => is_react_declaration(&export.declaration),
        Statement::ExportDefaultDeclaration(export) => match &export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                func.r#type == FunctionType::FunctionDeclaration
                    && func.id.as_ref().is_none_or(|id| is_component_or_hook_name(&id.name))
            }
            declaration => declaration.as_expression().is_some_and(is_function_expression),
        },
        _ => statement.as_declaration().is_some_and(is_react_declaration),
    }
}

fn is_react_declaration(declaration: &Declaration) -> bool {
    match declaration {
        Declaration::FunctionDeclaration(func) => {
            func.r#type == FunctionType::FunctionDeclaration
                && func.id.as_ref().is_some_and(|id| is_component_or_hook_name(&id.name))
        }
        Declaration::VariableDeclaration(declaration) => {
            declaration.declarations.iter().any(|declarator| {
                matches!(&declarator.id, BindingPattern::BindingIdentifier(id) if is_component_or_hook_name(&id.name))
                    && declarator.init.as_ref().is_some_and(is_function_expression)
            })
        }
        _ => false,
    }
}

fn is_function_expression(expr: &Expression) -> bool {
    matches!(
        expr.without_parentheses(),
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
    )
}

/// `/^[A-Z]/` or `/^use[A-Z0-9]/`.
fn is_component_or_hook_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.first().is_some_and(u8::is_ascii_uppercase)
        || (bytes.starts_with(b"use")
            && bytes.get(3).is_some_and(|b| b.is_ascii_uppercase() || b.is_ascii_digit()))
}

/// Shared `run_once` body for the React Compiler family of rules: report the
/// shared run's findings for `category` under the calling rule's name.
pub fn run_react_compiler_rule(ctx: &LintContext, category: ErrorCategory) {
    for finding in ctx.react_compiler_results().diagnostics_for(category) {
        let mut diagnostic = finding.diagnostic.clone();

        // `LintContext` supplies the per-category Oxlint rule code and primary
        // rule URL. Keep the compiler's category-specific URL as extra context.
        if let Some(guidance_url) = diagnostic.url.take() {
            let guidance = format!("Additional guidance: {guidance_url}");
            diagnostic.note = Some(Cow::Owned(match diagnostic.note.take() {
                Some(note) => format!("{note}. {guidance}"),
                None => guidance,
            }));
        }
        diagnostic.code = OxcCode::default();
        ctx.diagnostic(diagnostic);
    }
}
