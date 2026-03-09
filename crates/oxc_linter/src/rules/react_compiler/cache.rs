//! Thread-local compilation cache for the React Compiler lint rules.
//!
//! The cache ensures the full React Compiler pipeline runs at most once per file,
//! even when multiple per-category rules are enabled. Results are cached as
//! `CachedDiagnostic` values keyed by the source text pointer.

use std::cell::RefCell;

use oxc_diagnostics::OxcDiagnostic;
use oxc_react_compiler::{
    compiler_error::{ErrorCategory, ErrorSeverity},
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
    shared::walk_statements(&program.body, &outer_bindings, config, &mut diagnostics);

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
