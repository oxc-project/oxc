//! Code related to navigating `Token`s from the lexer

use oxc_ast::{context::Context, Atom, Span};
use oxc_diagnostics::Result;

use crate::lexer::{Kind, LexerCheckpoint, LexerContext, Token};
use crate::{diagnostics, Parser};

pub struct ParserCheckpoint<'a> {
    lexer: LexerCheckpoint<'a>,
    cur_token: Token,
    prev_span_end: u32,
    errors_pos: usize,
}

impl<'a> Parser<'a> {
    #[must_use]
    pub fn start_span(&self) -> Span {
        let token = self.cur_token();
        Span::new(token.start, 0)
    }

    #[must_use]
    pub fn end_span(&self, span: Span) -> Span {
        let mut span = span;
        span.end = self.prev_token_end;
        span
    }

    /// Get current token
    #[must_use]
    pub fn cur_token(&self) -> &Token {
        &self.token
    }

    /// Get current Kind
    #[must_use]
    pub fn cur_kind(&self) -> Kind {
        self.token.kind
    }

    /// Get current source text
    #[must_use]
    pub fn cur_src(&self) -> &'a str {
        let range = self.cur_token().span();
        unsafe { self.source.get_unchecked(range.start as usize..range.end as usize) }
    }

    /// Get current atom
    #[must_use]
    pub fn cur_atom(&self) -> Option<&Atom> {
        self.cur_token().value.get_atom()
    }

    /// Peek next token, returns EOF for final peek
    #[must_use]
    pub fn peek_token(&mut self) -> &Token {
        self.lexer.lookahead(1)
    }

    /// Peek next kind, returns EOF for final peek
    #[must_use]
    pub fn peek_kind(&mut self) -> Kind {
        self.peek_token().kind
    }

    /// Peek at kind
    #[must_use]
    pub fn peek_at(&mut self, kind: Kind) -> bool {
        self.peek_token().kind == kind
    }

    /// Peek nth token
    pub fn nth(&mut self, n: u8) -> &Token {
        if n == 0 {
            return self.cur_token();
        }
        self.lexer.lookahead(n)
    }

    /// Peek at nth kind
    pub fn nth_at(&mut self, n: u8, kind: Kind) -> bool {
        self.nth(n).kind == kind
    }

    /// Peek nth kind
    pub fn nth_kind(&mut self, n: u8) -> Kind {
        self.nth(n).kind
    }

    /// Checks if the current index has token `Kind`
    #[must_use]
    pub fn at(&self, kind: Kind) -> bool {
        self.cur_kind() == kind
    }

    /// StringValue of IdentifierName normalizes any Unicode escape sequences
    /// in IdentifierName hence such escapes cannot be used to write an Identifier
    /// whose code point sequence is the same as a ReservedWord.
    #[inline]
    fn test_escaped_keyword(&mut self, kind: Kind) {
        if self.cur_token().escaped && kind.is_all_keyword() {
            let span = self.cur_token().span();
            self.error(diagnostics::EscapedKeyword(span));
        }
    }

    /// Move to the next token
    /// Checks if the current token is escaped if it is a keyword
    fn advance(&mut self, kind: Kind) {
        self.test_escaped_keyword(kind);
        self.prev_token_end = self.token.end;
        self.token = self.lexer.next_token();
    }

    /// Move to the next JSXChild
    /// Checks if the current token is escaped if it is a keyword
    fn advance_for_jsx_child(&mut self, kind: Kind) {
        self.test_escaped_keyword(kind);
        self.prev_token_end = self.token.end;
        self.token = self.lexer.next_jsx_child();
    }

    /// Advance and return true if we are at `Kind`, return false otherwise
    #[must_use]
    pub fn eat(&mut self, kind: Kind) -> bool {
        if self.at(kind) {
            self.advance(kind);
            return true;
        }
        false
    }

    /// Advance and return true if we are at `Kind`
    pub fn bump(&mut self, kind: Kind) {
        if self.at(kind) {
            self.advance(kind);
        }
    }

    /// Advance any token
    pub fn bump_any(&mut self) {
        self.advance(self.cur_kind());
    }

    /// Advance and change token type, useful for changing keyword to ident
    pub fn bump_remap(&mut self, kind: Kind) {
        self.advance(kind);
    }

    /// Automatic Semicolon Insertion
    /// `https://tc39.es/ecma262/#sec-automatic-semicolon-insertion`
    /// # Errors
    pub fn asi(&mut self) -> Result<()> {
        if !self.can_insert_semicolon() {
            let span = Span::new(self.prev_token_end, self.cur_token().start);
            return Err(diagnostics::AutoSemicolonInsertion(span).into());
        }
        if self.at(Kind::Semicolon) {
            self.advance(Kind::Semicolon);
        }
        Ok(())
    }

    #[must_use]
    pub fn can_insert_semicolon(&self) -> bool {
        let kind = self.cur_kind();
        if kind == Kind::Semicolon {
            return true;
        }
        kind == Kind::RCurly || kind == Kind::Eof || self.cur_token().is_on_new_line
    }

    pub fn expect_without_advance(&mut self, kind: Kind) -> Result<()> {
        if !self.at(kind) {
            let range = self.current_range();
            return Err(
                diagnostics::ExpectToken(kind.to_str(), self.cur_kind().to_str(), range).into()
            );
        }
        Ok(())
    }

    /// Expect a `Kind` or return error
    /// # Errors
    pub fn expect(&mut self, kind: Kind) -> Result<()> {
        self.expect_without_advance(kind)?;
        self.advance(kind);
        Ok(())
    }

    #[must_use]
    pub fn current_range(&self) -> Span {
        let cur_token = self.cur_token();
        match self.cur_kind() {
            Kind::Eof => {
                if self.prev_token_end < cur_token.end {
                    Span::new(self.prev_token_end, self.prev_token_end)
                } else {
                    Span::new(self.prev_token_end - 1, self.prev_token_end)
                }
            }
            _ => cur_token.span(),
        }
    }

    /// Expect the next next token to be a `JsxChild`, i.e. `<` or `{` or `JSXText`
    /// # Errors
    pub fn expect_jsx_child(&mut self, kind: Kind) -> Result<()> {
        self.expect_without_advance(kind)?;
        self.advance_for_jsx_child(kind);
        Ok(())
    }

    /// Expect the next next token to be a `JsxString` or any other token
    /// # Errors
    pub fn expect_jsx_attribute_value(&mut self, kind: Kind) -> Result<()> {
        self.lexer.set_context(LexerContext::JsxAttributeValue);
        self.expect(kind)?;
        self.lexer.set_context(LexerContext::Regular);
        Ok(())
    }

    /// Tell lexer to read a regex
    pub fn read_regex(&mut self) {
        self.token = self.lexer.next_regex(self.cur_kind());
    }

    /// Tell lexer to read a template substitution tail
    pub fn re_lex_template_substitution_tail(&mut self) {
        if self.at(Kind::RCurly) {
            self.token = self.lexer.next_template_substitution_tail();
        }
    }

    /// Tell lexer to re-read a jsx identifier
    pub fn re_lex_jsx_identifier(&mut self) {
        self.token = self.lexer.next_jsx_identifier(self.prev_token_end);
    }

    pub fn re_lex_right_angle(&mut self) -> Kind {
        let kind = self.cur_kind();
        if kind == Kind::RAngle {
            self.token = self.lexer.next_right_angle();
            self.token.kind
        } else {
            kind
        }
    }

    pub fn re_lex_ts_l_angle(&mut self) {
        let kind = self.cur_kind();
        if matches!(kind, Kind::ShiftLeft | Kind::ShiftLeftEq | Kind::LtEq) {
            self.token = self.lexer.re_lex_as_typescript_l_angle(kind);
        }
    }

    pub fn re_lex_ts_r_angle(&mut self) {
        let kind = self.cur_kind();
        if matches!(
            kind,
            Kind::ShiftRight
                | Kind::ShiftRight3
                | Kind::ShiftRightEq
                | Kind::ShiftRight3Eq
                | Kind::GtEq
        ) {
            self.token = self.lexer.re_lex_as_typescript_r_angle(kind);
        }
    }

    #[must_use]
    pub fn checkpoint(&self) -> ParserCheckpoint<'a> {
        ParserCheckpoint {
            lexer: self.lexer.checkpoint(),
            cur_token: self.token.clone(),
            prev_span_end: self.prev_token_end,
            errors_pos: self.errors.borrow().len(),
        }
    }

    pub fn rewind(&mut self, checkpoint: ParserCheckpoint<'a>) {
        let ParserCheckpoint { lexer, cur_token, prev_span_end, errors_pos: errors_lens } =
            checkpoint;

        self.lexer.rewind(lexer);
        self.token = cur_token;
        self.prev_token_end = prev_span_end;
        self.errors.borrow_mut().truncate(errors_lens);
    }

    /// # Errors
    pub fn try_parse<T>(&mut self, func: impl FnOnce(&mut Parser<'a>) -> Result<T>) -> Result<T> {
        let checkpoint = self.checkpoint();
        let ctx = self.ctx;
        let result = func(self);
        if result.is_err() {
            self.ctx = ctx;
            self.rewind(checkpoint);
        }
        result
    }

    pub fn lookahead<U>(&mut self, predicate: impl Fn(&mut Parser<'a>) -> U) -> U {
        let checkpoint = self.checkpoint();
        let answer = predicate(self);
        self.rewind(checkpoint);
        answer
    }

    pub fn without_context<F, T>(&mut self, flags: Context, cb: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let context_flags_to_clear = flags & self.ctx;
        if !context_flags_to_clear.is_empty() {
            self.ctx &= !context_flags_to_clear;
            let result = cb(self);
            self.ctx |= context_flags_to_clear;
            return result;
        }
        cb(self)
    }

    pub fn with_context<F, T>(&mut self, flags: Context, cb: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let context_flags_to_set = flags & !self.ctx;
        if !context_flags_to_set.is_empty() {
            self.ctx |= context_flags_to_set;
            let result = cb(self);
            self.ctx &= !context_flags_to_set;
            return result;
        }
        cb(self)
    }
}
