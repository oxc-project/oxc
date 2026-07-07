use std::sync::Arc;

use napi_derive::napi;

use oxc_diagnostics::{LabeledSpan, NamedSource, OxcDiagnostic};

#[napi(object, use_nullable = true)]
pub struct OxcError {
    pub severity: Severity,
    pub message: String,
    pub labels: Vec<ErrorLabel>,
    pub help_message: Option<String>,
    pub codeframe: Option<String>,
}

impl OxcError {
    pub fn new(message: String) -> Self {
        Self {
            severity: Severity::Error,
            message,
            labels: vec![],
            help_message: None,
            codeframe: None,
        }
    }

    pub fn from_diagnostics(
        filename: &str,
        source_text: &str,
        diagnostics: impl IntoIterator<Item = OxcDiagnostic>,
    ) -> Vec<Self> {
        let diagnostics = diagnostics.into_iter().collect::<Vec<_>>();
        if diagnostics.is_empty() {
            return vec![];
        }
        let source = Arc::new(NamedSource::new(filename, source_text.to_string()));
        diagnostics.into_iter().map(|e| Self::from_diagnostic(&source, e)).collect()
    }

    /// Rendering the codeframe requires miette's graphical renderer, which is
    /// only linked when the `fancy` feature is enabled.
    #[cfg(feature = "fancy")]
    pub fn from_diagnostic(
        named_source: &Arc<NamedSource<String>>,
        diagnostic: OxcDiagnostic,
    ) -> Self {
        let mut error = Self::from(&diagnostic);
        let codeframe = diagnostic.with_source_code(Arc::clone(named_source));
        error.codeframe = Some(format!("{codeframe:?}"));
        error
    }

    /// Without the `fancy` feature the graphical renderer is not linked and
    /// `codeframe` stays `None`.
    #[cfg(not(feature = "fancy"))]
    pub fn from_diagnostic(
        _named_source: &Arc<NamedSource<String>>,
        diagnostic: OxcDiagnostic,
    ) -> Self {
        let error = Self::from(&diagnostic);
        drop(diagnostic);
        error
    }
}

impl From<&OxcDiagnostic> for OxcError {
    fn from(diagnostic: &OxcDiagnostic) -> Self {
        let labels = diagnostic.labels.iter().map(ErrorLabel::from).collect::<Vec<_>>();
        Self {
            severity: Severity::from(diagnostic.severity),
            message: diagnostic.message.to_string(),
            labels,
            help_message: diagnostic.help.as_ref().map(ToString::to_string),
            codeframe: None,
        }
    }
}

#[napi(object, use_nullable = true)]
pub struct ErrorLabel {
    pub message: Option<String>,
    pub start: u32,
    pub end: u32,
}

impl From<&LabeledSpan> for ErrorLabel {
    fn from(label: &LabeledSpan) -> Self {
        Self {
            message: label.label().map(ToString::to_string),
            start: label.offset(),
            end: label.offset() + label.len(),
        }
    }
}

#[napi(string_enum)]
pub enum Severity {
    Error,
    Warning,
    Advice,
}

impl From<oxc_diagnostics::Severity> for Severity {
    fn from(value: oxc_diagnostics::Severity) -> Self {
        match value {
            oxc_diagnostics::Severity::Error => Self::Error,
            oxc_diagnostics::Severity::Warning => Self::Warning,
            oxc_diagnostics::Severity::Advice => Self::Advice,
        }
    }
}
