//! Code related to error handling.

use oxc_allocator::Dummy;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::{ParserConfig as Config, ParserImpl, diagnostics, lexer::Kind};

/// Fatal parsing error.
#[derive(Debug, Clone)]
pub struct FatalError {
    /// What went wrong. The actual `OxcDiagnostic` is only materialized at the
    /// top-level parser exit — before that, `FatalErrorKind` carries just the
    /// inputs needed to rebuild the message.
    pub kind: FatalErrorKind,
    /// Length of `errors` at time fatal error is recorded.
    pub errors_len: usize,
}

/// Lazy description of a fatal error.
///
/// Variants are `Copy` so that `set_fatal_error` inside a speculative
/// block allocates nothing even when the error will be rewound.
/// `Other` is the escape hatch for cold call sites that already hold an
/// owned `OxcDiagnostic`.
#[derive(Debug, Clone)]
pub enum FatalErrorKind {
    /// `diagnostics::unexpected_token(span)`
    Unexpected(Span),
    /// `diagnostics::expect_token(expected, actual, span)`
    Expect { expected: Kind, actual: Kind, span: Span },
    /// `diagnostics::expect_closing(expected, actual, span, opening)`
    ExpectClosing { expected: Kind, actual: Kind, span: Span, opening: Span },
    /// `diagnostics::expect_conditional_alternative(actual, span, question)`
    ExpectConditionalAlternative { actual: Kind, span: Span, question: Span },
    /// `diagnostics::auto_semicolon_insertion(span)`
    AutoSemicolonInsertion(Span),
    /// Pre-built diagnostic. Used by cold call sites (not in any speculative path)
    /// and by `set_unexpected`'s lexer-error-pop case.
    Other(OxcDiagnostic),
}

impl FatalErrorKind {
    #[cold]
    pub fn into_diagnostic(self) -> OxcDiagnostic {
        match self {
            Self::Unexpected(span) => diagnostics::unexpected_token(span),
            Self::Expect { expected, actual, span } => {
                diagnostics::expect_token(expected.to_str(), actual.to_str(), span)
            }
            Self::ExpectClosing { expected, actual, span, opening } => {
                diagnostics::expect_closing(expected.to_str(), actual.to_str(), span, opening)
            }
            Self::ExpectConditionalAlternative { actual, span, question } => {
                diagnostics::expect_conditional_alternative(actual.to_str(), span, question)
            }
            Self::AutoSemicolonInsertion(span) => diagnostics::auto_semicolon_insertion(span),
            Self::Other(d) => d,
        }
    }
}

impl From<OxcDiagnostic> for FatalErrorKind {
    fn from(error: OxcDiagnostic) -> Self {
        Self::Other(error)
    }
}

impl<'a, C: Config> ParserImpl<'a, C> {
    #[cold]
    pub(crate) fn set_unexpected(&mut self) {
        // The lexer should have reported a more meaningful diagnostic
        // when it is a undetermined kind.
        if matches!(self.cur_kind(), Kind::Eof | Kind::Undetermined)
            && let Some(error) = self.lexer.errors.pop()
        {
            self.set_fatal_error(error);
            return;
        }

        // Check if this looks like a merge conflict marker
        if let Some(start_span) = self.is_merge_conflict_marker() {
            let (middle_span, end_span) = self.find_merge_conflict_markers();
            let error = diagnostics::merge_conflict_marker(start_span, middle_span, end_span);
            self.set_fatal_error(error);
            return;
        }

        self.set_fatal_error(FatalErrorKind::Unexpected(self.cur_token().span()));
    }

    /// Return error info at current token
    ///
    /// # Panics
    ///
    ///   * The lexer did not push a diagnostic when `Kind::Undetermined` is returned
    #[must_use]
    #[cold]
    pub(crate) fn unexpected<T: Dummy<'a>>(&mut self) -> T {
        self.set_unexpected();
        Dummy::dummy(self.ast.allocator)
    }

    /// Push a Syntax Error
    #[cold]
    pub(crate) fn error(&mut self, error: OxcDiagnostic) {
        self.errors.push(error);
    }

    /// Defer an error that is only valid if the file is a Script (not a Module).
    ///
    /// For `ModuleKind::Unambiguous`, we don't know the module type until parsing is complete.
    /// Errors like "await outside async function" are only valid if the file ends up being
    /// a Script. If ESM syntax is found, the file becomes a Module and these errors are discarded.
    #[cold]
    pub(crate) fn error_on_script(&mut self, error: OxcDiagnostic) {
        if self.source_type.is_unambiguous() {
            self.deferred_script_errors.push(error);
        } else {
            self.errors.push(error);
        }
    }

    /// Count of all parser and lexer errors.
    pub(crate) fn errors_count(&self) -> usize {
        self.errors.len() + self.lexer.errors.len()
    }

    /// Record a fatal error and advance the lexer to EOF.
    ///
    /// Cheap call sites build a `FatalErrorKind` variant directly
    /// (no heap). Call sites that already hold an `OxcDiagnostic`
    /// rely on the `From<OxcDiagnostic>` impl to auto-wrap in
    /// `FatalErrorKind::Other`.
    #[cold]
    pub(crate) fn set_fatal_error(&mut self, kind: impl Into<FatalErrorKind>) {
        if self.fatal_error.is_none() {
            self.lexer.advance_to_end();
            self.fatal_error =
                Some(FatalError { kind: kind.into(), errors_len: self.errors.len() });
        }
    }

    #[cold]
    pub(crate) fn fatal_error<T: Dummy<'a>>(&mut self, kind: impl Into<FatalErrorKind>) -> T {
        self.set_fatal_error(kind);
        Dummy::dummy(self.ast.allocator)
    }

    pub(crate) fn has_fatal_error(&self) -> bool {
        matches!(self.cur_kind(), Kind::Eof | Kind::Undetermined) || self.fatal_error.is_some()
    }
}

