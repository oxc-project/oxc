//! An Ecma-262 Lexer / Tokenizer
//! Prior Arts:
//!     * [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/crates/parser/src)
//!     * [rome](https://github.com/rome/tools/tree/main/crates/rome_js_parser/src/lexer)
//!     * [rustc](https://github.com/rust-lang/rust/blob/master/compiler/rustc_lexer/src)
//!     * [v8](https://v8.dev/blog/scanner)

mod byte_handlers;
mod comment;
mod identifier;
mod jsx;
mod kind;
mod number;
mod numeric;
mod punctuation;
mod regex;
mod string;
mod string_builder;
mod template;
mod token;
mod trivia_builder;
mod typescript;
mod unicode;

use rustc_hash::FxHashMap;
use std::{collections::VecDeque, str::Chars};

use oxc_allocator::Allocator;
use oxc_ast::ast::RegExpFlags;
use oxc_diagnostics::Error;
use oxc_span::{SourceType, Span};

use self::{byte_handlers::handle_byte, string_builder::AutoCow, trivia_builder::TriviaBuilder};
pub use self::{
    kind::Kind,
    number::{parse_big_int, parse_float, parse_int},
    token::Token,
};
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
    pub fn new(allocator: &'a Allocator, mut source: &'a str, source_type: SourceType) -> Self {
        // If source exceeds size limit, substitute a short source which will fail to parse.
        // `Parser::parse` will convert error to `diagnostics::OverlongSource`.
        if source.len() > MAX_LEN {
            source = "\0";
        }

        // The first token is at the start of file, so is allows on a new line
        let token = Token::new_on_new_line();
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

    fn finish_next(&mut self, kind: Kind) -> Token {
        self.current.token.kind = kind;
        self.current.token.end = self.offset();
        debug_assert!(self.current.token.start <= self.current.token.end);
        let token = self.current.token;
        self.current.token = Token::default();
        token
    }

    // ---------- Private Methods ---------- //
    fn error<T: Into<Error>>(&mut self, error: T) {
        self.errors.push(error.into());
    }

    /// Get the length offset from the source, in UTF-8 bytes
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn offset(&self) -> u32 {
        // Offset = current position of `chars` relative to start of `source`.
        // Previously was `self.source.len() - self.current.chars.as_str().len()`,
        // but that was slower because `std::str::Chars` internally is a current pointer + end pointer,
        // whereas `&str` internally is a start pointer and len.
        // So comparing `len()` of the two requires an extra memory read, and addition operation.
        // https://godbolt.org/z/v46MWddTM
        // This function is on hot path, so saving even a single instruction makes a measurable difference.
        (self.current.chars.as_str().as_ptr() as usize - self.source.as_ptr() as usize) as u32
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
}
