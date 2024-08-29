//! Diagnostics Wrapper
//! Exports `miette`

mod graphic_reporter;
mod graphical_theme;
mod reporter;
mod service;

use std::{
    borrow::Cow,
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

#[derive(Debug, Default, Clone)]
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

#[derive(Debug, Clone)]
pub struct OxcDiagnosticInner {
    pub message: Cow<'static, str>,
    pub labels: Option<Vec<LabeledSpan>>,
    pub help: Option<Cow<'static, str>>,
    pub severity: Severity,
    pub code: OxcCode,
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

    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.code.is_some().then(|| Box::new(&self.code) as Box<dyn Display>)
    }
}

impl OxcDiagnostic {
    pub fn error<T: Into<Cow<'static, str>>>(message: T) -> Self {
        Self {
            inner: Box::new(OxcDiagnosticInner {
                message: message.into(),
                labels: None,
                help: None,
                severity: Severity::Error,
                code: OxcCode::default(),
            }),
        }
    }

    pub fn warn<T: Into<Cow<'static, str>>>(message: T) -> Self {
        Self {
            inner: Box::new(OxcDiagnosticInner {
                message: message.into(),
                labels: None,
                help: None,
                severity: Severity::Warning,
                code: OxcCode::default(),
            }),
        }
    }

    #[inline]
    pub fn with_error_code<T: Into<Cow<'static, str>>, U: Into<Cow<'static, str>>>(
        self,
        scope: T,
        number: U,
    ) -> Self {
        self.with_error_code_scope(scope).with_error_code_num(number)
    }

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

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.inner.severity = severity;
        self
    }

    pub fn with_help<T: Into<Cow<'static, str>>>(mut self, help: T) -> Self {
        self.inner.help = Some(help.into());
        self
    }

    pub fn with_label<T: Into<LabeledSpan>>(mut self, label: T) -> Self {
        self.inner.labels = Some(vec![label.into()]);
        self
    }

    pub fn with_labels<L: Into<LabeledSpan>, T: IntoIterator<Item = L>>(
        mut self,
        labels: T,
    ) -> Self {
        self.inner.labels = Some(labels.into_iter().map(Into::into).collect());
        self
    }

    pub fn and_label<T: Into<LabeledSpan>>(mut self, label: T) -> Self {
        let mut labels = self.inner.labels.unwrap_or_default();
        labels.push(label.into());
        self.inner.labels = Some(labels);
        self
    }

    pub fn and_labels<L: Into<LabeledSpan>, T: IntoIterator<Item = L>>(
        mut self,
        labels: T,
    ) -> Self {
        let mut all_labels = self.inner.labels.unwrap_or_default();
        all_labels.extend(labels.into_iter().map(Into::into));
        self.inner.labels = Some(all_labels);
        self
    }

    pub fn with_source_code<T: SourceCode + Send + Sync + 'static>(self, code: T) -> Error {
        Error::from(self).with_source_code(code)
    }
}
