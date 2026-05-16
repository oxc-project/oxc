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

use oxc_ast::ast::{
    BindingPattern, Declaration, ExportDefaultDeclarationKind, Expression, Statement,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_react_compiler::{
    compiler_error::{CompilerErrorEntry, ErrorCategory, ErrorSeverity, SourceLocation},
    entrypoint::{
        program::validate_no_dynamically_created_components_or_hooks,
        suppression::{
            DEFAULT_ESLINT_SUPPRESSION_RULES, SuppressionRange, find_program_suppressions,
        },
    },
    hir::build_hir::collect_import_bindings,
};
use oxc_span::Span;
use rustc_hash::FxHashSet;

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
    /// Indices into `diagnostics` that have already been emitted in the
    /// current lint pass.
    ///
    /// Implements first-reporter-wins de-duplication: when both the catch-all
    /// `react-compiler/react-compiler-rule` and per-category rules (e.g.
    /// `react-compiler/hooks`) are enabled, each cached diagnostic would
    /// otherwise be emitted twice. We track which diagnostic indices have
    /// been emitted (after rule-level `disable_directives` filtering — so a
    /// per-category rule that's suppressed by
    /// `// oxlint-disable react-compiler/hooks` does NOT poison the catch-all
    /// rule's emission).
    reported_diagnostic_idx: FxHashSet<usize>,
    /// Identity of `LintContext`s that have called `ensure_compiled` in the
    /// current lint pass.
    ///
    /// In debug builds, oxlint re-runs every rule a second time within the
    /// same `Semantic` to validate diagnostic-count parity between the
    /// runtime-optimized and unoptimized execution paths (`lib.rs:399`).
    /// Both passes share the same `Semantic`, the same `file_id`, and the
    /// same spawned `LintContext` per rule — so the only way to detect a
    /// fresh pass is by observing the same `LintContext` revisiting
    /// `ensure_compiled`. When that happens, we reset both
    /// `reported_diagnostic_idx` and this set so the new pass starts clean.
    seen_lint_ctxs: FxHashSet<usize>,
}

thread_local! {
    static CACHE: RefCell<Option<CompilerCache>> = const { RefCell::new(None) };
}

