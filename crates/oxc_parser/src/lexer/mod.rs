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

use rustc_hash::FxHashMap;
use std::{
    collections::VecDeque,
    str::{Bytes, Chars},
};

use oxc_allocator::{Allocator, String};
use oxc_ast::ast::RegExpFlags;
use oxc_diagnostics::Error;
use oxc_span::{SourceType, Span};
use oxc_syntax::identifier::{
    is_identifier_part, is_identifier_part_ascii_byte, is_identifier_part_unicode,
    is_identifier_start, is_identifier_start_ascii_byte, is_identifier_start_unicode,
    is_irregular_line_terminator, is_irregular_whitespace, is_line_terminator, CR, FF, LF, LS, PS,
    TAB, VT,
};

pub use self::{
    kind::Kind,
    number::{parse_big_int, parse_float, parse_int},
    token::Token,
};
use self::{string_builder::AutoCow, trivia_builder::TriviaBuilder};
use crate::{diagnostics, MAX_LEN};

#[derive(Debug, Clone)]
pub struct LexerCheckpoint<'a> {
    /// Remaining chars to be tokenized
    chars: Chars<'a>,

    token: Token,

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

    /// Data store for escaped strings, indexed by [Token::start] when [Token::escaped] is true
    pub escaped_strings: FxHashMap<u32, &'a str>,

    /// Data store for escaped templates, indexed by [Token::start] when [Token::escaped] is true
    /// `None` is saved when the string contains an invalid escape sequence.
    pub escaped_templates: FxHashMap<u32, Option<&'a str>>,
}

