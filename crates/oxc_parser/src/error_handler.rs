//! Code related to error handling.

use oxc_allocator::{Dummy, GetAllocator};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::{ParserConfig as Config, ParserImpl, diagnostics, lexer::Kind};

/// Fatal parsing error.
#[derive(Debug, Clone)]
pub struct FatalError {
    /// The fatal error, kept lazy. The parser records a fatal error on every
    /// `expect` failure, including the many speculative ones during
    /// lookahead/backtracking that are discarded on `rewind`; keeping it
    /// un-built avoids paying for a diagnostic that is never reported.
    pub error: LazyDiagnostic,
    /// Length of `errors` at time fatal error is recorded
    pub errors_len: usize,
}

/// A fatal-error diagnostic kept in cheap, un-built form.
///
/// Building the `OxcDiagnostic` for an `expect` failure (two `format!` strings
/// plus a label vec) is pure waste when the failure is speculative and gets
/// discarded on `rewind` — on real files this is ~80-98% of parser heap
/// allocations. The hot `expect_*` failures store their (all-`Copy`) operands
/// here instead, and the diagnostic is built only when actually reported, via
/// [`LazyDiagnostic::into_diagnostic`]. All operands are `&'static str` (from
/// [`Kind::to_str`](crate::lexer::Kind::to_str)) plus `Span`, so recording a
/// fatal error allocates nothing.
#[derive(Debug, Clone)]
pub enum LazyDiagnostic {
    ExpectToken {
        expected: &'static str,
        found: &'static str,
        span: Span,
    },
    ExpectClosingOrSeparator {
        closing: &'static str,
        separator: &'static str,
        found: &'static str,
        span: Span,
        opening_span: Span,
    },
    ExpectClosing {
        closing: &'static str,
        found: &'static str,
        span: Span,
        opening_span: Span,
    },
    ExpectConditionalAlternative {
        found: &'static str,
        span: Span,
        question_span: Span,
    },
    /// Already-built diagnostic for the cold error sites. Stored inline (not
    /// boxed) so the cold path allocates no more than before; the larger enum is
    /// only ever moved (never cloned) through the parser's checkpoint/rewind.
    Eager(OxcDiagnostic),
}

impl LazyDiagnostic {
    /// Build the `OxcDiagnostic`. Called only when a fatal error is reported, so
    /// per-`expect`-failure work stays allocation-free during speculation. The
    /// output is byte-identical to building eagerly: it replays the same
    /// `diagnostics::` constructor with the same operands.
    #[cold]
    pub fn into_diagnostic(self) -> OxcDiagnostic {
        match self {
            LazyDiagnostic::ExpectToken { expected, found, span } => {
                diagnostics::expect_token(expected, found, span)
            }
            LazyDiagnostic::ExpectClosingOrSeparator {
                closing,
                separator,
                found,
                span,
                opening_span,
            } => diagnostics::expect_closing_or_separator(
                closing,
                separator,
                found,
                span,
                opening_span,
            ),
            LazyDiagnostic::ExpectClosing { closing, found, span, opening_span } => {
                diagnostics::expect_closing(closing, found, span, opening_span)
            }
            LazyDiagnostic::ExpectConditionalAlternative { found, span, question_span } => {
                diagnostics::expect_conditional_alternative(found, span, question_span)
            }
            LazyDiagnostic::Eager(error) => error,
        }
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

        let error = diagnostics::unexpected_token(self.cur_token().span());
        self.set_fatal_error(error);
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
        Dummy::dummy(self.allocator())
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

    /// Record an already-built diagnostic as the fatal error (cold error sites).
    #[cold]
    pub(crate) fn set_fatal_error(&mut self, error: OxcDiagnostic) {
        self.set_fatal_error_lazy(LazyDiagnostic::Eager(error));
    }

    /// Record a fatal error from a cheap, un-built [`LazyDiagnostic`] (the hot
    /// speculative `expect_*` failures). Advances the lexer to end of file so the
    /// usual `has_fatal_error()`/EOF loop guards terminate parsing, exactly like
    /// the eager path — only the diagnostic build is deferred to report time.
    pub(crate) fn set_fatal_error_lazy(&mut self, error: LazyDiagnostic) {
        if self.fatal_error.is_none() {
            self.lexer.advance_to_end();
            self.fatal_error = Some(FatalError { error, errors_len: self.errors.len() });
        }
    }

    #[cold]
    pub(crate) fn fatal_error<T: Dummy<'a>>(&mut self, error: OxcDiagnostic) -> T {
        self.set_fatal_error(error);
        Dummy::dummy(self.allocator())
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
            return Some(Span::sized(span.start, 7));
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