/// Ensure the React Compiler pipeline has been run for the current file.
///
/// On a cache miss (different `file_id`), runs the full pipeline and stores
/// the resulting diagnostics. On a cache hit, this is a no-op for the
/// compilation work itself but still bookkeeps the calling `LintContext` to
/// support fresh-lint-pass detection (see `seen_lint_ctxs`).
pub fn ensure_compiled(ctx: &LintContext<'_>, config: &ReactCompilerConfig) {
    let file_id = ctx.source_text().as_ptr() as usize;
    let ctx_id = std::ptr::from_ref(ctx) as usize;

    let needs_compile = CACHE.with(|cache| {
        let mut borrow = cache.borrow_mut();
        match borrow.as_mut() {
            Some(c) if c.file_id == file_id => {
                // Cache hit: same source. Detect "fresh lint pass" by
                // observing the same `LintContext` revisiting
                // `ensure_compiled`. Each spawned per-rule `LintContext` is
                // reused across the debug double-run (see field doc), so a
                // re-entry from a previously-seen ctx means a new pass began.
                if !c.seen_lint_ctxs.insert(ctx_id) {
                    c.reported_diagnostic_idx.clear();
                    c.seen_lint_ctxs.clear();
                    c.seen_lint_ctxs.insert(ctx_id);
                }
                false
            }
            _ => true,
        }
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

    // Pre-pass: validate that no component/hook is defined inside a
    // non-component, non-hook helper function.
    // Port of `validateNoDynamicallyCreatedComponentsOrHooks` in Program.ts:517-519.
    // This validation runs at the top-level program scope, not inside individual
    // function compilations, so it must be called separately here.
    if config.environment.validate_no_dynamically_created_components_or_hooks {
        for stmt in &program.body {
            if let Some((name, name_span, fn_span, body)) = extract_outer_function_info(stmt)
                && let Err(err) = validate_no_dynamically_created_components_or_hooks(
                    name, name_span, fn_span, body, None,
                )
            {
                for entry in &err.details {
                    let severity = entry.severity();
                    if matches!(severity, ErrorSeverity::Hint | ErrorSeverity::Off) {
                        continue;
                    }
                    let span = match entry {
                        CompilerErrorEntry::Diagnostic(d) => match d.primary_location() {
                            Some(SourceLocation::Source(s)) => s,
                            _ => fn_span,
                        },
                        CompilerErrorEntry::Detail(d) => match d.primary_location() {
                            Some(SourceLocation::Source(s)) => s,
                            _ => fn_span,
                        },
                    };
                    diagnostics.push(CachedDiagnostic {
                        category: entry.category(),
                        severity,
                        message: entry.to_string(),
                        span,
                    });
                }
            }
        }
    }

    CACHE.with(|cache| {
        let mut seen_lint_ctxs = FxHashSet::default();
        seen_lint_ctxs.insert(ctx_id);
        *cache.borrow_mut() = Some(CompilerCache {
            file_id,
            diagnostics,
            reported_diagnostic_idx: FxHashSet::default(),
            seen_lint_ctxs,
        });
    });
}

/// Report all cached diagnostics (used by the monolithic catch-all rule).
///
/// `rule_name` must be the caller's `Self::NAME` — for the catch-all this is
/// `"react-compiler-rule"` (the kebab-case form of `ReactCompilerRule`) — so
/// we can pre-filter through `disable_directives` and avoid poisoning the
/// de-dup state with a diagnostic the caller would have had suppressed
/// (see `report_for_category`).
///
/// Implements first-reporter-wins de-duplication: a cached diagnostic already
/// emitted by an earlier rule in this pass is skipped.
pub fn report_all(ctx: &LintContext<'_>, rule_name: &str) {
    let to_emit = collect_emissions(ctx, rule_name, None);
    for (_, d) in to_emit {
        ctx.diagnostic(d);
    }
}

/// Report cached diagnostics matching a specific `ErrorCategory`.
///
/// `rule_name` must be the caller's `Self::NAME` (e.g. `"hooks"`).
///
/// Implements first-reporter-wins de-duplication: a cached diagnostic already
/// emitted by an earlier rule in this pass is skipped. **Crucially**, a
/// diagnostic suppressed for `rule_name` via `disable_directives` (e.g.
/// `// oxlint-disable react-compiler/hooks`) is NOT marked as reported — the
/// catch-all rule downstream can still emit it under its own
/// `disable_directives` evaluation.
pub fn report_for_category(ctx: &LintContext<'_>, category: ErrorCategory, rule_name: &str) {
    let to_emit = collect_emissions(ctx, rule_name, Some(category));
    for (_, d) in to_emit {
        ctx.diagnostic(d);
    }
}

/// Pick cached diagnostics this rule may emit, marking the corresponding
/// indices as reported in the cache.
///
/// Filters out diagnostics that are either (a) already reported by an earlier
/// rule in this pass, or (b) suppressed for `rule_name` by an inline
/// `disable_directives` comment covering the diagnostic's span. The latter
/// must NOT mark the diagnostic as reported so a later rule whose
/// `disable_directives` evaluation differs can still emit it.
///
/// `ctx.diagnostic` performs its own `disable_directives` check too (an
/// additional safety net — if it suppresses, no further harm is done because
/// we've already pre-filtered the same way).
fn collect_emissions(
    ctx: &LintContext<'_>,
    rule_name: &str,
    only_category: Option<ErrorCategory>,
) -> Vec<(usize, OxcDiagnostic)> {
    let disable = ctx.disable_directives();
    CACHE.with(|cache| {
        let mut borrow = cache.borrow_mut();
        let Some(c) = borrow.as_mut() else { return Vec::new() };
        let picked = select_emission_indices(
            &c.diagnostics,
            &mut c.reported_diagnostic_idx,
            only_category,
            |span| disable.contains(rule_name, span),
        );
        picked.into_iter().map(|i| (i, make_oxc_diagnostic(&c.diagnostics[i]))).collect()
    })
}

/// Pure decision helper for `collect_emissions`.
///
/// Returns the indices of `diagnostics` that should be emitted given:
/// - `reported`: indices already emitted earlier in this pass (mutated to
///   record this emission).
/// - `only_category`: optional category filter (per-category rules).
/// - `is_disabled`: predicate that returns `true` if `disable_directives`
///   would suppress this diagnostic's span for the calling rule.
///
/// A suppressed diagnostic is dropped from the returned indices BUT NOT
/// marked as reported, so a later rule whose `is_disabled` differs can
/// still emit it.
fn select_emission_indices(
    diagnostics: &[CachedDiagnostic],
    reported: &mut FxHashSet<usize>,
    only_category: Option<ErrorCategory>,
    mut is_disabled: impl FnMut(Span) -> bool,
) -> Vec<usize> {
    let mut out = Vec::new();
    for (idx, diag) in diagnostics.iter().enumerate() {
        if let Some(cat) = only_category
            && diag.category != cat
        {
            continue;
        }
        if reported.contains(&idx) {
            continue;
        }
        if is_disabled(diag.span) {
            continue;
        }
        out.push(idx);
    }
    for idx in &out {
        reported.insert(*idx);
    }
    out
}

fn make_oxc_diagnostic(diag: &CachedDiagnostic) -> OxcDiagnostic {
    match diag.severity {
        ErrorSeverity::Error => OxcDiagnostic::error(diag.message.clone()).with_label(diag.span),
        _ => OxcDiagnostic::warn(diag.message.clone()).with_label(diag.span),
    }
}

/// Extract name, name span, function span, and body statements from a top-level
/// function declaration or arrow/function expression assigned to a variable.
///
/// Returns `None` for non-function statements (classes, imports, etc.).
/// This mirrors `extract_top_level_function_info` from the transformer and
/// the `nestedFnVisitor` code in upstream `Entrypoint/Program.ts`.
fn extract_outer_function_info<'a>(
    stmt: &'a Statement<'a>,
) -> Option<(&'a str, Option<Span>, Span, &'a [Statement<'a>])> {
    match stmt {
        Statement::FunctionDeclaration(func) => {
            let (name, name_span) = if let Some(id) = &func.id {
                (id.name.as_str(), Some(id.span))
            } else {
                return None;
            };
            let body = func.body.as_ref()?.statements.as_slice();
            Some((name, name_span, func.span, body))
        }
        Statement::ExportNamedDeclaration(export) => {
            let decl = export.declaration.as_ref()?;
            match decl {
                Declaration::FunctionDeclaration(func) => {
                    let (name, name_span) = if let Some(id) = &func.id {
                        (id.name.as_str(), Some(id.span))
                    } else {
                        return None;
                    };
                    let body = func.body.as_ref()?.statements.as_slice();
                    Some((name, name_span, func.span, body))
                }
                Declaration::VariableDeclaration(var_decl) if var_decl.declarations.len() == 1 => {
                    let declarator = &var_decl.declarations[0];
                    let (name, name_span) =
                        if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                            (id.name.as_str(), id.span)
                        } else {
                            return None;
                        };
                    let init = declarator.init.as_ref()?;
                    extract_fn_expr_info(init, name, name_span)
                }
                _ => None,
            }
        }
        Statement::ExportDefaultDeclaration(export) => match &export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(func)
            | ExportDefaultDeclarationKind::FunctionExpression(func) => {
                let (name, name_span) = if let Some(id) = &func.id {
                    (id.name.as_str(), Some(id.span))
                } else {
                    return None;
                };
                let body = func.body.as_ref()?.statements.as_slice();
                Some((name, name_span, func.span, body))
            }
            _ => None,
        },
        Statement::VariableDeclaration(var_decl) if var_decl.declarations.len() == 1 => {
            let declarator = &var_decl.declarations[0];
            let (name, name_span) = if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                (id.name.as_str(), id.span)
            } else {
                return None;
            };
            let init = declarator.init.as_ref()?;
            extract_fn_expr_info(init, name, name_span)
        }
        _ => None,
    }
}

