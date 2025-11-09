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
    #[cold]
    fn report_escaped_keyword(&mut self, span: Span) {
        self.error(diagnostics::escaped_keyword(span));
    }

    /// Move to the next token
    /// Checks if the current token is escaped if it is a keyword
    #[inline]
    fn advance(&mut self, kind: Kind) {
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
            let span = Span::empty(self.prev_token_end);
            let error = diagnostics::auto_semicolon_insertion(span);
            self.set_fatal_error(error);
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
            prev_span_end: self.prev_token_end,
            errors_pos: self.errors.len(),
            fatal_error: self.fatal_error.take(),
        }
    }

    pub(crate) fn checkpoint_with_error_recovery(&mut self) -> ParserCheckpoint<'a> {
        ParserCheckpoint {
            lexer: self.lexer.checkpoint_with_error_recovery(),
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
        let checkpoint = self.checkpoint_with_error_recovery();
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
    pub(crate) fn context_add<F, T>(&mut self, add_flags: Context, cb: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let ctx = self.ctx;
        self.ctx = ctx.union(add_flags);
        let result = cb(self);
        self.ctx = ctx;
        result
    }

    #[expect(clippy::inline_always)]
    #[inline(always)] // inline because this is always on a hot path
    pub(crate) fn context_remove<F, T>(&mut self, remove_flags: Context, cb: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let ctx = self.ctx;
        self.ctx = ctx.difference(remove_flags);
        let result = cb(self);
        self.ctx = ctx;
        result
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
        F: Fn(&mut Self) -> T,
    {
        let opening_span = self.cur_token().span();
        self.expect(open);
        let mut list = self.ast.vec();
        loop {
            let kind = self.cur_kind();
            if kind == close
                || matches!(kind, Kind::Eof | Kind::Undetermined)
                || self.fatal_error.is_some()
            {
                break;
            }
            list.push(f(self));
        }
        self.expect_closing(close, opening_span);
        list
    }

    pub(crate) fn parse_normal_list_breakable<F, T>(
        &mut self,
        open: Kind,
        close: Kind,
        f: F,
    ) -> Vec<'a, T>
    where
        F: Fn(&mut Self) -> Option<T>,
    {
        let opening_span = self.cur_token().span();
        self.expect(open);
        let mut list = self.ast.vec();
        loop {
            if self.at(close) || self.has_fatal_error() {
                break;
            }
            if let Some(e) = f(self) {
                list.push(e);
            } else {
                break;
            }
        }
        self.expect_closing(close, opening_span);
        list
    }

    pub(crate) fn parse_delimited_list<F, T>(
        &mut self,
        close: Kind,
        separator: Kind,
        opening_span: Span,
        f: F,
    ) -> (Vec<'a, T>, Option<u32>)
    where
        F: Fn(&mut Self) -> T,
    {
        let mut list = self.ast.vec();
        // Cache cur_kind() to avoid redundant calls in compound checks
        let kind = self.cur_kind();
        if kind == close
            || matches!(kind, Kind::Eof | Kind::Undetermined)
            || self.fatal_error.is_some()
        {
            return (list, None);
        }
        list.push(f(self));
        loop {
            let kind = self.cur_kind();
            if kind == close
                || matches!(kind, Kind::Eof | Kind::Undetermined)
                || self.fatal_error.is_some()
            {
                return (list, None);
            }
            if !self.at(separator) {
                self.set_fatal_error(diagnostics::expect_closing_or_separator(
                    close.to_str(),
                    separator.to_str(),
                    kind.to_str(),
                    self.cur_token().span(),
                    opening_span,
                ));
                return (list, None);
            }
            self.advance(separator);
            if self.cur_kind() == close {
                let trailing_separator = self.prev_token_end - 1;
                return (list, Some(trailing_separator));
            }
            list.push(f(self));
        }
    }

    pub(crate) fn parse_delimited_list_with_rest<E, A, D>(
        &mut self,
        close: Kind,
        opening_span: Span,
        parse_element: E,
        rest_last_diagnostic: D,
    ) -> (Vec<'a, A>, Option<BindingRestElement<'a>>)
    where
        E: Fn(&mut Self) -> A,
        D: Fn(Span) -> OxcDiagnostic,
    {
        let mut list = self.ast.vec();
        let mut rest: Option<BindingRestElement<'a>> = None;
        let mut first = true;
        loop {
            let kind = self.cur_kind();
            if kind == close
                || matches!(kind, Kind::Eof | Kind::Undetermined)
                || self.fatal_error.is_some()
            {
                break;
            }

            if first {
                first = false;
            } else {
                let comma_span = self.cur_token().span();
                if kind != Kind::Comma {
                    let error = diagnostics::expect_closing_or_separator(
                        close.to_str(),
                        Kind::Comma.to_str(),
                        kind.to_str(),
                        comma_span,
                        opening_span,
                    );
                    self.set_fatal_error(error);
                    break;
                }
                self.bump_any();
                let kind = self.cur_kind();
                if kind == close {
                    if rest.is_some() && !self.ctx.has_ambient() {
                        self.error(diagnostics::rest_element_trailing_comma(comma_span));
                    }
                    break;
                }
            }

            if let Some(r) = &rest {
                self.set_fatal_error(rest_last_diagnostic(r.span()));
                break;
            }

            // Re-capture kind to get the current token (may have changed after else branch)
            let kind = self.cur_kind();
            if kind == Kind::Dot3 {
                rest.replace(self.parse_rest_element());
            } else {
                list.push(parse_element(self));
            }
        }

        (list, rest)
    }
}
