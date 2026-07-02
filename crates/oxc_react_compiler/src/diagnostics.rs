// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Turn one internal compiler error detail into an [`OxcDiagnostic`].
//!
//! `oxc_react_compiler` accumulates [`OxcDiagnostic`]s directly on the
//! `ProgramContext` during compilation (see `program.rs`). Both emitting paths —
//! the per-function error path and the lint/telemetry path (`log_errors_as_events`)
//! — funnel through this one converter; the one-off messages (pipeline / unexpected
//! errors) are built inline at their call sites.

use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_span::Span;

use crate::react_compiler_diagnostics::{
    CompilerDiagnosticDetail, CompilerErrorOrDiagnostic, ErrorSeverity, SourceLocation,
};

/// Byte-offset span for a source location. Locations that flow through the oxc
/// frontend carry offsets in `Position::index`; synthetic locations yield `None`.
fn loc_to_span(loc: &SourceLocation) -> Option<Span> {
    Some(Span::new(loc.start.index?, loc.end.index?))
}

/// Labels for a detail: the detail's own location (`ErrorDetail`) or its sub-detail
/// locations (`Diagnostic`), falling back to the enclosing function (`fn_loc`).
fn detail_labels(detail: &CompilerErrorOrDiagnostic, fn_loc: Option<Span>) -> Vec<LabeledSpan> {
    match detail {
        CompilerErrorOrDiagnostic::ErrorDetail(d) => {
            if let Some(span) = d.loc.as_ref().and_then(loc_to_span) {
                return vec![LabeledSpan::underline(span)];
            }
        }
        CompilerErrorOrDiagnostic::Diagnostic(d) => {
            let labels: Vec<LabeledSpan> = d
                .details
                .iter()
                .filter_map(|item| match item {
                    CompilerDiagnosticDetail::Error { loc, message, .. } => {
                        let span = loc.as_ref().and_then(loc_to_span)?;
                        Some(match message {
                            Some(message) => span.label(message.clone()),
                            None => LabeledSpan::underline(span),
                        })
                    }
                    CompilerDiagnosticDetail::Hint { .. } => None,
                })
                .collect();
            if !labels.is_empty() {
                return labels;
            }
        }
    }
    fn_loc.map(|span| vec![LabeledSpan::underline(span)]).unwrap_or_default()
}

/// One internal error detail → an [`OxcDiagnostic`] at its display severity
/// (`Error`/`Warning`/`Hint`; `Off` is suppressed → `None`). `fn_loc` supplies a
/// fallback label when the detail carries no location of its own.
#[cold]
pub fn detail_to_diagnostic(
    detail: &CompilerErrorOrDiagnostic,
    fn_loc: Option<Span>,
) -> Option<OxcDiagnostic> {
    let (category, reason, description, severity) = match detail {
        CompilerErrorOrDiagnostic::Diagnostic(d) => {
            (d.category, &d.reason, &d.description, d.logged_severity())
        }
        CompilerErrorOrDiagnostic::ErrorDetail(d) => {
            (d.category, &d.reason, &d.description, d.logged_severity())
        }
    };

    let message = format!("[ReactCompiler] {category:?}: {reason}");
    let mut diagnostic = match severity {
        ErrorSeverity::Off => return None,
        ErrorSeverity::Error => OxcDiagnostic::error(message),
        // `Warning`, `Hint`, and any unknown future value surface as warnings.
        _ => OxcDiagnostic::warn(message),
    };

    if let Some(description) = description {
        diagnostic = diagnostic.with_help(description.clone());
    }

    let labels = detail_labels(detail, fn_loc);
    if !labels.is_empty() {
        diagnostic = diagnostic.with_labels(labels);
    }

    Some(diagnostic)
}
