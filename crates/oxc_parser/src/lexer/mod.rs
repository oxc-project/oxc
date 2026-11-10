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
use token::ScanState;
pub use token::Token;

use source::{Source, SourcePosition};
use trivia_builder::{AnnotationComments, TriviaBuilder};

#[derive(Debug)]
pub struct LexerCheckpoint<'a> {
    standalone_previous: Token,
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

    /// Previous token for standalone lexer entrypoints.
    /// Parser advancement supplies its current token directly instead.
    standalone_previous: Token,

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
            standalone_previous: Token::undetermined(),
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
        let errors_snapshot = if self.errors.is_empty() {
            ErrorSnapshot::Empty
        } else {
            ErrorSnapshot::Count(self.errors.len())
        };

        LexerCheckpoint {
            standalone_previous: self.standalone_previous,
            source_position: self.source.position(),
            errors_snapshot,
            tokens_len: self.tokens.len(),
            annotation_comments: self.trivia_builder.annotation_comments,
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
            standalone_previous: self.standalone_previous,
            source_position: self.source.position(),
            errors_snapshot,
            tokens_len: self.tokens.len(),
            annotation_comments: self.trivia_builder.annotation_comments,
        }
    }

    /// Rewinds the lexer to the same state as when the passed in `checkpoint` was created.
    pub fn rewind(&mut self, checkpoint: LexerCheckpoint<'a>) {
        self.standalone_previous = checkpoint.standalone_previous;

        match checkpoint.errors_snapshot {
            ErrorSnapshot::Empty => self.errors.clear(),
            ErrorSnapshot::Count(len) => self.errors.truncate(len),
            ErrorSnapshot::Full(errors) => self.errors = errors,
        }

        self.tokens.truncate(checkpoint.tokens_len);
        self.source.set_position(checkpoint.source_position);
        self.trivia_builder.annotation_comments = checkpoint.annotation_comments;
    }

    #[cfg(any(test, feature = "benchmarking"))]
    pub fn peek_token(&mut self) -> Token {
        let checkpoint = self.checkpoint();
        let token = self.next_token();
        self.rewind(checkpoint);
        token
    }

    pub(crate) fn peek_token_after(&mut self, previous: &Token) -> Token {
        let checkpoint = self.checkpoint();
        let mut token = *previous;
        self.advance_normal(&mut token);
        self.rewind(checkpoint);
        token
    }

    /// Initialize comment attachment only when this token boundary contains a comment.
    fn begin_comment(&mut self, state: &mut ScanState, previous: &Token) {
        if !state.has_trivia() {
            self.trivia_builder.begin_boundary(*previous, state.is_on_new_line());
            state.set_has_trivia();
        }
    }

    /// Read first token in file.
    pub fn first_token(&mut self) -> Token {
        let previous = self.standalone_previous;
        let previous = &previous;
        // The first token is at the start of the file, so it is always on a new line.
        let state = &mut ScanState::new_on_new_line();
        // HashbangComment ::
        //     `#!` SingleLineCommentChars?
        let token = if let Some([b'#', b'!']) = self.peek_2_bytes() {
            // SAFETY: Next 2 bytes are `#!`
            let kind = unsafe { self.read_hashbang_comment() };
            // Hashbangs are not included in tokens
            self.finish_next_inner::<true>(*state, kind, FinishTokenMode::Discard)
        } else {
            let kind = self.read_next_token(state, previous);
            self.finish_next(*state, kind)
        };
        self.standalone_previous = token;
        token
    }

    /// Read next token in file.
    /// Use `first_token` for first token, and this method for all further tokens.
    #[cfg(any(test, feature = "benchmarking"))]
    pub fn next_token(&mut self) -> Token {
        let previous = self.standalone_previous;
        let previous = &previous;
        let state = &mut ScanState::default();
        let kind = self.read_next_token(state, previous);
        let token = self.finish_next(*state, kind);
        self.standalone_previous = token;
        token
    }

    /// Advance the parser's current token without storing a second copy in the lexer.
    #[inline(never)]
    pub(crate) fn advance_normal(&mut self, current: &mut Token) {
        let mut state = ScanState::default();
        let mut pos = self.source.position();
        let end = self.source.end();
        if pos == end {
            state.set_start(self.offset());
            *current = self.finish_next(state, Kind::Eof);
            return;
        }
        // SAFETY: Source cursor is before the end.
        let mut byte = unsafe { pos.read() };
        if byte == b' ' {
            // SAFETY: The consumed ASCII space is before the end.
            pos = unsafe { pos.add(1) };
            self.source.set_position(pos);
            if pos == end {
                state.set_start(self.offset());
                *current = self.finish_next(state, Kind::Eof);
                return;
            }
            // SAFETY: The cursor remains before the end.
            byte = unsafe { pos.read() };
        }
        if matches!(byte, b'\n' | b'\r') {
            return self.advance_after_line_break(current);
        }
        state.set_start(self.source.offset_of(pos));
        // SAFETY: `byte` is at the source cursor, which is before the end.
        unsafe { self.advance_at(state.start(), current, byte) };
    }

    /// Keep newline scanning out of ordinary token dispatch so it can remain a tail call.
    #[inline(never)]
    fn advance_after_line_break(&mut self, current: &mut Token) {
        let mut state = ScanState::default();
        // No comment boundary has started. This consumes ordinary line breaks and
        // indentation together and retains the newline flag for ASI and comments.
        self.line_break_handler(&mut state);
        state.set_start(self.offset());
        let kind = if let Some(byte) = self.source.peek_byte() {
            // SAFETY: `peek_byte` returned the byte at the non-exhausted cursor.
            unsafe { self.handle_byte(&mut state, current, byte) }
        } else {
            Kind::Eof
        };
        self.complete_or_continue(state, kind, current);
    }

    /// Read the next token after `=` in a JSX attribute.
    pub(crate) fn next_jsx_attribute_value(&mut self, previous: &Token) -> Token {
        let state = &mut ScanState::default();
        let kind = self.read_next_jsx_attribute_value(state, previous);
        self.finish_next(*state, kind)
    }

    // This is a workaround for a problem where `next_token` is not inlined in lexer benchmark.
    // Must be kept in sync with `next_token` above, and contain exactly the same code.
    #[cfg(feature = "benchmarking")]
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn next_token_for_benchmarks(&mut self) -> Token {
        let previous = self.standalone_previous;
        let previous = &previous;
        let state = &mut ScanState::default();
        let kind = self.read_next_token(state, previous);
        let token = self.finish_next(*state, kind);
        self.standalone_previous = token;
        token
    }

    #[inline]
    fn finish_next(&mut self, state: ScanState, kind: Kind) -> Token {
        self.finish_next_inner::<true>(state, kind, FinishTokenMode::Push)
    }

    #[inline]
    fn finish_next_retokenized(&mut self, state: ScanState, kind: Kind) -> Token {
        self.finish_next_inner::<true>(state, kind, FinishTokenMode::Replace)
    }

    // `#[inline(always)]` to ensure is inlined into `finish_next` and `finish_next_retokenized`,
    // so that `mode` is statically known
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn finish_next_inner<const TRIVIA: bool>(
        &mut self,
        state: ScanState,
        kind: Kind,
        mode: FinishTokenMode,
    ) -> Token {
        let token = state.finish(kind, self.offset());
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
        if TRIVIA && state.has_trivia() {
            self.trivia_builder.handle_token(token);
        }
        token
    }

    /// Finish a re-lexed token used only for parser disambiguation.
    /// This must not mutate the externally collected token stream.
    fn finish_re_lex(&self, state: ScanState, kind: Kind) -> Token {
        state.finish(kind, self.offset())
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
    fn unterminated_range(&self, state: ScanState) -> Span {
        Span::new(state.start(), self.offset())
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
    #[inline(never)]
    fn read_next_token(&mut self, state: &mut ScanState, previous: &Token) -> Kind {
        let end_pos = self.source.end();
        loop {
            // Single spaces between tokens are common, so consume a space before processing the next token.
            // Can only do this if there are at least 2 bytes left in source.
            // If there aren't 2 bytes left, delegate to `read_next_token_at_end` (cold branch).
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

                // Set token start
                let offset = self.source.offset_of(pos);
                state.set_start(offset);

                // SAFETY: `byte` is byte value at current position in source
                let kind = unsafe { self.handle_byte(state, previous, byte) };
                if kind != Kind::Skip {
                    return kind;
                }
            } else {
                // Only 0 or 1 bytes left in source.
                // Delegate to `#[cold]` function as this is a very rare case.
                return self.read_next_token_at_end(state, previous);
            }
        }
    }

    /// Cold path for reading next token where only 0 or 1 bytes are left in source.
    #[inline(never)]
    #[cold]
    fn read_next_token_at_end(&mut self, state: &mut ScanState, previous: &Token) -> Kind {
        let offset = self.offset();
        state.set_start(offset);

        if let Some(byte) = self.peek_byte() {
            // SAFETY: `byte` is byte value at current position in source
            let kind = unsafe { self.handle_byte(state, previous, byte) };
            if kind != Kind::Skip {
                return kind;
            }
            // Last byte was whitespace/line break (`Kind::Skip`), so now at EOF
            state.set_start(offset + 1);
        }

        Kind::Eof
    }

    #[inline]
    fn read_next_jsx_attribute_value(&mut self, state: &mut ScanState, previous: &Token) -> Kind {
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
                state.set_start(offset);

                if matches!(byte, b'"' | b'\'') {
                    // SAFETY: `byte` is the current ASCII quote delimiter.
                    return unsafe { self.read_jsx_string_literal(state, byte) };
                }

                // SAFETY: `byte` is byte value at current position in source.
                let kind = unsafe { self.handle_byte(state, previous, byte) };
                if kind != Kind::Skip {
                    return kind;
                }
            } else {
                return self.read_next_jsx_attribute_value_at_end(state, previous);
            }
        }
    }

    #[inline(never)]
    #[cold]
    fn read_next_jsx_attribute_value_at_end(
        &mut self,
        state: &mut ScanState,
        previous: &Token,
    ) -> Kind {
        let offset = self.offset();
        state.set_start(offset);

        if let Some(byte) = self.peek_byte() {
            if matches!(byte, b'"' | b'\'') {
                // SAFETY: `byte` is the current ASCII quote delimiter.
                return unsafe { self.read_jsx_string_literal(state, byte) };
            }

            // SAFETY: `byte` is byte value at current position in source.
            let kind = unsafe { self.handle_byte(state, previous, byte) };
            if kind != Kind::Skip {
                return kind;
            }
            state.set_start(offset + 1);
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
