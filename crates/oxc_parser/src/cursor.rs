//! Token cursor: reading, consuming, expecting and re-lexing the current token,
//! and checkpoints for backtracking.

use oxc_ast::ast::RegExpFlags;
use oxc_span::Span;
use oxc_str::JSStr;

use crate::{
    ParserConfig as Config, ParserImpl, diagnostics,
    error_handler::FatalError,
    lexer::{Kind, LexerCheckpoint, Token, cold_branch},
};

pub struct ParserCheckpoint<'a> {
    lexer: LexerCheckpoint<'a>,
    cur_token: Token,
    prev_token_end: u32,
    errors_pos: usize,
    fatal_error: Option<FatalError<'a>>,
}

impl<'a, C: Config> ParserImpl<'a, C> {
    /// Get current token's span start.
    #[inline]
    pub(crate) fn cur_start(&self) -> u32 {
        self.token.start()
    }

    /// Create a [`Span`] from provided `start` to end of previous token.
    #[inline]
    pub(crate) fn end_span(&self, start: u32) -> Span {
        Span::new(start, self.prev_token_end)
    }

    /// Get current token
    #[inline]
    pub(crate) fn cur_token(&self) -> Token {
        self.token
    }

    /// Get current Kind
    #[inline]
    pub(crate) fn cur_kind(&self) -> Kind {
        self.token.kind()
    }

