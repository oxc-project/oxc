//! Errors raised by the [`Printer`](super::Printer) when a [`Document`](crate::Document)
//! has an invalid structure (unbalanced or mismatched tags).

use std::error::Error;

use crate::TagKind;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InvalidDocumentError {
    /// Mismatching start/end kinds
    ///
    /// ```plain
    /// StartIndent
    /// ...
    /// EndGroup
    /// ```
    StartEndTagMismatch { start_kind: TagKind, end_kind: TagKind },

    /// End tag without a corresponding start tag.
    ///
    /// ```plain
    /// Text
    /// EndGroup
    /// ```
    StartTagMissing { kind: TagKind },

    /// Expected a specific start tag but instead is:
    /// * at the end of the document
    /// * at another start tag
    /// * at an end tag
    ExpectedStart { expected_start: TagKind, actual: ActualStart },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ActualStart {
    /// The actual element is not a tag.
    Content,

    /// The actual element was a start tag of another kind.
    Start(TagKind),

    /// The actual element is an end tag instead of a start tag.
    End(TagKind),

    /// Reached the end of the document
    EndOfDocument,
}

impl std::fmt::Display for InvalidDocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidDocumentError::StartEndTagMismatch { start_kind, end_kind } => {
                std::write!(f, "Expected end tag of kind {start_kind:?} but found {end_kind:?}.")
            }
            InvalidDocumentError::StartTagMissing { kind } => {
                std::write!(f, "End tag of kind {kind:?} without matching start tag.")
            }
            InvalidDocumentError::ExpectedStart { expected_start, actual } => match actual {
                ActualStart::EndOfDocument => {
                    std::write!(
                        f,
                        "Expected start tag of kind {expected_start:?} but at the end of document."
                    )
                }
                ActualStart::Start(start) => {
                    std::write!(
                        f,
                        "Expected start tag of kind {expected_start:?} but found start tag of kind {start:?}."
                    )
                }
                ActualStart::End(end) => {
                    std::write!(
                        f,
                        "Expected start tag of kind {expected_start:?} but found end tag of kind {end:?}."
                    )
                }
                ActualStart::Content => {
                    std::write!(
                        f,
                        "Expected start tag of kind {expected_start:?} but found non-tag element."
                    )
                }
            },
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PrintError {
    InvalidDocument(InvalidDocumentError),
}

impl Error for PrintError {}

impl std::fmt::Display for PrintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrintError::InvalidDocument(inner) => {
                std::write!(f, "Invalid document: {inner}")
            }
        }
    }
}
