//! An Ecma-262 Lexer / Tokenizer
//! Prior Arts:
//!     * [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/24004745a8ed4939fc0dc7332bfd1268ac52285f/crates/parser/src)
//!     * [rome](https://github.com/rome/tools/tree/lsp/v0.28.0/crates/rome_js_parser/src/lexer)
//!     * [rustc](https://github.com/rust-lang/rust/blob/1.82.0/compiler/rustc_lexer/src)
//!     * [v8](https://v8.dev/blog/scanner)

use std::mem;

use rustc_hash::FxHashMap;

use oxc_allocator::{Allocator, ArenaVec};
use oxc_ast::ast::RegExpFlags;
use oxc_span::{SourceType, Span};
use oxc_str::JSStr;

use crate::{
    UniquePromise,
    config::LexerConfig as Config,
    diagnostics::{self, ParserDiagnostic},
};

mod byte_handlers;
mod comment;
mod identifier;
mod jsx;
mod keyword;
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

pub use kind::Kind;
pub use number::{parse_big_int, parse_float, parse_int};
pub use token::Token;

use source::{AsciiPosition, Source, SourcePosition};
use token::PendingToken;
use trivia_builder::{AnnotationComments, TriviaBuilder};

#[derive(Debug)]
pub struct LexerCheckpoint<'a> {
    source_position: SourcePosition<'a>,
    errors_snapshot: ErrorSnapshot<'a>,
    tokens_len: usize,
    annotation_comments: AnnotationComments,
}

#[derive(Debug)]
enum ErrorSnapshot<'a> {
    Empty,
    Count(usize),
    Full(Vec<ParserDiagnostic<'a>>),
}

/// Action to take when finishing a token.
/// Passed to [`Lexer::finish_next_inner`].
#[derive(Clone, Copy, Eq, PartialEq)]
enum FinishTokenMode {
    Push,
    Replace,
    Discard,
}

