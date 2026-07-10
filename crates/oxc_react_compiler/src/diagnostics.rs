// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Compiler diagnostics, built directly on [`oxc_diagnostics`].
//!
//! Passes construct [`OxcDiagnostic`]s eagerly via [`ErrorCategory::diagnostic`],
//! whose deterministic `[ReactCompiler] <Category>: ` message prefix lets the
//! aggregate [`CompilerError`] recover the category for control flow
//! (Invariant/Config checks, panic-threshold severity) without a parallel data
//! model. No other field encodes compiler state, so consumers see plain
//! `OxcDiagnostic`s.

use oxc_diagnostics::{OxcDiagnostic, Severity};
use oxc_span::Span;

/// Error categories matching the TS `ErrorCategory` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Hooks,
    CapitalizedCalls,
    StaticComponents,
    UseMemo,
    VoidUseMemo,
    PreserveManualMemo,
    MemoDependencies,
    IncompatibleLibrary,
    Immutability,
    Globals,
    Refs,
    EffectExhaustiveDependencies,
    EffectSetState,
    EffectDerivationsOfState,
    ErrorBoundaries,
    Purity,
    RenderSetState,
    Invariant,
    Todo,
    Syntax,
    UnsupportedSyntax,
    Config,
    Gating,
    Suppression,
}

impl ErrorCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hooks => "Hooks",
            Self::CapitalizedCalls => "CapitalizedCalls",
            Self::StaticComponents => "StaticComponents",
            Self::UseMemo => "UseMemo",
            Self::VoidUseMemo => "VoidUseMemo",
            Self::PreserveManualMemo => "PreserveManualMemo",
            Self::MemoDependencies => "MemoDependencies",
            Self::IncompatibleLibrary => "IncompatibleLibrary",
            Self::Immutability => "Immutability",
            Self::Globals => "Globals",
            Self::Refs => "Refs",
            Self::EffectExhaustiveDependencies => "EffectExhaustiveDependencies",
            Self::EffectSetState => "EffectSetState",
            Self::EffectDerivationsOfState => "EffectDerivationsOfState",
            Self::ErrorBoundaries => "ErrorBoundaries",
            Self::Purity => "Purity",
            Self::RenderSetState => "RenderSetState",
            Self::Invariant => "Invariant",
            Self::Todo => "Todo",
            Self::Syntax => "Syntax",
            Self::UnsupportedSyntax => "UnsupportedSyntax",
            Self::Config => "Config",
            Self::Gating => "Gating",
            Self::Suppression => "Suppression",
        }
    }

    /// Displayed severity, matching the TS compiler's `getRuleForCategory()`.
    /// `PreserveManualMemo` displays as an error but does not count towards
    /// `panicThreshold: critical_errors` (see [`CompilerError::has_errors`]).
    const fn severity(self) -> Severity {
        match self {
            Self::IncompatibleLibrary | Self::UnsupportedSyntax | Self::Todo => Severity::Warning,
            _ => Severity::Error,
        }
    }

    /// Build a diagnostic for this category: `[ReactCompiler] <Category>: <reason>`.
    /// Attach spans with `.with_label(span)` (plain underline) or
    /// `.with_label(span.label(text))`, and a description with `.with_help(..)`.
    pub fn diagnostic(self, reason: impl AsRef<str>) -> OxcDiagnostic {
        let message = format!("[ReactCompiler] {}: {}", self.as_str(), reason.as_ref());
        match self.severity() {
            Severity::Error => OxcDiagnostic::error(message),
            _ => OxcDiagnostic::warn(message),
        }
    }

    /// Whether `diagnostic` was built for this category via [`Self::diagnostic`],
    /// recovered from the deterministic message prefix.
    pub fn matches(self, diagnostic: &OxcDiagnostic) -> bool {
        Self::of(diagnostic) == Some(self.as_str())
    }

    /// The category segment of a message built by [`Self::diagnostic`].
    fn of(diagnostic: &OxcDiagnostic) -> Option<&str> {
        let rest = diagnostic.message.strip_prefix("[ReactCompiler] ")?;
        rest.split_once(": ").map(|(category, _)| category)
    }
}