    /// Get current source text
    #[inline]
    pub(crate) fn cur_src(&self) -> &'a str {
        self.token_source(&self.token)
    }

    /// Get source text for a token
    #[inline]
    pub(crate) fn token_source(&self, token: &Token) -> &'a str {
        let span = token.span();
        if cfg!(debug_assertions) {
            &self.source_text[span.start as usize..span.end as usize]
        } else {
            // SAFETY:
            // Span comes from the lexer, which ensures:
            // * `start` and `end` are in bounds of source text.
            // * `end >= start`.
            // * `start` and `end` are both on UTF-8 char boundaries.
            // * `self.source_text` is same text that `Token`s are generated from.
            //
            // TODO: I (@overlookmotel) don't think we should really be doing this.
            // We don't have static guarantees of these properties.
            unsafe { self.source_text.get_unchecked(span.start as usize..span.end as usize) }
        }
    }

    /// Get current string
    pub(crate) fn cur_string(&self) -> &'a str {
        self.lexer.get_string(self.token)
    }

    /// Get the decoded value of the current string literal.
    pub(crate) fn cur_js_string(&self) -> JSStr<'a> {
        self.lexer.get_js_string(self.token)
    }

    /// Get current template string
    pub(crate) fn cur_template_string(&self) -> Option<JSStr<'a>> {
        self.lexer.get_template_string(self.token.start())
    }

    /// Checks if the current index has token `Kind`
    #[inline]
    pub(crate) fn at(&self, kind: Kind) -> bool {
        self.cur_kind() == kind
    }

    /// `StringValue` of `IdentifierName` normalizes any Unicode escape sequences
    /// in `IdentifierName` hence such escapes cannot be used to write an Identifier
    /// whose code point sequence is the same as a `ReservedWord`.
    #[cold]
    fn report_escaped_keyword(&mut self, span: Span) {
        self.error(diagnostics::escaped_keyword(span));
    }

    /// Move to the next token
    /// Checks if the current token is escaped if it is a keyword
    #[inline]
    pub(crate) fn advance(&mut self, kind: Kind) {
        // Manually inlined escaped keyword check - escaped identifiers are extremely rare
        if self.token.escaped() && kind.is_any_keyword() {
            self.report_escaped_keyword(self.token.span());
        }
        self.prev_token_end = self.token.end();
        self.token = self.lexer.next_token();
    }

    /// Move to the next `JSXChild`
    /// Checks if the current token is escaped if it is a keyword
    pub(crate) fn advance_for_jsx_child(&mut self) {
        self.prev_token_end = self.token.end();
        self.token = self.lexer.next_jsx_child();
    }

    /// Advance and return true if we are at `Kind`, return false otherwise
    #[inline]
    #[must_use = "Use `bump` instead of `eat` if you are ignoring the return value"]
    pub(crate) fn eat(&mut self, kind: Kind) -> bool {
        if self.at(kind) {
            self.advance(kind);
            return true;
        }
        false
    }

    /// Advance if we are at `Kind`
    #[inline]
    pub(crate) fn bump(&mut self, kind: Kind) {
        if self.at(kind) {
            self.advance(kind);
        }
    }

    /// Advance any token
    #[inline]
    pub(crate) fn bump_any(&mut self) {
        self.advance(self.cur_kind());
    }

    /// Advance if `accept` holds for the token after the current one.
    ///
    /// Use this instead of [`Lexer::peek_token`] where the answer is nearly always yes, such as
    /// a modifier keyword, which is nearly always followed by the member it modifies. The token
    /// after is then lexed once and kept. Peeking lexes it, discards it, and leaves it to be
    /// lexed again.
    ///
    /// [`Lexer::peek_token`]: crate::lexer::Lexer::peek_token
    #[inline]
    pub(crate) fn eat_if_next(&mut self, accept: impl FnOnce(Token) -> bool) -> bool {
        let checkpoint = self.lexer.checkpoint();
        let next = self.lexer.next_token();
        if !accept(next) {
            self.lexer.rewind(checkpoint);
            return false;
        }
        if self.token.escaped() && self.token.kind().is_any_keyword() {
            self.report_escaped_keyword(self.token.span());
        }
        self.prev_token_end = self.token.end();
        self.token = next;
        true
    }

    /// [Automatic Semicolon Insertion](https://tc39.es/ecma262/#sec-automatic-semicolon-insertion)
    /// # Errors
    pub(crate) fn asi(&mut self) {
        if self.eat(Kind::Semicolon) || self.can_insert_semicolon() {
            /* no op */
        } else {
            // ASI failure is a syntax error (cold). Build the diagnostic out of line so the
            // ~232-byte `OxcDiagnostic` buffer does not inflate `asi`'s stack frame on the
            // common (valid) path.
            cold_branch(|| {
                let span = Span::empty(self.prev_token_end);
                let error = diagnostics::auto_semicolon_insertion(span);
                self.set_fatal_error(error);
            });
        }
    }

    #[inline]
    pub(crate) fn can_insert_semicolon(&self) -> bool {
        let token = self.cur_token();
        matches!(token.kind(), Kind::Semicolon | Kind::RCurly | Kind::Eof) || token.is_on_new_line()
    }

    /// Cold path for expect failures - separated to improve branch prediction
    #[cold]
    #[inline(never)]
    fn handle_expect_failure(&mut self, expected_kind: Kind) {
        let range = self.cur_token().span();
        let error =
            diagnostics::expect_token(expected_kind.to_str(), self.cur_kind().to_str(), range);
        self.set_fatal_error(error);
    }

    /// # Errors
    #[inline]
    pub(crate) fn expect_without_advance(&mut self, kind: Kind) {
        if !self.at(kind) {
            self.handle_expect_failure(kind);
        }
    }

    /// Expect a `Kind` or return error
    /// # Errors
    #[inline]
    pub(crate) fn expect(&mut self, kind: Kind) {
        if !self.at(kind) {
            self.handle_expect_failure(kind);
        }
        self.advance(kind);
    }

    #[inline]
    pub(crate) fn expect_closing(&mut self, kind: Kind, opening_span: Span) {
        if !self.at(kind) {
            let range = self.cur_token().span();
            let error = diagnostics::expect_closing(
                kind.to_str(),
                self.cur_kind().to_str(),
                range,
                opening_span,
            );
            self.set_fatal_error(error);
        }
        self.advance(kind);
    }

    #[inline]
    pub(crate) fn expect_conditional_alternative(&mut self, question_span: Span) {
        if !self.at(Kind::Colon) {
            let range = self.cur_token().span();
            let error = diagnostics::expect_conditional_alternative(
                self.cur_kind().to_str(),
                range,
                question_span,
            );
            self.set_fatal_error(error);
        }
        self.bump_any(); // bump `:`
    }

    /// Expect the next next token to be a `JsxChild`, i.e. `<` or `{` or `JSXText`
    /// # Errors
    pub(crate) fn expect_jsx_child(&mut self, kind: Kind) {
        self.expect_without_advance(kind);
        self.advance_for_jsx_child();
    }

    /// Move to the next token, lexing it as a JSX attribute value.
    pub(crate) fn advance_for_jsx_attribute_value(&mut self) {
        self.prev_token_end = self.token.end();
        self.token = self.lexer.next_jsx_attribute_value();
    }

    /// Tell lexer to read a regex
    pub(crate) fn read_regex(&mut self) -> (u32, RegExpFlags) {
        let (token, pattern_end, flags) = self.lexer.next_regex(self.cur_kind());
        self.token = token;
        (pattern_end, flags)
    }

    /// Tell lexer to read a template substitution tail
    pub(crate) fn re_lex_template_substitution_tail(&mut self) {
        if self.fatal_error.is_some() {
            return;
        }
        if self.at(Kind::RCurly) {
            self.token = self.lexer.next_template_substitution_tail();
        }
    }

    /// Tell lexer to continue reading jsx identifier if the lexer character position is at `-` for `<component-name>`.
    ///
    /// Returns `true` if was continued.
    pub(crate) fn continue_lex_jsx_identifier(&mut self) -> bool {
        if let Some(token) = self.lexer.continue_lex_jsx_identifier(self.token.start()) {
            self.token = token;
            true
        } else {
            false
        }
    }

    #[inline]
    pub(crate) fn re_lex_right_angle(&mut self) -> Kind {
        if self.fatal_error.is_some() {
            return Kind::Eof;
        }
        let kind = self.cur_kind();
        if kind == Kind::RAngle {
            self.token = self.lexer.re_lex_right_angle();
            self.token.kind()
        } else {
            kind
        }
    }

    pub(crate) fn re_lex_ts_l_angle(&mut self) -> bool {
        if self.fatal_error.is_some() {
            return false;
        }
        let kind = self.cur_kind();
        if kind == Kind::ShiftLeft || kind == Kind::LtEq {
            self.token = self.lexer.re_lex_as_typescript_l_angle(2);
            true
        } else if kind == Kind::ShiftLeftEq {
            self.token = self.lexer.re_lex_as_typescript_l_angle(3);
            true
        } else {
            kind == Kind::LAngle
        }
    }

    pub(crate) fn re_lex_ts_r_angle(&mut self) -> bool {
        if self.fatal_error.is_some() {
            return false;
        }
        let kind = self.cur_kind();
        if kind == Kind::ShiftRight {
            self.token = self.lexer.re_lex_as_typescript_r_angle(2);
            true
        } else if kind == Kind::ShiftRight3 {
            self.token = self.lexer.re_lex_as_typescript_r_angle(3);
            true
        } else {
            kind == Kind::RAngle
        }
    }

    pub(crate) fn checkpoint(&mut self) -> ParserCheckpoint<'a> {
        ParserCheckpoint {
            lexer: self.lexer.checkpoint(),
            cur_token: self.token,
            prev_token_end: self.prev_token_end,
            errors_pos: self.errors.len(),
            fatal_error: self.fatal_error.take(),
        }
    }

    pub(crate) fn checkpoint_with_error_recovery(&mut self) -> ParserCheckpoint<'a> {
        ParserCheckpoint {
            lexer: self.lexer.checkpoint_with_error_recovery(),
            cur_token: self.token,
            prev_token_end: self.prev_token_end,
            errors_pos: self.errors.len(),
            fatal_error: self.fatal_error.take(),
        }
    }

    pub(crate) fn rewind(&mut self, checkpoint: ParserCheckpoint<'a>) {
        let ParserCheckpoint { lexer, cur_token, prev_token_end, errors_pos, fatal_error } =
            checkpoint;

        self.lexer.rewind(lexer);
        self.token = cur_token;
        self.prev_token_end = prev_token_end;
        self.errors.truncate(errors_pos);
        self.fatal_error = fatal_error;
    }

    pub(crate) fn lookahead<U>(&mut self, predicate: impl Fn(&mut ParserImpl<'a, C>) -> U) -> U {
        let checkpoint = self.checkpoint();
        let answer = predicate(self);
        self.rewind(checkpoint);
        answer
    }
}
