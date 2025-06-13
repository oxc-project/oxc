//! Code related to navigating `Token`s from the lexer

use oxc_allocator::Vec;
use oxc_ast::ast::{BindingRestElement, RegExpFlags};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};

use crate::{
    Context, ParserImpl, diagnostics,
    error_handler::FatalError,
    lexer::{Kind, LexerCheckpoint, LexerContext, Token},
};

#[derive(Clone)]
pub struct ParserCheckpoint<'a> {
    lexer: LexerCheckpoint<'a>,
    cur_token: Token,
    prev_span_end: u32,
    errors_pos: usize,
    fatal_error: Option<FatalError>,
}

impl<'a> ParserImpl<'a> {
    #[inline]
    pub(crate) fn start_span(&self) -> u32 {
        self.token.start()
    }

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
    pub(crate) fn cur_src(&self) -> &'a str {
        let range = self.cur_token().span();
        // SAFETY:
        // range comes from the lexer, which are ensured to meeting the criteria of `get_unchecked`.

        unsafe { self.source_text.get_unchecked(range.start as usize..range.end as usize) }
    }

    /// Get current string
    pub(crate) fn cur_string(&self) -> &'a str {
        self.lexer.get_string(self.token)
    }

    /// Get current template string
    pub(crate) fn cur_template_string(&self) -> Option<&'a str> {
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
    #[inline]
    fn test_escaped_keyword(&mut self, kind: Kind) {
        if self.cur_token().escaped() && kind.is_any_keyword() {
            let span = self.cur_token().span();
            self.error(diagnostics::escaped_keyword(span));
        }
    }

    /// Move to the next token
    /// Checks if the current token is escaped if it is a keyword
    #[inline]
    fn advance(&mut self, kind: Kind) {
        self.test_escaped_keyword(kind);
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

    /// Advance and change token type, useful for changing keyword to ident
    #[inline]
    pub(crate) fn bump_remap(&mut self, kind: Kind) {
        self.advance(kind);
    }

    /// [Automatic Semicolon Insertion](https://tc39.es/ecma262/#sec-automatic-semicolon-insertion)
    /// # Errors
    pub(crate) fn asi(&mut self) {
        if self.eat(Kind::Semicolon) || self.can_insert_semicolon() {
            /* no op */
        } else {
            let span = Span::new(self.prev_token_end, self.prev_token_end);
            let error = diagnostics::auto_semicolon_insertion(span);
            self.set_fatal_error(error);
        }
    }

    #[inline]
    pub(crate) fn can_insert_semicolon(&self) -> bool {
        let token = self.cur_token();
        let kind = token.kind();
        kind == Kind::Semicolon || kind == Kind::RCurly || kind.is_eof() || token.is_on_new_line()
    }

    /// # Errors
    pub(crate) fn expect_without_advance(&mut self, kind: Kind) {
        if !self.at(kind) {
            let range = self.cur_token().span();
            let error = diagnostics::expect_token(kind.to_str(), self.cur_kind().to_str(), range);
            self.set_fatal_error(error);
        }
    }

    /// Expect a `Kind` or return error
    /// # Errors
    #[inline]
    pub(crate) fn expect(&mut self, kind: Kind) {
        self.expect_without_advance(kind);
        self.advance(kind);
    }

    /// Expect the next next token to be a `JsxChild`, i.e. `<` or `{` or `JSXText`
    /// # Errors
    pub(crate) fn expect_jsx_child(&mut self, kind: Kind) {
        self.expect_without_advance(kind);
        self.advance_for_jsx_child();
    }

    /// Expect the next next token to be a `JsxString` or any other token
    /// # Errors
    pub(crate) fn expect_jsx_attribute_value(&mut self, kind: Kind) {
        self.lexer.set_context(LexerContext::JsxAttributeValue);
        self.expect(kind);
        self.lexer.set_context(LexerContext::Regular);
    }

    /// Tell lexer to read a regex
    pub(crate) fn read_regex(&mut self) -> (u32, RegExpFlags, bool) {
        let (token, pattern_end, flags, flags_error) = self.lexer.next_regex(self.cur_kind());
        self.token = token;
        (pattern_end, flags, flags_error)
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

    pub(crate) fn re_lex_l_angle(&mut self) -> Kind {
        if self.fatal_error.is_some() {
            return Kind::Eof;
        }
        let kind = self.cur_kind();
        if matches!(kind, Kind::ShiftLeft | Kind::ShiftLeftEq | Kind::LtEq) {
            self.token = self.lexer.re_lex_as_typescript_l_angle(kind);
            self.token.kind()
        } else {
            kind
        }
    }

    pub(crate) fn re_lex_ts_r_angle(&mut self) -> Kind {
        if self.fatal_error.is_some() {
            return Kind::Eof;
        }
        let kind = self.cur_kind();
        if matches!(kind, Kind::ShiftRight | Kind::ShiftRight3) {
            self.token = self.lexer.re_lex_as_typescript_r_angle(kind);
            self.token.kind()
        } else {
            kind
        }
    }

    pub(crate) fn checkpoint(&mut self) -> ParserCheckpoint<'a> {
        ParserCheckpoint {
            lexer: self.lexer.checkpoint(),
            cur_token: self.token,
            prev_span_end: self.prev_token_end,
            errors_pos: self.errors.len(),
            fatal_error: self.fatal_error.take(),
        }
    }

    pub(crate) fn rewind(&mut self, checkpoint: ParserCheckpoint<'a>) {
        let ParserCheckpoint { lexer, cur_token, prev_span_end, errors_pos, fatal_error } =
            checkpoint;

        self.lexer.rewind(lexer);
        self.token = cur_token;
        self.prev_token_end = prev_span_end;
        self.errors.truncate(errors_pos);
        self.fatal_error = fatal_error;
    }

    pub(crate) fn try_parse<T>(
        &mut self,
        func: impl FnOnce(&mut ParserImpl<'a>) -> T,
    ) -> Option<T> {
        let checkpoint = self.checkpoint();
        let ctx = self.ctx;
        let node = func(self);
        if self.fatal_error.is_none() {
            Some(node)
        } else {
            self.ctx = ctx;
            self.rewind(checkpoint);
            None
        }
    }

    pub(crate) fn lookahead<U>(&mut self, predicate: impl Fn(&mut ParserImpl<'a>) -> U) -> U {
        let checkpoint = self.checkpoint();
        let answer = predicate(self);
        self.rewind(checkpoint);
        answer
    }

    #[expect(clippy::inline_always)]
    #[inline(always)] // inline because this is always on a hot path
    pub(crate) fn context<F, T>(&mut self, add_flags: Context, remove_flags: Context, cb: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let ctx = self.ctx;
        self.ctx = ctx.difference(remove_flags).union(add_flags);
        let result = cb(self);
        self.ctx = ctx;
        result
    }

    pub(crate) fn parse_normal_list<F, T>(&mut self, open: Kind, close: Kind, f: F) -> Vec<'a, T>
    where
        F: Fn(&mut Self) -> Option<T>,
    {
        self.expect(open);
        let mut list = self.ast.vec();
        loop {
            let kind = self.cur_kind();
            if kind == close || self.has_fatal_error() {
                break;
            }
            match f(self) {
                Some(e) => {
                    list.push(e);
                }
                None => {
                    break;
                }
            }
        }
        self.expect(close);
        list
    }

    pub(crate) fn parse_delimited_list<F, T>(
        &mut self,
        close: Kind,
        separator: Kind,
        f: F,
    ) -> (Vec<'a, T>, Option<u32>)
    where
        F: Fn(&mut Self) -> T,
    {
        let mut list = self.ast.vec();
        let mut first = true;
        let mut trailing_separator = None;
        loop {
            if self.cur_kind() == close || self.has_fatal_error() {
                break;
            }
            if first {
                first = false;
            } else {
                let separator_span = self.start_span();
                self.expect(separator);
                if self.at(close) {
                    trailing_separator = Some(separator_span);
                    break;
                }
            }
            list.push(f(self));
        }
        (list, trailing_separator)
    }

    pub(crate) fn parse_delimited_list_with_rest<E, A, D>(
        &mut self,
        close: Kind,
        parse_element: E,
        rest_last_diagnostic: D,
    ) -> (Vec<'a, A>, Option<BindingRestElement<'a>>)
    where
        E: Fn(&mut Self) -> A,
        D: Fn(Span) -> OxcDiagnostic,
    {
        let mut list = self.ast.vec();
        let mut rest = None;
        let mut first = true;
        let mut rest_comma_span: Option<Span> = None;
        loop {
            let kind = self.cur_kind();
            if self.has_fatal_error() {
                break;
            }
            if kind == close {
                break;
            }

            if first {
                first = false;
            } else {
                self.expect(Kind::Comma);
                if self.at(close) {
                    break;
                }
            }

            if self.at(Kind::Dot3) {
                let r = self.parse_rest_element();
                if self.at(Kind::Comma) {
                    rest_comma_span.replace(self.cur_token().span());
                    if !self.ctx.has_ambient() {
                        self.error(rest_last_diagnostic(r.span()));
                    }
                } else {
                    rest_comma_span = None;
                }
                rest.replace(r);
            } else {
                rest_comma_span = None;
                list.push(parse_element(self));
            }
        }

        if !self.ctx.has_ambient() {
            if let Some(span) = rest_comma_span {
                self.error(diagnostics::rest_element_trailing_comma(span));
            }
        }

        (list, rest)
    }
}
