//! An Ecma-262 Lexer / Tokenizer
//! Prior Arts:
//!     * [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/crates/parser/src)
//!     * [rome](https://github.com/rome/tools/tree/main/crates/rome_js_parser/src/lexer)
//!     * [rustc](https://github.com/rust-lang/rust/blob/master/compiler/rustc_lexer/src)
//!     * [v8](https://v8.dev/blog/scanner)

mod kind;
mod number;
mod string_builder;
mod token;
mod trivia_builder;

use std::{collections::VecDeque, str::Chars};

use oxc_allocator::{Allocator, String};
use oxc_ast::ast::RegExpFlags;
use oxc_diagnostics::Error;
use oxc_span::{SourceType, Span};
use oxc_syntax::{
    identifier::{
        is_identifier_part, is_identifier_start_all, is_irregular_line_terminator,
        is_irregular_whitespace, is_line_terminator, CR, FF, LF, LS, PS, TAB, VT,
    },
    unicode_id_start::is_id_start_unicode,
};
pub use token::{RegExp, Token, TokenValue};

pub use self::kind::Kind;
use self::{
    number::{parse_big_int, parse_float, parse_int},
    string_builder::AutoCow,
    trivia_builder::TriviaBuilder,
};
use crate::diagnostics;

#[derive(Debug, Clone)]
pub struct LexerCheckpoint<'a> {
    /// Remaining chars to be tokenized
    chars: Chars<'a>,

    token: Token<'a>,

    errors_pos: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LexerContext {
    Regular,
    /// Lex the next token, returns `JsxString` or any other token
    JsxAttributeValue,
}

pub struct Lexer<'a> {
    allocator: &'a Allocator,

    source: &'a str,

    source_type: SourceType,

    current: LexerCheckpoint<'a>,

    pub(crate) errors: Vec<Error>,

    lookahead: VecDeque<LexerCheckpoint<'a>>,

    context: LexerContext,

    pub(crate) trivia_builder: TriviaBuilder,
}

#[allow(clippy::unused_self)]
impl<'a> Lexer<'a> {
    pub fn new(allocator: &'a Allocator, source: &'a str, source_type: SourceType) -> Self {
        let token = Token {
            // the first token is at the start of file, so is allows on a new line
            is_on_new_line: true,
            ..Token::default()
        };
        let current = LexerCheckpoint { chars: source.chars(), token, errors_pos: 0 };
        Self {
            allocator,
            source,
            source_type,
            current,
            errors: vec![],
            lookahead: VecDeque::with_capacity(4), // 4 is the maximum lookahead for TypeScript
            context: LexerContext::Regular,
            trivia_builder: TriviaBuilder::default(),
        }
    }

