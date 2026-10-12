use std::{borrow::Cow, fmt, sync::Arc};

use serde::{Deserialize, Serialize};

use oxc_diagnostics::{Diagnostic, Error, LabeledSpan, NamedSource, Severity, SourceCode};
use oxc_span::Span;

/// A diagnostic as stored in the cache file.
///
/// Only the parts that output formatters read are stored. The source text is not stored;
/// it is read from the (unchanged) file when the diagnostic is replayed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedDiagnostic {
    message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    severity: Option<CachedSeverity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    help: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    labels: Vec<CachedLabel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CachedSeverity {
    Advice,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CachedLabel {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    offset: u32,
    len: u32,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    primary: bool,
}

impl CachedDiagnostic {
    pub fn from_diagnostic(diagnostic: &dyn Diagnostic) -> Self {
        Self {
            message: diagnostic.to_string(),
            code: diagnostic.code().map(Cow::into_owned),
            severity: diagnostic.severity().map(|severity| match severity {
                Severity::Advice => CachedSeverity::Advice,
                Severity::Warning => CachedSeverity::Warning,
                Severity::Error => CachedSeverity::Error,
            }),
            help: diagnostic.help().map(Cow::into_owned),
            note: diagnostic.note().map(Cow::into_owned),
            url: diagnostic.url().map(Cow::into_owned),
            labels: diagnostic
                .labels()
                .iter()
                .map(|label| CachedLabel {
                    label: label.label().map(ToString::to_string),
                    offset: label.offset(),
                    len: label.len(),
                    primary: label.primary(),
                })
                .collect(),
        }
    }

    /// Rebuild a diagnostic that renders the same as the one it was captured from.
    pub fn replay(&self, source: Arc<NamedSource<String>>) -> Error {
        Box::new(ReplayedDiagnostic {
            message: self.message.clone(),
            code: self.code.clone(),
            severity: self.severity.map(|severity| match severity {
                CachedSeverity::Advice => Severity::Advice,
                CachedSeverity::Warning => Severity::Warning,
                CachedSeverity::Error => Severity::Error,
            }),
            help: self.help.clone(),
            note: self.note.clone(),
            url: self.url.clone(),
            labels: self
                .labels
                .iter()
                .map(|label| {
                    let span = Span::sized(label.offset, label.len);
                    if label.primary {
                        LabeledSpan::new_primary_with_span(label.label.clone(), span)
                    } else {
                        LabeledSpan::new_with_span(label.label.clone(), span)
                    }
                })
                .collect(),
            source,
        })
    }
}

#[derive(Debug)]
struct ReplayedDiagnostic {
    message: String,
    code: Option<String>,
    severity: Option<Severity>,
    help: Option<String>,
    note: Option<String>,
    url: Option<String>,
    labels: Vec<LabeledSpan>,
    source: Arc<NamedSource<String>>,
}

impl fmt::Display for ReplayedDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ReplayedDiagnostic {}

impl Diagnostic for ReplayedDiagnostic {
    fn code(&self) -> Option<Cow<'_, str>> {
        self.code.as_deref().map(Cow::Borrowed)
    }

    fn severity(&self) -> Option<Severity> {
        self.severity
    }

    fn help(&self) -> Option<Cow<'_, str>> {
        self.help.as_deref().map(Cow::Borrowed)
    }

    fn note(&self) -> Option<Cow<'_, str>> {
        self.note.as_deref().map(Cow::Borrowed)
    }

    fn url(&self) -> Option<Cow<'_, str>> {
        self.url.as_deref().map(Cow::Borrowed)
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&*self.source)
    }

    fn labels(&self) -> &[LabeledSpan] {
        &self.labels
    }
}
