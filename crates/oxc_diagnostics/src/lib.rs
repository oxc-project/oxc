//! Error data types and utilities for handling/reporting them.
//!
//! The main type in this module is [`OxcDiagnostic`], which is used by all other oxc tools to
//! report problems. It implements [miette]'s [`Diagnostic`] trait, making it compatible with other
//! tooling you may be using.
//!
//! ```rust
//! use oxc_diagnostics::{OxcDiagnostic, Result};
//! fn my_tool() -> Result<()> {
//!     try_something().map_err(|e| OxcDiagnostic::error(e.to_string()))?;
//!     Ok(())
//! }
//! ```
//!
//! See the [miette] documentation for more information on how to interact with diagnostics.
//!
//! ## Reporting
//! If you are writing your own tools that may produce their own errors, you can use
//! [`DiagnosticService`] to format and render them to a string or a stream. It can receive
//! [`Error`]s over a multi-producer, single consumer
//!
//! ```
//! use std::{path::PathBuf, sync::Arc, thread};
//! use oxc_diagnostics::{DiagnosticService, Error, OxcDiagnostic, GraphicalReportHandler, NamedSource};
//!
//! fn my_tool() -> Result<()> {
//!     try_something().map_err(|e| OxcDiagnostic::error(e.to_string()))?;
//!     Ok(())
//! }
//!
//! let (mut service, sender) = DiagnosticService::new(Box::new(GraphicalReportHandler::new()));
//!
//! thread::spawn(move || {
//!     let file_path_being_processed = PathBuf::from("file.txt");
//!     let file_being_processed = Arc::new(NamedSource::new(file_path_being_processed.clone()));
//!
//!     for _ in 0..10 {
//!         if let Err(diagnostic) = my_tool() {
//!             let report = diagnostic.with_source_code(Arc::clone(&file_being_processed));
//!             sender.send((file_path_being_processed, vec![Error::new(e)]));
//!         }
//!         // The service will stop when all senders are dropped
//!     }
//! });
//!
//! service.run();
//! ```

mod service;

use std::{
    borrow::Cow,
    fmt::{self, Display},
    ops::{Deref, DerefMut},
    sync::OnceLock,
};

pub mod reporter;

pub use crate::reporter::GraphicalReportHandlerWithDiff;
pub use crate::service::{DiagnosticSender, DiagnosticService, OxcDiagnosticWithSource};

pub type Error = miette::Error;
pub type Severity = miette::Severity;

pub type Result<T> = std::result::Result<T, OxcDiagnostic>;

use miette::{Diagnostic, SourceCode};
pub use miette::{GraphicalReportHandler, GraphicalTheme, LabeledSpan, NamedSource, SourceSpan};

use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag for forcing fix diff display in tests.
static FORCE_SHOW_FIX_DIFF: AtomicBool = AtomicBool::new(false);

/// Returns `true` if fix diffs should be shown in diagnostic output.
///
/// Controlled by the `OXC_DIAGNOSTIC_SHOW_FIX_DIFF` environment variable.
/// Set to "1" or "true" to enable.
///
/// Can also be enabled programmatically via [`enable_fix_diff`] for testing.
pub fn show_fix_diff() -> bool {
    // Check the force flag first (for testing)
    if FORCE_SHOW_FIX_DIFF.load(Ordering::Relaxed) {
        return true;
    }
    static SHOW_FIX_DIFF: OnceLock<bool> = OnceLock::new();
    *SHOW_FIX_DIFF.get_or_init(|| {
        std::env::var("OXC_DIAGNOSTIC_SHOW_FIX_DIFF")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    })
}

/// Force-enable fix diff display.
///
/// This is useful for testing when you want to ensure fix diffs are captured
/// and displayed regardless of environment variables.
pub fn enable_fix_diff() {
    FORCE_SHOW_FIX_DIFF.store(true, Ordering::Relaxed);
}

/// Force-disable fix diff display (revert to env var behavior).
pub fn disable_fix_diff() {
    FORCE_SHOW_FIX_DIFF.store(false, Ordering::Relaxed);
}

