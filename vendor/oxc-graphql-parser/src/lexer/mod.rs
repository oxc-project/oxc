mod cursor;
mod lookup;
mod token;
mod token_kind;

use crate::Error;
use crate::LimitTracker;
use crate::lexer::cursor::Cursor;
use crate::lexer::lookup::ByteClass;
use std::hint::cold_path;
pub use token::Token;
pub use token_kind::TokenKind;

/// Parses GraphQL source text into tokens.
/// ```rust
/// use oxc_graphql_parser::Lexer;
///
/// let query = "
/// {
///     animal
///     ...snackSelection
///     ... on Pet {
///       playmates {
///         count
///       }
///     }
/// }
/// ";
/// let (tokens, errors) = Lexer::new(query).lex();
/// assert_eq!(errors.len(), 0);
/// ```
#[derive(Debug)]
pub struct Lexer<'a> {
    finished: bool,
    cursor: Cursor<'a>,
    pub(crate) limit_tracker: LimitTracker,
}

/// States of the number token state machine.
enum NumberState {
    MinusSign,
    LeadingZero,
    IntegerPart,
    DecimalPoint,
    FractionalPart,
    ExponentIndicator,
    ExponentSign,
    ExponentDigit,
}

impl<'a> Lexer<'a> {
    /// Create a lexer for a GraphQL source text.
    ///
    /// The Lexer is an iterator over tokens and errors:
    /// ```rust
    /// use oxc_graphql_parser::Lexer;
    ///
    /// let query = "# --- GraphQL here ---";
    ///
    /// let mut lexer = Lexer::new(query);
    /// let mut tokens = vec![];
    /// for token in lexer {
    ///     match token {
    ///         Ok(token) => tokens.push(token),
    ///         Err(error) => panic!("{:?}", error),
    ///     }
    /// }
    /// ```
    pub fn new(input: &'a str) -> Self {
        Self {
            cursor: Cursor::new(input),
            finished: false,
            limit_tracker: LimitTracker::new(usize::MAX),
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit_tracker = LimitTracker::new(limit);
        self
    }

    /// Lex the full source text, consuming the lexer.
    pub fn lex(self) -> (Vec<Token<'a>>, Vec<Error>) {
        let mut tokens = vec![];
        let mut errors = vec![];

        for item in self {
            match item {
                Ok(token) => tokens.push(token),
                Err(error) => errors.push(error),
            }
        }

        (tokens, errors)
    }

    /// Returns the next token, skipping whitespace and comma trivia without
    /// materializing tokens for them. Comments are returned so the caller can
    /// record their spans.
    ///
    /// Each skipped trivia token still counts toward the token limit, exactly
    /// as if it had been yielded by the iterator.
    pub(crate) fn next_significant(&mut self) -> Option<Result<Token<'a>, Error>> {
        if self.finished {
            return None;
        }

        loop {
            if self.limit_tracker.check_and_increment() {
                self.finished = true;
                return Some(Err(Error::limit(
                    "token limit reached, aborting lexing",
                    self.cursor.index(),
                )));
            }

            if self.cursor.skip_trivia() {
                continue;
            }

            return match self.cursor.advance() {
                Ok(token) => {
                    if matches!(token.kind(), TokenKind::Eof) {
                        self.finished = true;
                    }

                    Some(Ok(token))
                }
                Err(err) => Some(Err(err)),
            };
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        if self.limit_tracker.check_and_increment() {
            self.finished = true;
            return Some(Err(Error::limit(
                "token limit reached, aborting lexing",
                self.cursor.index(),
            )));
        }

        match self.cursor.advance() {
            Ok(token) => {
                if matches!(token.kind(), TokenKind::Eof) {
                    self.finished = true;
                }

                Some(Ok(token))
            }
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'a> Cursor<'a> {
    fn advance(&mut self) -> Result<Token<'a>, Error> {
        // A pending error is only ever set and consumed within `lex_string`;
        // every other token starts with a clean slate.
        debug_assert!(self.err.is_none());

        let mut token = Token { kind: TokenKind::Eof, data: "", index: self.index() };

        let Some(c) = self.bump() else {
            cold_path();
            // Report EOF at the end of the input rather than one byte past it.
            let end = self.source.len();
            self.offset = end;
            token.index = end;
            return Ok(token);
        };

        match lookup::byte_class(c) {
            ByteClass::Bang => self.punctuation(token, TokenKind::Bang),
            ByteClass::Dollar => self.punctuation(token, TokenKind::Dollar),
            ByteClass::Amp => self.punctuation(token, TokenKind::Amp),
            ByteClass::LParen => self.punctuation(token, TokenKind::LParen),
            ByteClass::RParen => self.punctuation(token, TokenKind::RParen),
            ByteClass::Comma => self.punctuation(token, TokenKind::Comma),
            ByteClass::Colon => self.punctuation(token, TokenKind::Colon),
            ByteClass::Eq => self.punctuation(token, TokenKind::Eq),
            ByteClass::At => self.punctuation(token, TokenKind::At),
            ByteClass::LBracket => self.punctuation(token, TokenKind::LBracket),
            ByteClass::RBracket => self.punctuation(token, TokenKind::RBracket),
            ByteClass::LCurly => self.punctuation(token, TokenKind::LCurly),
            ByteClass::RCurly => self.punctuation(token, TokenKind::RCurly),
            ByteClass::Pipe => self.punctuation(token, TokenKind::Pipe),
            ByteClass::Name => {
                token.kind = TokenKind::Name;
                token.data = self.consume_name();
                Ok(token)
            }
            ByteClass::Whitespace => {
                token.kind = TokenKind::Whitespace;
                token.data = self.consume_whitespace();
                Ok(token)
            }
            ByteClass::Bom => {
                if self.eat_bom() {
                    token.kind = TokenKind::Whitespace;
                    token.data = self.consume_whitespace();
                    Ok(token)
                } else {
                    self.unexpected_character(c, &token)
                }
            }
            ByteClass::Quote => self.lex_string_start(token),
            ByteClass::Hash => self.lex_comment(token),
            ByteClass::Dot => self.lex_spread(token),
            ByteClass::Zero => self.lex_number(NumberState::LeadingZero, token),
            ByteClass::Digit => self.lex_number(NumberState::IntegerPart, token),
            ByteClass::Minus => self.lex_number(NumberState::MinusSign, token),
            ByteClass::Other => {
                cold_path();
                self.unexpected_character(c, &token)
            }
        }
    }

    /// Skips one trivia token (whitespace run or comma) without materializing
    /// it. Returns `false` when the next token is significant. Comments are
    /// not skipped: callers record their spans, so they lex as normal tokens.
    fn skip_trivia(&mut self) -> bool {
        let Some(&c) = self.bytes.get(self.next) else {
            return false;
        };
        match lookup::byte_class(c) {
            ByteClass::Whitespace => {
                self.bump();
                self.consume_whitespace();
                true
            }
            ByteClass::Comma => {
                self.bump();
                // Update the cursor position exactly like lexing the token would.
                let _ = self.current_str();
                true
            }
            ByteClass::Bom if self.at_bom() => {
                self.bump();
                self.eat_bom();
                self.consume_whitespace();
                true
            }
            _ => false,
        }
    }

    #[inline]
    fn punctuation(&mut self, mut token: Token<'a>, kind: TokenKind) -> Result<Token<'a>, Error> {
        token.kind = kind;
        token.data = self.current_str();
        Ok(token)
    }

    fn lex_comment(&mut self, mut token: Token<'a>) -> Result<Token<'a>, Error> {
        token.kind = TokenKind::Comment;
        let start = self.index;
        let end = self.seek_line_end();
        token.data = &self.source[start..end];
        Ok(token)
    }

    fn lex_spread(&mut self, mut token: Token<'a>) -> Result<Token<'a>, Error> {
        token.kind = TokenKind::Spread;
        if let Some(c) = self.bump() {
            if c == b'.' {
                if self.eatc(b'.') {
                    token.data = self.current_str();
                    return Ok(token);
                }
            } else if !c.is_ascii() {
                // Consume the whole character so the error data slices at a
                // character boundary.
                self.consume_current_char();
            }
        }
        let data = self.current_str();
        Err(Error::with_loc("Unterminated spread operator", data.to_string(), token.index))
    }

    fn lex_string_start(&mut self, mut token: Token<'a>) -> Result<Token<'a>, Error> {
        token.kind = TokenKind::StringValue;

        if self.eatc(b'"') {
            if self.eatc(b'"') {
                return self.lex_block_string(token);
            }

            // Empty string: `""`.
            token.data = self.current_str();
            return Ok(token);
        }

        if self.next == self.bytes.len() {
            // A lone `"` at the end of the input.
            return Err(Error::with_loc(
                "unexpected end of data while lexing string value",
                self.current_str().to_string(),
                token.index,
            ));
        }

        self.lex_string(token)
    }

    fn lex_string(&mut self, mut token: Token<'a>) -> Result<Token<'a>, Error> {
        loop {
            let Some(found) = memchr::memchr2(b'"', b'\\', &self.bytes[self.next..]) else {
                cold_path();
                return self.unterminated_string(&token);
            };
            let stop = self.next + found;

            if memchr::memchr2(b'\n', b'\r', &self.bytes[self.next..stop]).is_some() {
                cold_path();
                self.add_err(Error::with_loc("unexpected line terminator", String::new(), 0));
            }

            // Consume through the stop byte.
            self.offset = stop;
            self.next = stop + 1;

            if self.bytes[stop] == b'"' {
                token.data = self.current_str();
                return self.done(token);
            }

            // Backslash escape sequence.
            let Some(c) = self.bump() else {
                cold_path();
                return self.unterminated_string(&token);
            };
            if c == b'u' {
                // `\uXXXX`: four hex digits. A non-hex byte is consumed as
                // plain string content after recording an error.
                for remaining in (1..=4usize).rev() {
                    let Some(c) = self.bump() else {
                        cold_path();
                        return self.unterminated_string(&token);
                    };
                    if c == b'"' {
                        self.add_err(Error::with_loc(
                            "incomplete unicode escape sequence",
                            char::from(c).to_string(),
                            token.index,
                        ));
                        token.data = self.current_str();
                        return self.done(token);
                    }
                    if !c.is_ascii_hexdigit() {
                        self.add_err(Error::with_loc(
                            "invalid unicode escape sequence",
                            c.to_string(),
                            0,
                        ));
                        break;
                    }
                    if remaining == 1 {
                        let hex_end = self.offset + 1;
                        let hex_start = hex_end - 4;
                        let hex = &self.source[hex_start..hex_end];
                        // `is_ascii_hexdigit()` checks in previous iterations ensures
                        // this `unwrap()` does not panic:
                        let code_point = u32::from_str_radix(hex, 16).unwrap();
                        if char::from_u32(code_point).is_none() {
                            // TODO: https://github.com/oxc-project/oxc-graphql-parser/issues/657 needs
                            // changes both here and in `ast/node_ext.rs`
                            let escape_sequence_start = hex_start - 2; // include "\u"
                            let escape_sequence = &self.source[escape_sequence_start..hex_end];
                            self.add_err(Error::with_loc(
                                "surrogate code point is invalid in unicode escape sequence \
                                 (paired surrogate not supported yet: \
                                 https://github.com/oxc-project/oxc-graphql-parser/issues/657)",
                                escape_sequence.to_owned(),
                                0,
                            ));
                        }
                    }
                }
            } else if !is_escaped_char(c) {
                cold_path();
                let c = self.char_for_error(c);
                self.add_err(Error::with_loc("unexpected escaped character", c.to_string(), 0));
            }
        }
    }

    fn lex_block_string(&mut self, mut token: Token<'a>) -> Result<Token<'a>, Error> {
        loop {
            let Some(found) = memchr::memchr2(b'"', b'\\', &self.bytes[self.next..]) else {
                cold_path();
                return self.unterminated_string(&token);
            };
            let stop = self.next + found;

            // Consume through the stop byte.
            self.offset = stop;
            self.next = stop + 1;

            if self.bytes[stop] == b'"' {
                // Require two additional quotes to complete the triple quote;
                // a lone second quote is consumed as content.
                if self.eatc(b'"') && self.eatc(b'"') {
                    token.data = self.current_str();
                    return self.done(token);
                }
                continue;
            }

            // Backslash. If this is \""", we need to eat 3 in total, and then
            // continue. The lexer does not un-escape escape sequences so it's
            // OK if we take this path for \"", even if that is technically not
            // an escape sequence. It's also legal to write \\\""" with two
            // literal backslashes and then the escape sequence.
            loop {
                let Some(c) = self.bump() else {
                    cold_path();
                    return self.unterminated_string(&token);
                };
                match c {
                    b'\\' => {}
                    b'"' => {
                        if self.eatc(b'"') {
                            self.eatc(b'"');
                        }
                        break;
                    }
                    _ => break,
                }
            }
        }
    }

    fn lex_number(
        &mut self,
        mut state: NumberState,
        mut token: Token<'a>,
    ) -> Result<Token<'a>, Error> {
        token.kind = TokenKind::Int;

        loop {
            let Some(c) = self.bump() else {
                return match state {
                    NumberState::MinusSign => Err(Error::with_loc(
                        "Unexpected character \"-\"",
                        self.current_str().to_string(),
                        token.index,
                    )),
                    NumberState::DecimalPoint
                    | NumberState::ExponentIndicator
                    | NumberState::ExponentSign => Err(Error::with_loc(
                        "Unexpected EOF in float value",
                        self.current_str().to_string(),
                        token.index,
                    )),
                    NumberState::LeadingZero
                    | NumberState::IntegerPart
                    | NumberState::FractionalPart
                    | NumberState::ExponentDigit => {
                        token.data = self.current_str();
                        Ok(token)
                    }
                };
            };

            match state {
                NumberState::MinusSign => match c {
                    b'0' => {
                        state = NumberState::LeadingZero;
                    }
                    curr if curr.is_ascii_digit() => {
                        state = NumberState::IntegerPart;
                    }
                    _ => {
                        let c = self.char_for_error(c);
                        return Err(Error::with_loc(
                            format!("Unexpected character `{c}`"),
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                },
                NumberState::LeadingZero => match c {
                    b'.' => {
                        token.kind = TokenKind::Float;
                        state = NumberState::DecimalPoint;
                    }
                    b'e' | b'E' => {
                        token.kind = TokenKind::Float;
                        state = NumberState::ExponentIndicator;
                    }
                    _ if c.is_ascii_digit() => {
                        return Err(Error::with_loc(
                            "Numbers must not have non-significant leading zeroes",
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                    _ if lookup::is_namestart(c) => {
                        let c = char::from(c);
                        return Err(Error::with_loc(
                            format!("Unexpected character `{c}` as integer suffix"),
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                    _ => {
                        token.data = self.prev_str();
                        return Ok(token);
                    }
                },
                NumberState::IntegerPart => match c {
                    curr if curr.is_ascii_digit() => {}
                    b'.' => {
                        token.kind = TokenKind::Float;
                        state = NumberState::DecimalPoint;
                    }
                    b'e' | b'E' => {
                        token.kind = TokenKind::Float;
                        state = NumberState::ExponentIndicator;
                    }
                    _ if lookup::is_namestart(c) => {
                        let c = char::from(c);
                        return Err(Error::with_loc(
                            format!("Unexpected character `{c}` as integer suffix"),
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                    _ => {
                        token.data = self.prev_str();
                        return Ok(token);
                    }
                },
                NumberState::DecimalPoint => match c {
                    curr if curr.is_ascii_digit() => {
                        state = NumberState::FractionalPart;
                    }
                    _ => {
                        let c = self.char_for_error(c);
                        return Err(Error::with_loc(
                            format!("Unexpected character `{c}`, expected fractional digit"),
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                },
                NumberState::FractionalPart => match c {
                    curr if curr.is_ascii_digit() => {}
                    b'e' | b'E' => {
                        state = NumberState::ExponentIndicator;
                    }
                    _ if c == b'.' || lookup::is_namestart(c) => {
                        let c = char::from(c);
                        return Err(Error::with_loc(
                            format!("Unexpected character `{c}` as float suffix"),
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                    _ => {
                        token.data = self.prev_str();
                        return Ok(token);
                    }
                },
                NumberState::ExponentIndicator => match c {
                    _ if c.is_ascii_digit() => {
                        state = NumberState::ExponentDigit;
                    }
                    b'+' | b'-' => {
                        state = NumberState::ExponentSign;
                    }
                    _ => {
                        let c = self.char_for_error(c);
                        return Err(Error::with_loc(
                            format!("Unexpected character `{c}`, expected exponent digit or sign"),
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                },
                NumberState::ExponentSign => match c {
                    _ if c.is_ascii_digit() => {
                        state = NumberState::ExponentDigit;
                    }
                    _ => {
                        let c = self.char_for_error(c);
                        return Err(Error::with_loc(
                            format!("Unexpected character `{c}`, expected exponent digit"),
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                },
                NumberState::ExponentDigit => match c {
                    _ if c.is_ascii_digit() => {}
                    _ if c == b'.' || lookup::is_namestart(c) => {
                        let c = char::from(c);
                        return Err(Error::with_loc(
                            format!("Unexpected character `{c}` as float suffix"),
                            self.current_str().to_string(),
                            token.index,
                        ));
                    }
                    _ => {
                        token.data = self.prev_str();
                        return Ok(token);
                    }
                },
            }
        }
    }

    fn unexpected_character(&mut self, c: u8, token: &Token<'a>) -> Result<Token<'a>, Error> {
        let c = self.char_for_error(c);
        Err(Error::with_loc(
            format!(r#"Unexpected character "{c}""#),
            self.current_str().to_string(),
            token.index,
        ))
    }

    fn unterminated_string(&mut self, token: &Token<'a>) -> Result<Token<'a>, Error> {
        // Any pending in-string error is superseded by the unterminated error
        // (it was never observable: only the EOF token can follow a drain).
        self.err = None;
        Err(Error::with_loc("unterminated string value", self.drain().to_string(), token.index))
    }

    fn char_for_error(&mut self, c: u8) -> char {
        if c.is_ascii() { char::from(c) } else { self.consume_current_char() }
    }

    #[inline]
    fn done(&mut self, token: Token<'a>) -> Result<Token<'a>, Error> {
        if let Some(mut err) = self.err.take() {
            cold_path();
            err.set_data(token.data.to_string());
            err.index = token.index;
            return Err(err);
        }
        Ok(token)
    }
}

/// Ignored tokens other than comments and commas are assimilated to whitespace
/// <https://spec.graphql.org/October2021/#Ignored>
fn is_whitespace_assimilated(c: u8) -> bool {
    matches!(
        c,
        // https://spec.graphql.org/October2021/#WhiteSpace
        b'\t'
        | b' '
        // https://spec.graphql.org/October2021/#LineTerminator
        | b'\n'
        | b'\r'
    )
}

/// <https://spec.graphql.org/October2021/#NameContinue>
fn is_name_continue(c: u8) -> bool {
    matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')
}

// EscapedCharacter
//     "  \  /  b  f  n  r  t
fn is_escaped_char(c: u8) -> bool {
    matches!(c, b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't')
}