pub struct Lexer<'a, C: Config> {
    allocator: &'a Allocator,

    // Wrapper around source text. Must not be changed after initialization.
    source: Source<'a>,

    source_type: SourceType,

    /// Start of the token currently being lexed.
    token_start: u32,

    /// Flags of the token currently being lexed, and trivia state since the previous token.
    /// The `Token` itself is only constructed by `Token::new` once its kind and end are known.
    ///
    /// Between lexer calls, `pending` is always `PendingToken::NONE`. Every method which produces
    /// a token ends in `finish_next_inner`, which resets it, or `finish_re_lex`, which never alters it.
    ///
    /// The one exception is before the first token is lexed, when it is `PendingToken::START_OF_FILE`.
    /// `checkpoint` and `rewind` can never see that state, because the parser lexes the first token
    /// before it does anything else, and they are the only things which depend on `pending` being
    /// `PendingToken::NONE`. Both assert it.
    pending: PendingToken,

    pub(crate) errors: Vec<ParserDiagnostic<'a>>,

    /// Errors that are only emitted if the file is determined to be a Module.
    /// For `ModuleKind::Unambiguous`, HTML-like comments are allowed during lexing,
    /// but if ESM syntax is found later, these comments become invalid.
    /// If resolved to Module → emit these errors.
    /// If resolved to Script → discard these errors.
    pub(crate) deferred_module_errors: Vec<ParserDiagnostic<'a>>,

    pub(crate) trivia_builder: TriviaBuilder<'a>,

    /// Decoded identifiers and string literals, indexed by [Token::start] when
    /// [Token::escaped] is true.
    pub escaped_strings: FxHashMap<u32, JSStr<'a>>,

    /// Data store for escaped templates, indexed by [Token::start] when [Token::escaped] is true
    /// `None` is saved when the string contains an invalid escape sequence.
    pub escaped_templates: FxHashMap<u32, Option<JSStr<'a>>>,

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

        // If collecting tokens, allocate enough space so that the `Vec<Token>` will not have to grow during parsing.
        // `source_text.len() + 1` is almost always a large overestimate of number of tokens, but it's impossible to
        // have more than N + 1 tokens in a file which is N bytes long, so it'll never be an underestimate.
        //
        // + 1 is to account for the final `Eof` token. Without adding 1, the capacity could be too small for
        // minified files which have no space between any tokens. It would also be too small for empty files.
        //
        // Our largest benchmark file `binder.ts` is 190 KB, and `Token` is 16 bytes, so the `Vec<Token>`
        // would be ~3 MB even in the case of this unusually large file. That's not a huge amount of memory.
        //
        // However, we should choose a better heuristic based on real-world observation, and bring this usage down.
        let tokens = if config.tokens() {
            ArenaVec::with_capacity_in(source_text.len() + 1, &allocator)
        } else {
            ArenaVec::new_in(&allocator)
        };

        Self {
            allocator,
            source,
            source_type,
            token_start: 0,
            // The first token is at the start of file, so is always on a new line
            pending: PendingToken::START_OF_FILE,
            errors: vec![],
            deferred_module_errors: vec![],
            trivia_builder: TriviaBuilder::new_in(allocator),
            escaped_strings: FxHashMap::default(),
            escaped_templates: FxHashMap::default(),
            multi_line_comment_end_finder: None,
            tokens,
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
    pub fn errors(&self) -> &[ParserDiagnostic<'a>] {
        &self.errors
    }

    /// Remaining string from `Source`
    pub fn remaining(&self) -> &'a str {
        self.source.remaining()
    }

    /// Creates a checkpoint storing the current lexer state.
    /// Use `rewind` to restore the lexer to the state stored in the checkpoint.
    pub fn checkpoint(&self) -> LexerCheckpoint<'a> {
        // `pending` is always `PendingToken::NONE` here, so it does not need to be saved. See `pending` field docs.
        debug_assert_eq!(self.pending, PendingToken::NONE);

        let errors_snapshot = if self.errors.is_empty() {
            ErrorSnapshot::Empty
        } else {
            ErrorSnapshot::Count(self.errors.len())
        };

        LexerCheckpoint {
            source_position: self.source.position(),
            errors_snapshot,
            tokens_len: self.tokens.len(),
            annotation_comments: self.trivia_builder.annotation_comments,
        }
    }

    /// Create a checkpoint that can handle error popping.
    /// This is more expensive as it clones the errors vector.
    pub(crate) fn checkpoint_with_error_recovery(&self) -> LexerCheckpoint<'a> {
        // `pending` is always `PendingToken::NONE` here, so it does not need to be saved. See `pending` field docs.
        debug_assert_eq!(self.pending, PendingToken::NONE);

        let errors_snapshot = if self.errors.is_empty() {
            ErrorSnapshot::Empty
        } else {
            ErrorSnapshot::Full(self.errors.clone())
        };

        LexerCheckpoint {
            source_position: self.source.position(),
            errors_snapshot,
            tokens_len: self.tokens.len(),
            annotation_comments: self.trivia_builder.annotation_comments,
        }
    }

    /// Rewinds the lexer to the same state as when the passed in `checkpoint` was created.
    pub fn rewind(&mut self, checkpoint: LexerCheckpoint<'a>) {
        // `pending` is always `PendingToken::NONE` here, so it does not need to be restored. See `pending` field docs.
        debug_assert_eq!(self.pending, PendingToken::NONE);

        match checkpoint.errors_snapshot {
            ErrorSnapshot::Empty => self.errors.clear(),
            ErrorSnapshot::Count(len) => self.errors.truncate(len),
            ErrorSnapshot::Full(errors) => self.errors = errors,
        }

        self.tokens.truncate(checkpoint.tokens_len);
        self.source.set_position(checkpoint.source_position);
        self.trivia_builder.annotation_comments = checkpoint.annotation_comments;
    }

    pub fn peek_token(&mut self) -> Token {
        let checkpoint = self.checkpoint();
        let token = self.next_token();
        self.rewind(checkpoint);
        token
    }

    /// Read first token in file.
    pub fn first_token(&mut self) -> Token {
        // HashbangComment ::
        //     `#!` SingleLineCommentChars?
        if let Some([b'#', b'!']) = self.peek_2_bytes() {
            // SAFETY: Next 2 bytes are `#!`
            let kind = unsafe { self.read_hashbang_comment() };
            // Hashbangs are not included in tokens
            self.finish_next_inner(kind, self.source.position(), FinishTokenMode::Discard)
        } else {
            let (kind, end) = self.read_next_token();
            self.finish_next_inner(kind, end, FinishTokenMode::Push)
        }
    }

    /// Read next token in file.
    /// Use `first_token` for first token, and this method for all further tokens.
    pub fn next_token(&mut self) -> Token {
        let (kind, end) = self.read_next_token();
        self.finish_next_inner(kind, end, FinishTokenMode::Push)
    }

    /// Read the next token after `=` in a JSX attribute.
    pub(crate) fn next_jsx_attribute_value(&mut self) -> Token {
        let kind = self.read_next_jsx_attribute_value();
        self.finish_next(kind)
    }

    // This is a workaround for a problem where `next_token` is not inlined in lexer benchmark.
    // Must be kept in sync with `next_token` above, and contain exactly the same code.
    #[cfg(feature = "benchmarking")]
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn next_token_for_benchmarks(&mut self) -> Token {
        let (kind, end) = self.read_next_token();
        self.finish_next_inner(kind, end, FinishTokenMode::Push)
    }

    #[inline]
    fn finish_next(&mut self, kind: Kind) -> Token {
        self.finish_next_inner(kind, self.source.position(), FinishTokenMode::Push)
    }

    #[inline]
    fn finish_next_retokenized(&mut self, kind: Kind) -> Token {
        self.finish_next_inner(kind, self.source.position(), FinishTokenMode::Replace)
    }

    // `#[inline(always)]` to ensure is inlined into `finish_next` and `finish_next_retokenized`,
    // so that `mode` is statically known
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn finish_next_inner(
        &mut self,
        kind: Kind,
        end: SourcePosition<'a>,
        mode: FinishTokenMode,
    ) -> Token {
        debug_assert_eq!(end, self.source.position());
        let pending = mem::replace(&mut self.pending, PendingToken::NONE);
        let token = Token::new(kind, self.token_start, self.source.offset_of(end), pending);
        if self.config.tokens() {
            match mode {
                FinishTokenMode::Push => {
                    // We allocated sufficient capacity in the `Vec` for all tokens at the start.
                    // `push_fast` is optimized for the "doesn't need to grow" case.
                    self.tokens.push_fast(token);
                }
                FinishTokenMode::Replace => {
                    debug_assert!(
                        self.tokens.last().is_some_and(|last| last.start() == token.start())
                    );
                    let last = self.tokens.last_mut().unwrap();
                    *last = token;
                }
                FinishTokenMode::Discard => {}
            }
        }
        self.trivia_builder.handle_token(token, pending);
        token
    }

    /// Finish a re-lexed token used only for parser disambiguation.
    /// This must not mutate the externally collected token stream.
    fn finish_re_lex(&self, kind: Kind) -> Token {
        debug_assert_eq!(self.pending, PendingToken::NONE);
        Token::new(kind, self.token_start, self.offset(), PendingToken::NONE)
    }

    /// Overwrite the last token in the collected token stream.
    ///
    /// Used by `re_lex_as_typescript_l_angle` to replace a compound token (`<<`, `<=` or `<<=`)
    /// with the single `<` it re-lexed, and to restore the compound token when `try_parse`
    /// fails and rewinds.
    #[inline]
    pub(crate) fn rewrite_last_collected_token(&mut self, token: Token) {
        // Make this function a no-op when tokens are statically disabled (`NoTokensLexerConfig`)
        if C::TOKENS_METHOD_IS_STATIC && !self.config.tokens() {
            return;
        }

        // Because of the static check above, there's no need to check `self.config.tokens()` here.
        //
        // * If tokens are statically disabled, we already exited.
        // * If tokens are statically enabled, then `self.tokens` is always non-empty.
        // * If tokens are runtime disabled, then `self.tokens` is always empty.
        // * If tokens are runtime enabled, then `self.tokens` is always non-empty.
        //
        // So checking `self.config.tokens()` too here would be redundant,
        // and would be an extra branch with runtime config (`RuntimeLexerConfig`).
        if let Some(last) = self.tokens.last_mut() {
            *last = token;
        } else {
            // When tokens are enabled, this should be unreachable
            debug_assert!(!self.config.tokens());
        }
    }

    pub(crate) fn take_tokens(&mut self) -> ArenaVec<'a, Token> {
        mem::replace(&mut self.tokens, ArenaVec::new_in(&self.allocator))
    }

    pub(crate) fn set_tokens(&mut self, tokens: ArenaVec<'a, Token>) {
        self.tokens = tokens;
    }

    /// Finalize tokens and return them.
    /// Called at very end of parsing.
    pub(crate) fn finalize_tokens(&mut self) -> ArenaVec<'a, Token> {
        if self.config.tokens() {
            // Tokens are enabled. Discard the last token, which marks the end of input.
            // Usually that is `Eof`. Where the lexer cannot lex any further - an unterminated string,
            // for example - it emits `Undetermined` instead, and no `Eof` ever follows.
            // The parser treats the two alike, and neither is a real token, so both are discarded.
            let mut tokens = self.take_tokens();
            let last_token = tokens.pop();
            debug_assert!(
                last_token
                    .is_some_and(|token| matches!(token.kind(), Kind::Eof | Kind::Undetermined))
            );
            tokens
        } else {
            // Tokens are disabled. Just return an empty vec.
            debug_assert!(self.tokens.is_empty());
            ArenaVec::new_in(&self.allocator)
        }
    }

    /// Advance source cursor to end of file.
    #[inline]
    pub fn advance_to_end(&mut self) {
        self.source.advance_to_end();
    }

    // ---------- Private Methods ---------- //
    fn error(&mut self, error: ParserDiagnostic<'a>) {
        self.errors.push(error);
    }

    /// Get the length offset from the source, in UTF-8 bytes
    #[inline]
    fn offset(&self) -> u32 {
        self.source.offset()
    }

    /// Get the current unterminated token range
    fn unterminated_range(&self) -> Span {
        Span::new(self.token_start, self.offset())
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
    /// Returns the `Kind` of the token, and the position of its end, which is current position of source.
    #[inline] // Make sure is inlined into `next_token`
    fn read_next_token(&mut self) -> (Kind, SourcePosition<'a>) {
        self.trivia_builder.clear_annotation_comments();

        let mut pos = self.source.position();
        loop {
            // Byte handlers return the position which they leave the source at
            debug_assert_eq!(pos, self.source.position());

            // Single spaces between tokens are common, so `byte_start` skips a space
            // before the byte which the next token starts with
            let kind;
            if let Some((start, byte)) = self.source.byte_start(pos) {
                // Set token start
                self.token_start = self.source.offset_of(start.position());

                if let Some(kind) = byte_handlers::single_byte_token(byte) {
                    let end = start.after();
                    self.source.set_position(end);
                    return (kind, end);
                }

                (kind, pos) = Self::byte_handler(byte)(self, start);
            } else {
                // Only 0 or 1 bytes left in source, or a Unicode character.
                // Delegate to `#[cold]` function as these are rare cases.
                kind = self.read_next_token_uncommon();
                pos = self.source.position();
            }

            if kind != Kind::Skip {
                return (kind, pos);
            }
        }
    }

    /// Cold path for reading next token from the current position of source, where either
    /// only 0 or 1 bytes are left in source, or the next byte is not ASCII.
    ///
    /// Also called where a single space is followed by a byte which is not ASCII.
    /// The space is then skipped as whitespace.
    #[inline(never)]
    #[cold]
    fn read_next_token_uncommon(&mut self) -> Kind {
        self.token_start = self.offset();

        if let Some((start, byte)) = self.source.ascii_position(self.source.position()) {
            self.handle_ascii_byte(start, byte)
        } else if self.source.is_eof() {
            Kind::Eof
        } else {
            self.unicode_char_handler()
        }
    }

    #[inline]
    fn read_next_jsx_attribute_value(&mut self) -> Kind {
        self.trivia_builder.clear_annotation_comments();

        loop {
            let pos = self.source.position();
            let kind = if let Some((start, byte)) = self.source.byte_start(pos) {
                self.token_start = self.source.offset_of(start.position());

                if matches!(byte, b'"' | b'\'') {
                    return self.read_jsx_string_literal(start, byte);
                }

                self.handle_ascii_byte(start, byte)
            } else {
                self.read_next_jsx_attribute_value_uncommon()
            };

            if kind != Kind::Skip {
                return kind;
            }
        }
    }

    /// Cold path for `read_next_jsx_attribute_value`. As for `read_next_token_uncommon`.
    #[inline(never)]
    #[cold]
    fn read_next_jsx_attribute_value_uncommon(&mut self) -> Kind {
        if let Some((start, byte)) = self.source.ascii_position(self.source.position())
            && matches!(byte, b'"' | b'\'')
        {
            self.token_start = self.offset();
            return self.read_jsx_string_literal(start, byte);
        }

        self.read_next_token_uncommon()
    }
}

/// Call a closure while hinting to compiler that this branch is rarely taken.
///
/// Unlike [`cold_path`], which only guides branch layout, this `#[cold]` trampoline also moves
/// the closure's code and stack usage out of the caller's frame (unless compiler chooses to
/// inline it). Use `cold_path` for pure branch hints. Use `cold_branch` where keeping large
/// stack objects (e.g. an `OxcDiagnostic` return buffer) out of the hot function's stack frame
/// is the aim.
///
/// "Cold trampoline function", suggested in:
/// <https://users.rust-lang.org/t/is-cold-the-only-reliable-way-to-hint-to-branch-predictor/106509/2>
///
/// [`cold_path`]: oxc_data_structures::branch_hints::cold_path
#[cold]
pub fn cold_branch<F: FnOnce() -> T, T>(f: F) -> T {
    f()
}
