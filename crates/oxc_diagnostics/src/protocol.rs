//! Core diagnostic protocol used by Oxc's renderers.
use std::{borrow::Cow, error::Error};

use oxc_span::LabeledSpan;

/// Rich metadata that renderers use to produce human-friendly error messages.
pub trait Diagnostic: Error {
    /// Unique diagnostic code that can be used to look up more information
    /// about this `Diagnostic`. Ideally also globally unique, and documented
    /// in the top-level crate's documentation for easy searching. Rust path
    /// format (`foo::bar::baz`) is recommended, but more classic codes like
    /// `E0123` or enums will work just fine.
    fn code(&self) -> Option<Cow<'_, str>> {
        None
    }

    /// Diagnostic severity. Renderers may use this to change the display format
    /// of this diagnostic.
    ///
    /// If `None`, reporters should treat this as [`Severity::Error`].
    fn severity(&self) -> Option<Severity> {
        None
    }

    /// Additional help text related to this diagnostic.
    fn help(&self) -> Option<Cow<'_, str>> {
        None
    }

    /// Supplementary context for this `Diagnostic`, separate from help text.
    /// Notes mirror rustc-style `= note:` lines and offer additional
    /// information when guidance (help) is insufficient.
    fn note(&self) -> Option<Cow<'_, str>> {
        None
    }

    /// URL to visit for a more detailed explanation/help about this
    /// `Diagnostic`.
    fn url(&self) -> Option<Cow<'_, str>> {
        None
    }

    /// Source code to apply this `Diagnostic`'s [`Diagnostic::labels`] to.
    fn source_code(&self) -> Option<&dyn SourceCode> {
        None
    }

    /// Labels to apply to this `Diagnostic`'s [`Diagnostic::source_code`]
    ///
    /// The diagnostic retains ownership of the labels; renderers only borrow
    /// them for the duration of a report.
    fn labels(&self) -> &[LabeledSpan] {
        &[]
    }
}

/// [`Diagnostic`] severity. Renderers use this to change the way diagnostics are
/// displayed. Defaults to [`Severity::Error`].
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Default)]
pub enum Severity {
    /// Advice for improving the reported code.
    Advice,
    /// A non-fatal warning.
    Warning,
    /// An error. This is the default severity.
    #[default]
    Error,
}

/// Contiguous source text used by diagnostic renderers.
pub trait SourceCode: Send + Sync {
    /// Returns the complete source as bytes.
    fn data(&self) -> &[u8];

    /// Returns the name of this source code, if any.
    fn name(&self) -> Option<&str> {
        None
    }
}
