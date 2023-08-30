use std::{
    fmt::{Debug, Display},
    path::PathBuf,
    sync::Arc,
};

use oxc_diagnostics::{
    miette::{self, Diagnostic, NamedSource, SourceSpan},
    thiserror::{self, Error},
    Report,
};
use oxc_span::Span;

#[derive(Debug, Error, Diagnostic)]
pub enum ErrorFromLinterPlugin {
    #[error("{0}")]
    PluginGenerated(String, String, #[label("{1}")] Span),
    #[error("{error_message}")]
    Trustfall {
        error_message: String,
        #[source_code]
        query_source: Arc<NamedSource>,
        #[label = "This query failed."]
        query_span: SourceSpan,
    },
    #[error(transparent)]
    Ignore(ignore::Error),
    #[error(transparent)]
    ReadFile(std::io::Error),
    #[error("Failed to parse file at path: {0}")]
    QueryParse(PathBuf, #[related] Vec<ParseError>),
    #[error(
        "Expected span_start and span_end to be List of Int or Int, instead got\nspan_start = {span_start}\nspan_end = {span_end}"
    )]
    WrongTypeForSpanStartSpanEnd {
        span_start: String,
        span_end: String,
        #[source_code]
        query_source: Arc<NamedSource>,
        #[label = "This query failed."]
        query_span: SourceSpan,
    },
    #[error("Expected {which_span} to fit into a u32, however {number} didn't fit into u32.")]
    SpanStartOrEndDoesntFitInU32 {
        which_span: SpanStartOrEnd,
        number: i128, // i128 because it can fit i64 and u64
        #[source_code]
        query_source: Arc<NamedSource>,
        #[label = "This query failed."]
        query_span: SourceSpan,
    },
}

#[derive(Debug, Error, Diagnostic)]
#[error("Test expected to pass, but failed.")]
pub struct ExpectedTestToPassButFailed {
    #[source_code]
    pub query: NamedSource,
    #[label = "This test failed."]
    pub err_span: SourceSpan,
    #[related]
    pub errors: Vec<Report>,
}

#[derive(Debug, Error, Diagnostic)]
#[error("Test expected to fail, but passed.")]
pub struct ExpectedTestToFailButPassed {
    #[source_code]
    pub query: NamedSource,
    #[label = "This test should have failed but it passed."]
    pub err_span: SourceSpan,
}

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected errors in fail test.")]
pub struct UnexpectedErrorsInFailTest {
    #[related]
    pub errors: Vec<Report>,
    #[source_code]
    pub query: NamedSource,
    #[label = "This test should have failed but it passed."]
    pub err_span: SourceSpan,
}

#[derive(Debug, Error, Diagnostic)]
#[error(transparent)]
pub struct ParseError(serde_yaml::Error);

impl From<serde_yaml::Error> for ParseError {
    fn from(error: serde_yaml::Error) -> Self {
        Self(error)
    }
}

/// Represents either the start or the end of a span, which is used
/// for error messages that need to point to a specific part of a
/// span that failed to be used for some reason.
pub enum SpanStartOrEnd {
    Start,
    End,
}

impl Debug for SpanStartOrEnd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Start => write!(f, "SpanStart"),
            Self::End => write!(f, "SpanEnd"),
        }
    }
}

impl Display for SpanStartOrEnd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Start => write!(f, "span_start"),
            Self::End => write!(f, "span_end"),
        }
    }
}
