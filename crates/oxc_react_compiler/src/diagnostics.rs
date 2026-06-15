// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use oxc_diagnostics::{Diagnostics, OxcDiagnostic};
use react_compiler::entrypoint::compile_result::{
    CompileResult, CompilerErrorDetailInfo, LoggerEvent,
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

fn error_info_to_diagnostic(
    error: &react_compiler::entrypoint::compile_result::CompilerErrorInfo,
) -> OxcDiagnostic {
    let message = format!("[ReactCompiler] {}", error.reason);
    let mut diag = OxcDiagnostic::error(message);

    if let Some(description) = &error.description {
        diag = diag.with_help(description.clone());
    }

    diag
}

/// Map a detail to an [`OxcDiagnostic`] at the compiler's own *display* severity
/// (`Error`/`Warning`/`Hint`; `Off` is suppressed). Fatality is separate, decided
/// by `panicThreshold` ([`CompileResult::Error`]).
fn error_detail_to_diagnostic(detail: &CompilerErrorDetailInfo) -> Option<OxcDiagnostic> {
    let message = if let Some(description) = &detail.description {
        format!("[ReactCompiler] {}: {}. {}", detail.category, detail.reason, description)
    } else {
        format!("[ReactCompiler] {}: {}", detail.category, detail.reason)
    };

    let diagnostic = match detail.severity.as_str() {
        "Off" => return None,
        "Error" => OxcDiagnostic::error(message),
        // `Warning`, `Hint`, and any unknown future value surface as warnings.
        _ => OxcDiagnostic::warn(message),
    };
    Some(diagnostic)
}

fn event_to_diagnostic(event: &LoggerEvent) -> Option<OxcDiagnostic> {
    match event {
        LoggerEvent::CompileSuccess { .. } | LoggerEvent::CompileSkip { .. } => None,
        LoggerEvent::CompileError { detail, .. }
        | LoggerEvent::CompileErrorWithLoc { detail, .. } => error_detail_to_diagnostic(detail),
        LoggerEvent::CompileUnexpectedThrow { data, .. } => {
            Some(OxcDiagnostic::error(format!("[ReactCompiler] Unexpected error: {}", data)))
        }
        LoggerEvent::PipelineError { data, .. } => {
            Some(OxcDiagnostic::error(format!("[ReactCompiler] Pipeline error: {}", data)))
        }
    }
}