/// Aggregate compiler error: the diagnostics of one failed compilation attempt.
/// This is the main error type returned by pipeline passes.
#[derive(Debug, Clone)]
pub struct CompilerError {
    pub diagnostics: Vec<OxcDiagnostic>,
    /// When false, this error was accumulated on the Environment via
    /// `record_error()` / `record_diagnostic()` and returned at the end
    /// of the pipeline. In TS, `CompileUnexpectedThrow` is only emitted
    /// for errors that are **thrown** (not accumulated). Defaults to `true`
    /// because errors created directly (e.g., via `?` from a pass) are
    /// analogous to thrown errors in the TS code.
    pub is_thrown: bool,
}

impl CompilerError {
    pub fn new() -> Self {
        Self { diagnostics: Vec::new(), is_thrown: true }
    }

    pub fn push(&mut self, diagnostic: OxcDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn merge(&mut self, other: CompilerError) {
        self.diagnostics.extend(other.diagnostics);
    }

    /// Whether any diagnostic is an error at the TS compiler's *internal*
    /// severity, which decides `panicThreshold: critical_errors`. Internal and
    /// displayed severity agree except for `PreserveManualMemo`, which displays
    /// as an error but is internally a warning (it must not trigger the panic
    /// threshold).
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error && !ErrorCategory::PreserveManualMemo.matches(d))
    }

    pub fn has_any_errors(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    pub fn has_invariant_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| ErrorCategory::Invariant.matches(d))
    }

    /// In TS, this is used to determine if an error thrown during compilation
    /// should be logged as CompileUnexpectedThrow.
    pub fn is_all_non_invariant(&self) -> bool {
        !self.diagnostics.iter().any(|d| ErrorCategory::Invariant.matches(d))
    }

    /// Format as a string matching the TS `CompilerError.toString()` output,
    /// used for the `data` field of `CompileUnexpectedThrow` events:
    /// `"<Heading>: <reason>. <description>."` per diagnostic, joined by `"\n\n"`.
    /// The reason is the message minus the deterministic prefix added by
    /// [`ErrorCategory::diagnostic`].
    pub fn to_string_for_event(&self) -> String {
        self.diagnostics
            .iter()
            .map(|d| {
                let category = ErrorCategory::of(d);
                let heading = match category {
                    Some("IncompatibleLibrary" | "PreserveManualMemo" | "UnsupportedSyntax") => {
                        "Compilation Skipped"
                    }
                    Some(heading @ ("Invariant" | "Todo")) => heading,
                    _ => "Error",
                };
                let reason = category
                    .and_then(|c| d.message.strip_prefix(&format!("[ReactCompiler] {c}: ")))
                    .unwrap_or(&d.message);
                let mut buf = format!("{heading}: {reason}");
                if let Some(help) = &d.help {
                    buf.push_str(&format!(". {help}."));
                }
                buf
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

impl Default for CompilerError {
    fn default() -> Self {
        Self::new()
    }
}

impl From<OxcDiagnostic> for CompilerError {
    fn from(diagnostic: OxcDiagnostic) -> Self {
        let mut error = CompilerError::new();
        error.push(diagnostic);
        error
    }
}

/// Allow `?` to convert a `CompilerError` into an `OxcDiagnostic` when the
/// enclosing function returns `Result<T, OxcDiagnostic>`. This typically happens
/// when `record_error()` returns `Err(CompilerError)` for an Invariant error;
/// the conversion extracts the first diagnostic from the aggregate error.
impl From<CompilerError> for OxcDiagnostic {
    fn from(err: CompilerError) -> Self {
        err.diagnostics
            .into_iter()
            .next()
            .unwrap_or_else(|| ErrorCategory::Invariant.diagnostic("Unknown compiler error"))
    }
}

/// Owned copy of a diagnostic for the log accumulator, labelling the enclosing
/// function (`fn_span`) when the diagnostic carries no location of its own.
#[cold]
pub fn with_fallback_label(diagnostic: &OxcDiagnostic, fn_span: Option<Span>) -> OxcDiagnostic {
    let diagnostic = diagnostic.clone();
    match fn_span {
        Some(span) if diagnostic.labels.is_empty() => diagnostic.with_label(span),
        _ => diagnostic,
    }
}