    /// Remaining string from `Chars`
    pub fn remaining(&self) -> &'a str {
        self.current.chars.as_str()
    }

    /// Creates a checkpoint storing the current lexer state.
    /// Use `rewind` to restore the lexer to the state stored in the checkpoint.
    pub fn checkpoint(&self) -> LexerCheckpoint<'a> {
        LexerCheckpoint {
            chars: self.current.chars.clone(),
            token: self.current.token.clone(),
            errors_pos: self.errors.len(),
        }
    }

    /// Rewinds the lexer to the same state as when the passed in `checkpoint` was created.
    pub fn rewind(&mut self, checkpoint: LexerCheckpoint<'a>) {
        self.errors.truncate(checkpoint.errors_pos);
        self.current = checkpoint;
        self.lookahead.clear();
    }

    /// Find the nth lookahead token lazily
    pub fn lookahead(&mut self, n: u8) -> &Token<'a> {
        let n = n as usize;
        debug_assert!(n > 0);

        if self.lookahead.len() > n - 1 {
            return &self.lookahead[n - 1].token;
        }

        let checkpoint = self.checkpoint();

        if let Some(checkpoint) = self.lookahead.back() {
            self.current = checkpoint.clone();
        }

        // reset the current token for `read_next_token`,
        // otherwise it will contain the token from
        // `self.current = checkpoint`
        self.current.token = Token::default();

        for _i in self.lookahead.len()..n {
            let kind = self.read_next_token();
            let peeked = self.finish_next(kind);
            self.lookahead.push_back(LexerCheckpoint {
                chars: self.current.chars.clone(),
                token: peeked,
                errors_pos: self.errors.len(),
            });
        }

        self.current = checkpoint;

        &self.lookahead[n - 1].token
    }

    /// Set context
    pub fn set_context(&mut self, context: LexerContext) {
        self.context = context;
    }

    /// Main entry point
    pub fn next_token(&mut self) -> Token<'a> {
        if let Some(checkpoint) = self.lookahead.pop_front() {
            self.current.chars = checkpoint.chars;
            self.current.errors_pos = checkpoint.errors_pos;
            return checkpoint.token;
        }
        let kind = self.read_next_token();
        self.finish_next(kind)
    }

    pub fn next_jsx_child(&mut self) -> Token<'a> {
        self.current.token.start = self.offset();
        let kind = self.read_jsx_child();
        self.finish_next(kind)
    }

    fn finish_next(&mut self, kind: Kind) -> Token<'a> {
        self.current.token.kind = kind;
        self.current.token.end = self.offset();
        debug_assert!(self.current.token.start <= self.current.token.end);
        std::mem::take(&mut self.current.token)
    }

    /// Re-tokenize the current `/` or `/=` and return `RegExp`
    /// See Section 12:
    ///   The `InputElementRegExp` goal symbol is used in all syntactic grammar contexts
    ///   where a `RegularExpressionLiteral` is permitted
    /// Which meams the parser needs to re-tokenize on `PrimaryExpression`,
    /// `RegularExpressionLiteral` only appear on the right hand side of `PrimaryExpression`
    pub fn next_regex(&mut self, kind: Kind) -> Token<'a> {
        self.current.token.start = self.offset()
            - match kind {
                Kind::Slash => 1,
                Kind::SlashEq => 2,
                _ => unreachable!(),
            };
        let kind = self.read_regex();
        self.lookahead.clear();
        self.finish_next(kind)
    }

    pub fn next_right_angle(&mut self) -> Token<'a> {
        let kind = self.read_right_angle();
        self.lookahead.clear();
        self.finish_next(kind)
    }

    /// Re-tokenize the current `}` token for `TemplateSubstitutionTail`
    /// See Section 12, the parser needs to re-tokenize on `TemplateSubstitutionTail`,
    pub fn next_template_substitution_tail(&mut self) -> Token<'a> {
        self.current.token.start = self.offset() - 1;
        let kind = self.read_template_literal(Kind::TemplateMiddle, Kind::TemplateTail);
        self.lookahead.clear();
        self.finish_next(kind)
    }

    /// Expand the current token for `JSXIdentifier`
    pub fn next_jsx_identifier(&mut self, start_offset: u32) -> Token<'a> {
        let kind = self.read_jsx_identifier(start_offset);
        self.lookahead.clear();
        self.finish_next(kind)
    }

    /// Re-tokenize '<<' or '<=' or '<<=' to '<'
    pub fn re_lex_as_typescript_l_angle(&mut self, kind: Kind) -> Token<'a> {
        let offset = match kind {
            Kind::ShiftLeft | Kind::LtEq => 2,
            Kind::ShiftLeftEq => 3,
            _ => unreachable!(),
        };
        self.current.token.start = self.offset() - offset;
        self.current.chars = self.source[self.current.token.start as usize + 1..].chars();
        let kind = Kind::LAngle;
        self.lookahead.clear();
        self.finish_next(kind)
    }

    // ---------- Private Methods ---------- //
    fn error<T: Into<Error>>(&mut self, error: T) {
        self.errors.push(error.into());
    }

    /// Get the length offset from the source, in UTF-8 bytes
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn offset(&self) -> u32 {
        (self.source.len() - self.current.chars.as_str().len()) as u32
    }

    /// Get the current unterminated token range
    fn unterminated_range(&self) -> Span {
        Span::new(self.current.token.start, self.offset())
    }

    /// Consume the current char
    #[inline]
    fn consume_char(&mut self) -> char {
        self.current.chars.next().unwrap()
    }

    /// Peek the next char without advancing the position
    #[inline]
    fn peek(&self) -> Option<char> {
        self.current.chars.clone().next()
    }

    /// Peek the next next char without advancing the position
    #[inline]
    fn peek2(&self) -> Option<char> {
        let mut chars = self.current.chars.clone();
        chars.next();
        chars.next()
    }

    /// Peek the next character, and advance the current position if it matches
    #[inline]
    fn next_eq(&mut self, c: char) -> bool {
        let matched = self.peek() == Some(c);
        if matched {
            self.current.chars.next();
        }
        matched
    }

    fn current_offset(&self) -> Span {
        let offset = self.offset();
        Span::new(offset, offset)
    }

    /// Return `IllegalCharacter` Error or `UnexpectedEnd` if EOF
    fn unexpected_err(&mut self) {
        let offset = self.current_offset();
        match self.peek() {
            Some(c) => self.error(diagnostics::InvalidCharacter(c, offset)),
            None => self.error(diagnostics::UnexpectedEnd(offset)),
        }
    }

    /// Add string to `SourceAtomSet` and get `TokenValue::Atom`
    fn string_to_token_value(&mut self, s: &'a str) -> TokenValue<'a> {
        TokenValue::String(s)
    }

    fn set_numeric_value(&mut self, kind: Kind, src: &'a str) {
        let value = match kind {
            Kind::Decimal | Kind::Binary | Kind::Octal | Kind::Hex => {
                src.strip_suffix('n').map_or_else(
                    || parse_int(src, kind).map(TokenValue::Number),
                    |src| parse_big_int(src, kind).map(TokenValue::BigInt),
                )
            }
            Kind::Float | Kind::PositiveExponential | Kind::NegativeExponential => {
                parse_float(src).map(TokenValue::Number)
            }
            Kind::Undetermined => Ok(TokenValue::Number(std::f64::NAN)),
            _ => unreachable!("{kind}"),
        };

        match value {
            Ok(value) => self.current.token.value = value,
            Err(err) => {
                self.error(diagnostics::InvalidNumber(
                    err,
                    Span::new(self.current.token.start, self.offset()),
                ));
                self.current.token.value = TokenValue::Number(std::f64::NAN);
            }
        };
    }

    /// Read each char and set the current token
    /// Whitespace and line terminators are skipped
    fn read_next_token(&mut self) -> Kind {
        self.current.token.start = self.offset();

        loop {
            let offset = self.offset();
            self.current.token.start = offset;

            if let Some(c) = self.current.chars.clone().next() {
                let kind = self.match_char(c);
                if !matches!(
                    kind,
                    Kind::WhiteSpace | Kind::NewLine | Kind::Comment | Kind::MultiLineComment
                ) {
                    return kind;
                }
            } else {
                return Kind::Eof;
            }
        }
    }

    #[inline]
    fn match_char(&mut self, c: char) -> Kind {
        let size = c as usize;

        if size < 128 {
            return BYTE_HANDLERS[size](self);
        }

        match c {
            c if is_id_start_unicode(c) => {
                let mut builder = AutoCow::new(self);
                let c = self.consume_char();
                builder.push_matching(c);
                self.identifier_name(builder);
                Kind::Ident
            }
            c if is_irregular_whitespace(c) => {
                self.consume_char();
                Kind::WhiteSpace
            }
            c if is_irregular_line_terminator(c) => {
                self.consume_char();
                self.current.token.is_on_new_line = true;
                Kind::NewLine
            }
            _ => {
                self.consume_char();
                self.error(diagnostics::InvalidCharacter(c, self.unterminated_range()));
                Kind::Undetermined
            }
        }
    }

    /// Section 12.4 Single Line Comment
    fn skip_single_line_comment(&mut self) -> Kind {
        while let Some(c) = self.current.chars.next().as_ref() {
            if is_line_terminator(*c) {
                break;
            }
        }
        self.current.token.is_on_new_line = true;
        self.trivia_builder.add_single_line_comment(self.current.token.start, self.offset());
        Kind::Comment
    }

    /// Section 12.4 Multi Line Comment
    fn skip_multi_line_comment(&mut self) -> Kind {
        while let Some(c) = self.current.chars.next() {
            if c == '*' && self.next_eq('/') {
                self.trivia_builder.add_multi_line_comment(self.current.token.start, self.offset());
                return Kind::MultiLineComment;
            }
            if is_line_terminator(c) {
                self.current.token.is_on_new_line = true;
            }
        }
        self.error(diagnostics::UnterminatedMultiLineComment(self.unterminated_range()));
        Kind::Eof
    }

    /// Section 12.5 Hashbang Comments
    fn read_hashbang_comment(&mut self) -> Kind {
        while let Some(c) = self.current.chars.next().as_ref() {
            if is_line_terminator(*c) {
                break;
            }
        }
        self.current.token.is_on_new_line = true;
        Kind::HashbangComment
    }

    /// Section 12.6.1 Identifier Names
    fn identifier_tail(&mut self, mut builder: AutoCow<'a>) -> (bool, &'a str) {
        // ident tail
        while let Some(c) = self.peek() {
            if !is_identifier_part(c) {
                if c == '\\' {
                    self.current.chars.next();
                    builder.force_allocation_without_current_ascii_char(self);
                    self.identifier_unicode_escape_sequence(&mut builder, false);
                    continue;
                }
                break;
            }
            self.current.chars.next();
            builder.push_matching(c);
        }
        let has_escape = builder.has_escape();
        (has_escape, builder.finish(self))
    }

    fn identifier_name(&mut self, builder: AutoCow<'a>) -> &'a str {
        let (has_escape, text) = self.identifier_tail(builder);
        self.current.token.escaped = has_escape;
        self.current.token.value = TokenValue::String(text);
        text
    }

    fn identifier_name_handler(&mut self) -> &'a str {
        let builder = AutoCow::new(self);
        self.consume_char();
        self.identifier_name(builder)
    }

    /// Section 12.7 Punctuators
    fn read_dot(&mut self, builder: &mut AutoCow<'a>) -> Kind {
        if self.peek() == Some('.') && self.peek2() == Some('.') {
            self.current.chars.next();
            self.current.chars.next();
            return Kind::Dot3;
        }
        if self.peek().is_some_and(|c| c.is_ascii_digit()) {
            builder.push_matching('.');
            self.decimal_literal_after_decimal_point(builder)
        } else {
            Kind::Dot
        }
    }

    /// returns None for `SingleLineHTMLOpenComment` `<!--` in script mode
    fn read_left_angle(&mut self) -> Option<Kind> {
        if self.next_eq('<') {
            if self.next_eq('=') {
                Some(Kind::ShiftLeftEq)
            } else {
                Some(Kind::ShiftLeft)
            }
        } else if self.next_eq('=') {
            Some(Kind::LtEq)
        } else if self.peek() == Some('!')
            // SingleLineHTMLOpenComment `<!--` in script mode
            && self.source_type.is_script()
            && self.remaining().starts_with("!--")
        {
            None
        } else {
            Some(Kind::LAngle)
        }
    }

    fn read_right_angle(&mut self) -> Kind {
        if self.next_eq('>') {
            if self.next_eq('>') {
                if self.next_eq('=') {
                    Kind::ShiftRight3Eq
                } else {
                    Kind::ShiftRight3
                }
            } else if self.next_eq('=') {
                Kind::ShiftRightEq
            } else {
                Kind::ShiftRight
            }
        } else if self.next_eq('=') {
            Kind::GtEq
        } else {
            Kind::RAngle
        }
    }

    /// returns None for `SingleLineHTMLCloseComment` `-->` in script mode
    fn read_minus(&mut self) -> Option<Kind> {
        if self.next_eq('-') {
            // SingleLineHTMLCloseComment `-->` in script mode
            if self.current.token.is_on_new_line
                && self.source_type.is_script()
                && self.next_eq('>')
            {
                None
            } else {
                Some(Kind::Minus2)
            }
        } else if self.next_eq('=') {
            Some(Kind::MinusEq)
        } else {
            Some(Kind::Minus)
        }
    }

    fn private_identifier(&mut self, mut builder: AutoCow<'a>) -> Kind {
        let start = self.offset();
        match self.current.chars.next() {
            Some(c) if is_identifier_start_all(c) => {
                builder.push_matching(c);
            }
            Some('\\') => {
                builder.force_allocation_without_current_ascii_char(self);
                self.identifier_unicode_escape_sequence(&mut builder, true);
            }
            Some(c) => {
                #[allow(clippy::cast_possible_truncation)]
                self.error(diagnostics::InvalidCharacter(
                    c,
                    Span::new(start, start + c.len_utf8() as u32),
                ));
                return Kind::Undetermined;
            }
            None => {
                self.error(diagnostics::UnexpectedEnd(Span::new(start, start)));
                return Kind::Undetermined;
            }
        }
        let (_, name) = self.identifier_tail(builder);
        self.current.token.value = self.string_to_token_value(name);
        Kind::PrivateIdentifier
    }

    /// 12.8.3 Numeric Literals with `0` prefix
    fn read_zero(&mut self, builder: &mut AutoCow<'a>) -> Kind {
        match self.peek() {
            Some('b' | 'B') => self.read_non_decimal(Kind::Binary, builder),
            Some('o' | 'O') => self.read_non_decimal(Kind::Octal, builder),
            Some('x' | 'X') => self.read_non_decimal(Kind::Hex, builder),
            Some(c @ ('e' | 'E')) => {
                self.current.chars.next();
                builder.push_matching(c);
                self.read_decimal_exponent(builder)
            }
            Some('.') => {
                self.current.chars.next();
                builder.push_matching('.');
                self.decimal_literal_after_decimal_point_after_digits(builder)
            }
            Some('n') => {
                self.current.chars.next();
                builder.push_matching('n');
                self.check_after_numeric_literal(Kind::Decimal)
            }
            Some(n) if n.is_ascii_digit() => self.read_legacy_octal(builder),
            _ => self.check_after_numeric_literal(Kind::Decimal),
        }
    }

    fn read_non_decimal(&mut self, kind: Kind, builder: &mut AutoCow<'a>) -> Kind {
        let c = self.current.chars.next().unwrap();
        builder.push_matching(c);

        if self.peek().is_some_and(|c| kind.matches_number_char(c)) {
            let c = self.current.chars.next().unwrap();
            builder.push_matching(c);
        } else {
            self.unexpected_err();
            return Kind::Undetermined;
        }

        while let Some(c) = self.peek() {
            match c {
                '_' => {
                    self.current.chars.next();
                    builder.force_allocation_without_current_ascii_char(self);
                    if self.peek().is_some_and(|c| kind.matches_number_char(c)) {
                        let c = self.current.chars.next().unwrap();
                        builder.push_matching(c);
                    } else {
                        self.unexpected_err();
                        return Kind::Undetermined;
                    }
                }
                c if kind.matches_number_char(c) => {
                    self.current.chars.next();
                    builder.push_matching(c);
                }
                _ => break,
            }
        }
        if self.peek() == Some('n') {
            self.current.chars.next();
            builder.push_matching('n');
        }
        self.check_after_numeric_literal(kind)
    }

    fn read_legacy_octal(&mut self, builder: &mut AutoCow<'a>) -> Kind {
        let mut kind = Kind::Octal;
        loop {
            match self.peek() {
                Some('0'..='7') => {
                    self.current.chars.next();
                }
                Some('8'..='9') => {
                    self.current.chars.next();
                    kind = Kind::Decimal;
                }
                _ => break,
            }
        }

        match self.peek() {
            // allow 08.5 and 09.5
            Some('.') if kind == Kind::Decimal => {
                self.current.chars.next();
                builder.push_matching('.');
                self.decimal_literal_after_decimal_point_after_digits(builder)
            }
            // allow 08e1 and 09e1
            Some('e') if kind == Kind::Decimal => {
                self.current.chars.next();
                builder.push_matching('e');
                self.read_decimal_exponent(builder)
            }
            _ => self.check_after_numeric_literal(kind),
        }
    }

    fn decimal_literal_after_first_digit(&mut self, builder: &mut AutoCow<'a>) -> Kind {
        self.read_decimal_digits_after_first_digit(builder);
        if self.next_eq('.') {
            builder.push_matching('.');
            return self.decimal_literal_after_decimal_point_after_digits(builder);
        } else if self.next_eq('n') {
            builder.push_matching('n');
            return self.check_after_numeric_literal(Kind::Decimal);
        }

        let kind = self.optional_exponent(builder).map_or(Kind::Decimal, |kind| kind);
        self.check_after_numeric_literal(kind)
    }

    fn read_decimal_exponent(&mut self, builder: &mut AutoCow<'a>) -> Kind {
        let kind = match self.peek() {
            Some('-') => {
                self.current.chars.next();
                builder.push_matching('-');
                Kind::NegativeExponential
            }
            Some('+') => {
                self.current.chars.next();
                builder.push_matching('+');
                Kind::PositiveExponential
            }
            _ => Kind::PositiveExponential,
        };
        self.read_decimal_digits(builder);
        kind
    }

    fn read_decimal_digits(&mut self, builder: &mut AutoCow<'a>) {
        if self.peek().is_some_and(|c| c.is_ascii_digit()) {
            let c = self.current.chars.next().unwrap();
            builder.push_matching(c);
        } else {
            self.unexpected_err();
            return;
        }

        self.read_decimal_digits_after_first_digit(builder);
    }

    fn read_decimal_digits_after_first_digit(&mut self, builder: &mut AutoCow<'a>) {
        while let Some(c) = self.peek() {
            match c {
                '_' => {
                    self.current.chars.next();
                    builder.force_allocation_without_current_ascii_char(self);
                    if self.peek().is_some_and(|c| c.is_ascii_digit()) {
                        let c = self.current.chars.next().unwrap();
                        builder.push_matching(c);
                    } else {
                        self.unexpected_err();
                        return;
                    }
                }
                c @ '0'..='9' => {
                    self.current.chars.next();
                    builder.push_matching(c);
                }
                _ => break,
            }
        }
    }

    fn decimal_literal_after_decimal_point(&mut self, builder: &mut AutoCow<'a>) -> Kind {
        self.read_decimal_digits(builder);
        self.optional_exponent(builder);
        self.check_after_numeric_literal(Kind::Float)
    }

    fn decimal_literal_after_decimal_point_after_digits(
        &mut self,
        builder: &mut AutoCow<'a>,
    ) -> Kind {
        self.optional_decimal_digits(builder);
        self.optional_exponent(builder);
        self.check_after_numeric_literal(Kind::Float)
    }

    fn optional_decimal_digits(&mut self, builder: &mut AutoCow<'a>) {
        if self.peek().is_some_and(|c| c.is_ascii_digit()) {
            let c = self.current.chars.next().unwrap();
            builder.push_matching(c);
        } else {
            return;
        }
        self.read_decimal_digits_after_first_digit(builder);
    }

    fn optional_exponent(&mut self, builder: &mut AutoCow<'a>) -> Option<Kind> {
        if matches!(self.peek(), Some('e' | 'E')) {
            let c = self.current.chars.next().unwrap();
            builder.push_matching(c);
            return Some(self.read_decimal_exponent(builder));
        }
        None
    }

    fn check_after_numeric_literal(&mut self, kind: Kind) -> Kind {
        let offset = self.offset();
        // The SourceCharacter immediately following a NumericLiteral must not be an IdentifierStart or DecimalDigit.
        let c = self.peek();
        if c.is_none() || c.is_some_and(|ch| !ch.is_ascii_digit() && !is_identifier_start_all(ch)) {
            return kind;
        }
        self.current.chars.next();
        while let Some(c) = self.peek() {
            if is_identifier_start_all(c) {
                self.current.chars.next();
            } else {
                break;
            }
        }
        self.error(diagnostics::InvalidNumberEnd(Span::new(offset, self.offset())));
        Kind::Undetermined
    }

    /// 12.8.4 String Literals
    fn read_string_literal(&mut self, delimiter: char) -> Kind {
        let mut builder = AutoCow::new(self);
        loop {
            match self.current.chars.next() {
                None | Some('\r' | '\n') => {
                    self.error(diagnostics::UnterminatedString(self.unterminated_range()));
                    return Kind::Undetermined;
                }
                Some(c @ ('"' | '\'')) => {
                    if c == delimiter {
                        self.current.token.value =
                            self.string_to_token_value(builder.finish_without_push(self));
                        return Kind::Str;
                    }
                    builder.push_matching(c);
                }
                Some('\\') => {
                    let start = self.offset() - 1;
                    let text = builder.get_mut_string_without_current_ascii_char(self);
                    let mut is_valid_escape_sequence = true;
                    self.read_string_escape_sequence(text, false, &mut is_valid_escape_sequence);
                    if !is_valid_escape_sequence {
                        let range = Span::new(start, self.offset());
                        self.error(diagnostics::InvalidEscapeSequence(range));
                    }
                }
                Some(other) => {
                    builder.push_matching(other);
                }
            }
        }
    }

    /// 12.8.5 Regular Expression Literals
    fn read_regex(&mut self) -> Kind {
        let start = self.current.token.start + 1; // +1 to exclude `/`
        let mut in_escape = false;
        let mut in_character_class = false;
        loop {
            match self.current.chars.next() {
                None => {
                    self.error(diagnostics::UnterminatedRegExp(self.unterminated_range()));
                    return Kind::Undetermined;
                }
                Some(c) if is_line_terminator(c) => {
                    self.error(diagnostics::UnterminatedRegExp(self.unterminated_range()));
                    return Kind::Undetermined;
                }
                Some(c) => {
                    if in_escape {
                        in_escape = false;
                    } else if c == '/' && !in_character_class {
                        break;
                    } else if c == '[' {
                        in_character_class = true;
                    } else if c == '\\' {
                        in_escape = true;
                    } else if c == ']' {
                        in_character_class = false;
                    }
                }
            }
        }

        let end = self.offset() - 1; // -1 to exclude `/`
        let pattern = &self.source[start as usize..end as usize];

        let mut flags = RegExpFlags::empty();

        while let Some(ch @ ('$' | '_' | 'a'..='z' | 'A'..='Z' | '0'..='9')) = self.peek() {
            self.current.chars.next();
            if !ch.is_ascii_lowercase() {
                self.error(diagnostics::RegExpFlag(ch, self.current_offset()));
                continue;
            }
            let flag = match ch {
                'g' => RegExpFlags::G,
                'i' => RegExpFlags::I,
                'm' => RegExpFlags::M,
                's' => RegExpFlags::S,
                'u' => RegExpFlags::U,
                'y' => RegExpFlags::Y,
                'd' => RegExpFlags::D,
                'v' => RegExpFlags::V,
                _ => {
                    self.error(diagnostics::RegExpFlag(ch, self.current_offset()));
                    continue;
                }
            };
            if flags.contains(flag) {
                self.error(diagnostics::RegExpFlagTwice(ch, self.current_offset()));
                continue;
            }
            flags |= flag;
        }

        self.current.token.value = TokenValue::RegExp(RegExp { pattern, flags });

        Kind::RegExp
    }

    /// 12.8.6 Template Literal Lexical Components
    fn read_template_literal(&mut self, substitute: Kind, tail: Kind) -> Kind {
        let mut builder = AutoCow::new(self);
        let mut is_valid_escape_sequence = true;
        while let Some(c) = self.current.chars.next() {
            match c {
                '$' if self.peek() == Some('{') => {
                    if is_valid_escape_sequence {
                        self.current.token.value =
                            self.string_to_token_value(builder.finish_without_push(self));
                    }
                    self.current.chars.next();
                    return substitute;
                }
                '`' => {
                    if is_valid_escape_sequence {
                        self.current.token.value =
                            self.string_to_token_value(builder.finish_without_push(self));
                    }
                    return tail;
                }
                CR => {
                    builder.force_allocation_without_current_ascii_char(self);
                    if self.next_eq(LF) {
                        builder.push_different(LF);
                    }
                }
                '\\' => {
                    let text = builder.get_mut_string_without_current_ascii_char(self);
                    self.read_string_escape_sequence(text, true, &mut is_valid_escape_sequence);
                }
                _ => builder.push_matching(c),
            }
        }
        self.error(diagnostics::UnterminatedString(self.unterminated_range()));
        Kind::Undetermined
    }

    /// `JSXIdentifier` :
    ///   `IdentifierStart`
    ///   `JSXIdentifier` `IdentifierPart`
    ///   `JSXIdentifier` [no `WhiteSpace` or Comment here] -
    fn read_jsx_identifier(&mut self, start_offset: u32) -> Kind {
        let prev_str = &self.source[start_offset as usize..self.offset() as usize];

        let mut builder = AutoCow::new(self);
        while let Some(c) = self.peek() {
            if c == '-' || is_identifier_start_all(c) {
                self.current.chars.next();
                builder.push_matching(c);
                while let Some(c) = self.peek() {
                    if is_identifier_part(c) {
                        let c = self.current.chars.next().unwrap();
                        builder.push_matching(c);
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        }
        let mut s = String::from_str_in(prev_str, self.allocator);
        s.push_str(builder.finish(self));
        self.current.token.value = self.string_to_token_value(s.into_bump_str());
        Kind::Ident
    }

    /// [`JSXChild`](https://facebook.github.io/jsx/#prod-JSXChild)
    /// `JSXChild` :
    /// `JSXText`
    /// `JSXElement`
    /// `JSXFragment`
    /// { `JSXChildExpressionopt` }
    fn read_jsx_child(&mut self) -> Kind {
        match self.peek() {
            Some('<') => {
                self.current.chars.next();
                Kind::LAngle
            }
            Some('{') => {
                self.current.chars.next();
                Kind::LCurly
            }
            Some(c) => {
                let mut builder = AutoCow::new(self);
                builder.push_matching(c);
                loop {
                    // `>` and `}` are errors in TypeScript but not Babel
                    // let's make this less strict so we can parse more code
                    if matches!(self.peek(), Some('{' | '<')) {
                        break;
                    }
                    if let Some(c) = self.current.chars.next() {
                        builder.push_matching(c);
                    } else {
                        break;
                    }
                }
                self.current.token.value = self.string_to_token_value(builder.finish(self));
                Kind::JSXText
            }
            None => Kind::Eof,
        }
    }

    /// `JSXDoubleStringCharacters` ::
    ///   `JSXDoubleStringCharacter` `JSXDoubleStringCharactersopt`
    /// `JSXDoubleStringCharacter` ::
    ///   `JSXStringCharacter` but not "
    /// `JSXSingleStringCharacters` ::
    ///   `JSXSingleStringCharacter` `JSXSingleStringCharactersopt`
    /// `JSXSingleStringCharacter` ::
    ///   `JSXStringCharacter` but not '
    /// `JSXStringCharacter` ::
    ///   `SourceCharacter` but not one of `HTMLCharacterReference`
    fn read_jsx_string_literal(&mut self, delimiter: char) -> Kind {
        let mut builder = AutoCow::new(self);
        loop {
            match self.current.chars.next() {
                Some(c @ ('"' | '\'')) => {
                    if c == delimiter {
                        self.current.token.value =
                            self.string_to_token_value(builder.finish_without_push(self));
                        return Kind::Str;
                    }
                    builder.push_matching(c);
                }
                Some(other) => {
                    builder.push_matching(other);
                }
                None => {
                    self.error(diagnostics::UnterminatedString(self.unterminated_range()));
                    return Kind::Undetermined;
                }
            }
        }
    }

    /* ---------- utils ---------- */

    /// Identifier `UnicodeEscapeSequence`
    ///   \u `Hex4Digits`
    ///   \u{ `CodePoint` }
    fn identifier_unicode_escape_sequence(
        &mut self,
        builder: &mut AutoCow<'a>,
        check_identifier_start: bool,
    ) {
        let start = self.offset();
        if self.current.chars.next() != Some('u') {
            let range = Span::new(start, self.offset());
            self.error(diagnostics::UnicodeEscapeSequence(range));
            return;
        }

        let value = match self.peek() {
            Some('{') => self.unicode_code_point(),
            _ => self.surrogate_pair(),
        };

        let Some(value) = value else {
            let range = Span::new(start, self.offset());
            self.error(diagnostics::UnicodeEscapeSequence(range));
            return;
        };

        // For Identifiers, surrogate pair is an invalid grammar, e.g. `var \uD800\uDEA7`.
        let ch = match value {
            SurrogatePair::Astral(..) | SurrogatePair::HighLow(..) => {
                let range = Span::new(start, self.offset());
                self.error(diagnostics::UnicodeEscapeSequence(range));
                return;
            }
            SurrogatePair::CodePoint(code_point) => {
                if let Ok(ch) = char::try_from(code_point) {
                    ch
                } else {
                    let range = Span::new(start, self.offset());
                    self.error(diagnostics::UnicodeEscapeSequence(range));
                    return;
                }
            }
        };

        let is_valid = if check_identifier_start {
            is_identifier_start_all(ch)
        } else {
            is_identifier_part(ch)
        };

        if !is_valid {
            self.error(diagnostics::InvalidCharacter(ch, self.current_offset()));
            return;
        }

        builder.push_different(ch);
    }

    /// String `UnicodeEscapeSequence`
    ///   \u `Hex4Digits`
    ///   \u `Hex4Digits` \u `Hex4Digits`
    ///   \u{ `CodePoint` }
    fn string_unicode_escape_sequence(
        &mut self,
        text: &mut String<'a>,
        is_valid_escape_sequence: &mut bool,
    ) {
        let value = match self.peek() {
            Some('{') => self.unicode_code_point(),
            _ => self.surrogate_pair(),
        };

        let Some(value) = value else {
            // error raised within the parser by `diagnostics::TemplateLiteral`
            *is_valid_escape_sequence = false;
            return;
        };

        // For strings and templates, surrogate pairs are valid grammar, e.g. `"\uD83D\uDE00" === 😀`
        // values are interpreted as is if they fall out of range
        match value {
            SurrogatePair::CodePoint(code_point) | SurrogatePair::Astral(code_point) => {
                if let Ok(ch) = char::try_from(code_point) {
                    text.push(ch);
                } else {
                    text.push_str("\\u");
                    text.push_str(format!("{code_point:x}").as_str());
                }
            }
            SurrogatePair::HighLow(high, low) => {
                text.push_str("\\u");
                text.push_str(format!("{high:x}").as_str());
                text.push_str("\\u");
                text.push_str(format!("{low:x}").as_str());
            }
        }
    }

    fn unicode_code_point(&mut self) -> Option<SurrogatePair> {
        if !self.next_eq('{') {
            return None;
        }
        let value = self.code_point()?;
        if !self.next_eq('}') {
            return None;
        }
        Some(SurrogatePair::CodePoint(value))
    }

    fn hex_4_digits(&mut self) -> Option<u32> {
        let mut value = 0;
        for _ in 0..4 {
            value = (value << 4) | self.hex_digit()?;
        }
        Some(value)
    }

    fn hex_digit(&mut self) -> Option<u32> {
        let value = match self.peek() {
            Some(c @ '0'..='9') => c as u32 - '0' as u32,
            Some(c @ 'a'..='f') => 10 + (c as u32 - 'a' as u32),
            Some(c @ 'A'..='F') => 10 + (c as u32 - 'A' as u32),
            _ => return None,
        };
        self.current.chars.next();
        Some(value)
    }

    fn code_point(&mut self) -> Option<u32> {
        let mut value = self.hex_digit()?;
        while let Some(next) = self.hex_digit() {
            value = (value << 4) | next;
            if value > 0x0010_FFFF {
                return None;
            }
        }
        Some(value)
    }

    /// Surrogate pairs
    /// See background info:
    ///   * `https://mathiasbynens.be/notes/javascript-encoding#surrogate-formulae`
    ///   * `https://mathiasbynens.be/notes/javascript-identifiers-es6`
    fn surrogate_pair(&mut self) -> Option<SurrogatePair> {
        let high = self.hex_4_digits()?;
        // The first code unit of a surrogate pair is always in the range from 0xD800 to 0xDBFF, and is called a high surrogate or a lead surrogate.
        if !((0xD800..=0xDBFF).contains(&high)
            && self.peek() == Some('\\')
            && self.peek2() == Some('u'))
        {
            return Some(SurrogatePair::CodePoint(high));
        }

        self.current.chars.next();
        self.current.chars.next();

        let low = self.hex_4_digits()?;

        // The second code unit of a surrogate pair is always in the range from 0xDC00 to 0xDFFF, and is called a low surrogate or a trail surrogate.
        if !(0xDC00..=0xDFFF).contains(&low) {
            return Some(SurrogatePair::HighLow(high, low));
        }

        // `https://tc39.es/ecma262/#sec-utf16decodesurrogatepair`
        let astral_code_point = (high - 0xD800) * 0x400 + low - 0xDC00 + 0x10000;

        Some(SurrogatePair::Astral(astral_code_point))
    }

    // EscapeSequence ::
    fn read_string_escape_sequence(
        &mut self,
        text: &mut String<'a>,
        in_template: bool,
        is_valid_escape_sequence: &mut bool,
    ) {
        match self.current.chars.next() {
            None => {
                self.error(diagnostics::UnterminatedString(self.unterminated_range()));
            }
            Some(c) => match c {
                // CharacterEscapeSequence
                LF | LS | PS => {}
                CR => {
                    self.next_eq(LF);
                }
                '\'' | '"' | '\\' => text.push(c),
                'b' => text.push('\u{8}'),
                'f' => text.push(FF),
                'n' => text.push(LF),
                'r' => text.push(CR),
                't' => text.push(TAB),
                'v' => text.push(VT),
                // HexEscapeSequence
                'x' => {
                    self.hex_digit()
                        .and_then(|value1| {
                            let value2 = self.hex_digit()?;
                            Some((value1, value2))
                        })
                        .map(|(value1, value2)| (value1 << 4) | value2)
                        .and_then(|value| char::try_from(value).ok())
                        .map_or_else(
                            || {
                                *is_valid_escape_sequence = false;
                            },
                            |c| {
                                text.push(c);
                            },
                        );
                }
                // UnicodeEscapeSequence
                'u' => {
                    self.string_unicode_escape_sequence(text, is_valid_escape_sequence);
                }
                // 0 [lookahead ∉ DecimalDigit]
                '0' if !self.peek().is_some_and(|c| c.is_ascii_digit()) => text.push('\0'),
                // Section 12.8.4 String Literals
                // LegacyOctalEscapeSequence
                // NonOctalDecimalEscapeSequence
                a @ '0'..='7' if !in_template => {
                    let mut num = String::new_in(self.allocator);
                    num.push(a);
                    match a {
                        '4'..='7' => {
                            if matches!(self.peek(), Some('0'..='7')) {
                                let b = self.current.chars.next().unwrap();
                                num.push(b);
                            }
                        }
                        '0'..='3' => {
                            if matches!(self.peek(), Some('0'..='7')) {
                                let b = self.current.chars.next().unwrap();
                                num.push(b);
                                if matches!(self.peek(), Some('0'..='7')) {
                                    let c = self.current.chars.next().unwrap();
                                    num.push(c);
                                }
                            }
                        }
                        _ => {}
                    }

                    let value =
                        char::from_u32(u32::from_str_radix(num.as_str(), 8).unwrap()).unwrap();
                    text.push(value);
                }
                '0' if in_template && self.peek().is_some_and(|c| c.is_ascii_digit()) => {
                    self.current.chars.next();
                    // error raised within the parser by `diagnostics::TemplateLiteral`
                    *is_valid_escape_sequence = false;
                }
                // NotEscapeSequence :: DecimalDigit but not 0
                '1'..='9' if in_template => {
                    // error raised within the parser by `diagnostics::TemplateLiteral`
                    *is_valid_escape_sequence = false;
                }
                other => {
                    // NonOctalDecimalEscapeSequence \8 \9 in strict mode
                    text.push(other);
                }
            },
        }
    }
}

enum SurrogatePair {
    // valid \u Hex4Digits \u Hex4Digits
    Astral(u32),
    // valid \u Hex4Digits
    CodePoint(u32),
    // invalid \u Hex4Digits \u Hex4Digits
    HighLow(u32, u32),
}

type ByteHandler = fn(&mut Lexer<'_>) -> Kind;

/// Lookup table mapping any incoming byte to a handler function defined below.
/// <https://github.com/ratel-rust/ratel-core/blob/master/ratel/src/lexer/mod.rs>
#[rustfmt::skip]
static BYTE_HANDLERS: [ByteHandler; 128] = [
//  0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F    //
    ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, SPS, LIN, SPS, SPS, LIN, ERR, ERR, // 0
    ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, // 1
    SPS, EXL, QOT, HAS, IDT, PRC, AMP, QOT, PNO, PNC, ATR, PLS, COM, MIN, PRD, SLH, // 2
    ZER, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, COL, SEM, LSS, EQL, GTR, QST, // 3
    AT_, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, // 4
    IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, BTO, ESC, BTC, CRT, IDT, // 5
    TPL, L_A, L_B, L_C, L_D, L_E, L_F, L_G, IDT, L_I, IDT, L_K, L_L, L_M, L_N, L_O, // 6
    L_P, IDT, L_R, L_S, L_T, L_U, L_V, L_W, IDT, L_Y, IDT, BEO, PIP, BEC, TLD, ERR, // 7
];

const ERR: ByteHandler = |lexer| {
    let c = lexer.consume_char();
    lexer.error(diagnostics::InvalidCharacter(c, lexer.unterminated_range()));
    Kind::Undetermined
};

// <TAB> <VT> <FF>
const SPS: ByteHandler = |lexer| {
    lexer.consume_char();
    Kind::WhiteSpace
};

// '\r' '\n'
const LIN: ByteHandler = |lexer| {
    lexer.consume_char();
    lexer.current.token.is_on_new_line = true;
    Kind::NewLine
};

// !
const EXL: ByteHandler = |lexer| {
    lexer.consume_char();
    if lexer.next_eq('=') {
        if lexer.next_eq('=') {
            Kind::Neq2
        } else {
            Kind::Neq
        }
    } else {
        Kind::Bang
    }
};

// ' "
const QOT: ByteHandler = |lexer| {
    let c = lexer.consume_char();
    if lexer.context == LexerContext::JsxAttributeValue {
        lexer.read_jsx_string_literal(c)
    } else {
        lexer.read_string_literal(c)
    }
};

// #
const HAS: ByteHandler = |lexer| {
    let mut builder = AutoCow::new(lexer);
    let c = lexer.consume_char();
    builder.push_matching(c);
    // HashbangComment ::
    //     `#!` SingleLineCommentChars?
    if lexer.current.token.start == 0 && lexer.next_eq('!') {
        lexer.read_hashbang_comment()
    } else {
        builder.get_mut_string_without_current_ascii_char(lexer);
        lexer.private_identifier(builder)
    }
};

const IDT: ByteHandler = |lexer| {
    lexer.identifier_name_handler();
    Kind::Ident
};

// %
const PRC: ByteHandler = |lexer| {
    lexer.consume_char();
    if lexer.next_eq('=') {
        Kind::PercentEq
    } else {
        Kind::Percent
    }
};

// &
const AMP: ByteHandler = |lexer| {
    lexer.consume_char();
    if lexer.next_eq('&') {
        if lexer.next_eq('=') {
            Kind::Amp2Eq
        } else {
            Kind::Amp2
        }
    } else if lexer.next_eq('=') {
        Kind::AmpEq
    } else {
        Kind::Amp
    }
};

// (
const PNO: ByteHandler = |lexer| {
    lexer.consume_char();
    Kind::LParen
};

// )
const PNC: ByteHandler = |lexer| {
    lexer.consume_char();
    Kind::RParen
};

// *
const ATR: ByteHandler = |lexer| {
    lexer.consume_char();
    if lexer.next_eq('*') {
        if lexer.next_eq('=') {
            Kind::Star2Eq
        } else {
            Kind::Star2
        }
    } else if lexer.next_eq('=') {
        Kind::StarEq
    } else {
        Kind::Star
    }
};

// +
const PLS: ByteHandler = |lexer| {
    lexer.consume_char();
    if lexer.next_eq('+') {
        Kind::Plus2
    } else if lexer.next_eq('=') {
        Kind::PlusEq
    } else {
        Kind::Plus
    }
};

// ,
const COM: ByteHandler = |lexer| {
    lexer.consume_char();
    Kind::Comma
};

// -
const MIN: ByteHandler = |lexer| {
    lexer.consume_char();
    lexer.read_minus().unwrap_or_else(|| lexer.skip_single_line_comment())
};

// .
const PRD: ByteHandler = |lexer| {
    let mut builder = AutoCow::new(lexer);
    let c = lexer.consume_char();
    builder.push_matching(c);
    let kind = lexer.read_dot(&mut builder);
    if kind.is_number() {
        lexer.set_numeric_value(kind, builder.finish(lexer));
    }
    kind
};

// /
const SLH: ByteHandler = |lexer| {
    lexer.consume_char();
    match lexer.peek() {
        Some('/') => {
            lexer.current.chars.next();
            lexer.skip_single_line_comment()
        }
        Some('*') => {
            lexer.current.chars.next();
            lexer.skip_multi_line_comment()
        }
        _ => {
            // regex is handled separately, see `next_regex`
            if lexer.next_eq('=') {
                Kind::SlashEq
            } else {
                Kind::Slash
            }
        }
    }
};

// 0
const ZER: ByteHandler = |lexer| {
    let mut builder = AutoCow::new(lexer);
    let c = lexer.consume_char();
    builder.push_matching(c);
    let kind = lexer.read_zero(&mut builder);
    lexer.set_numeric_value(kind, builder.finish(lexer));
    kind
};

// 1 to 9
const DIG: ByteHandler = |lexer| {
    let mut builder = AutoCow::new(lexer);
    let c = lexer.consume_char();
    builder.push_matching(c);
    let kind = lexer.decimal_literal_after_first_digit(&mut builder);
    lexer.set_numeric_value(kind, builder.finish(lexer));
    kind
};

// :
const COL: ByteHandler = |lexer| {
    lexer.consume_char();
    Kind::Colon
};

// ;
const SEM: ByteHandler = |lexer| {
    lexer.consume_char();
    Kind::Semicolon
};

// <
const LSS: ByteHandler = |lexer| {
    lexer.consume_char();
    lexer.read_left_angle().unwrap_or_else(|| lexer.skip_single_line_comment())
};

// =
const EQL: ByteHandler = |lexer| {
    lexer.consume_char();
    if lexer.next_eq('=') {
        if lexer.next_eq('=') {
            Kind::Eq3
        } else {
            Kind::Eq2
        }
    } else if lexer.next_eq('>') {
        Kind::Arrow
    } else {
        Kind::Eq
    }
};

// >
const GTR: ByteHandler = |lexer| {
    lexer.consume_char();
    // `>=` is re-lexed with [Lexer::next_jsx_child]
    Kind::RAngle
};

// ?
const QST: ByteHandler = |lexer| {
    lexer.consume_char();
    if lexer.next_eq('?') {
        if lexer.next_eq('=') {
            Kind::Question2Eq
        } else {
            Kind::Question2
        }
    } else if lexer.peek() == Some('.') {
        // parse `?.1` as `?` `.1`
        if lexer.peek2().is_some_and(|c| c.is_ascii_digit()) {
            Kind::Question
        } else {
            lexer.current.chars.next();
            Kind::QuestionDot
        }
    } else {
        Kind::Question
    }
};

// @
const AT_: ByteHandler = |lexer| {
    lexer.consume_char();
    Kind::At
};

// [
const BTO: ByteHandler = |lexer| {
    lexer.consume_char();
    Kind::LBrack
};

// \
const ESC: ByteHandler = |lexer| {
    let mut builder = AutoCow::new(lexer);
    let c = lexer.consume_char();
    builder.push_matching(c);
    builder.force_allocation_without_current_ascii_char(lexer);
    lexer.identifier_unicode_escape_sequence(&mut builder, true);
    let text = lexer.identifier_name(builder);
    Kind::match_keyword(text)
};

// ]
const BTC: ByteHandler = |lexer| {
    lexer.consume_char();
    Kind::RBrack
};

// ^
const CRT: ByteHandler = |lexer| {
    lexer.consume_char();
    if lexer.next_eq('=') {
        Kind::CaretEq
    } else {
        Kind::Caret
    }
};

// `
const TPL: ByteHandler = |lexer| {
    lexer.consume_char();
    lexer.read_template_literal(Kind::TemplateHead, Kind::NoSubstitutionTemplate)
};

// {
const BEO: ByteHandler = |lexer| {
    lexer.consume_char();
    Kind::LCurly
};

// |
const PIP: ByteHandler = |lexer| {
    lexer.consume_char();
    if lexer.next_eq('|') {
        if lexer.next_eq('=') {
            Kind::Pipe2Eq
        } else {
            Kind::Pipe2
        }
    } else if lexer.next_eq('=') {
        Kind::PipeEq
    } else {
        Kind::Pipe
    }
};

// }
const BEC: ByteHandler = |lexer| {
    lexer.consume_char();
    Kind::RCurly
};

// ~
const TLD: ByteHandler = |lexer| {
    lexer.consume_char();
    Kind::Tilde
};

const L_A: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "wait" => Kind::Await,
    "sync" => Kind::Async,
    "bstract" => Kind::Abstract,
    "ccessor" => Kind::Accessor,
    "ny" => Kind::Any,
    "s" => Kind::As,
    "ssert" => Kind::Assert,
    "sserts" => Kind::Asserts,
    _ => Kind::Ident,
};

const L_B: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "reak" => Kind::Break,
    "oolean" => Kind::Boolean,
    "igint" => Kind::BigInt,
    _ => Kind::Ident,
};

const L_C: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "onst" => Kind::Const,
    "lass" => Kind::Class,
    "ontinue" => Kind::Continue,
    "atch" => Kind::Catch,
    "ase" => Kind::Case,
    "onstructor" => Kind::Constructor,
    _ => Kind::Ident,
};

const L_D: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "o" => Kind::Do,
    "elete" => Kind::Delete,
    "eclare" => Kind::Declare,
    "efault" => Kind::Default,
    "ebugger" => Kind::Debugger,
    _ => Kind::Ident,
};

