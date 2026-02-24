//! An Ecma-262 Lexer / Tokenizer
//! Prior Arts:
//!     * [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/24004745a8ed4939fc0dc7332bfd1268ac52285f/crates/parser/src)
//!     * [rome](https://github.com/rome/tools/tree/lsp/v0.28.0/crates/rome_js_parser/src/lexer)
//!     * [rustc](https://github.com/rust-lang/rust/blob/1.82.0/compiler/rustc_lexer/src)
//!     * [v8](https://v8.dev/blog/scanner)

use rustc_hash::FxHashMap;

use oxc_allocator::{Allocator, Vec as ArenaVec};
use oxc_ast::ast::RegExpFlags;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{SourceType, Span};

use crate::{UniquePromise, config::LexerConfig as Config, diagnostics};

mod byte_handlers;
mod comment;
mod identifier;
mod jsx;
mod kind;
mod number;
mod numeric;
mod punctuation;
mod regex;
mod search;
mod source;
mod string;
mod template;
mod token;
mod trivia_builder;
mod typescript;
mod unicode;
mod whitespace;

pub(crate) use byte_handlers::{ByteHandler, ByteHandlers, byte_handler_tables};
pub use kind::Kind;
pub use number::{parse_big_int, parse_float, parse_int};
pub use token::Token;

use source::{Source, SourcePosition};
use trivia_builder::TriviaBuilder;

#[derive(Debug, Clone)]
pub struct LexerCheckpoint<'a> {
    source_position: SourcePosition<'a>,
    token: Token,
    errors_snapshot: ErrorSnapshot,
    tokens_len: usize,
    has_pure_comment: bool,
    has_no_side_effects_comment: bool,
}

#[derive(Debug, Clone)]
enum ErrorSnapshot {
    Empty,
    Count(usize),
    Full(Vec<OxcDiagnostic>),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LexerContext {
    Regular,
    /// Lex the next token, returns `JsxString` or any other token
    JsxAttributeValue,
}

pub struct Lexer<'a, C: Config> {
    allocator: &'a Allocator,

    // Wrapper around source text. Must not be changed after initialization.
    source: Source<'a>,

    source_type: SourceType,

    token: Token,

    pub(crate) errors: Vec<OxcDiagnostic>,

    /// Errors that are only emitted if the file is determined to be a Module.
    /// For `ModuleKind::Unambiguous`, HTML-like comments are allowed during lexing,
    /// but if ESM syntax is found later, these comments become invalid.
    /// If resolved to Module → emit these errors.
    /// If resolved to Script → discard these errors.
    pub(crate) deferred_module_errors: Vec<OxcDiagnostic>,

    context: LexerContext,

    pub(crate) trivia_builder: TriviaBuilder,

    /// Data store for escaped strings, indexed by [Token::start] when [Token::escaped] is true
    pub escaped_strings: FxHashMap<u32, &'a str>,

    /// Data store for escaped templates, indexed by [Token::start] when [Token::escaped] is true
    /// `None` is saved when the string contains an invalid escape sequence.
    pub escaped_templates: FxHashMap<u32, Option<&'a str>>,

    /// `memchr` Finder for end of multi-line comments. Created lazily when first used.
    multi_line_comment_end_finder: Option<memchr::memmem::Finder<'static>>,

    /// Collected tokens in source order.
    tokens: ArenaVec<'a, Token>,

    /// Config
    pub(crate) config: C,
}

impl<'a, C: Config> Lexer<'a, C> {
    /// Create new `Lexer`.
    ///
    /// Requiring a `UniquePromise` to be provided guarantees only 1 `Lexer` can exist
    /// on a single thread at one time.
    pub(super) fn new(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        config: C,
        unique: UniquePromise,
    ) -> Self {
        let source = Source::new(source_text, unique);

        // The first token is at the start of file, so is allows on a new line
        let token = Token::new_on_new_line();
        Self {
            allocator,
            source,
            source_type,
            token,
            errors: vec![],
            deferred_module_errors: vec![],
            context: LexerContext::Regular,
            trivia_builder: TriviaBuilder::default(),
            escaped_strings: FxHashMap::default(),
            escaped_templates: FxHashMap::default(),
            multi_line_comment_end_finder: None,
            tokens: ArenaVec::new_in(allocator),
            config,
        }
    }

