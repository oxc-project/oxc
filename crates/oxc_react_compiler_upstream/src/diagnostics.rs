// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use oxc_diagnostics::{Diagnostics, LabeledSpan, OxcDiagnostic};
use oxc_span::Span;

use crate::react_compiler::entrypoint::compile_result::{
    CompileResult, CompilerErrorDetailInfo, CompilerErrorInfo, LoggerEvent, LoggerSourceLocation,
};

/// Convert a `CompileResult` into OXC diagnostics. Each diagnostic carries its
/// own severity (set by [`OxcDiagnostic::error`]/[`OxcDiagnostic::warn`]).
pub fn compile_result_to_diagnostics(result: &CompileResult) -> Diagnostics {
    let mut diagnostics = Diagnostics::new();

    match result {
        CompileResult::Success { events, .. } => {
            for event in events {
                if let Some(diag) = event_to_diagnostic(event) {
                    diagnostics.push(diag);
                }
            }
        }
        CompileResult::Error { error, events, .. } => {
            diagnostics.push(error_info_to_diagnostic(error));
            for event in events {
                if let Some(diag) = event_to_diagnostic(event) {
                    diagnostics.push(diag);
                }
            }
        }
    }

    diagnostics
}

/// Byte-offset span for a compiler source location. Locations that flow through
/// the oxc frontend carry span offsets in `Position::index`; synthetic locations
/// without offsets yield `None`.
fn loc_to_span(loc: &LoggerSourceLocation) -> Option<Span> {
    Some(Span::new(loc.start.index?, loc.end.index?))
}

/// Labels for an error detail: its own `loc` when present (`ErrorDetail`-style),
/// otherwise the locations of its sub-details (`Diagnostic`-style), otherwise
/// the enclosing function.
fn detail_labels(
    detail: &CompilerErrorDetailInfo,
    fn_loc: Option<&LoggerSourceLocation>,
) -> Vec<LabeledSpan> {
    if let Some(span) = detail.loc.as_ref().and_then(loc_to_span) {
        return vec![LabeledSpan::underline(span)];
    }
    if let Some(items) = &detail.details {
        let labels: Vec<LabeledSpan> = items
            .iter()
            .filter_map(|item| {
                let span = item.loc.as_ref().and_then(loc_to_span)?;
                Some(match &item.message {
                    Some(message) => span.label(message.clone()),
                    None => LabeledSpan::underline(span),
                })
            })
            .collect();
        if !labels.is_empty() {
            return labels;
        }
    }
    fn_loc.and_then(loc_to_span).map(|span| vec![LabeledSpan::underline(span)]).unwrap_or_default()
}

fn error_info_to_diagnostic(error: &CompilerErrorInfo) -> OxcDiagnostic {
    let message = format!("[ReactCompiler] {}", error.reason);
    let mut diag = OxcDiagnostic::error(message);

    if let Some(description) = &error.description {
        diag = diag.with_help(description.clone());
    }

    let labels: Vec<LabeledSpan> =
        error.details.iter().flat_map(|detail| detail_labels(detail, None)).collect();
    if !labels.is_empty() {
        diag = diag.with_labels(labels);
    }

    diag
}

/// Map a detail to an [`OxcDiagnostic`] at the compiler's own *display* severity
/// (`Error`/`Warning`/`Hint`; `Off` is suppressed). Fatality is separate, decided
/// by `panicThreshold` ([`CompileResult::Error`]).
fn error_detail_to_diagnostic(
    detail: &CompilerErrorDetailInfo,
    fn_loc: Option<&LoggerSourceLocation>,
) -> Option<OxcDiagnostic> {
    let message = format!("[ReactCompiler] {}: {}", detail.category, detail.reason);

    let mut diagnostic = match detail.severity.as_str() {
        "Off" => return None,
        "Error" => OxcDiagnostic::error(message),
        // `Warning`, `Hint`, and any unknown future value surface as warnings.
        _ => OxcDiagnostic::warn(message),
    };

    if let Some(description) = &detail.description {
        diagnostic = diagnostic.with_help(description.clone());
    }

    let labels = detail_labels(detail, fn_loc);
    if !labels.is_empty() {
        diagnostic = diagnostic.with_labels(labels);
    }

    Some(diagnostic)
}

fn event_to_diagnostic(event: &LoggerEvent) -> Option<OxcDiagnostic> {
    match event {
        LoggerEvent::CompileSuccess { .. } | LoggerEvent::CompileSkip { .. } => None,
        LoggerEvent::CompileError { detail, fn_loc } => {
            error_detail_to_diagnostic(detail, fn_loc.as_ref())
        }
        LoggerEvent::CompileErrorWithLoc { detail, fn_loc } => {
            error_detail_to_diagnostic(detail, Some(fn_loc))
        }
        LoggerEvent::CompileUnexpectedThrow { data, fn_loc } => {
            let mut diagnostic =
                OxcDiagnostic::error(format!("[ReactCompiler] Unexpected error: {}", data));
            if let Some(span) = fn_loc.as_ref().and_then(loc_to_span) {
                diagnostic = diagnostic.with_label(LabeledSpan::underline(span));
            }
            Some(diagnostic)
        }
        LoggerEvent::PipelineError { data, fn_loc } => {
            let mut diagnostic =
                OxcDiagnostic::error(format!("[ReactCompiler] Pipeline error: {}", data));
            if let Some(span) = fn_loc.as_ref().and_then(loc_to_span) {
                diagnostic = diagnostic.with_label(LabeledSpan::underline(span));
            }
            Some(diagnostic)
        }
    }
}