const L_E: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "lse" => Kind::Else,
    "num" => Kind::Enum,
    "xport" => Kind::Export,
    "xtends" => Kind::Extends,
    _ => Kind::Ident,
};

const L_F: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "unction" => Kind::Function,
    "alse" => Kind::False,
    "or" => Kind::For,
    "inally" => Kind::Finally,
    "rom" => Kind::From,
    _ => Kind::Ident,
};

const L_G: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "et" => Kind::Get,
    "lobal" => Kind::Global,
    _ => Kind::Ident,
};

const L_I: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "f" => Kind::If,
    "nstanceof" => Kind::Instanceof,
    "n" => Kind::In,
    "mplements" => Kind::Implements,
    "mport" => Kind::Import,
    "nfer" => Kind::Infer,
    "nterface" => Kind::Interface,
    "ntrinsic" => Kind::Intrinsic,
    "s" => Kind::Is,
    _ => Kind::Ident,
};

const L_K: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "eyof" => Kind::KeyOf,
    _ => Kind::Ident,
};

const L_L: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "et" => Kind::Let,
    _ => Kind::Ident,
};

const L_M: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "eta" => Kind::Meta,
    "odule" => Kind::Module,
    _ => Kind::Ident,
};

const L_N: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "ull" => Kind::Null,
    "ew" => Kind::New,
    "umber" => Kind::Number,
    "amespace" => Kind::Namespace,
    "ever" => Kind::Never,
    _ => Kind::Ident,
};

