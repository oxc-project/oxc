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
//! use std::{sync::Arc, thread};
//! use oxc_diagnostics::{DiagnosticService, Error, OxcDiagnostic};
//!
//! fn my_tool() -> Result<()> {
//!     try_something().map_err(|e| OxcDiagnostic::error(e.to_string()))?;
//!     Ok(())
//! }
//!
//! let mut service = DiagnosticService::default();
//! let mut sender = service.sender().clone();
//!
//! thread::spawn(move || {
//!     let file_path_being_processed = PathBuf::from("file.txt");
//!     let file_being_processed = Arc::new(NamedSource::new(file_path_being_processed.clone()));
//!
//!     for _ in 0..10 {
//!         if let Err(diagnostic) = my_tool() {
//!             let report = diagnostic.with_source_code(Arc::clone(&file_being_processed));
//!             sender.send(Some(file_path_being_processed, vec![Error::new(e)]));
//!         }
//!         // send None to stop the service
//!         sender.send(None);
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
};

pub mod reporter;

pub use crate::service::{DiagnosticSender, DiagnosticService, DiagnosticTuple};

pub type Error = miette::Error;
pub type Severity = miette::Severity;

pub type Result<T> = std::result::Result<T, OxcDiagnostic>;

use miette::{Diagnostic, SourceCode};
pub use miette::{GraphicalReportHandler, GraphicalTheme, LabeledSpan, NamedSource};

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

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct OxcCode {
    pub scope: Option<Cow<'static, str>>,
    pub number: Option<Cow<'static, str>>,
}

impl OxcCode {
    pub fn is_some(&self) -> bool {
        self.scope.is_some() || self.number.is_some()
    }
}

impl fmt::Display for OxcCode {
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
    pub severity: Severity,
    pub code: OxcCode,
    pub url: Option<Cow<'static, str>>,
}

impl fmt::Display for OxcDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.message)
    }
}

impl std::error::Error for OxcDiagnostic {}

impl Diagnostic for OxcDiagnostic {
    /// The secondary help message.
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.help.as_ref().map(Box::new).map(|c| c as Box<dyn Display>)
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
                help: None,
                severity: Severity::Error,
                code: OxcCode::default(),
                url: None,
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
                severity: Severity::Warning,
                code: OxcCode::default(),
                url: None,
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

    /// Add source code to this diagnostic and convert it into an [`Error`].
    ///
    /// You should use a [`NamedSource`] if you have a file name as well as the source code.
    pub fn with_source_code<T: SourceCode + Send + Sync + 'static>(self, code: T) -> Error {
        Error::from(self).with_source_code(code)
    }
}
