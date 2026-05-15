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
