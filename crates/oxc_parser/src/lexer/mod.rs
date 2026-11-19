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
use oxc_data_structures::brands::BrandOrigin;
#[cfg(any(test, feature = "benchmarking"))]
use oxc_data_structures::brands::BrandScope;
use oxc_span::{SourceType, Span};
use oxc_str::JSStr;

use crate::{
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

#[cfg_attr(not(feature = "benchmarking"), expect(clippy::redundant_pub_crate))]
pub(crate) use byte_handlers::{ByteHandler, ByteHandlers, byte_handler_tables};
pub use kind::Kind;
pub use number::{parse_big_int, parse_float, parse_int};
pub use token::Token;

use source::{Source, SourcePosition};
use token::PendingToken;
use trivia_builder::{AnnotationComments, TriviaBuilder};

#[derive(Debug, Clone, Copy)]
pub struct LexerCheckpoint<'a> {
    source_position: SourcePosition<'a>,
    errors_len: usize,
    tokens_len: usize,
    annotation_comments: AnnotationComments,
}

impl LexerCheckpoint<'_> {
    pub(crate) fn errors_len(&self) -> usize {
        self.errors_len
    }
}

/// Number of bytes which are always available to read from the position passed to a token handler.
/// i.e. a token handler can read the byte after the first byte of the token, without checking
/// whether it is at the end of the source.
const TOKEN_HANDLER_MIN_BYTES: usize = 2;

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
    /// a token ends in one of:
    /// * `finish_next_inner`, which resets it.
    /// * `finish_re_lex`, which never alters it.
    /// * `finish_fresh_token`, which is only for tokens which nothing has altered it for.
    ///
    /// The one exception is before the first token is lexed, when it is `PendingToken::START_OF_FILE`.
    /// `next_token`, `checkpoint` and `rewind` depend on `pending` being `PendingToken::NONE`,
    /// and assert it in debug builds. They can never see that state, because the parser lexes
    /// the first token with `first_token` before it does anything else.
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

    /// Collected tokens in source order.
    tokens: ArenaVec<'a, Token>,

    /// Config
    pub(crate) config: C,
}