const L_O: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "f" => Kind::Of,
    "bject" => Kind::Object,
    "ut" => Kind::Out,
    "verride" => Kind::Override,
    _ => Kind::Ident,
};

const L_P: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "ackage" => Kind::Package,
    "rivate" => Kind::Private,
    "rotected" => Kind::Protected,
    "ublic" => Kind::Public,
    _ => Kind::Ident,
};

const L_R: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "eturn" => Kind::Return,
    "equire" => Kind::Require,
    "eadonly" => Kind::Readonly,
    _ => Kind::Ident,
};

const L_S: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "et" => Kind::Set,
    "uper" => Kind::Super,
    "witch" => Kind::Switch,
    "tatic" => Kind::Static,
    "ymbol" => Kind::Symbol,
    "tring" => Kind::String,
    "atisfies" => Kind::Satisfies,
    _ => Kind::Ident,
};

const L_T: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "his" => Kind::This,
    "rue" => Kind::True,
    "hrow" => Kind::Throw,
    "ry" => Kind::Try,
    "ypeof" => Kind::Typeof,
    "arget" => Kind::Target,
    "ype" => Kind::Type,
    _ => Kind::Ident,
};

const L_U: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "ndefined" => Kind::Undefined,
    "sing" => Kind::Using,
    "nique" => Kind::Unique,
    "nknown" => Kind::Unknown,
    _ => Kind::Ident,
};

const L_V: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "ar" => Kind::Var,
    "oid" => Kind::Void,
    _ => Kind::Ident,
};

const L_W: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "hile" => Kind::While,
    "ith" => Kind::With,
    _ => Kind::Ident,
};

const L_Y: ByteHandler = |lexer| match &lexer.identifier_name_handler()[1..] {
    "ield" => Kind::Yield,
    _ => Kind::Ident,
};
