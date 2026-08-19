use std::borrow::Cow;

use oxc_diagnostics::OxcCode;
use oxc_react_compiler::{
    CompilerOutputMode, EnvironmentConfig, ErrorCategory, ExhaustiveEffectDepsMode, LintDiagnostic,
    PluginOptions,
};

use crate::{
    context::{ContextHost, LintContext},
    loader::LINT_PARTIAL_LOADER_EXTENSIONS,
};

/// The compiler options the React Compiler family of rules lints with — `lint`
/// output mode plus validations that are off by default in the compiler.
/// Mirrors `COMPILER_OPTIONS` in `eslint-plugin-react-hooks`'s
/// `src/shared/RunReactCompiler.ts`.
///
/// The options are a fixed superset shared by every rule in the family: which
/// rules are enabled only routes categories to reporters, it never changes
/// what the compiler analyzes. That keeps the single shared run valid for any
/// combination of enabled rules.
fn react_compiler_plugin_options() -> PluginOptions {
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
            validate_no_capitalized_calls: Some(vec![]),
            validate_hooks_usage: true,
            validate_no_derived_computations_in_effects: true,
            validate_exhaustive_memoization_dependencies: true,
            validate_exhaustive_effect_dependencies: ExhaustiveEffectDepsMode::All,
            ..EnvironmentConfig::default()
        },
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
        react_compiler_plugin_options(),
    );

    ReactCompilerResults { diagnostics: result.diagnostics }
}

/// Shared `should_run` for the React Compiler family: match the upstream
/// `node_modules` source filter and skip files with multiple `<script>` sections
/// (vue/astro/svelte). The latter is required for correctness, not just speed —
/// the cache on [`ContextHost`] is per file, so it would replay the first script
/// section's findings for every later section.
pub fn should_run_react_compiler(ctx: &ContextHost) -> bool {
    !ctx.file_path().to_string_lossy().contains("node_modules")
        && !ctx
            .file_extension()
            .is_some_and(|ext| LINT_PARTIAL_LOADER_EXTENSIONS.iter().any(|e| e == &ext))
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