// ==================== Merge Conflict Marker Detection ====================
//
// Git merge conflict markers detection and error recovery.
//
// This provides enhanced diagnostics when the parser encounters Git merge conflict markers
// (e.g., `<<<<<<<`, `=======`, `>>>>>>>`). Instead of showing a generic "Unexpected token"
// error, we detect these patterns and provide helpful guidance on how to resolve the conflict.
//
// Inspired by rust-lang/rust#106242
impl<C: Config> ParserImpl<'_, C> {
    /// Check if the current position looks like a merge conflict marker.
    ///
    /// Detects the following Git conflict markers:
    /// - `<<<<<<<` - Start marker (ours)
    /// - `=======` - Middle separator
    /// - `>>>>>>>` - End marker (theirs)
    /// - `|||||||` - Diff3 format (common ancestor)
    ///
    /// Returns the span of the marker if detected, None otherwise.
    ///
    /// # False Positive Prevention
    ///
    /// Git conflict markers always appear at the start of a line. To prevent false positives
    /// from operator sequences in valid code (e.g., `a << << b`), we verify that the first
    /// token is on a new line using the `is_on_new_line()` flag from the lexer.
    ///
    /// The special case `span.start == 0` handles the beginning of the file, where
    /// `is_on_new_line()` may be false but a conflict marker is still valid.
    fn is_merge_conflict_marker(&self) -> Option<Span> {
        let token = self.cur_token();
        let span = token.span();

        // Git conflict markers always appear at start of line.
        // This prevents false positives from operator sequences like `a << << b`.
        // At the start of the file (span.start == 0), we allow the check to proceed
        // even if is_on_new_line() is false, since there's no preceding line.
        if !token.is_on_new_line() && span.start != 0 {
            return None;
        }

        // Get the remaining source text from the current position
        let remaining = &self.source_text[span.start as usize..];

        // Check for each conflict marker pattern (all are exactly 7 ASCII characters)
        // Git conflict markers are always ASCII, so we can safely use byte slicing
        if remaining.starts_with("<<<<<<<")
            || remaining.starts_with("=======")
            || remaining.starts_with(">>>>>>>")
            || remaining.starts_with("|||||||")
        {
            // Marker length is 7 bytes (all ASCII characters)
            return Some(Span::new(span.start, span.start + 7));
        }

        None
    }

    /// Scans forward to find the middle and end markers of a merge conflict.
    ///
    /// After detecting the start marker (`<<<<<<<`), this function scans forward to find:
    /// - The middle marker (`=======`)
    /// - The end marker (`>>>>>>>`)
    ///
    /// The diff3 marker (`|||||||`) is recognized but not returned, as it appears between
    /// the start and middle markers and doesn't need separate labeling in the diagnostic.
    ///
    /// Returns `(middle_span, end_span)` where:
    /// - `middle_span` is the location of `=======` (if found)
    /// - `end_span` is the location of `>>>>>>>` (if found)
    ///
    /// Uses a checkpoint to rewind the parser state after scanning, leaving the parser
    /// positioned at the start marker.
    ///
    /// # Nested Conflicts
    ///
    /// If nested conflict markers are encountered (e.g., a conflict within a conflict),
    /// this function returns the first complete set of markers found. The parser will
    /// stop with a fatal error at the first conflict, so nested conflicts won't be
    /// fully analyzed until the outer conflict is resolved.
    ///
    /// The diagnostic message includes a note about nested conflicts to guide users
    /// to resolve the outermost conflict first.
    fn find_merge_conflict_markers(&mut self) -> (Option<Span>, Option<Span>) {
        let checkpoint = self.checkpoint();
        let mut middle_span = None;

        loop {
            self.bump_any();

            if self.cur_kind() == Kind::Eof {
                self.rewind(checkpoint);
                return (middle_span, None);
            }

            // Check if we've hit a conflict marker
            if let Some(marker_span) = self.is_merge_conflict_marker() {
                let span = self.cur_token().span();
                let remaining = &self.source_text[span.start as usize..];

                if remaining.starts_with("=======") && middle_span.is_none() {
                    // Found middle marker
                    middle_span = Some(marker_span);
                } else if remaining.starts_with(">>>>>>>") {
                    // Found end marker
                    let result = (middle_span, Some(marker_span));
                    self.rewind(checkpoint);
                    return result;
                }
                // Skip other markers (like diff3 `|||||||` or nested start markers `<<<<<<<`)
            }
        }
    }
}
