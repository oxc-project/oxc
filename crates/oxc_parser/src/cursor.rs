//! Code related to navigating `Token`s from the lexer

use oxc_ast::{context::Context, Atom, Node};
use oxc_diagnostics::{Diagnostic, Result};

use crate::lexer::{Kind, LexerCheckpoint, LexerContext, Token};
use crate::Parser;

pub struct ParserCheckpoint<'a> {
    lexer: LexerCheckpoint<'a>,
    cur_token: Token,
    prev_node_end: usize,
    errors_pos: usize,
}

impl<'a> Parser<'a> {
    #[must_use]
    pub const fn start_node(&self) -> Node {
        let token = self.cur_token();
        Node::new(token.start, 0, self.ctx)
    }

    #[must_use]
    pub const fn end_node(&self, node: Node) -> Node {
        let mut node = node;
        node.end = self.prev_token_end;
        node
    }

    /// Get current token
    #[must_use]
    pub const fn cur_token(&self) -> &Token {
        &self.token
    }

    /// Get current Kind
    #[must_use]
    pub const fn cur_kind(&self) -> Kind {
        self.token.kind
    }

    /// Get current source text
    #[must_use]
    pub fn cur_src(&self) -> &'a str {
        unsafe { self.source.get_unchecked(self.cur_token().range()) }
    }

    /// Get current atom
    #[must_use]
    pub const fn cur_atom(&self) -> Option<&Atom> {
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
    pub fn nth(&mut self, n: usize) -> &Token {
        if n == 0 {
            return self.cur_token();
        }
        self.lexer.lookahead(n)
    }

    /// Peek at nth kind
    pub fn nth_at(&mut self, n: usize, kind: Kind) -> bool {
        self.nth(n).kind == kind
    }

    /// Peek nth kind
    pub fn nth_kind(&mut self, n: usize) -> Kind {
        self.nth(n).kind
    }

    /// Checks if the current index has token `Kind`
    #[must_use]
    pub fn at(&self, kind: Kind) -> bool {
        self.cur_kind() == kind
    }

    /// Move to the next token
    /// Checks if the current token is escaped if it is a keyword
    fn advance(&mut self, kind: Kind) {
        // StringValue of IdentifierName normalizes any Unicode escape sequences
        // in IdentifierName hence such escapes cannot be used to write an Identifier
        // whose code point sequence is the same as a ReservedWord.
        if self.cur_token().escaped && kind.is_all_keyword() {
            let range = self.cur_token().range();
            self.error(Diagnostic::EscapedKeyword(range));
        }
        self.prev_token_end = self.token.end;
        self.token = self.lexer.next_token();
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
            let range = self.prev_token_end..self.cur_token().start;
            return Err(Diagnostic::AutoSemicolonInsertion(range));
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

    /// Expect a `Kind` or return error
    /// # Errors
    pub fn expect(&mut self, kind: Kind) -> Result<()> {
        if !self.at(kind) {
            let range = self.current_range();
            return Err(Diagnostic::ExpectToken(kind.to_str(), self.cur_kind().to_str(), range));
        }
        self.advance(kind);
        Ok(())
    }

    #[must_use]
    pub const fn current_range(&self) -> std::ops::Range<usize> {
        let cur_token = self.cur_token();
        match self.cur_kind() {
            Kind::Eof => {
                if self.prev_token_end < cur_token.end {
                    self.prev_token_end..self.prev_token_end
                } else {
                    self.prev_token_end - 1..self.prev_token_end
                }
            }
            _ => cur_token.range(),
        }
    }

    /// Expect the next next token to be a `JsxChild`, i.e. `<` or `{` or `JSXText`
    /// # Errors
    pub fn expect_jsx_child(&mut self, kind: Kind) -> Result<()> {
        self.lexer.context = LexerContext::JsxChild;
        self.expect(kind)?;
        self.lexer.context = LexerContext::Regular;
        Ok(())
    }

    /// Expect the next next token to be a `JsxString` or any other token
    /// # Errors
    pub fn expect_jsx_attribute_value(&mut self, kind: Kind) -> Result<()> {
        self.lexer.context = LexerContext::JsxAttributeValue;
        self.expect(kind)?;
        self.lexer.context = LexerContext::Regular;
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
            prev_node_end: self.prev_token_end,
            errors_pos: self.errors.borrow().len(),
        }
    }

    pub fn rewind(&mut self, checkpoint: ParserCheckpoint<'a>) {
        let ParserCheckpoint { lexer, cur_token, prev_node_end, errors_pos: errors_lens } =
            checkpoint;

        self.lexer.rewind(lexer);
        self.token = cur_token;
        self.prev_token_end = prev_node_end;
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
