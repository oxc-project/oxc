//! Code related to navigating `Token`s from the lexer

use oxc_ast::ast::RegExpFlags;
use oxc_diagnostics::Result;
use oxc_span::Span;

use crate::{
    diagnostics,
    lexer::{Kind, LexerCheckpoint, LexerContext, Token},
    Context, ParserImpl,
};

#[derive(Clone, Copy)]
pub struct ParserCheckpoint<'a> {
    lexer: LexerCheckpoint<'a>,
    cur_token: Token,
    prev_span_end: u32,
    errors_pos: usize,
}

impl<'a> ParserImpl<'a> {
    pub(crate) fn start_span(&self) -> Span {
        let token = self.cur_token();
        Span::new(token.start, 0)
    }

    #[inline]
    pub(crate) fn end_span(&self, mut span: Span) -> Span {
        span.end = self.prev_token_end;
        debug_assert!(span.end >= span.start);
        span
    }

    /// Get current token
    pub(crate) fn cur_token(&self) -> Token {
        self.token
    }

    /// Get current Kind
    pub(crate) fn cur_kind(&self) -> Kind {
        self.token.kind
    }

    /// Get current source text
    pub(crate) fn cur_src(&self) -> &'a str {
        let range = self.cur_token().span();
        // SAFETY:
        // range comes from the parser, which are ensured to meeting the criteria of `get_unchecked`.
        unsafe { self.source_text.get_unchecked(range.start as usize..range.end as usize) }
    }

    /// Get current string
    pub(crate) fn cur_string(&self) -> &'a str {
        self.lexer.get_string(self.token)
    }

    /// Get current template string
    pub(crate) fn cur_template_string(&self) -> Option<&'a str> {
        self.lexer.get_template_string(self.token)
    }

    /// Peek next token, returns EOF for final peek
    pub(crate) fn peek_token(&mut self) -> Token {
        self.lexer.lookahead(1)
    }

    /// Peek next kind, returns EOF for final peek
    pub(crate) fn peek_kind(&mut self) -> Kind {
        self.peek_token().kind
    }

    /// Peek at kind
    pub(crate) fn peek_at(&mut self, kind: Kind) -> bool {
        self.peek_token().kind == kind
    }

    /// Peek nth token
    pub(crate) fn nth(&mut self, n: u8) -> Token {
        if n == 0 {
            return self.cur_token();
        }
        self.lexer.lookahead(n)
    }

    /// Peek at nth kind
    pub(crate) fn nth_at(&mut self, n: u8, kind: Kind) -> bool {
        self.nth(n).kind == kind
    }

    /// Peek nth kind
    pub(crate) fn nth_kind(&mut self, n: u8) -> Kind {
        self.nth(n).kind
    }

    /// Checks if the current index has token `Kind`
    pub(crate) fn at(&self, kind: Kind) -> bool {
        self.cur_kind() == kind
    }

    /// `StringValue` of `IdentifierName` normalizes any Unicode escape sequences
    /// in `IdentifierName` hence such escapes cannot be used to write an Identifier
    /// whose code point sequence is the same as a `ReservedWord`.
    #[inline]
    fn test_escaped_keyword(&mut self, kind: Kind) {
        if self.cur_token().escaped() && kind.is_all_keyword() {
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

    /// Move to the next `JSXChild`
    /// Checks if the current token is escaped if it is a keyword
    fn advance_for_jsx_child(&mut self, kind: Kind) {
        self.test_escaped_keyword(kind);
        self.prev_token_end = self.token.end;
        self.token = self.lexer.next_jsx_child();
    }

    /// Advance and return true if we are at `Kind`, return false otherwise
    pub(crate) fn eat(&mut self, kind: Kind) -> bool {
        if self.at(kind) {
            self.advance(kind);
            return true;
        }
        false
    }

    /// Advance and return true if we are at `Kind`
    pub(crate) fn bump(&mut self, kind: Kind) {
        if self.at(kind) {
            self.advance(kind);
        }
    }

    /// Advance any token
    pub(crate) fn bump_any(&mut self) {
        self.advance(self.cur_kind());
    }

    /// Advance and change token type, useful for changing keyword to ident
    pub(crate) fn bump_remap(&mut self, kind: Kind) {
        self.advance(kind);
    }

    /// [Automatic Semicolon Insertion](https://tc39.es/ecma262/#sec-automatic-semicolon-insertion)
    /// # Errors
    pub(crate) fn asi(&mut self) -> Result<()> {
        if !self.can_insert_semicolon() {
            let span = Span::new(self.prev_token_end, self.cur_token().start);
            return Err(diagnostics::AutoSemicolonInsertion(span).into());
        }
        if self.at(Kind::Semicolon) {
            self.advance(Kind::Semicolon);
        }
        Ok(())
    }

    pub(crate) fn can_insert_semicolon(&self) -> bool {
        let kind = self.cur_kind();
        if kind == Kind::Semicolon {
            return true;
        }
        kind == Kind::RCurly || kind.is_eof() || self.cur_token().is_on_new_line
    }

    /// # Errors
    pub(crate) fn expect_without_advance(&mut self, kind: Kind) -> Result<()> {
        if !self.at(kind) {
            let range = self.cur_token().span();
            return Err(
                diagnostics::ExpectToken(kind.to_str(), self.cur_kind().to_str(), range).into()
            );
        }
        Ok(())
    }

    /// Expect a `Kind` or return error
    /// # Errors
    pub(crate) fn expect(&mut self, kind: Kind) -> Result<()> {
        self.expect_without_advance(kind)?;
        self.advance(kind);
        Ok(())
    }

    /// Expect the next next token to be a `JsxChild`, i.e. `<` or `{` or `JSXText`
    /// # Errors
    pub(crate) fn expect_jsx_child(&mut self, kind: Kind) -> Result<()> {
        self.expect_without_advance(kind)?;
        self.advance_for_jsx_child(kind);
        Ok(())
    }

    /// Expect the next next token to be a `JsxString` or any other token
    /// # Errors
    pub(crate) fn expect_jsx_attribute_value(&mut self, kind: Kind) -> Result<()> {
        self.lexer.set_context(LexerContext::JsxAttributeValue);
        self.expect(kind)?;
        self.lexer.set_context(LexerContext::Regular);
        Ok(())
    }

    /// Tell lexer to read a regex
    pub(crate) fn read_regex(&mut self) -> (u32, RegExpFlags) {
        let (token, pattern_end, flags) = self.lexer.next_regex(self.cur_kind());
        self.token = token;
        (pattern_end, flags)
    }

    /// Tell lexer to read a template substitution tail
    pub(crate) fn re_lex_template_substitution_tail(&mut self) {
        if self.at(Kind::RCurly) {
            self.token = self.lexer.next_template_substitution_tail();
        }
    }

    /// Tell lexer to continue reading jsx identifier if the lexer character position is at `-` for `<component-name>`
    pub(crate) fn continue_lex_jsx_identifier(&mut self) {
        if let Some(token) = self.lexer.continue_lex_jsx_identifier() {
            self.token = token;
        }
    }

    pub(crate) fn re_lex_right_angle(&mut self) -> Kind {
        let kind = self.cur_kind();
        if kind == Kind::RAngle {
            self.token = self.lexer.next_right_angle();
            self.token.kind
        } else {
            kind
        }
    }

    pub(crate) fn re_lex_ts_l_angle(&mut self) {
        let kind = self.cur_kind();
        if matches!(kind, Kind::ShiftLeft | Kind::ShiftLeftEq | Kind::LtEq) {
            self.token = self.lexer.re_lex_as_typescript_l_angle(kind);
        }
    }

    pub(crate) fn checkpoint(&self) -> ParserCheckpoint<'a> {
        ParserCheckpoint {
            lexer: self.lexer.checkpoint(),
            cur_token: self.token,
            prev_span_end: self.prev_token_end,
            errors_pos: self.errors.len(),
        }
    }

    pub(crate) fn rewind(&mut self, checkpoint: ParserCheckpoint<'a>) {
        let ParserCheckpoint { lexer, cur_token, prev_span_end, errors_pos: errors_lens } =
            checkpoint;

        self.lexer.rewind(lexer);
        self.token = cur_token;
        self.prev_token_end = prev_span_end;
        self.errors.truncate(errors_lens);
    }

    /// # Errors
    pub(crate) fn try_parse<T>(
        &mut self,
        func: impl FnOnce(&mut ParserImpl<'a>) -> Result<T>,
    ) -> Result<T> {
        let checkpoint = self.checkpoint();
        let ctx = self.ctx;
        let result = func(self);
        if result.is_err() {
            self.ctx = ctx;
            self.rewind(checkpoint);
        }
        result
    }

    pub(crate) fn lookahead<U>(&mut self, predicate: impl Fn(&mut ParserImpl<'a>) -> U) -> U {
        let checkpoint = self.checkpoint();
        let answer = predicate(self);
        self.rewind(checkpoint);
        answer
    }

    pub(crate) fn without_context<F, T>(&mut self, flags: Context, cb: F) -> T
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

    pub(crate) fn with_context<F, T>(&mut self, flags: Context, cb: F) -> T
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