/// A suggested edit to source code for fixing a diagnostic.
///
/// Modeled after annotate-snippets' Patch type. Represents a span to replace
/// and its replacement text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    /// The byte span in the source to replace.
    pub span: SourceSpan,
    /// The replacement text.
    pub replacement: Cow<'static, str>,
}

impl Patch {
    /// Create a new patch that replaces the given span with the replacement text.
    pub fn new<S: Into<SourceSpan>, R: Into<Cow<'static, str>>>(span: S, replacement: R) -> Self {
        Self { span: span.into(), replacement: replacement.into() }
    }
}

/// Describes an error or warning that occurred.
///
/// Used by all oxc tools.
#[derive(Debug, Clone, Eq, PartialEq)]
#[must_use]
pub struct OxcDiagnostic {
    // `Box` the data to make `OxcDiagnostic` 8 bytes so that `Result` is small.
    // This is required because rust does not performance return value optimization.
    inner: Box<OxcDiagnosticInner>,
}

impl Deref for OxcDiagnostic {
    type Target = Box<OxcDiagnosticInner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for OxcDiagnostic {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct OxcCode {
    pub scope: Option<Cow<'static, str>>,
    pub number: Option<Cow<'static, str>>,
}

impl OxcCode {
    pub fn is_some(&self) -> bool {
        self.scope.is_some() || self.number.is_some()
    }
}

impl Display for OxcCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.scope, &self.number) {
            (Some(scope), Some(number)) => write!(f, "{scope}({number})"),
            (Some(scope), None) => scope.fmt(f),
            (None, Some(number)) => number.fmt(f),
            (None, None) => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OxcDiagnosticInner {
    pub message: Cow<'static, str>,
    pub labels: Option<Vec<LabeledSpan>>,
    pub help: Option<Cow<'static, str>>,
    pub note: Option<Cow<'static, str>>,
    pub severity: Severity,
    pub code: OxcCode,
    pub url: Option<Cow<'static, str>>,
    /// Suggested fixes/patches for this diagnostic.
    ///
    /// Each inner `Vec<Patch>` represents one possible fix (which may consist of
    /// multiple edits that should be applied together). Multiple inner vecs
    /// represent alternative fixes.
    pub patches: Option<Vec<Vec<Patch>>>,
}

impl Display for OxcDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for OxcDiagnostic {}

impl Diagnostic for OxcDiagnostic {
    /// The secondary help message.
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.help.as_ref().map(Box::new).map(|c| c as Box<dyn Display>)
    }

    /// A note for the diagnostic.
    ///
    /// Similar to rustc - intended for additional explanation and information,
    /// e.g. why an error was emitted, how to turn it off.
    fn note<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.note.as_ref().map(Box::new).map(|c| c as Box<dyn Display>)
    }

    /// The severity level of this diagnostic.
    ///
    /// Diagnostics with missing severity levels should be treated as [errors](Severity::Error).
    fn severity(&self) -> Option<Severity> {
        Some(self.severity)
    }

    /// Labels covering problematic portions of source code.
    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        self.labels
            .as_ref()
            .map(|ls| ls.iter().cloned())
            .map(Box::new)
            .map(|b| b as Box<dyn Iterator<Item = LabeledSpan>>)
    }

    /// An error code uniquely identifying this diagnostic.
    ///
    /// Note that codes may be scoped, which will be rendered as `scope(code)`.
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.code.is_some().then(|| Box::new(&self.code) as Box<dyn Display>)
    }

    /// A URL that provides more information about the problem that occurred.
    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.url.as_ref().map(Box::new).map(|c| c as Box<dyn Display>)
    }
}

