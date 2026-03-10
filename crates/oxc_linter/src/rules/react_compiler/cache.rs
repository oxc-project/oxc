//! Thread-local compilation cache for the React Compiler lint rules.
//!
//! The cache ensures the full React Compiler pipeline runs at most once per file,
//! even when multiple per-category rules are enabled. Results are cached as
//! `CachedDiagnostic` values keyed by the source text pointer.
//!
//! **Design notes:**
//! - Single-slot cache: stores results for exactly one file. This works because
//!   oxlint runs all rules sequentially on each file before moving to the next.
//! - The first rule to call `ensure_compiled` populates the cache; all subsequent
//!   rules on the same file read from it. Per-category rules use
//!   `ReactCompilerConfig::default()`, matching the upstream ESLint plugin where
//!   per-category rules don't accept individual configuration.

use std::cell::RefCell;

use oxc_diagnostics::OxcDiagnostic;
use oxc_react_compiler::{
    compiler_error::{ErrorCategory, ErrorSeverity},
    entrypoint::suppression::{
        DEFAULT_ESLINT_SUPPRESSION_RULES, SuppressionRange, find_program_suppressions,
    },
    hir::build_hir::collect_import_bindings,
};
use oxc_span::Span;

use super::react_compiler_rule::ReactCompilerConfig;
use super::shared;
use crate::context::LintContext;

/// A single diagnostic produced by the React Compiler pipeline,
/// stored in the cache for later reporting.
pub struct CachedDiagnostic {
    pub category: ErrorCategory,
    pub severity: ErrorSeverity,
    pub message: String,
    pub span: Span,
}

/// Per-file compilation result cache.
struct CompilerCache {
    /// Identity key: `source_text().as_ptr() as usize`.
    file_id: usize,
    /// Collected diagnostics from the compilation pipeline.
    diagnostics: Vec<CachedDiagnostic>,
}

thread_local! {
    static CACHE: RefCell<Option<CompilerCache>> = const { RefCell::new(None) };
}

/// Ensure the React Compiler pipeline has been run for the current file.
///
/// On a cache miss (different `file_id`), runs the full pipeline and stores
/// the resulting diagnostics. On a cache hit, does nothing.
pub fn ensure_compiled(ctx: &LintContext<'_>, config: &ReactCompilerConfig) {
    let file_id = ctx.source_text().as_ptr() as usize;

    let needs_compile = CACHE.with(|cache| {
        let cache = cache.borrow();
        !matches!(cache.as_ref(), Some(c) if c.file_id == file_id)
    });

    if !needs_compile {
        return;
    }

    let program = ctx.nodes().program();
    let outer_bindings = collect_import_bindings(&program.body);
    let mut diagnostics = Vec::new();

    // Compute program-level ESLint suppression ranges (port of Program.ts:391-403).
    // If both validateExhaustiveMemoizationDependencies and validateHooksUsage are
    // enabled, the TS reference passes null for ruleNames (skipping suppression checks),
    // because those validations already catch the underlying issues.
    let suppressions = compute_suppressions(ctx, config);

    shared::walk_statements(
        &program.body,
        &outer_bindings,
        config,
        &suppressions,
        &mut diagnostics,
    );

    CACHE.with(|cache| {
        *cache.borrow_mut() = Some(CompilerCache { file_id, diagnostics });
    });
}

/// Report all cached diagnostics (used by the monolithic `react-compiler` rule).
pub fn report_all(ctx: &LintContext<'_>) {
    CACHE.with(|cache| {
        let cache = cache.borrow();
        let Some(c) = cache.as_ref() else { return };
        for diag in &c.diagnostics {
            ctx.diagnostic(make_oxc_diagnostic(diag));
        }
    });
}

/// Report cached diagnostics matching a specific `ErrorCategory`.
pub fn report_for_category(ctx: &LintContext<'_>, category: ErrorCategory) {
    CACHE.with(|cache| {
        let cache = cache.borrow();
        let Some(c) = cache.as_ref() else { return };
        for diag in &c.diagnostics {
            if diag.category == category {
                ctx.diagnostic(make_oxc_diagnostic(diag));
            }
        }
    });
}

fn make_oxc_diagnostic(diag: &CachedDiagnostic) -> OxcDiagnostic {
    match diag.severity {
        ErrorSeverity::Error => OxcDiagnostic::error(diag.message.clone()).with_label(diag.span),
        _ => OxcDiagnostic::warn(diag.message.clone()).with_label(diag.span),
    }
}

/// Compute program-level ESLint suppression ranges.
///
/// Port of Program.ts lines 391-403: if `validateExhaustiveMemoizationDependencies`
/// and `validateHooksUsage` are both true, we skip eslint-disable checks (pass null
/// for rule names) because those validations already catch the underlying issues.
/// Otherwise, we check for `DEFAULT_ESLINT_SUPPRESSION_RULES`.
///
/// Flow suppressions are always checked (matching `pass.opts.flowSuppressions`
/// which defaults to true).
fn compute_suppressions(
    ctx: &LintContext<'_>,
    config: &ReactCompilerConfig,
) -> Vec<SuppressionRange> {
    let skip_eslint_suppressions = config.environment.validate_exhaustive_memoization_dependencies
        && config.environment.validate_hooks_usage;

    let rule_names: Option<Vec<String>> = if skip_eslint_suppressions {
        None
    } else {
        Some(DEFAULT_ESLINT_SUPPRESSION_RULES.iter().map(|s| (*s).to_string()).collect())
    };

    let comments = ctx.semantic().comments();
    let source_text = ctx.semantic().source_text();

    find_program_suppressions(
        comments,
        source_text,
        rule_names.as_deref(),
        // Flow suppressions: always true (matching upstream default).
        true,
    )
}