fn extract_fn_expr_info<'a>(
    expr: &'a Expression<'a>,
    name: &'a str,
    name_span: Span,
) -> Option<(&'a str, Option<Span>, Span, &'a [Statement<'a>])> {
    match expr {
        Expression::FunctionExpression(func) => {
            let body = func.body.as_ref()?.statements.as_slice();
            let (eff_name, eff_name_span) = if let Some(id) = &func.id {
                (id.name.as_str(), Some(id.span))
            } else {
                (name, Some(name_span))
            };
            Some((eff_name, eff_name_span, func.span, body))
        }
        Expression::ArrowFunctionExpression(arrow) => {
            let body = arrow.body.statements.as_slice();
            Some((name, Some(name_span), arrow.span, body))
        }
        _ => None,
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

#[cfg(test)]
mod tests {
    //! Unit tests for the first-reporter-wins de-dup state machine.
    //!
    //! These tests exercise the pure helper `select_emission_indices` that
    //! backs `report_all` and `report_for_category`. The helper takes an
    //! `is_disabled` predicate so we can simulate `disable_directives`
    //! suppression without constructing a real `LintContext`.
    //!
    //! Invariants verified:
    //! 1. Each cached diagnostic is emitted at most once across catch-all +
    //!    per-category interleavings.
    //! 2. A diagnostic suppressed by an earlier rule's `disable_directives`
    //!    does NOT poison the de-dup state — a later rule whose
    //!    `disable_directives` evaluates the same span differently can still
    //!    emit it.
    use super::*;

    fn mk(cat: ErrorCategory, msg: &str, span: Span) -> CachedDiagnostic {
        CachedDiagnostic {
            category: cat,
            severity: ErrorSeverity::Error,
            message: msg.to_string(),
            span,
        }
    }

    fn never_disabled(_: Span) -> bool {
        false
    }

    #[test]
    fn per_category_then_catch_all_does_not_duplicate() {
        let diags = vec![
            mk(ErrorCategory::Hooks, "hooks-1", Span::default()),
            mk(ErrorCategory::Hooks, "hooks-2", Span::default()),
            mk(ErrorCategory::Purity, "purity-1", Span::default()),
        ];
        let mut reported = FxHashSet::default();

        // Per-category Hooks rule fires first.
        let first = select_emission_indices(
            &diags,
            &mut reported,
            Some(ErrorCategory::Hooks),
            never_disabled,
        );
        assert_eq!(first, vec![0, 1]);

        // Catch-all rule fires second — only emits the Purity diagnostic.
        let second = select_emission_indices(&diags, &mut reported, None, never_disabled);
        assert_eq!(second, vec![2]);
    }

    #[test]
    fn catch_all_then_per_category_does_not_duplicate() {
        let diags = vec![
            mk(ErrorCategory::Hooks, "hooks-1", Span::default()),
            mk(ErrorCategory::Purity, "purity-1", Span::default()),
        ];
        let mut reported = FxHashSet::default();

        let first = select_emission_indices(&diags, &mut reported, None, never_disabled);
        assert_eq!(first, vec![0, 1]);

        // Per-category Hooks rule fires second — no-op.
        let second = select_emission_indices(
            &diags,
            &mut reported,
            Some(ErrorCategory::Hooks),
            never_disabled,
        );
        assert!(second.is_empty());
    }

    #[test]
    fn per_category_only_emits_matching_diagnostics() {
        let diags = vec![
            mk(ErrorCategory::Hooks, "hooks-1", Span::default()),
            mk(ErrorCategory::Purity, "purity-1", Span::default()),
            mk(ErrorCategory::Hooks, "hooks-2", Span::default()),
        ];
        let mut reported = FxHashSet::default();
        let picked = select_emission_indices(
            &diags,
            &mut reported,
            Some(ErrorCategory::Hooks),
            never_disabled,
        );
        assert_eq!(picked, vec![0, 2]);
        // Purity diag was NOT picked; not marked.
        assert!(!reported.contains(&1));
    }

    #[test]
    fn suppressed_diagnostic_does_not_poison_dedup_state() {
        // Regression test for the bug Codex adversarial review caught:
        // if `report_for_category` marked a category as reported BEFORE
        // emitting (i.e., before disable_directives filtering), a
        // `// oxlint-disable react-compiler/hooks` directive on the
        // per-category rule would silently drop the diagnostic AND prevent
        // the catch-all rule from emitting it.
        //
        // With per-diagnostic marking and pre-filter through
        // `disable_directives`, the suppressed diagnostic stays unreported
        // and the catch-all rule (whose disable evaluation differs) emits
        // it.
        let span = Span::new(10, 20);
        let diags = vec![mk(ErrorCategory::Hooks, "hooks-1", span)];
        let mut reported = FxHashSet::default();

        // Per-category Hooks rule fires first, but disable_directives
        // suppresses this span for `react-compiler/hooks`.
        let first =
            select_emission_indices(&diags, &mut reported, Some(ErrorCategory::Hooks), |s| {
                s == span
            });
        assert!(first.is_empty(), "suppressed diagnostic must not be picked");
        assert!(reported.is_empty(), "suppressed diagnostic must NOT be marked as reported");

        // Catch-all rule fires second — `disable_directives` for
        // `react-compiler/react-compiler-rule` does NOT suppress this
        // span, so the diagnostic is emitted by the catch-all.
        let second = select_emission_indices(&diags, &mut reported, None, never_disabled);
        assert_eq!(second, vec![0]);
    }

    #[test]
    fn same_rule_called_twice_is_idempotent() {
        let diags = vec![mk(ErrorCategory::Hooks, "hooks-1", Span::default())];
        let mut reported = FxHashSet::default();

        let first = select_emission_indices(
            &diags,
            &mut reported,
            Some(ErrorCategory::Hooks),
            never_disabled,
        );
        assert_eq!(first, vec![0]);

        let second = select_emission_indices(
            &diags,
            &mut reported,
            Some(ErrorCategory::Hooks),
            never_disabled,
        );
        assert!(second.is_empty());
    }

    #[test]
    fn empty_cache_is_noop() {
        let diags: Vec<CachedDiagnostic> = Vec::new();
        let mut reported = FxHashSet::default();
        let picked = select_emission_indices(&diags, &mut reported, None, never_disabled);
        assert!(picked.is_empty());
        assert!(reported.is_empty());
    }

    #[test]
    fn catch_all_does_not_mark_disabled_diagnostics() {
        // Symmetric to the per-category regression test: if catch-all runs
        // first under a `// oxlint-disable react-compiler/react-compiler-rule`
        // directive but the per-category rule is NOT disabled, the
        // per-category rule must still emit. (Note: a bare
        // `// oxlint-disable react-compiler` directive would disable the
        // whole plugin and thus all per-category rules too — that's not the
        // scenario we're testing here.)
        let span = Span::new(10, 20);
        let diags = vec![
            mk(ErrorCategory::Hooks, "hooks-1", span),
            mk(ErrorCategory::Purity, "purity-1", Span::default()),
        ];
        let mut reported = FxHashSet::default();

        // Catch-all suppresses the Hooks span but not the Purity span.
        let first = select_emission_indices(&diags, &mut reported, None, |s| s == span);
        assert_eq!(first, vec![1]);
        assert!(!reported.contains(&0), "suppressed diag must NOT be marked");
        assert!(reported.contains(&1));

        // Per-category Hooks rule (not suppressed) emits the leftover.
        let second = select_emission_indices(
            &diags,
            &mut reported,
            Some(ErrorCategory::Hooks),
            never_disabled,
        );
        assert_eq!(second, vec![0]);
    }
}

/// Integration test: enable both the catch-all `react-compiler-rule` AND a
/// per-category rule on the same lint run, and verify that a single
/// Hooks-category violation produces exactly one diagnostic (not two).
///
/// This exercises the full oxlint pipeline — `Linter::new` + `LintService` +
/// the thread-local cache + `disable_directives` — and is the integration
/// counterpart to the pure-helper tests above. Without the first-reporter-wins
/// de-dup logic, the same diagnostic would be emitted twice (once by each
/// enabled rule).
#[cfg(test)]
mod integration_tests {
    use std::{ffi::OsStr, path::Path, sync::Arc, sync::mpsc};

    use oxc_allocator::Allocator;

    use super::super::react_compiler_rule::ReactCompilerRule;
    use crate::{
        AllowWarnDeny, ConfigStore, ConfigStoreBuilder, ExternalPluginStore, LintOptions,
        LintPlugins, LintService, LintServiceOptions, Linter, rule::RuleMeta,
        service::RuntimeFileSystem, utils::read_to_arena_str,
    };

    /// Minimal `RuntimeFileSystem` that serves a single in-memory source file.
    struct InMemoryFileSystem {
        path: std::path::PathBuf,
        source: String,
    }

    impl RuntimeFileSystem for InMemoryFileSystem {
        fn read_to_arena_str<'a>(
            &self,
            path: &Path,
            allocator: &'a Allocator,
        ) -> Result<&'a str, std::io::Error> {
            if path == self.path {
                return Ok(allocator.alloc_str(&self.source));
            }
            read_to_arena_str(path, allocator)
        }

        fn write_file(&self, _path: &Path, _content: &str) -> Result<(), std::io::Error> {
            panic!("writing file should not be allowed in integration test");
        }
    }

    /// Build a `Linter` with the catch-all `ReactCompilerRule` AND the
    /// per-category `Hooks` rule both enabled, then lint `source` and return
    /// the number of diagnostics emitted.
    fn lint_with_both_rules(source: &str) -> usize {
        // Find both rules in the global registry and build default-configured
        // instances by cloning the templates and re-deserializing null config.
        let find_rule = |name: &str| -> crate::rules::RuleEnum {
            let template = crate::rules::RULES
                .iter()
                .find(|r| r.plugin_name() == ReactCompilerRule::PLUGIN && r.name() == name)
                .unwrap_or_else(|| panic!("rule react-compiler/{name} must be registered"));
            template
                .from_configuration(serde_json::Value::Null)
                .unwrap_or_else(|_| panic!("rule react-compiler/{name} accepts null config"))
        };
        let catch_all = find_rule(ReactCompilerRule::NAME);
        let hooks_rule = find_rule(super::super::hooks::Hooks::NAME);

        let mut external_plugin_store = ExternalPluginStore::default();
        let config_store_builder = ConfigStoreBuilder::empty()
            .with_builtin_plugins(LintPlugins::REACT_COMPILER)
            .with_rule(catch_all, AllowWarnDeny::Warn)
            .with_rule(hooks_rule, AllowWarnDeny::Warn);
        let config_store = ConfigStore::new(
            config_store_builder.build(&mut external_plugin_store).unwrap(),
            rustc_hash::FxHashMap::default(),
            external_plugin_store,
        );

        let linter = Linter::new(LintOptions::default(), config_store, None);

        let cwd = std::env::current_dir().unwrap().into_boxed_path();
        // Use a .tsx extension so the JSX in the fixture parses.
        let path_to_lint = cwd.join("integration_dedup.tsx");
        let paths = vec![Arc::<OsStr>::from(path_to_lint.as_os_str())];
        let options = LintServiceOptions::new(cwd).with_cross_module(false);
        let lint_service = LintService::new(linter, options);
        let file_system = InMemoryFileSystem { path: path_to_lint, source: source.to_string() };

        let (sender, _receiver) = mpsc::channel();
        lint_service.run_test_source(&file_system, paths, false, &sender).len()
    }

    #[test]
    fn catch_all_and_per_category_emit_each_diagnostic_once() {
        // A conditional hook call triggers exactly one Hooks-category
        // violation. With both `react-compiler/react-compiler-rule` AND
        // `react-compiler/hooks` enabled, the cache's first-reporter-wins
        // de-dup must collapse the two would-be emissions into one.
        let source = r"
            function Component() {
              if (cond) {
                useConditionalHook();
              }
              return <div />;
            }
            ";
        let count = lint_with_both_rules(source);
        assert_eq!(
            count, 1,
            "expected exactly one diagnostic from the single Hooks violation, got {count}; \
             this likely means the catch-all + per-category de-dup regressed"
        );
    }
}