#[allow(clippy::unused_self)]
impl<'a> Lexer<'a> {
    pub fn new(allocator: &'a Allocator, source: &'a str, source_type: SourceType) -> Self {
        // Token's start and end are u32s, so limit for length of source is u32::MAX bytes.
        // Only a debug assertion is required, as parser checks length of source before calling
        // this method.
        debug_assert!(source.len() <= MAX_LEN, "Source length exceeds MAX_LEN");

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
            escaped_strings: FxHashMap::default(),
            escaped_templates: FxHashMap::default(),
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
            token: self.current.token,
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
    pub fn lookahead(&mut self, n: u8) -> Token {
        let n = n as usize;
        debug_assert!(n > 0);

        if self.lookahead.len() > n - 1 {
            return self.lookahead[n - 1].token;
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

        self.lookahead[n - 1].token
    }

    /// Set context
    pub fn set_context(&mut self, context: LexerContext) {
        self.context = context;
    }

    /// Main entry point
    pub fn next_token(&mut self) -> Token {
        if let Some(checkpoint) = self.lookahead.pop_front() {
            self.current.chars = checkpoint.chars;
            self.current.errors_pos = checkpoint.errors_pos;
            return checkpoint.token;
        }
        let kind = self.read_next_token();
        self.finish_next(kind)
    }

    pub fn next_jsx_child(&mut self) -> Token {
        self.current.token.start = self.offset();
        let kind = self.read_jsx_child();
        self.finish_next(kind)
    }

    fn finish_next(&mut self, kind: Kind) -> Token {
        self.current.token.kind = kind;
        self.current.token.end = self.offset();
        debug_assert!(self.current.token.start <= self.current.token.end);
        let token = self.current.token;
        self.current.token = Token::default();
        token
    }

    /// Re-tokenize the current `/` or `/=` and return `RegExp`
    /// See Section 12:
    ///   The `InputElementRegExp` goal symbol is used in all syntactic grammar contexts
    ///   where a `RegularExpressionLiteral` is permitted
    /// Which means the parser needs to re-tokenize on `PrimaryExpression`,
    /// `RegularExpressionLiteral` only appear on the right hand side of `PrimaryExpression`
    pub fn next_regex(&mut self, kind: Kind) -> (Token, u32, RegExpFlags) {
        self.current.token.start = self.offset()
            - match kind {
                Kind::Slash => 1,
                Kind::SlashEq => 2,
                _ => unreachable!(),
            };
        let (pattern_end, flags) = self.read_regex();
        self.lookahead.clear();
        let token = self.finish_next(Kind::RegExp);
        (token, pattern_end, flags)
    }

    pub fn next_right_angle(&mut self) -> Token {
        let kind = self.read_right_angle();
        self.lookahead.clear();
        self.finish_next(kind)
    }

    /// Re-tokenize the current `}` token for `TemplateSubstitutionTail`
    /// See Section 12, the parser needs to re-tokenize on `TemplateSubstitutionTail`,
    pub fn next_template_substitution_tail(&mut self) -> Token {
        self.current.token.start = self.offset() - 1;
        let kind = self.read_template_literal(Kind::TemplateMiddle, Kind::TemplateTail);
        self.lookahead.clear();
        self.finish_next(kind)
    }

    /// Expand the current token for `JSXIdentifier`
    pub fn next_jsx_identifier(&mut self, start_offset: u32) -> Token {
        let kind = self.read_jsx_identifier(start_offset);
        self.lookahead.clear();
        self.finish_next(kind)
    }

    /// Re-tokenize '<<' or '<=' or '<<=' to '<'
    pub fn re_lex_as_typescript_l_angle(&mut self, kind: Kind) -> Token {
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

    /// Save the string if it is escaped
    /// This reduces the overall memory consumption while keeping the `Token` size small
    /// Strings without escaped values can be retrieved as is from the token span
    fn save_string(&mut self, has_escape: bool, s: &'a str) {
        if !has_escape {
            return;
        }
        self.escaped_strings.insert(self.current.token.start, s);
        self.current.token.escaped = true;
    }

    pub(crate) fn get_string(&self, token: Token) -> &'a str {
        if token.escaped {
            return self.escaped_strings[&token.start];
        }

        let raw = &self.source[token.start as usize..token.end as usize];
        match token.kind {
            Kind::Str => {
                &raw[1..raw.len() - 1] // omit surrounding quotes
            }
            Kind::PrivateIdentifier => {
                &raw[1..] // omit leading `#`
            }
            _ => raw,
        }
    }

    /// Save the template if it is escaped
    fn save_template_string(
        &mut self,
        is_valid_escape_sequence: bool,
        has_escape: bool,
        s: &'a str,
    ) {
        if !has_escape {
            return;
        }
        self.escaped_templates
            .insert(self.current.token.start, is_valid_escape_sequence.then(|| s));
        self.current.token.escaped = true;
    }

    pub(crate) fn get_template_string(&self, token: Token) -> Option<&'a str> {
        if token.escaped {
            return self.escaped_templates[&token.start];
        }
        let raw = &self.source[token.start as usize..token.end as usize];
        Some(match token.kind {
            Kind::NoSubstitutionTemplate | Kind::TemplateTail => {
                &raw[1..raw.len() - 1] // omit surrounding quotes or leading "}" and trailing "`"
            }
            Kind::TemplateHead | Kind::TemplateMiddle => {
                &raw[1..raw.len() - 2] // omit leading "`" or "}" and trailing "${"
            }
            _ => raw,
        })
    }

    /// Read each char and set the current token
    /// Whitespace and line terminators are skipped
    fn read_next_token(&mut self) -> Kind {
        loop {
            let offset = self.offset();
            self.current.token.start = offset;

            let remaining = self.current.chars.as_str();
            if remaining.is_empty() {
                return Kind::Eof;
            }

            let byte = remaining.as_bytes()[0];
            // SAFETY: Check for `remaining.is_empty()` ensures not at end of file,
            // and `byte` is the byte at current position of `self.current.chars`.
            let kind = unsafe { handle_byte(byte, self) };
            if kind != Kind::Skip {
                return kind;
            }
        }
    }

    fn unicode_char_handler(&mut self) -> Kind {
        let mut chars = self.current.chars.clone();
        let c = chars.next().unwrap();
        match c {
            c if is_identifier_start_unicode(c) => {
                // `bytes` is positioned after this char
                let bytes = chars.as_str().bytes();
                self.identifier_tail_after_no_escape(bytes);
                Kind::Ident
            }
            c if is_irregular_whitespace(c) => {
                self.trivia_builder
                    .add_irregular_whitespace(self.current.token.start, self.offset());
                self.consume_char();
                Kind::Skip
            }
            c if is_irregular_line_terminator(c) => {
                self.consume_char();
                self.current.token.is_on_new_line = true;
                Kind::Skip
            }
            _ => {
                self.consume_char();
                self.error(diagnostics::InvalidCharacter(c, self.unterminated_range()));
                Kind::Undetermined
            }
        }
    }

    /// Section 12.4 Single Line Comment
    #[allow(clippy::cast_possible_truncation)]
    fn skip_single_line_comment(&mut self) -> Kind {
        let start = self.current.token.start;
        while let Some(c) = self.current.chars.next() {
            if is_line_terminator(c) {
                self.current.token.is_on_new_line = true;
                self.trivia_builder
                    .add_single_line_comment(start, self.offset() - c.len_utf8() as u32);
                return Kind::Skip;
            }
        }
        // EOF
        self.trivia_builder.add_single_line_comment(start, self.offset());
        Kind::Skip
    }

    /// Section 12.4 Multi Line Comment
    fn skip_multi_line_comment(&mut self) -> Kind {
        while let Some(c) = self.current.chars.next() {
            if c == '*' && self.next_eq('/') {
                self.trivia_builder.add_multi_line_comment(self.current.token.start, self.offset());
                return Kind::Skip;
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

    /// Section 12.7.1 Identifier Names

    /// TODO: Move all the identifier stuff into separate module to contain the unsafe.

    /// Handle identifier with ASCII start character.
    /// Start character should not be consumed from `self.current.chars` prior to calling this.
    /// SAFETY: Next char in `self.current.chars` must be ASCII.
    /// TODO: Can we get a gain by avoiding returning slice if it's not used (IDT handler)?
    unsafe fn identifier_name_handler(&mut self) -> &'a str {
        // `bytes` skip the character which caller guarantees is ASCII
        let bytes = self.remaining().get_unchecked(1..).bytes();
        let text = self.identifier_tail_after_no_escape(bytes);

        // Return identifier minus its first character
        // Caller guaranteed first char was ASCII.
        // Everything we've done since guarantees this is safe.
        // TODO: Write this comment better!
        text.get_unchecked(1..)
    }

    /// Handle identifier after 1st char dealt with.
    /// 1st char can have been ASCII or Unicode, but cannot have been a `\` escape.
    /// 1st character should not be consumed from `self.current.chars` prior to calling this,
    /// but `bytes` iterator should be positioned *after* 1st char.
    // `#[inline]` because we want this inlined into `identifier_name_handler`,
    // which is the fast path for common cases.
    #[inline]
    fn identifier_tail_after_no_escape(&mut self, mut bytes: Bytes<'a>) -> &'a str {
        // Find first byte which isn't valid ASCII identifier part
        let next_byte = match self.identifier_consume_ascii_identifier_bytes(&mut bytes) {
            Some(b) => b,
            None => {
                return self.identifier_eof();
            }
        };

        // Handle the byte which isn't ASCII identifier part.
        // Most likely we're at the end of the identifier, but handle `\` escape and Unicode chars.
        // Fast path for normal ASCII identifiers, by marking the 2 uncommon cases `#[cold]`.
        if next_byte == b'\\' {
            self.identifier_after_backslash(bytes, false)
        } else if !next_byte.is_ascii() {
            self.identifier_tail_after_unicode_byte(bytes)
        } else {
            // End of identifier found.
            // Advance chars iterator to the byte we just found which isn't part of the identifier.
            self.identifier_end(&bytes)
        }
    }

    /// Consume bytes from `Bytes` iterator which are ASCII identifier part bytes.
    /// `bytes` iterator is left positioned on next non-matching byte.
    /// Returns next non-matching byte, or `None` if EOF.
    // `#[inline]` because we want this inlined into `identifier_tail_after_no_escape`,
    // which is on the fast path for common cases.
    #[inline]
    fn identifier_consume_ascii_identifier_bytes(&mut self, bytes: &mut Bytes<'a>) -> Option<u8> {
        loop {
            match bytes.clone().next() {
                Some(b) => {
                    if !is_identifier_part_ascii_byte(b) {
                        return Some(b);
                    }
                    bytes.next();
                }
                None => {
                    return None;
                }
            }
        }
    }

    /// End of identifier found.
    /// `bytes` iterator must be positioned on next byte after end of identifier.
    // `#[inline]` because we want this inlined into `identifier_tail_after_no_escape`,
    // which is on the fast path for common cases.
    #[inline]
    fn identifier_end(&mut self, bytes: &Bytes) -> &'a str {
        // TODO: Could do this unchecked.
        // This fn would have to become unsafe, with proviso that `bytes` is on a UTF-8 boundary.
        // ```
        // let remaining = self.remaining();
        // let len = remaining.len() - bytes.len();
        // self.current.chars = remaining.get_unchecked(len..);
        // remaining.get_unchecked(..len)
        // ```
        // SAFETY: Only safe if `self.remaining().as_bytes()[self.remaining.len() - bytes.len()]`
        // is a UTF-8 character boundary, and within bounds of `self.remaining()`
        unsafe {
            let remaining = self.remaining();
            let len = remaining.len() - bytes.len();
            self.current.chars = remaining.get_unchecked(len..).chars();
            remaining.get_unchecked(..len)
        }
    }

    /// Identifier end at EOF.
    /// Return text of identifier, and advance `self.current.chars` to end of file.
    // This could be replaced with `identifier_end` in `identifier_tail_after_no_escape`
    // but doing that causes a 3% drop in lexer benchmarks, for some reason.
    fn identifier_eof(&mut self) -> &'a str {
        let text = self.remaining();
        self.current.chars = text[text.len()..].chars();
        text
    }

    /// Handle continuation of identifier after 1st byte of a multi-byte unicode char found.
    /// Any number of characters can have already been eaten from `bytes` iterator prior to it.
    /// `bytes` iterator should be positioned at start of Unicode character.
    /// Nothing should have been consumed from `self.current.chars` prior to calling this.
    // `#[cold]` to guide branch predictor that Unicode chars in identifiers are rare.
    #[cold]
    fn identifier_tail_after_unicode_byte(&mut self, mut bytes: Bytes<'a>) -> &'a str {
        let at_end = self.identifier_consume_unicode_char_if_identifier_part(&mut bytes);
        if !at_end {
            let at_end = self.identifier_tail_consume_until_end_or_escape(&mut bytes);
            if !at_end {
                return self.identifier_after_backslash(bytes, false);
            }
        }

        self.identifier_end(&bytes)
    }

    /// Consume valid identifier bytes (ASCII or Unicode) from `bytes`
    /// until reach end of identifier or a `\`.
    /// Returns `true` if at end of identifier, or `false` if found `\`.
    fn identifier_tail_consume_until_end_or_escape(&mut self, bytes: &mut Bytes<'a>) -> bool {
        loop {
            // Eat ASCII chars from `bytes`
            let next_byte = match self.identifier_consume_ascii_identifier_bytes(bytes) {
                Some(b) => b,
                None => {
                    return true;
                }
            };

            if next_byte.is_ascii() {
                return next_byte != b'\\';
            }

            // Unicode char
            let at_end = self.identifier_consume_unicode_char_if_identifier_part(bytes);
            if at_end {
                return true;
            }
            // Char was part of identifier. Keep eating.
        }
    }

    /// Consume unicode character from `bytes` if it's part of identifier.
    /// Returns `true` if at end of identifier (this character is not part of identifier)
    /// or `false` if character was consumed and potentially more of identifier still to come.
    fn identifier_consume_unicode_char_if_identifier_part(&self, bytes: &mut Bytes<'a>) -> bool {
        let mut chars = self.source[self.source.len() - bytes.len()..].chars();
        let c = chars.next().unwrap();
        if is_identifier_part_unicode(c) {
            // Advance `bytes` iterator past this character
            *bytes = chars.as_str().bytes();
            false
        } else {
            // Reached end of identifier
            true
        }
    }

    /// Handle identifier after a `\` found.
    /// Any number of characters can have been eaten from `bytes` iterator prior to the `\`.
    /// `\` byte must not have been eaten from `bytes`.
    /// Nothing should have been consumed from `self.current.chars` prior to calling this.
    // `check_identifier_start` should be `true` if this is 1st char in the identifier,
    // and `false` otherwise.
    // `#[cold]` to guide branch predictor that escapes in identifiers are rare and keep a fast path
    // in `identifier_tail_after_no_escape` for the common case.
    #[cold]
    fn identifier_after_backslash(
        &mut self,
        mut bytes: Bytes<'a>,
        mut check_identifier_start: bool,
    ) -> &'a str {
        // All the other identifier lexer functions only iterate through `bytes`,
        // leaving `self.current.chars` unchanged until the end of the identifier is found.
        // At this point, after finding an escape, we change approach.
        // In this function, the unescaped identifier is built up in an arena `String`.
        // Each time an escape is found, all the previous non-escaped bytes are pushed into the `String`
        // and `chars` iterator advanced to after the escape sequence.
        // We then search again for another run of unescaped bytes, and push them to the `String`
        // as a single chunk. If another escape is found, loop back and do same again.

        // Create an arena string to hold unescaped identifier.
        // We don't know how long identifier will end up being. Take a guess that total length
        // will be double what we've seen so far, or 16 minimum.
        const MIN_LEN: usize = 16;
        let len = self.remaining().len() - bytes.len();
        let capacity = (len * 2).max(MIN_LEN);
        let mut str = String::with_capacity_in(capacity, self.allocator);

        loop {
            // Add bytes before this escape to `str` and advance `chars` iterator to after the `\`
            let len = self.remaining().len() - bytes.len();
            str.push_str(&self.remaining()[0..len]);
            self.current.chars = self.remaining()[len + 1..].chars();

            // Consume escape sequence from `chars` and add char to `str`
            self.identifier_unicode_escape_sequence(&mut str, check_identifier_start);
            check_identifier_start = false;

            // Bring `bytes` iterator back into sync with `chars` iterator.
            // i.e. advance `bytes` to after the escape sequence.
            bytes = self.remaining().bytes();

            // Consume bytes until reach end of identifier or another escape
            let at_end = self.identifier_tail_consume_until_end_or_escape(&mut bytes);
            if at_end {
                break;
            }
            // Found another `\` escape
        }

        // Add bytes after last escape to `str`, and advance `chars` iterator to end of identifier
        let last_chunk = self.identifier_end(&bytes);
        str.push_str(last_chunk);

        // Convert to arena slice and save to `escaped_strings`
        let text = str.into_bump_str();
        self.save_string(true, text);
        text
    }

    /// Section 12.8 Punctuators
    fn read_dot(&mut self) -> Kind {
        if self.peek() == Some('.') && self.peek2() == Some('.') {
            self.current.chars.next();
            self.current.chars.next();
            return Kind::Dot3;
        }
        if self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.decimal_literal_after_decimal_point()
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

    fn private_identifier(&mut self) -> Kind {
        let mut bytes = self.remaining().bytes();
        if let Some(b) = bytes.clone().next() {
            if is_identifier_start_ascii_byte(b) {
                // Consume byte from `bytes`
                bytes.next();
                self.identifier_tail_after_no_escape(bytes);
                Kind::PrivateIdentifier
            } else {
                // Do not consume byte from `bytes`
                self.private_identifier_not_ascii_id(bytes)
            }
        } else {
            let start = self.offset();
            self.error(diagnostics::UnexpectedEnd(Span::new(start, start)));
            Kind::Undetermined
        }
    }

    #[cold]
    fn private_identifier_not_ascii_id(&mut self, bytes: Bytes<'a>) -> Kind {
        let b = bytes.clone().next().unwrap();
        if b == b'\\' {
            // Do not consume `\` byte from `bytes`
            self.identifier_after_backslash(bytes, true);
            return Kind::PrivateIdentifier;
        }

        if !b.is_ascii() {
            let mut chars = self.current.chars.clone();
            let c = chars.next().unwrap();
            if is_identifier_start_unicode(c) {
                // Char has been eaten from `bytes` (but not from `self.current.chars`)
                let bytes = chars.as_str().bytes();
                self.identifier_tail_after_no_escape(bytes);
                return Kind::PrivateIdentifier;
            }
        };

        let start = self.offset();
        let c = self.consume_char();
        self.error(diagnostics::InvalidCharacter(c, Span::new(start, self.offset())));
        Kind::Undetermined
    }

    /// 12.9.3 Numeric Literals with `0` prefix
    fn read_zero(&mut self) -> Kind {
        match self.peek() {
            Some('b' | 'B') => self.read_non_decimal(Kind::Binary),
            Some('o' | 'O') => self.read_non_decimal(Kind::Octal),
            Some('x' | 'X') => self.read_non_decimal(Kind::Hex),
            Some('e' | 'E') => {
                self.current.chars.next();
                self.read_decimal_exponent()
            }
            Some('.') => {
                self.current.chars.next();
                self.decimal_literal_after_decimal_point_after_digits()
            }
            Some('n') => {
                self.current.chars.next();
                self.check_after_numeric_literal(Kind::Decimal)
            }
            Some(n) if n.is_ascii_digit() => self.read_legacy_octal(),
            _ => self.check_after_numeric_literal(Kind::Decimal),
        }
    }

    fn read_non_decimal(&mut self, kind: Kind) -> Kind {
        self.current.chars.next();

        if self.peek().is_some_and(|c| kind.matches_number_char(c)) {
            self.current.chars.next();
        } else {
            self.unexpected_err();
            return Kind::Undetermined;
        }

        while let Some(c) = self.peek() {
            match c {
                '_' => {
                    self.current.chars.next();
                    if self.peek().is_some_and(|c| kind.matches_number_char(c)) {
                        self.current.chars.next();
                    } else {
                        self.unexpected_err();
                        return Kind::Undetermined;
                    }
                }
                c if kind.matches_number_char(c) => {
                    self.current.chars.next();
                }
                _ => break,
            }
        }
        if self.peek() == Some('n') {
            self.current.chars.next();
        }
        self.check_after_numeric_literal(kind)
    }

    fn read_legacy_octal(&mut self) -> Kind {
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
                self.decimal_literal_after_decimal_point_after_digits()
            }
            // allow 08e1 and 09e1
            Some('e') if kind == Kind::Decimal => {
                self.current.chars.next();
                self.read_decimal_exponent()
            }
            _ => self.check_after_numeric_literal(kind),
        }
    }

    fn decimal_literal_after_first_digit(&mut self) -> Kind {
        self.read_decimal_digits_after_first_digit();
        if self.next_eq('.') {
            return self.decimal_literal_after_decimal_point_after_digits();
        } else if self.next_eq('n') {
            return self.check_after_numeric_literal(Kind::Decimal);
        }

        let kind = self.optional_exponent().map_or(Kind::Decimal, |kind| kind);
        self.check_after_numeric_literal(kind)
    }

    fn read_decimal_exponent(&mut self) -> Kind {
        let kind = match self.peek() {
            Some('-') => {
                self.current.chars.next();
                Kind::NegativeExponential
            }
            Some('+') => {
                self.current.chars.next();
                Kind::PositiveExponential
            }
            _ => Kind::PositiveExponential,
        };
        self.read_decimal_digits();
        kind
    }

    fn read_decimal_digits(&mut self) {
        if self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.current.chars.next();
        } else {
            self.unexpected_err();
            return;
        }

        self.read_decimal_digits_after_first_digit();
    }

