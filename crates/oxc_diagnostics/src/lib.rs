//! Diagnostics Wrapper
//! Exports `miette`

mod graphic_reporter;
mod graphical_theme;
mod reporter;
mod service;

use std::{
    fmt::{self, Display},
    ops::Deref,
};

pub use crate::{
    graphic_reporter::GraphicalReportHandler,
    graphical_theme::GraphicalTheme,
    service::{DiagnosticSender, DiagnosticService, DiagnosticTuple},
};

pub type Error = miette::Error;
pub type Severity = miette::Severity;

pub type Result<T> = std::result::Result<T, OxcDiagnostic>;

use miette::{Diagnostic, SourceCode};
pub use miette::{LabeledSpan, NamedSource};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct OxcDiagnosticInner {
    pub message: String,
    pub labels: Option<Vec<LabeledSpan>>,
    pub help: Option<String>,
    pub severity: Severity,
}

impl fmt::Display for OxcDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.message)
    }
}

impl std::error::Error for OxcDiagnostic {}

impl Diagnostic for OxcDiagnostic {
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.help.as_ref().map(Box::new).map(|c| c as Box<dyn Display>)
    }

    fn severity(&self) -> Option<Severity> {
        Some(self.severity)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        self.labels
            .as_ref()
            .map(|ls| ls.iter().cloned())
            .map(Box::new)
            .map(|b| b as Box<dyn Iterator<Item = LabeledSpan>>)
    }
}

impl OxcDiagnostic {
    #[must_use]
    pub fn error<T: Into<String>>(message: T) -> Self {
        Self {
            inner: Box::new(OxcDiagnosticInner {
                message: message.into(),
                labels: None,
                help: None,
                severity: Severity::Error,
            }),
        }
    }

    #[must_use]
    pub fn warn<T: Into<String>>(message: T) -> Self {
        Self {
            inner: Box::new(OxcDiagnosticInner {
                message: message.into(),
                labels: None,
                help: None,
                severity: Severity::Warning,
            }),
        }
    }

    #[must_use]
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.inner.severity = severity;
        self
    }

    #[must_use]
    pub fn with_help<T: Into<String>>(mut self, help: T) -> Self {
        self.inner.help = Some(help.into());
        self
    }

    #[must_use]
    pub fn with_label<T: Into<LabeledSpan>>(mut self, label: T) -> Self {
        self.inner.labels = Some(vec![label.into()]);
        self
    }

    #[must_use]
    pub fn with_labels<T: IntoIterator<Item = LabeledSpan>>(mut self, labels: T) -> Self {
        self.inner.labels = Some(labels.into_iter().collect());
        self
    }

    #[must_use]
    pub fn and_label<T: Into<LabeledSpan>>(mut self, label: T) -> Self {
        let mut labels = self.inner.labels.unwrap_or_default();
        labels.push(label.into());
        self.inner.labels = Some(labels);
        self
    }

    #[must_use]
    pub fn and_labels<T: IntoIterator<Item = LabeledSpan>>(mut self, labels: T) -> Self {
        let mut all_labels = self.inner.labels.unwrap_or_default();
        all_labels.extend(labels);
        self.inner.labels = Some(all_labels);
        self
    }

    #[must_use]
    pub fn with_source_code<T: SourceCode + Send + Sync + 'static>(self, code: T) -> Error {
        Error::from(self).with_source_code(code)
    }
}
