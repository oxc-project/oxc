// All methods are `#[inline(always)]` to ensure compiler removes dead code resulting from static values
#![expect(clippy::inline_always)]

use std::ops::Index;

use crate::lexer::{ByteHandler, ByteHandlers, byte_handler_tables};

/// Parser config.
///
/// The purpose of parser config (as opposed to `ParseOptions`) is to allow setting options at either
/// compile time or runtime.
///
/// 3 configs are provided:
/// * [`NoTokensParserConfig`]: Parse without tokens, static (default)
/// * [`TokensParserConfig`]: Parse with tokens, static
/// * [`RuntimeParserConfig`]: Parse with or without tokens, decided at runtime
///
/// The trade-off is:
///
/// * The 2 static configs will produce better performance, because compiler can remove code that relates
///   to the other option as dead code, and remove branches.
///
/// * The runtime config will produce a smaller binary than using 2 different configs in the same application,
///   which would cause 2 polymorphic variants of the parser to be compiled.
///
/// Advised usage:
/// * If your application uses only a specific set of options, use a static config.
/// * If your application uses multiple sets of options, probably a runtime config is preferable.
///
/// At present the only option controlled by `ParserConfig` is whether to parse with or without tokens.
/// Other options will be added in future.
///
/// You can also create your own config by implementing [`ParserConfig`] on a type.
pub trait ParserConfig: Default {
    /// Type of [`LexerConfig`] this [`ParserConfig`] uses.
    type LexerConfig: LexerConfig;

    /// Get [`LexerConfig`] for this [`ParserConfig`].
    fn lexer_config(&self) -> Self::LexerConfig;
}

/// Parser config for parsing without tokens (default).
///
/// See [`ParserConfig`] for more details.
#[derive(Copy, Clone, Default)]
pub struct NoTokensParserConfig;

impl ParserConfig for NoTokensParserConfig {
    type LexerConfig = NoTokensLexerConfig;

    #[inline(always)]
    fn lexer_config(&self) -> NoTokensLexerConfig {
        NoTokensLexerConfig
    }
}

/// Parser config for parsing with tokens.
///
/// See [`ParserConfig`] for more details.
#[derive(Copy, Clone, Default)]
pub struct TokensParserConfig;

impl ParserConfig for TokensParserConfig {
    type LexerConfig = TokensLexerConfig;

    #[inline(always)]
    fn lexer_config(&self) -> TokensLexerConfig {
        TokensLexerConfig
    }
}

/// Parser config for parsing with / without tokens, decided at runtime.
///
/// See [`ParserConfig`] for more details.
#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct RuntimeParserConfig {
    lexer_config: RuntimeLexerConfig,
}

impl RuntimeParserConfig {
    #[inline(always)]
    pub fn new(tokens: bool) -> Self {
        Self { lexer_config: RuntimeLexerConfig::new(tokens) }
    }
}

impl ParserConfig for RuntimeParserConfig {
    type LexerConfig = RuntimeLexerConfig;

    #[inline(always)]
    fn lexer_config(&self) -> RuntimeLexerConfig {
        self.lexer_config
    }
}

/// Lexer config.
///
/// See [`ParserConfig`] for more details.
///
/// We have to define a different byte handler table for each config, as byte handler functions
/// are generic over the [`LexerConfig`].
///
/// Byte handler tables are defined within lexer (`lexer/byte_handlers.rs`).
/// We need them to be defined as `static` items, for performance.
/// 1. `static` ensures that only 1 copy of each byte handler table appears in the binary
///    (even if `Lexer::handle_byte` is inlined in multiple places).
/// 2. Indexing into byte handlers table is an extremely hot path, and even 1 extra indirection (pointer chasing)
///    has a sizeable impact on performance.
///
/// An associated type and `byte_handlers` method is the only way to make this work with Rust's type system
/// and borrow checker.
pub trait LexerConfig: Default {
    /// Byte handlers table type.
    type ByteHandlers: Index<usize, Output = ByteHandler<Self>>;

    /// `true` if [`tokens`] method returns a static value.
    ///
    /// This const is useful in situations where code can be more efficient
    /// when the return value of [`tokens`] method is known at compile time.
    ///
    /// Implementors of this trait MUST only set this const to `true` if [`tokens`] method returns a static value.
    ///
    /// [`tokens`]: LexerConfig::tokens
    const TOKENS_METHOD_IS_STATIC: bool;

    /// Returns `true` if tokens are enabled.
    fn tokens(&self) -> bool;

    /// Get byte handlers table to use in lexer with this config.
    fn byte_handlers(&self) -> &Self::ByteHandlers;
}

/// Lexer config for lexing without tokens.
#[derive(Copy, Clone, Default)]
pub struct NoTokensLexerConfig;

impl LexerConfig for NoTokensLexerConfig {
    type ByteHandlers = ByteHandlers<Self>;

    const TOKENS_METHOD_IS_STATIC: bool = true;

    #[inline(always)]
    fn tokens(&self) -> bool {
        false
    }

    #[inline(always)]
    fn byte_handlers(&self) -> &Self::ByteHandlers {
        &byte_handler_tables::NO_TOKENS
    }
}

/// Lexer config for parsing with tokens.
#[derive(Copy, Clone, Default)]
pub struct TokensLexerConfig;

impl LexerConfig for TokensLexerConfig {
    type ByteHandlers = ByteHandlers<Self>;

    const TOKENS_METHOD_IS_STATIC: bool = true;

    #[inline(always)]
    fn tokens(&self) -> bool {
        true
    }

    #[inline(always)]
    fn byte_handlers(&self) -> &Self::ByteHandlers {
        &byte_handler_tables::WITH_TOKENS
    }
}

/// Lexer config for lexing with / without tokens, decided at runtime.
#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct RuntimeLexerConfig {
    tokens: bool,
}

impl RuntimeLexerConfig {
    #[inline(always)]
    pub fn new(tokens: bool) -> Self {
        Self { tokens }
    }
}

impl LexerConfig for RuntimeLexerConfig {
    type ByteHandlers = ByteHandlers<Self>;

    const TOKENS_METHOD_IS_STATIC: bool = false;

    #[inline(always)]
    fn tokens(&self) -> bool {
        self.tokens
    }

    #[inline(always)]
    fn byte_handlers(&self) -> &Self::ByteHandlers {
        &byte_handler_tables::RUNTIME_TOKENS
    }
}
