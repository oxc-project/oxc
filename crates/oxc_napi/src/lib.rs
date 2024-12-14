use napi_derive::napi;

use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};

#[napi(object)]
pub struct OxcError {
    pub severity: Severity,
    pub message: String,
    pub labels: Vec<ErrorLabel>,
    pub help_message: Option<String>,
}

impl OxcError {
    pub fn new(message: String) -> Self {
        Self { severity: Severity::Error, message, labels: vec![], help_message: None }
    }
}

impl From<OxcDiagnostic> for OxcError {
    fn from(diagnostic: OxcDiagnostic) -> Self {
        let labels = diagnostic
            .labels
            .as_ref()
            .map(|labels| labels.iter().map(ErrorLabel::from).collect::<Vec<_>>())
            .unwrap_or_default();
        Self {
            severity: Severity::from(diagnostic.severity),
            message: diagnostic.message.to_string(),
            labels,
            help_message: diagnostic.help.as_ref().map(ToString::to_string),
        }
    }
}

#[napi(object)]
pub struct ErrorLabel {
    pub message: Option<String>,
    pub start: u32,
    pub end: u32,
}

impl From<&LabeledSpan> for ErrorLabel {
    #[allow(clippy::cast_possible_truncation)]
    fn from(label: &LabeledSpan) -> Self {
        Self {
            message: label.label().map(ToString::to_string),
            start: label.offset() as u32,
            end: (label.offset() + label.len()) as u32,
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