    fn read_decimal_digits_after_first_digit(&mut self) {
        while let Some(c) = self.peek() {
            match c {
                '_' => {
                    self.current.chars.next();
                    if self.peek().is_some_and(|c| c.is_ascii_digit()) {
                        self.current.chars.next();
                    } else {
                        self.unexpected_err();
                        return;
                    }
                }
                '0'..='9' => {
                    self.current.chars.next();
                }
                _ => break,
            }
        }
    }

    fn decimal_literal_after_decimal_point(&mut self) -> Kind {
        self.read_decimal_digits();
        self.optional_exponent();
        self.check_after_numeric_literal(Kind::Float)
    }

    fn decimal_literal_after_decimal_point_after_digits(&mut self) -> Kind {
        self.optional_decimal_digits();
        self.optional_exponent();
        self.check_after_numeric_literal(Kind::Float)
    }

    fn optional_decimal_digits(&mut self) {
        if self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.current.chars.next();
        } else {
            return;
        }
        self.read_decimal_digits_after_first_digit();
    }

    fn optional_exponent(&mut self) -> Option<Kind> {
        if matches!(self.peek(), Some('e' | 'E')) {
            self.current.chars.next();
            return Some(self.read_decimal_exponent());
        }
        None
    }

    fn check_after_numeric_literal(&mut self, kind: Kind) -> Kind {
        let offset = self.offset();
        // The SourceCharacter immediately following a NumericLiteral must not be an IdentifierStart or DecimalDigit.
        let c = self.peek();
        if c.is_none() || c.is_some_and(|ch| !ch.is_ascii_digit() && !is_identifier_start(ch)) {
            return kind;
        }
        self.current.chars.next();
        while let Some(c) = self.peek() {
            if is_identifier_start(c) {
                self.current.chars.next();
            } else {
                break;
            }
        }
        self.error(diagnostics::InvalidNumberEnd(Span::new(offset, self.offset())));
        Kind::Undetermined
    }

    /// 12.9.4 String Literals
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
                        self.save_string(builder.has_escape(), builder.finish_without_push(self));
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
                Some(c) => {
                    builder.push_matching(c);
                }
            }
        }
    }

    /// 12.9.5 Regular Expression Literals
    fn read_regex(&mut self) -> (u32, RegExpFlags) {
        let mut in_escape = false;
        let mut in_character_class = false;
        loop {
            match self.current.chars.next() {
                None => {
                    self.error(diagnostics::UnterminatedRegExp(self.unterminated_range()));
                    return (self.offset(), RegExpFlags::empty());
                }
                Some(c) if is_line_terminator(c) => {
                    self.error(diagnostics::UnterminatedRegExp(self.unterminated_range()));
                    #[allow(clippy::cast_possible_truncation)]
                    let pattern_end = self.offset() - c.len_utf8() as u32;
                    return (pattern_end, RegExpFlags::empty());
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

        let pattern_end = self.offset() - 1; // -1 to exclude `/`
        let mut flags = RegExpFlags::empty();

        while let Some(ch @ ('$' | '_' | 'a'..='z' | 'A'..='Z' | '0'..='9')) = self.peek() {
            self.current.chars.next();
            let flag = if let Ok(flag) = RegExpFlags::try_from(ch) {
                flag
            } else {
                self.error(diagnostics::RegExpFlag(ch, self.current_offset()));
                continue;
            };
            if flags.contains(flag) {
                self.error(diagnostics::RegExpFlagTwice(ch, self.current_offset()));
                continue;
            }
            flags |= flag;
        }

        (pattern_end, flags)
    }

    /// 12.8.6 Template Literal Lexical Components
    fn read_template_literal(&mut self, substitute: Kind, tail: Kind) -> Kind {
        let mut builder = AutoCow::new(self);
        let mut is_valid_escape_sequence = true;
        while let Some(c) = self.current.chars.next() {
            match c {
                '$' if self.peek() == Some('{') => {
                    self.save_template_string(
                        is_valid_escape_sequence,
                        builder.has_escape(),
                        builder.finish_without_push(self),
                    );
                    self.current.chars.next();
                    return substitute;
                }
                '`' => {
                    self.save_template_string(
                        is_valid_escape_sequence,
                        builder.has_escape(),
                        builder.finish_without_push(self),
                    );
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
    fn read_jsx_identifier(&mut self, _start_offset: u32) -> Kind {
        while let Some(c) = self.peek() {
            if c == '-' || is_identifier_start(c) {
                self.current.chars.next();
                while let Some(c) = self.peek() {
                    if is_identifier_part(c) {
                        self.current.chars.next();
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        }
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
            Some(_) => {
                loop {
                    // The tokens `{`, `<`, `>` and `}` cannot appear in a jsx text.
                    // The TypeScript compiler raises the error "Unexpected token. Did you mean `{'>'}` or `&gt;`?".
                    // Where as the Babel compiler does not raise any errors.
                    // The following check omits `>` and `}` so that more Babel tests can be passed.
                    if self.peek().is_some_and(|c| c == '{' || c == '<') {
                        break;
                    }
                    if self.current.chars.next().is_none() {
                        break;
                    }
                }
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
                        self.save_string(builder.has_escape(), builder.finish_without_push(self));
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
        str: &mut String,
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

        let is_valid =
            if check_identifier_start { is_identifier_start(ch) } else { is_identifier_part(ch) };

        if !is_valid {
            self.error(diagnostics::InvalidCharacter(ch, self.current_offset()));
            return;
        }

        str.push(ch);
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
                // \ LineTerminatorSequence
                // LineTerminatorSequence ::
                // <LF>
                // <CR> [lookahead ≠ <LF>]
                // <LS>
                // <PS>
                // <CR> <LF>
                LF | LS | PS => {}
                CR => {
                    self.next_eq(LF);
                }
                // SingleEscapeCharacter :: one of
                //   ' " \ b f n r t v
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
                // Section 12.9.4 String Literals
                // LegacyOctalEscapeSequence
                // NonOctalDecimalEscapeSequence
                a @ '0'..='7' if !in_template => {
                    let mut num = String::new_in(self.allocator);
                    num.push(a);
                    match a {
                        '4'..='7' => {
                            if matches!(self.peek(), Some('0'..='7')) {
                                let b = self.consume_char();
                                num.push(b);
                            }
                        }
                        '0'..='3' => {
                            if matches!(self.peek(), Some('0'..='7')) {
                                let b = self.consume_char();
                                num.push(b);
                                if matches!(self.peek(), Some('0'..='7')) {
                                    let c = self.consume_char();
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

#[allow(clippy::unnecessary_safety_comment)]
/// Handle next byte of source.
/// SAFETY:
/// * Lexer must not be at end of file.
/// * `byte` must be next byte of source code, corresponding to current position
///   of `lexer.current.chars`.
/// * Only `BYTE_HANDLERS` for ASCII characters may use the `ascii_byte_handler!()` macro.
unsafe fn handle_byte(byte: u8, lexer: &mut Lexer) -> Kind {
    BYTE_HANDLERS[byte as usize](lexer)
}

type ByteHandler = fn(&mut Lexer<'_>) -> Kind;

/// Lookup table mapping any incoming byte to a handler function defined below.
/// <https://github.com/ratel-rust/ratel-core/blob/master/ratel/src/lexer/mod.rs>
#[rustfmt::skip]
static BYTE_HANDLERS: [ByteHandler; 256] = [
//  0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F    //
    ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, SPS, LIN, SPS, SPS, LIN, ERR, ERR, // 0
    ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, // 1
    SPS, EXL, QOT, HAS, IDT, PRC, AMP, QOT, PNO, PNC, ATR, PLS, COM, MIN, PRD, SLH, // 2
    ZER, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, COL, SEM, LSS, EQL, GTR, QST, // 3
    AT_, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, // 4
    IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, BTO, ESC, BTC, CRT, IDT, // 5
    TPL, L_A, L_B, L_C, L_D, L_E, L_F, L_G, IDT, L_I, IDT, L_K, L_L, L_M, L_N, L_O, // 6
    L_P, IDT, L_R, L_S, L_T, L_U, L_V, L_W, IDT, L_Y, IDT, BEO, PIP, BEC, TLD, ERR, // 7
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 8
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 9
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // A
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // B
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // C
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // D
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // E
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // F
];

#[allow(clippy::unnecessary_safety_comment)]
/// Macro for defining byte handler for an ASCII character.
///
/// In addition to defining a `const` for the handler, it also asserts that lexer
/// is not at end of file, and that next char is ASCII.
/// Where the handler is for an ASCII character, these assertions are self-evidently true.
///
/// These assertions produce no runtime code, but hint to the compiler that it can assume that
/// next char is ASCII, and it uses that information to optimize the rest of the handler.
/// e.g. `lexer.current.chars.next()` becomes just a single assembler instruction.
/// Without the assertions, the compiler is unable to deduce the next char is ASCII, due to
/// the indirection of the `BYTE_HANDLERS` jump table.
///
/// These assertions are unchecked (i.e. won't panic) and will cause UB if they're incorrect.
///
/// SAFETY: Only use this macro to define byte handlers for ASCII characters.
///
/// ```
/// ascii_byte_handler!(SPS(lexer) {
///   lexer.consume_char();
///   Kind::WhiteSpace
/// });
/// ```
///
/// expands to:
///
/// ```
/// const SPS: ByteHandler = |lexer| {
///   unsafe {
///     use ::assert_unchecked::assert_unchecked;
///     let s = lexer.current.chars.as_str();
///     assert_unchecked!(!s.is_empty());
///     assert_unchecked!(s.as_bytes()[0] < 128);
///   }
///   lexer.consume_char();
///   Kind::WhiteSpace
/// };
/// ```
macro_rules! ascii_byte_handler {
    ($id:ident($lex:ident) $body:expr) => {
        const $id: ByteHandler = |$lex| {
            // SAFETY: This macro is only used for ASCII characters
            unsafe {
                use assert_unchecked::assert_unchecked;
                let s = $lex.current.chars.as_str();
                assert_unchecked!(!s.is_empty());
                assert_unchecked!(s.as_bytes()[0] < 128);
            }
            $body
        };
    };
}

// TODO: Write comment explaining this macro
macro_rules! ascii_identifier_handler {
    ($id:ident($lex:ident, $id_handler:ident) $body:expr) => {
        const $id: ByteHandler = |$lex| {
            fn $id_handler<'a>(lexer: &mut Lexer<'a>) -> &'a str {
                // SAFETY: This macro is only used for ASCII characters
                unsafe { lexer.identifier_name_handler() }
            }
            // SAFETY: This macro is only used for ASCII characters
            unsafe {
                use assert_unchecked::assert_unchecked;
                let s = $lex.current.chars.as_str();
                assert_unchecked!(!s.is_empty());
                assert_unchecked!(s.as_bytes()[0] < 128);
            }
            $body
        };
    };
}

// `\0` `\1` etc
ascii_byte_handler!(ERR(lexer) {
    let c = lexer.consume_char();
    lexer.error(diagnostics::InvalidCharacter(c, lexer.unterminated_range()));
    Kind::Undetermined
});

// <SPACE> <TAB> <VT> <FF>
ascii_byte_handler!(SPS(lexer) {
    lexer.consume_char();
    Kind::Skip
});

// '\r' '\n'
ascii_byte_handler!(LIN(lexer) {
    lexer.consume_char();
    lexer.current.token.is_on_new_line = true;
    Kind::Skip
});

// !
ascii_byte_handler!(EXL(lexer) {
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
});

// ' "
ascii_byte_handler!(QOT(lexer) {
    let c = lexer.consume_char();
    if lexer.context == LexerContext::JsxAttributeValue {
        lexer.read_jsx_string_literal(c)
    } else {
        lexer.read_string_literal(c)
    }
});

// #
ascii_byte_handler!(HAS(lexer) {
    lexer.consume_char();
    // HashbangComment ::
    //     `#!` SingleLineCommentChars?
    if lexer.current.token.start == 0 && lexer.next_eq('!') {
        lexer.read_hashbang_comment()
    } else {
        lexer.private_identifier()
    }
});

// `A..=Z`, `a..=z` (except special cases below), `_`, `$`
ascii_identifier_handler!(IDT(lexer, id_handler) {
    id_handler(lexer);
    Kind::Ident
});

// %
ascii_byte_handler!(PRC(lexer) {
    lexer.consume_char();
    if lexer.next_eq('=') {
        Kind::PercentEq
    } else {
        Kind::Percent
    }
});

// &
ascii_byte_handler!(AMP(lexer) {
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
});

// (
ascii_byte_handler!(PNO(lexer) {
    lexer.consume_char();
    Kind::LParen
});

// )
ascii_byte_handler!(PNC(lexer) {
    lexer.consume_char();
    Kind::RParen
});

// *
ascii_byte_handler!(ATR(lexer) {
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
});

// +
ascii_byte_handler!(PLS(lexer) {
    lexer.consume_char();
    if lexer.next_eq('+') {
        Kind::Plus2
    } else if lexer.next_eq('=') {
        Kind::PlusEq
    } else {
        Kind::Plus
    }
});

// ,
ascii_byte_handler!(COM(lexer) {
    lexer.consume_char();
    Kind::Comma
});

// -
ascii_byte_handler!(MIN(lexer) {
    lexer.consume_char();
    lexer.read_minus().unwrap_or_else(|| lexer.skip_single_line_comment())
});

// .
ascii_byte_handler!(PRD(lexer) {
    lexer.consume_char();
    lexer.read_dot()
});

// /
ascii_byte_handler!(SLH(lexer) {
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
});

// 0
ascii_byte_handler!(ZER(lexer) {
    lexer.consume_char();
    lexer.read_zero()
});

// 1 to 9
ascii_byte_handler!(DIG(lexer) {
    lexer.consume_char();
    lexer.decimal_literal_after_first_digit()
});

// :
ascii_byte_handler!(COL(lexer) {
    lexer.consume_char();
    Kind::Colon
});

// ;
ascii_byte_handler!(SEM(lexer) {
    lexer.consume_char();
    Kind::Semicolon
});

// <
ascii_byte_handler!(LSS(lexer) {
    lexer.consume_char();
    lexer.read_left_angle().unwrap_or_else(|| lexer.skip_single_line_comment())
});

// =
ascii_byte_handler!(EQL(lexer) {
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
});

// >
ascii_byte_handler!(GTR(lexer) {
    lexer.consume_char();
    // `>=` is re-lexed with [Lexer::next_jsx_child]
    Kind::RAngle
});

// ?
ascii_byte_handler!(QST(lexer) {
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
});

// @
ascii_byte_handler!(AT_(lexer) {
    lexer.consume_char();
    Kind::At
});

// [
ascii_byte_handler!(BTO(lexer) {
    lexer.consume_char();
    Kind::LBrack
});

// \
ascii_byte_handler!(ESC(lexer) {
    // `bytes` iterator positioned on `\`
    let bytes = lexer.remaining().bytes();
    let text = lexer.identifier_after_backslash(bytes, true);
    Kind::match_keyword(text)
});

// ]
ascii_byte_handler!(BTC(lexer) {
    lexer.consume_char();
    Kind::RBrack
});

// ^
ascii_byte_handler!(CRT(lexer) {
    lexer.consume_char();
    if lexer.next_eq('=') {
        Kind::CaretEq
    } else {
        Kind::Caret
    }
});

// `
ascii_byte_handler!(TPL(lexer) {
    lexer.consume_char();
    lexer.read_template_literal(Kind::TemplateHead, Kind::NoSubstitutionTemplate)
});

// {
ascii_byte_handler!(BEO(lexer) {
    lexer.consume_char();
    Kind::LCurly
});

// |
ascii_byte_handler!(PIP(lexer) {
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
});

// }
ascii_byte_handler!(BEC(lexer) {
    lexer.consume_char();
    Kind::RCurly
});

// ~
ascii_byte_handler!(TLD(lexer) {
    lexer.consume_char();
    Kind::Tilde
});

ascii_identifier_handler!(L_A(lexer, id_handler) match id_handler(lexer) {
    "wait" => Kind::Await,
    "sync" => Kind::Async,
    "bstract" => Kind::Abstract,
    "ccessor" => Kind::Accessor,
    "ny" => Kind::Any,
    "s" => Kind::As,
    "ssert" => Kind::Assert,
    "sserts" => Kind::Asserts,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_B(lexer, id_handler) match id_handler(lexer) {
    "reak" => Kind::Break,
    "oolean" => Kind::Boolean,
    "igint" => Kind::BigInt,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_C(lexer, id_handler) match id_handler(lexer) {
    "onst" => Kind::Const,
    "lass" => Kind::Class,
    "ontinue" => Kind::Continue,
    "atch" => Kind::Catch,
    "ase" => Kind::Case,
    "onstructor" => Kind::Constructor,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_D(lexer, id_handler) match id_handler(lexer) {
    "o" => Kind::Do,
    "elete" => Kind::Delete,
    "eclare" => Kind::Declare,
    "efault" => Kind::Default,
    "ebugger" => Kind::Debugger,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_E(lexer, id_handler) match id_handler(lexer) {
    "lse" => Kind::Else,
    "num" => Kind::Enum,
    "xport" => Kind::Export,
    "xtends" => Kind::Extends,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_F(lexer, id_handler) match id_handler(lexer) {
    "unction" => Kind::Function,
    "alse" => Kind::False,
    "or" => Kind::For,
    "inally" => Kind::Finally,
    "rom" => Kind::From,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_G(lexer, id_handler) match id_handler(lexer) {
    "et" => Kind::Get,
    "lobal" => Kind::Global,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_I(lexer, id_handler) match id_handler(lexer) {
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
});

ascii_identifier_handler!(L_K(lexer, id_handler) match id_handler(lexer) {
    "eyof" => Kind::KeyOf,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_L(lexer, id_handler) match id_handler(lexer) {
    "et" => Kind::Let,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_M(lexer, id_handler) match id_handler(lexer) {
    "eta" => Kind::Meta,
    "odule" => Kind::Module,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_N(lexer, id_handler) match id_handler(lexer) {
    "ull" => Kind::Null,
    "ew" => Kind::New,
    "umber" => Kind::Number,
    "amespace" => Kind::Namespace,
    "ever" => Kind::Never,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_O(lexer, id_handler) match id_handler(lexer) {
    "f" => Kind::Of,
    "bject" => Kind::Object,
    "ut" => Kind::Out,
    "verride" => Kind::Override,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_P(lexer, id_handler) match id_handler(lexer) {
    "ackage" => Kind::Package,
    "rivate" => Kind::Private,
    "rotected" => Kind::Protected,
    "ublic" => Kind::Public,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_R(lexer, id_handler) match id_handler(lexer) {
    "eturn" => Kind::Return,
    "equire" => Kind::Require,
    "eadonly" => Kind::Readonly,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_S(lexer, id_handler) match id_handler(lexer) {
    "et" => Kind::Set,
    "uper" => Kind::Super,
    "witch" => Kind::Switch,
    "tatic" => Kind::Static,
    "ymbol" => Kind::Symbol,
    "tring" => Kind::String,
    "atisfies" => Kind::Satisfies,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_T(lexer, id_handler) match id_handler(lexer) {
    "his" => Kind::This,
    "rue" => Kind::True,
    "hrow" => Kind::Throw,
    "ry" => Kind::Try,
    "ypeof" => Kind::Typeof,
    "arget" => Kind::Target,
    "ype" => Kind::Type,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_U(lexer, id_handler) match id_handler(lexer) {
    "ndefined" => Kind::Undefined,
    "sing" => Kind::Using,
    "nique" => Kind::Unique,
    "nknown" => Kind::Unknown,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_V(lexer, id_handler) match id_handler(lexer) {
    "ar" => Kind::Var,
    "oid" => Kind::Void,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_W(lexer, id_handler) match id_handler(lexer) {
    "hile" => Kind::While,
    "ith" => Kind::With,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_Y(lexer, id_handler) match id_handler(lexer) {
    "ield" => Kind::Yield,
    _ => Kind::Ident,
});

// Non-ASCII characters.
// NB: Must not use `ascii_byte_handler!()` macro, as this handler is for non-ASCII chars.
#[allow(clippy::redundant_closure_for_method_calls)]
const UNI: ByteHandler = |lexer| lexer.unicode_char_handler();