impl<'a, C: Config> Lexer<'a, C> {
    /// Create new `Lexer`.
    ///
    /// `brand_origin` is for the `Lexer`'s `Source` (see `Source` docs).
    pub(super) fn new(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        config: C,
        brand_origin: BrandOrigin<'a>,
    ) -> Self {
        let source = Source::new(source_text, brand_origin);

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
            tokens,
            config,
        }
    }

    /// Create a [`Lexer`], and call `f` with it. Returns what `f` returns.
    ///
    /// The `Lexer` is passed to `f`, not returned, because that's what makes `'brand`
    /// unique to it. `f` must accept a `Lexer` with any lifetime `'brand`, so code in `f`
    /// can't treat `'brand` as the same as any other lifetime. That makes `'brand` a brand
    /// (see [`brands`]).
    ///
    /// For tests and benchmarks only.
    /// The parser creates its `Lexer` with `ParserImpl::scope`, and `Lexer::new`.
    ///
    /// [`brands`]: oxc_data_structures::brands
    #[cfg(any(test, feature = "benchmarking"))]
    pub fn scope<R>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        config: C,
        f: impl for<'brand> FnOnce(Lexer<'brand, C>) -> R,
    ) -> R {
        BrandOrigin::scope_within(|brand_origin, _: BrandScope<'_, 'a>| {
            // `allocator` and `source_text` are shortened from `'a` to `'brand` here
            let lexer = Lexer::new(allocator, source_text, source_type, config, brand_origin);
            f(lexer)
        })
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

        LexerCheckpoint {
            source_position: self.source.position(),
            errors_len: self.errors.len(),
            tokens_len: self.tokens.len(),
            annotation_comments: self.trivia_builder.annotation_comments,
        }
    }

    /// Rewinds the lexer to the same state as when the passed in `checkpoint` was created.
    pub fn rewind(&mut self, checkpoint: LexerCheckpoint<'a>) {
        // `pending` is always `PendingToken::NONE` here, so it does not need to be restored. See `pending` field docs.
        debug_assert_eq!(self.pending, PendingToken::NONE);

        self.errors.truncate(checkpoint.errors_len);

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
            self.first_token_without_hashbang()
        }
    }

    /// Read first token, without treating `#!` as the start of a hashbang comment.
    /// For source which is not a whole file e.g. a lone expression.
    pub fn first_token_without_hashbang(&mut self) -> Token {
        let (kind, end) = self.read_next_token();
        self.finish_next_inner(kind, end, FinishTokenMode::Push)
    }

    /// Read next token in file.
    ///
    /// Use `first_token` or `first_token_without_hashbang` for first token, and this method for
    /// all further tokens. This method relies on `pending` being `PendingToken::NONE`, which it is
    /// after any token has been read, but not at the start of the file. Reading the first token
    /// with this method gives that token, and a later one, the wrong `is_on_new_line` flag.
    /// That is only checked in debug builds.
    pub fn next_token(&mut self) -> Token {
        self.next_token_impl()
    }

    /// Read next token in file.
    ///
    /// This is inlined into every place the parser advances, so it only contains what most tokens
    /// need: skip one space, then either complete a single-byte token, or call a token handler.
    // `#[inline(always)]` so `next_token` and `next_token_for_benchmarks` contain the same code
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn next_token_impl(&mut self) -> Token {
        // No trivia has been seen since the previous token. See `pending` field docs.
        debug_assert_eq!(self.pending, PendingToken::NONE);

        self.trivia_builder.clear_annotation_comments();

        let mut pos = self.source.position();
        // SAFETY: `source.end()` is always equal to or after `source.position()`
        let remaining_bytes = unsafe { self.source.end().offset_from(pos) };
        if remaining_bytes > TOKEN_HANDLER_MIN_BYTES {
            // SAFETY: There are more than `TOKEN_HANDLER_MIN_BYTES` bytes remaining in source
            let mut byte = unsafe { pos.read() };
            if byte == b' ' {
                // SAFETY: A space is ASCII, and `TOKEN_HANDLER_MIN_BYTES` bytes remain after it
                pos = unsafe { pos.add(1) };
                // SAFETY: `TOKEN_HANDLER_MIN_BYTES` bytes remain after the space
                byte = unsafe { pos.read() };
            }

            if let Some(kind) = byte_handlers::single_byte_token(byte) {
                // SAFETY: Single-byte tokens are ASCII, and at least 1 byte remains after `pos`
                let end = unsafe { pos.add(1) };
                self.source.set_position(end);
                return self.finish_fresh_token(kind, pos, end);
            }

            let token_handlers = &Self::TOKEN_HANDLERS;
            // SAFETY: `byte` is the byte at `pos`. `TOKEN_HANDLER_MIN_BYTES` bytes remain from
            // `pos`. No trivia has been seen since the previous token, other than one space.
            return unsafe { token_handlers[byte as usize](self, pos) };
        }

        self.next_token_after_trivia()
    }

    /// Read next token, starting from the current position of source.
    ///
    /// Token handlers continue here once they have skipped whitespace or a comment.
    /// `next_token_impl` calls this if it is at the end of the source.
    #[inline(never)]
    fn next_token_after_trivia(&mut self) -> Token {
        let (kind, end) = self.read_next_token_after_trivia();
        self.finish_next_inner(kind, end, FinishTokenMode::Push)
    }

    /// Finish a token which starts at `start` and ends at `end`, where all of these are true:
    ///
    /// * No trivia was seen between the previous token and it, other than one space.
    /// * It has no flags (`escaped` or `has_separator`).
    /// * Position of source has been set to `end`.
    ///
    /// Such a token is not on a new line, and has no comments to attach, so `pending` is neither
    /// read nor reset. `token_start` is not read either, so does not need to have been set.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn finish_fresh_token(
        &mut self,
        kind: Kind,
        start: SourcePosition<'a>,
        end: SourcePosition<'a>,
    ) -> Token {
        self.finish_token_without_comments(kind, start, end, PendingToken::NONE)
    }

    /// Same as `finish_fresh_token`, for a token which only line breaks and regular whitespace
    /// were seen before. The token is on a new line.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn finish_fresh_token_on_new_line(
        &mut self,
        kind: Kind,
        start: SourcePosition<'a>,
        end: SourcePosition<'a>,
    ) -> Token {
        self.finish_token_without_comments(kind, start, end, PendingToken::AFTER_LINE_BREAK)
    }

    /// Finish a token which has no flags, and which no comments were seen before.
    /// `trivia` is what else was seen before it. `pending` must not have been altered.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn finish_token_without_comments(
        &mut self,
        kind: Kind,
        start: SourcePosition<'a>,
        end: SourcePosition<'a>,
        trivia: PendingToken,
    ) -> Token {
        debug_assert_eq!(end, self.source.position());
        debug_assert_ne!(kind, Kind::Skip);
        #[cfg(debug_assertions)]
        self.assert_byte_handler_produces_token(kind, start, end);

        let token =
            Token::new(kind, self.source.offset_of(start), self.source.offset_of(end), trivia);
        self.record_token_without_comments(token)
    }

    /// Check that the byte handler for the first byte of a token produces that same token,
    /// with no flags.
    ///
    /// Code which finishes a token with `finish_fresh_token` decides for itself where the token
    /// ends, what `Kind` it is, and that it has no flags. Byte handlers lex the same token where
    /// it follows a comment or other trivia, so the two must agree. This checks that they do,
    /// in debug builds, which all tests and conformance runs are.
    #[cfg(debug_assertions)]
    fn assert_byte_handler_produces_token(
        &mut self,
        kind: Kind,
        start: SourcePosition<'a>,
        end: SourcePosition<'a>,
    ) {
        let token_start = self.token_start;
        let error_count = self.errors.len();

        self.source.set_position(start);
        self.token_start = self.source.offset_of(start);
        // SAFETY: A token starts at `start`, so it is not the end of the source
        let byte = unsafe { start.read() };
        // SAFETY: `byte` is the byte at `start`, which is the current position of source
        let byte_handler_kind = unsafe { self.handle_byte(byte) };

        assert_eq!(
            (byte_handler_kind, self.source.position()),
            (kind, end),
            "Byte handler lexes a different token"
        );
        assert_eq!(self.pending, PendingToken::NONE, "Byte handler gives the token flags");
        assert_eq!(self.errors.len(), error_count, "Byte handler reports an error for the token");

        self.token_start = token_start;
    }

    /// Record a finished token, where no comments were seen since the previous token,
    /// and `pending` has not been altered since then.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn record_token_without_comments(&mut self, token: Token) -> Token {
        debug_assert_eq!(self.pending, PendingToken::NONE);
        if self.config.tokens() {
            // We allocated sufficient capacity in the `Vec` for all tokens at the start.
            // `push_fast` is optimized for the "doesn't need to grow" case.
            self.tokens.push_fast(token);
        }
        self.trivia_builder.set_previous_token(token);
        token
    }

    /// Read the next token after `=` in a JSX attribute.
    pub(crate) fn next_jsx_attribute_value(&mut self) -> Token {
        let kind = self.read_next_jsx_attribute_value();
        self.finish_next(kind)
    }

    // This is a workaround for a problem where `next_token` is not inlined in lexer benchmark.
    // It contains exactly the same code as `next_token` above, as both only call `next_token_impl`.
    #[cfg(feature = "benchmarking")]
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn next_token_for_benchmarks(&mut self) -> Token {
        self.next_token_impl()
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
    #[inline] // Make sure is inlined into `first_token`
    fn read_next_token(&mut self) -> (Kind, SourcePosition<'a>) {
        self.trivia_builder.clear_annotation_comments();
        self.read_next_token_after_trivia()
    }

    /// Same as `read_next_token`, but keeps any annotation comments found before the token so far.
    #[inline]
    fn read_next_token_after_trivia(&mut self) -> (Kind, SourcePosition<'a>) {
        let end_pos = self.source.end();
        let mut pos = self.source.position();
        loop {
            // Single spaces between tokens are common, so consume a space before processing the next token.
            // Can only do this if there are at least 2 bytes left in source.
            // If there aren't 2 bytes left, delegate to `read_next_token_at_end` (cold branch).
            // SAFETY: `source.end()` is always equal to or after `source.position()`
            let remaining_bytes = unsafe { end_pos.offset_from(pos) };
            if remaining_bytes >= 2 {
                // SAFETY: There are at least 2 bytes remaining in source.
                let mut byte = unsafe { pos.read() };

                // Only update the cursor and read again when consuming a space.
                if byte == b' ' {
                    // SAFETY: At least 2 bytes remain, so advancing past one ASCII space
                    // leaves at least one byte to read.
                    pos = unsafe { pos.add(1) };
                    self.source.set_position(pos);
                    // SAFETY: At least one byte remains after the space.
                    byte = unsafe { pos.read() };
                }

                // Set token start
                let offset = self.source.offset_of(pos);
                self.token_start = offset;

                if let Some(kind) = byte_handlers::single_byte_token(byte) {
                    // SAFETY: Single-byte tokens are ASCII, and at least 1 byte remains after `pos`
                    let end = unsafe { pos.add(1) };
                    self.source.set_position(end);
                    return (kind, end);
                }

                let byte_handlers = self.config.byte_handlers();
                let kind;
                // SAFETY: `byte` is byte value at current position in source
                (kind, pos) = unsafe { byte_handlers[byte as usize](self, pos) };
                if kind != Kind::Skip {
                    return (kind, pos);
                }
            } else {
                // Only 0 or 1 bytes left in source.
                // Delegate to `#[cold]` function as this is a very rare case.
                let kind = self.read_next_token_at_end();
                return (kind, self.source.position());
            }
        }
    }

    /// Cold path for reading next token where only 0 or 1 bytes are left in source.
    #[inline(never)]
    #[cold]
    fn read_next_token_at_end(&mut self) -> Kind {
        let offset = self.offset();
        self.token_start = offset;

        if let Some(byte) = self.peek_byte() {
            // SAFETY: `byte` is byte value at current position in source
            let kind = unsafe { self.handle_byte(byte) };
            if kind != Kind::Skip {
                return kind;
            }
            // Last byte was whitespace/line break (`Kind::Skip`), so now at EOF
            self.token_start = offset + 1;
        }

        Kind::Eof
    }

    #[inline]
    fn read_next_jsx_attribute_value(&mut self) -> Kind {
        self.trivia_builder.clear_annotation_comments();

        let end_pos = self.source.end();
        loop {
            let mut pos = self.source.position();
            // SAFETY: `source.end()` is always equal to or after `source.position()`
            let remaining_bytes = unsafe { end_pos.offset_from(pos) };
            if remaining_bytes >= 2 {
                // SAFETY: There are at least 2 bytes remaining in source.
                let mut byte = unsafe { pos.read() };

                // Only update the cursor and read again when consuming a space.
                if byte == b' ' {
                    // SAFETY: At least 2 bytes remain, so advancing past one ASCII space
                    // leaves at least one byte to read.
                    pos = unsafe { pos.add(1) };
                    self.source.set_position(pos);
                    // SAFETY: At least one byte remains after the space.
                    byte = unsafe { pos.read() };
                }

                let offset = self.source.offset_of(pos);
                self.token_start = offset;

                if matches!(byte, b'"' | b'\'') {
                    // SAFETY: `byte` is the current ASCII quote delimiter.
                    return unsafe { self.read_jsx_string_literal(byte) };
                }

                // SAFETY: `byte` is byte value at current position in source.
                let kind = unsafe { self.handle_byte(byte) };
                if kind != Kind::Skip {
                    return kind;
                }
            } else {
                return self.read_next_jsx_attribute_value_at_end();
            }
        }
    }

    #[inline(never)]
    #[cold]
    fn read_next_jsx_attribute_value_at_end(&mut self) -> Kind {
        let offset = self.offset();
        self.token_start = offset;

        if let Some(byte) = self.peek_byte() {
            if matches!(byte, b'"' | b'\'') {
                // SAFETY: `byte` is the current ASCII quote delimiter.
                return unsafe { self.read_jsx_string_literal(byte) };
            }

            // SAFETY: `byte` is byte value at current position in source.
            let kind = unsafe { self.handle_byte(byte) };
            if kind != Kind::Skip {
                return kind;
            }
            self.token_start = offset + 1;
        }

        Kind::Eof
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