impl OxcDiagnostic {
    /// Create new an error-level [`OxcDiagnostic`].
    pub fn error<T: Into<Cow<'static, str>>>(message: T) -> Self {
        Self {
            inner: Box::new(OxcDiagnosticInner {
                message: message.into(),
                labels: None,
                note: None,
                help: None,
                severity: Severity::Error,
                code: OxcCode::default(),
                url: None,
                patches: None,
            }),
        }
    }

    /// Create new a warning-level [`OxcDiagnostic`].
    pub fn warn<T: Into<Cow<'static, str>>>(message: T) -> Self {
        Self {
            inner: Box::new(OxcDiagnosticInner {
                message: message.into(),
                labels: None,
                help: None,
                note: None,
                severity: Severity::Warning,
                code: OxcCode::default(),
                url: None,
                patches: None,
            }),
        }
    }

    /// Add a scoped error code to this diagnostic.
    ///
    /// This is a shorthand for `with_error_code_scope(scope).with_error_code_num(number)`.
    #[inline]
    pub fn with_error_code<T: Into<Cow<'static, str>>, U: Into<Cow<'static, str>>>(
        self,
        scope: T,
        number: U,
    ) -> Self {
        self.with_error_code_scope(scope).with_error_code_num(number)
    }

    /// Add an error code scope to this diagnostic.
    ///
    /// Use [`OxcDiagnostic::with_error_code`] to set both the scope and number at once.
    #[inline]
    pub fn with_error_code_scope<T: Into<Cow<'static, str>>>(mut self, code_scope: T) -> Self {
        self.inner.code.scope = match self.inner.code.scope {
            Some(scope) => Some(scope),
            None => Some(code_scope.into()),
        };
        debug_assert!(
            self.inner.code.scope.as_ref().is_some_and(|s| !s.is_empty()),
            "Error code scopes cannot be empty"
        );

        self
    }

    /// Add an error code number to this diagnostic.
    ///
    /// Use [`OxcDiagnostic::with_error_code`] to set both the scope and number at once.
    #[inline]
    pub fn with_error_code_num<T: Into<Cow<'static, str>>>(mut self, code_num: T) -> Self {
        self.inner.code.number = match self.inner.code.number {
            Some(num) => Some(num),
            None => Some(code_num.into()),
        };
        debug_assert!(
            self.inner.code.number.as_ref().is_some_and(|n| !n.is_empty()),
            "Error code numbers cannot be empty"
        );

        self
    }

    /// Set the severity level of this diagnostic.
    ///
    /// Use [`OxcDiagnostic::error`] or [`OxcDiagnostic::warn`] to create a diagnostic at the
    /// severity you want.
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.inner.severity = severity;
        self
    }

    /// Suggest a possible solution for a problem to the user.
    ///
    /// ## Example
    /// ```
    /// use std::path::PathBuf;
    /// use oxc_diagnostics::OxcDiagnostic
    ///
    /// let config_file_path = Path::from("config.json");
    /// if !config_file_path.exists() {
    ///     return Err(OxcDiagnostic::error("No config file found")
    ///         .with_help("Run my_tool --init to set up a new config file"));
    /// }
    /// ```
    pub fn with_help<T: Into<Cow<'static, str>>>(mut self, help: T) -> Self {
        self.inner.help = Some(help.into());
        self
    }

    /// Show a note to the user.
    ///
    /// ## Example
    /// ```
    /// use std::path::PathBuf;
    /// use oxc_diagnostics::OxcDiagnostic
    ///
    /// let config_file_path = Path::from("config.json");
    /// if !config_file_path.exists() {
    ///     return Err(OxcDiagnostic::error("No config file found")
    ///         .with_help("Run my_tool --init to set up a new config file")
    ///         .with_note("Some useful information or suggestion"));
    /// }
    /// ```
    pub fn with_note<T: Into<Cow<'static, str>>>(mut self, note: T) -> Self {
        self.inner.note = Some(note.into());
        self
    }

    /// Set the label covering a problematic portion of source code.
    ///
    /// Existing labels will be removed. Use [`OxcDiagnostic::and_label`] append a label instead.
    ///
    /// You need to add some source code to this diagnostic (using
    /// [`OxcDiagnostic::with_source_code`]) for this to actually be useful. Use
    /// [`OxcDiagnostic::with_labels`] to add multiple labels all at once.
    ///
    /// Note that this pairs nicely with [`oxc_span::Span`], particularly the [`label`] method.
    ///
    /// [`oxc_span::Span`]: https://docs.rs/oxc_span/latest/oxc_span/struct.Span.html
    /// [`label`]: https://docs.rs/oxc_span/latest/oxc_span/struct.Span.html#method.label
    pub fn with_label<T: Into<LabeledSpan>>(mut self, label: T) -> Self {
        self.inner.labels = Some(vec![label.into()]);
        self
    }

    /// Add multiple labels covering problematic portions of source code.
    ///
    /// Existing labels will be removed. Use [`OxcDiagnostic::and_labels`] to append labels
    /// instead.
    ///
    /// You need to add some source code using [`OxcDiagnostic::with_source_code`] for this to
    /// actually be useful. If you only have a single label, consider using
    /// [`OxcDiagnostic::with_label`] instead.
    ///
    /// Note that this pairs nicely with [`oxc_span::Span`], particularly the [`label`] method.
    ///
    /// [`oxc_span::Span`]: https://docs.rs/oxc_span/latest/oxc_span/struct.Span.html
    /// [`label`]: https://docs.rs/oxc_span/latest/oxc_span/struct.Span.html#method.label
    pub fn with_labels<L: Into<LabeledSpan>, T: IntoIterator<Item = L>>(
        mut self,
        labels: T,
    ) -> Self {
        self.inner.labels = Some(labels.into_iter().map(Into::into).collect());
        self
    }

    /// Add a label to this diagnostic without clobbering existing labels.
    pub fn and_label<T: Into<LabeledSpan>>(mut self, label: T) -> Self {
        let mut labels = self.inner.labels.unwrap_or_default();
        labels.push(label.into());
        self.inner.labels = Some(labels);
        self
    }

    /// Add multiple labels to this diagnostic without clobbering existing labels.
    pub fn and_labels<L: Into<LabeledSpan>, T: IntoIterator<Item = L>>(
        mut self,
        labels: T,
    ) -> Self {
        let mut all_labels = self.inner.labels.unwrap_or_default();
        all_labels.extend(labels.into_iter().map(Into::into));
        self.inner.labels = Some(all_labels);
        self
    }

    /// Add a URL that provides more information about this diagnostic.
    pub fn with_url<S: Into<Cow<'static, str>>>(mut self, url: S) -> Self {
        self.inner.url = Some(url.into());
        self
    }

    /// Set a single fix (which may consist of multiple patches) for this diagnostic.
    ///
    /// Existing patches will be removed. Use [`OxcDiagnostic::and_fix`] to add additional
    /// alternative fixes.
    ///
    /// Each patch in the provided iterator represents an edit that should be applied together
    /// as part of this fix.
    pub fn with_fix<P: Into<Patch>, T: IntoIterator<Item = P>>(mut self, patches: T) -> Self {
        self.inner.patches = Some(vec![patches.into_iter().map(Into::into).collect()]);
        self
    }

    /// Add an alternative fix to this diagnostic without removing existing fixes.
    ///
    /// Each call adds another possible fix. When rendered, all alternative fixes will be shown.
    pub fn and_fix<P: Into<Patch>, T: IntoIterator<Item = P>>(mut self, patches: T) -> Self {
        let mut all_patches = self.inner.patches.unwrap_or_default();
        all_patches.push(patches.into_iter().map(Into::into).collect());
        self.inner.patches = Some(all_patches);
        self
    }

    /// Set multiple alternative fixes for this diagnostic.
    ///
    /// Each inner iterator represents one possible fix (which may consist of multiple patches
    /// that should be applied together). The outer iterator contains all alternatives.
    pub fn with_fixes<P: Into<Patch>, I: IntoIterator<Item = P>, T: IntoIterator<Item = I>>(
        mut self,
        fixes: T,
    ) -> Self {
        self.inner.patches =
            Some(fixes.into_iter().map(|f| f.into_iter().map(Into::into).collect()).collect());
        self
    }

    /// Add source code to this diagnostic and convert it into an [`Error`].
    ///
    /// You should use a [`NamedSource`] if you have a file name as well as the source code.
    pub fn with_source_code<T: SourceCode + Send + Sync + 'static>(self, code: T) -> Error {
        Error::from(self).with_source_code(code)
    }

    /// Consumes the diagnostic and returns the inner owned data.
    pub fn inner_owned(self) -> OxcDiagnosticInner {
        *self.inner
    }
}