    /// Backdoor to create a `Lexer` without holding a `UniquePromise`, for benchmarks.
    /// This function must NOT be exposed in public API as it breaks safety invariants.
    #[cfg(feature = "benchmarking")]
    pub fn new_for_benchmarks(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        config: C,
    ) -> Self {
        let unique = UniquePromise::new_for_tests_and_benchmarks();
        Self::new(allocator, source_text, source_type, config, unique)
    }

    /// Get errors.
    /// Only used in benchmarks.
    #[cfg(feature = "benchmarking")]
    pub fn errors(&self) -> &[OxcDiagnostic] {
        &self.errors
    }

    /// Remaining string from `Source`
    pub fn remaining(&self) -> &'a str {
        self.source.remaining()
    }

    /// Creates a checkpoint storing the current lexer state.
    /// Use `rewind` to restore the lexer to the state stored in the checkpoint.
    pub fn checkpoint(&self) -> LexerCheckpoint<'a> {
        let errors_snapshot = if self.errors.is_empty() {
            ErrorSnapshot::Empty
        } else {
            ErrorSnapshot::Count(self.errors.len())
        };
        LexerCheckpoint {
            source_position: self.source.position(),
            token: self.token,
            errors_snapshot,
            tokens_len: self.tokens.len(),
            has_pure_comment: self.trivia_builder.has_pure_comment,
            has_no_side_effects_comment: self.trivia_builder.has_no_side_effects_comment,
        }
    }

    /// Create a checkpoint that can handle error popping.
    /// This is more expensive as it clones the errors vector.
    pub(crate) fn checkpoint_with_error_recovery(&self) -> LexerCheckpoint<'a> {
        let errors_snapshot = if self.errors.is_empty() {
            ErrorSnapshot::Empty
        } else {
            ErrorSnapshot::Full(self.errors.clone())
        };
        LexerCheckpoint {
            source_position: self.source.position(),
            token: self.token,
            errors_snapshot,
            tokens_len: self.tokens.len(),
            has_pure_comment: self.trivia_builder.has_pure_comment,
            has_no_side_effects_comment: self.trivia_builder.has_no_side_effects_comment,
        }
    }

    /// Rewinds the lexer to the same state as when the passed in `checkpoint` was created.
    pub fn rewind(&mut self, checkpoint: LexerCheckpoint<'a>) {
        match checkpoint.errors_snapshot {
            ErrorSnapshot::Empty => self.errors.clear(),
            ErrorSnapshot::Count(len) => self.errors.truncate(len),
            ErrorSnapshot::Full(errors) => self.errors = errors,
        }
        self.tokens.truncate(checkpoint.tokens_len);
        self.source.set_position(checkpoint.source_position);
        self.token = checkpoint.token;
        self.trivia_builder.has_pure_comment = checkpoint.has_pure_comment;
        self.trivia_builder.has_no_side_effects_comment = checkpoint.has_no_side_effects_comment;
    }

    pub fn peek_token(&mut self) -> Token {
        let checkpoint = self.checkpoint();
        let token = self.next_token();
        self.rewind(checkpoint);
        token
    }

    /// Set context
    pub fn set_context(&mut self, context: LexerContext) {
        self.context = context;
    }

    /// Read first token in file.
    pub fn first_token(&mut self) -> Token {
        // HashbangComment ::
        //     `#!` SingleLineCommentChars?
        let kind = if let Some([b'#', b'!']) = self.peek_2_bytes() {
            // SAFETY: Next 2 bytes are `#!`
            unsafe { self.read_hashbang_comment() }
        } else {
            self.read_next_token()
        };
        self.finish_next(kind)
    }

    /// Read next token in file.
    /// Use `first_token` for first token, and this method for all further tokens.
    pub fn next_token(&mut self) -> Token {
        let kind = self.read_next_token();
        self.finish_next(kind)
    }

    // This is a workaround for a problem where `next_token` is not inlined in lexer benchmark.
    // Must be kept in sync with `next_token` above, and contain exactly the same code.
    #[cfg(feature = "benchmarking")]
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn next_token_for_benchmarks(&mut self) -> Token {
        let kind = self.read_next_token();
        self.finish_next(kind)
    }

    #[inline]
    fn finish_next(&mut self, kind: Kind) -> Token {
        self.finish_next_inner::<false>(kind)
    }

    #[inline]
    fn finish_next_retokenized(&mut self, kind: Kind) -> Token {
        self.finish_next_inner::<true>(kind)
    }

    #[inline]
    fn finish_next_inner<const REPLACE_SAME_START: bool>(&mut self, kind: Kind) -> Token {
        self.token.set_kind(kind);
        self.token.set_end(self.offset());
        let token = self.token;
        if self.config.tokens() && !matches!(token.kind(), Kind::Eof | Kind::HashbangComment) {
            if REPLACE_SAME_START {
                debug_assert!(self.tokens.last().is_some_and(|last| last.start() == token.start()));
                let last = self.tokens.last_mut().unwrap();
                *last = token;
            } else {
                self.tokens.push(token);
            }
        }
        self.trivia_builder.handle_token(token);
        self.token = Token::default();
        token
    }

    /// Finish a re-lexed token used only for parser disambiguation.
    /// This must not mutate the externally collected token stream.
    fn finish_re_lex(&mut self, kind: Kind) -> Token {
        self.token.set_kind(kind);
        self.token.set_end(self.offset());
        let token = self.token;
        self.token = Token::default();
        token
    }

    pub(crate) fn take_tokens(&mut self) -> ArenaVec<'a, Token> {
        std::mem::replace(&mut self.tokens, ArenaVec::new_in(self.allocator))
    }

    pub(crate) fn set_tokens(&mut self, tokens: ArenaVec<'a, Token>) {
        self.tokens = tokens;
    }

    /// Advance source cursor to end of file.
    #[inline]
    pub fn advance_to_end(&mut self) {
        self.source.advance_to_end();
    }

    // ---------- Private Methods ---------- //
    fn error(&mut self, error: OxcDiagnostic) {
        self.errors.push(error);
    }

    /// Get the length offset from the source, in UTF-8 bytes
    #[inline]
    fn offset(&self) -> u32 {
        self.source.offset()
    }

    /// Get the current unterminated token range
    fn unterminated_range(&self) -> Span {
        Span::new(self.token.start(), self.offset())
    }

    /// Consume the current char if not at EOF
    #[inline]
    fn next_char(&mut self) -> Option<char> {
        self.source.next_char()
    }

    /// Consume the current char
    #[inline]
    fn consume_char(&mut self) -> char {
        self.source.next_char().unwrap()
    }

    /// Consume the current char and the next if not at EOF
    #[inline]
    fn next_2_chars(&mut self) -> Option<[char; 2]> {
        self.source.next_2_chars()
    }

    /// Consume the current char and the next
    #[inline]
    fn consume_2_chars(&mut self) -> [char; 2] {
        self.next_2_chars().unwrap()
    }

    /// Peek the next byte without advancing the position
    #[inline]
    fn peek_byte(&self) -> Option<u8> {
        self.source.peek_byte()
    }

    /// Peek the next two bytes without advancing the position
    #[inline]
    fn peek_2_bytes(&self) -> Option<[u8; 2]> {
        self.source.peek_2_bytes()
    }

    /// Peek the next char without advancing the position
    #[inline]
    fn peek_char(&self) -> Option<char> {
        self.source.peek_char()
    }

    /// Peek the next byte, and advance the current position if it matches
    /// the given ASCII char.
    // `#[inline(always)]` to make sure the `assert!` gets optimized out.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn next_ascii_byte_eq(&mut self, b: u8) -> bool {
        // TODO: can be replaced by `std::ascii:Char` once stabilized.
        // https://github.com/rust-lang/rust/issues/110998
        assert!(b.is_ascii());
        // SAFETY: `b` is a valid ASCII char.
        unsafe { self.source.advance_if_ascii_eq(b) }
    }

    fn current_offset(&self) -> Span {
        Span::empty(self.offset())
    }

    /// Return `IllegalCharacter` Error or `UnexpectedEnd` if EOF
    fn unexpected_err(&mut self) {
        let offset = self.current_offset();
        match self.peek_char() {
            Some(c) => self.error(diagnostics::invalid_character(c, offset)),
            None => self.error(diagnostics::unexpected_end(offset)),
        }
    }

    /// Read each char and set the current token
    /// Whitespace and line terminators are skipped
    #[inline] // Make sure is inlined into `next_token`
    fn read_next_token(&mut self) -> Kind {
        self.trivia_builder.has_pure_comment = false;
        self.trivia_builder.has_no_side_effects_comment = false;

        let end_pos = self.source.end();
        loop {
            // Single spaces between tokens are common, so consume a space before processing the next token.
            // Do this without a branch. This produces more instructions, but avoids an unpredictable branch.
            // Can only do this if there are at least 2 bytes left in source.
            // If there aren't 2 bytes left, delegate to `read_next_token_at_end` (cold branch).
            let mut pos = self.source.position();
            // SAFETY: `source.end()` is always equal to or after `source.position()`
            let remaining_bytes = unsafe { end_pos.offset_from(pos) };
            if remaining_bytes >= 2 {
                // Read next byte.
                // SAFETY: There are at least 2 bytes remaining in source.
                let byte = unsafe { pos.read() };

                // If next byte is a space, advance by 1 byte.
                // Do this with maths, instead of a branch.
                let is_space = byte == b' ';
                // SAFETY: There are at least 2 bytes remaining in source, so advancing 1 byte cannot be out of bounds
                pos = unsafe { pos.add(usize::from(is_space)) };
                self.source.set_position(pos);

                // Read next byte again, in case we skipped a space.
                // SAFETY: We checked above that there were at least 2 bytes to read,
                // and we skipped a maximum of 1 byte, so there's still at least 1 byte left to read.
                let byte = unsafe { pos.read() };

                // Set token start
                let offset = self.source.offset_of(pos);
                self.token.set_start(offset);

                // SAFETY: `byte` is byte value at current position in source
                let kind = unsafe { self.handle_byte(byte) };
                if kind != Kind::Skip {
                    return kind;
                }
            } else {
                // Only 0 or 1 bytes left in source.
                // Delegate to `#[cold]` function as this is a very rare case.
                return self.read_next_token_at_end();
            }
        }
    }

    /// Cold path for reading next token where only 0 or 1 bytes are left in source.
    #[inline(never)]
    #[cold]
    fn read_next_token_at_end(&mut self) -> Kind {
        let offset = self.offset();
        self.token.set_start(offset);

        if let Some(byte) = self.peek_byte() {
            // SAFETY: `byte` is byte value at current position in source
            let kind = unsafe { self.handle_byte(byte) };
            if kind != Kind::Skip {
                return kind;
            }
            // Last byte was whitespace/line break (`Kind::Skip`), so now at EOF
            self.token.set_start(offset + 1);
        }

        Kind::Eof
    }
}

/// Call a closure while hinting to compiler that this branch is rarely taken.
///
/// "Cold trampoline function", suggested in:
/// <https://users.rust-lang.org/t/is-cold-the-only-reliable-way-to-hint-to-branch-predictor/106509/2>
#[cold]
pub fn cold_branch<F: FnOnce() -> T, T>(f: F) -> T {
    f()
}
