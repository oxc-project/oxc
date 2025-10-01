//! Code related to error handling.

use oxc_allocator::Dummy;
use oxc_diagnostics::OxcDiagnostic;

use crate::{ParserImpl, diagnostics, lexer::Kind};

/// Fatal parsing error.
#[derive(Debug, Clone)]
pub struct FatalError {
    /// The fatal error
    pub error: OxcDiagnostic,
    /// Length of `errors` at time fatal error is recorded
    pub errors_len: usize,
}

impl<'a> ParserImpl<'a> {
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
        Dummy::dummy(self.ast.allocator)
    }

    /// Push a Syntax Error
    #[cold]
    pub(crate) fn error(&mut self, error: OxcDiagnostic) {
        self.errors.push(error);
    }

    /// Count of all parser and lexer errors.
    pub(crate) fn errors_count(&self) -> usize {
        self.errors.len() + self.lexer.errors.len()
    }

    /// Advance lexer's cursor to end of file.
    #[cold]
    pub(crate) fn set_fatal_error(&mut self, error: OxcDiagnostic) {
        if self.fatal_error.is_none() {
            self.lexer.advance_to_end();
            self.fatal_error = Some(FatalError { error, errors_len: self.errors.len() });
        }
    }

    #[cold]
    pub(crate) fn fatal_error<T: Dummy<'a>>(&mut self, error: OxcDiagnostic) -> T {
        self.set_fatal_error(error);
        Dummy::dummy(self.ast.allocator)
    }

    pub(crate) fn has_fatal_error(&self) -> bool {
        matches!(self.cur_kind(), Kind::Eof | Kind::Undetermined) || self.fatal_error.is_some()
    }
}
