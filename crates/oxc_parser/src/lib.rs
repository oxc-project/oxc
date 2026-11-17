//! Oxc Parser for JavaScript and TypeScript
//!
//! Oxc's [`Parser`] has full support for
//! - The latest stable ECMAScript syntax
//! - TypeScript
//! - JSX and TSX
//! - [Stage 3 Decorators](https://github.com/tc39/proposal-decorator-metadata)
//!
//! # Usage
//!
//! The parser has a minimal API with three inputs (a [memory arena](oxc_allocator::Allocator), a
//! source string, and a [`SourceType`]) and one return struct (a [ParserReturn]).
//! Create a [`Source`] from the inputs, and call [`Source::parse`].
//!
//! ```rust,ignore
//! let parser_return = Source::from_str(&source_text, source_type, &allocator).parse();
//! ```
//!
//! # Abstract Syntax Tree (AST)
//! Oxc's AST is located in a separate [`oxc_ast`] crate. You can find type definitions for AST
//! nodes [here][`oxc_ast::ast`].
//!
//! # Performance
//!
//! The following optimization techniques are used:
//! * AST is allocated in a memory arena ([oxc_allocator](https://docs.rs/oxc_allocator)) for fast AST drop
//! * [`oxc_span::Span`] offsets uses `u32` instead of `usize`
//! * Scope binding, symbol resolution and complicated syntax errors are not done in the parser,
//! they are delegated to the [semantic analyzer](https://docs.rs/oxc_semantic)
//!
//! <div class="warning">
//! Because [`oxc_span::Span`] uses `u32` instead of `usize`, Oxc can only parse files up
//! to 4 GiB in size. This shouldn't be a limitation in almost all cases.
//! </div>
//!
//! # Examples
//!
//! <https://github.com/oxc-project/oxc/blob/main/crates/oxc_parser/examples/parser.rs>
//!
//! ```rust,ignore
#![doc = include_str!("../examples/parser.rs")]
//! ```
//!
//! ### Parsing TSX
//! ```rust,ignore
#![doc = include_str!("../examples/parser_tsx.rs")]
//! ```
//!
//! # Visitor
//!
//! See [`Visit`](http://docs.rs/oxc_ast_visit) and [`VisitMut`](http://docs.rs/oxc_ast_visit).
//!
//! # Visiting without a visitor
//!
//! For ad-hoc tasks, the semantic analyzer can be used to get a parent pointing tree with untyped nodes,
//! the nodes can be iterated through a sequential loop.
//!
//! ```rust,ignore
//! for node in semantic.nodes().iter() {
//!     match node.kind() {
//!         // check node
//!     }
//! }
//! ```
//!
//! See [full linter example](https://github.com/Boshen/oxc/blob/ab2ef4f89ba3ca50c68abb2ca43e36b7793f3673/crates/oxc_linter/examples/linter.rs#L38-L39)

use std::any::Any;

mod ast_builder;
pub mod config;
mod context;
mod cursor;
mod error_handler;
mod list;
mod modifiers;
mod module_record;
mod source;
mod state;
mod top_level_await;

mod js;
mod jsx;
mod ts;

mod diagnostics;

// Expose lexer only in benchmarks
#[cfg(not(feature = "benchmarking"))]
mod lexer;
#[cfg(feature = "benchmarking")]
#[doc(hidden)]
pub mod lexer;

use oxc_allocator::{Allocator, ArenaBox, ArenaVec, Dummy, GetAllocator};
use oxc_ast::{
    ast::{Expression, Program},
    builder::{AstBuilder, GetAstBuilder},
};
use oxc_data_structures::brands::{BrandOrigin, BrandScope, Unbranded};
use oxc_diagnostics::Diagnostics;
use oxc_span::{SourceType, Span};
use oxc_syntax::module_record::ModuleRecord;

use crate::{
    ast_builder::ParserAstBuilder,
    config::{NoTokensParserConfig, ParserConfig, RuntimeParserConfig, TokensParserConfig},
    context::{Context, StatementContext},
    diagnostics::ParserDiagnostic,
    error_handler::FatalError,
    lexer::Lexer,
    module_record::ModuleRecordBuilder,
    source::ValidSource,
    state::ParserState,
};
pub use crate::{
    lexer::{Kind, Token},
    source::{Source, SourceFileError},
};

/// Maximum length of source which can be parsed (in bytes).
/// ~4 GiB on 64-bit systems, ~2 GiB on 32-bit systems.
//
// Length is constrained by 2 factors:
// 1. `Span`'s `start` and `end` are `u32`s, which limits length to `u32::MAX` bytes.
// 2. Rust's allocator APIs limit allocations to `isize::MAX`.
//    https://doc.rust-lang.org/std/alloc/struct.Layout.html#method.from_size_align
//
// The limit is 256 bytes below Rust's limit:
//
// 1. `oxc_lexer` will require source text to be followed by 64 bytes of padding.
//    This means *padded* length can fit in a `u32`.
//    256 instead of 64 to leave headroom, in case the padding requirement expands in future.
// 2. Counts of tokens, comments and errors are all bounded by source length plus a small constant,
//    so they can be stored as `u32`s without any possibility of overflow.
// 3. No real `Span` can have `start` or `end` of `u32::MAX`, so such a `Span` can be used as a sentinel.
//    `oxc_transformer`'s styled-components plugin relies on this.
// 4. Source is allocated in the arena, which adds the overhead of the allocator's chunk footer
//    (metadata). The total allocation still needs to be below the system limit.
const MAX_LEN: usize = cfg_select! {
    target_pointer_width = "64" => u32::MAX as usize - PADDING_HEADROOM,
    _ => isize::MAX as usize - PADDING_HEADROOM,
};

const PADDING_HEADROOM: usize = 256;
const _: () = assert!(PADDING_HEADROOM >= Source::PADDING);

static MAX_LEN_ERROR: &str = cfg_select! {
    target_pointer_width = "64" => "Source length exceeds 4 GiB limit",
    _ => "Source length exceeds 2 GiB limit",
};

/// Return value of [`Parser::parse`] and [`Source::parse`], consisting of AST, errors and comments.
///
/// ## AST Validity
///
/// [`program`] will always contain a structurally valid AST, even if there are syntax errors.
/// However, the AST may be semantically invalid. To ensure a valid AST,
/// 1. Check that [`diagnostics`] is empty
/// 2. Run semantic analysis with [syntax error checking
///    enabled](https://docs.rs/oxc_semantic/latest/oxc_semantic/struct.SemanticBuilder.html#method.with_check_syntax_error)
///
/// ## Errors
///
/// Oxc's [`Parser`] is able to recover from some syntax errors and continue parsing.
/// When this happens:
///
/// 1. [`diagnostics`] will be non-empty
/// 2. [`program`] will contain a full AST
/// 3. [`fatal_error`] will be false
///
/// When the parser cannot recover, it will abort and terminate parsing early.
/// [`program`] will be empty and [`fatal_error`] will be `true`.
///
/// [`program`]: ParserReturn::program
/// [`diagnostics`]: ParserReturn::diagnostics
/// [`fatal_error`]: ParserReturn::fatal_error
#[non_exhaustive]
pub struct ParserReturn<'a> {
    /// The parsed AST.
    ///
    /// Will be empty (e.g. no statements, directives, etc) if the parser encountered a fatal error.
    ///
    /// Every node has a unique node ID within this AST. Program has ID zero;
    /// other IDs follow construction order and may have gaps from discarded nodes.
    /// Semantic analysis replaces these with its own dense node IDs.
    ///
    /// ## Validity
    /// It is possible for the AST to be present and semantically invalid. This will happen if
    /// 1. The [`Parser`] encounters a recoverable syntax error
    /// 2. The logic for checking the violation is in the semantic analyzer
    ///
    /// To ensure a valid AST, check that [`diagnostics`](ParserReturn::diagnostics) is empty. Then, run
    /// semantic analysis with syntax error checking enabled.
    pub program: Program<'a>,

    /// See <https://tc39.es/ecma262/#sec-abstract-module-records>
    pub module_record: ModuleRecord<'a>,

    /// Syntax errors encountered while parsing.
    ///
    /// This list is not comprehensive. Oxc offloads more-expensive checks to [semantic
    /// analysis](https://docs.rs/oxc_semantic), which can be enabled using
    /// [`SemanticBuilder::with_check_syntax_error`](https://docs.rs/oxc_semantic/latest/oxc_semantic/struct.SemanticBuilder.html#method.with_check_syntax_error).
    pub diagnostics: Diagnostics,

    /// Irregular whitespaces for `Oxlint`
    pub irregular_whitespaces: Box<[Span]>,

    /// Lexed tokens in source order.
    ///
    /// Tokens are only collected when tokens are enabled in [`ParserConfig`].
    pub tokens: ArenaVec<'a, Token>,

    /// Whether the parser encountered a fatal error and terminated early.
    ///
    /// If `false`, either parsing was successful, or parsing was able to recover from a syntax error.
    /// [`diagnostics`] may still contain errors, indicating that the program is not syntactically valid,
    /// but parser was able to recover enough to produce an AST.
    ///
    /// If `true`, parser was unable to parse the source.
    /// [`program`] will be empty and [`diagnostics`] will contain at least one error.
    ///
    /// [`program`]: ParserReturn::program
    /// [`diagnostics`]: ParserReturn::diagnostics
    pub fatal_error: bool,

    /// Whether the file is [flow](https://flow.org).
    pub is_flow_language: bool,
}

// `ParserReturn` is the output of `ParserImpl::parse`.
// It's created inside a closure, with the closure's `'brand` lifetime.
// `BrandScope::extend_lifetime` changes that lifetime to the lifetime of the allocator
// and source text.
// That's only allowed for types which implement `Unbranded`.
//
// SAFETY: `ParserReturn` meets `Unbranded`'s contract.
// - It doesn't name `BrandOrigin` or `Brand`, or any type which does. In this crate,
//   those types are `LexerSource` and `SourcePosition`, and the types which contain them
//   (`Lexer`, `LexerCheckpoint`, `ParserCheckpoint`, `ParserState`, `ParserImpl`).
// - It doesn't contain a `dyn Trait + 'a`.
// - Its lifetime is the lifetime of the arena. Nothing uses it as a brand.
unsafe impl Unbranded for ParserReturn<'_> {
    type WithLifetime<'a> = ParserReturn<'a>;
}

/// Parse options
///
/// You may provide options using [`Source::parse_with_options`].
#[derive(Debug, Clone, Copy)]
pub struct ParseOptions {
    /// Whether to parse regular expressions or not.
    ///
    /// Default: `false`
    #[cfg(feature = "regular_expression")]
    pub parse_regular_expression: bool,

    /// Allow [`return`] statements outside of functions.
    ///
    /// By default, a return statement at the top level raises an error (`false`).
    ///
    /// Default: `false`
    ///
    /// [`return`]: oxc_ast::ast::ReturnStatement
    pub allow_return_outside_function: bool,

    /// Emit [`ParenthesizedExpression`]s and [`TSParenthesizedType`] in AST.
    ///
    /// If this option is `true`, parenthesized expressions are represented by
    /// (non-standard) [`ParenthesizedExpression`] and [`TSParenthesizedType`] nodes
    /// that have a single `expression` property containing the expression inside parentheses.
    ///
    /// Default: `true`
    ///
    /// [`ParenthesizedExpression`]: oxc_ast::ast::ParenthesizedExpression
    /// [`TSParenthesizedType`]: oxc_ast::ast::TSParenthesizedType
    pub preserve_parens: bool,

    /// Allow V8 runtime calls in the AST.
    /// See: [V8's Parser::ParseV8Intrinsic](https://chromium.googlesource.com/v8/v8/+/35a14c75e397302655d7b3fbe648f9490ae84b7d/src/parsing/parser.cc#4811).
    ///
    /// Default: `false`
    ///
    /// [`V8IntrinsicExpression`]: oxc_ast::ast::V8IntrinsicExpression
    pub allow_v8_intrinsics: bool,

    /// Precompute hashes for identifier names (`Ident`s) in the AST.
    ///
    /// Precomputed hashes speed up `Ident`-keyed hash map operations in semantic analysis
    /// and later compilation stages, at a small cost to parsing speed.
    ///
    /// Only disable this for parse-only pipelines that never rely on `Ident` hashing, such as
    /// parse + serialize or formatting. Unhashed `Ident`s do not compare equal to hashed ones,
    /// so semantic analysis and anything else relying on `Ident` hashing must not run on an AST
    /// parsed with this option disabled.
    ///
    /// Default: `true`
    pub enable_ident_hashes: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            #[cfg(feature = "regular_expression")]
            parse_regular_expression: false,
            allow_return_outside_function: false,
            preserve_parens: true,
            allow_v8_intrinsics: false,
            enable_ident_hashes: true,
        }
    }
}

/// Recursive Descent Parser for ECMAScript and TypeScript
///
/// See [`Parser::parse`] for entry function.
pub struct Parser<'a, C: ParserConfig = NoTokensParserConfig> {
    source: Source<'a>,
    options: ParseOptions,
    config: C,
}

impl<'a> Parser<'a> {
    /// Create a new [`Parser`]
    ///
    /// `source_text` is copied into `allocator`.
    /// [`Program::source_text`] and all strings in the AST refer to that copy.
    ///
    /// # Parameters
    /// - `allocator`: [Memory arena](oxc_allocator::Allocator) for allocating AST nodes
    /// - `source_text`: Source code to parse
    /// - `source_type`: Source type (e.g. JavaScript, TypeScript, JSX, ESM Module, Script)
    ///
    /// # Panics
    ///
    /// Panics if `source_text.len() + Source::PADDING` exceeds `isize::MAX`.
    pub fn new(allocator: &'a Allocator, source_text: &'a str, source_type: SourceType) -> Self {
        let source = Source::from_str(source_text, source_type, allocator);
        Self { source, options: ParseOptions::default(), config: NoTokensParserConfig }
    }
}

impl<'a, C: ParserConfig> Parser<'a, C> {
    /// Set parse options
    #[must_use]
    pub fn with_options(mut self, options: ParseOptions) -> Self {
        self.options = options;
        self
    }

    /// Set parser config.
    ///
    /// See [`ParserConfig`] for more details.
    #[must_use]
    pub fn with_config<Config: ParserConfig>(self, config: Config) -> Parser<'a, Config> {
        Parser { source: self.source, options: self.options, config }
    }

    /// Main entry point
    ///
    /// Returns an empty `Program` on unrecoverable error,
    /// Recoverable errors are stored inside `errors`.
    ///
    /// See the [module-level documentation](crate) for examples and more information.
    #[inline]
    pub fn parse(self) -> ParserReturn<'a> {
        self.source.parse_with_config(self.options, self.config)
    }

    /// Parse a single [`Expression`].
    ///
    /// Every node receives a unique, nonzero node ID. IDs follow construction
    /// order and may have gaps from discarded nodes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxc_allocator::Allocator;
    /// use oxc_ast::ast::Expression;
    /// use oxc_parser::Parser;
    /// use oxc_span::SourceType;
    ///
    /// let src = "1 + 2";
    /// let allocator = Allocator::new();
    /// let source_type = SourceType::default();
    ///
    /// let expr: Expression<'_> = Parser::new(&allocator, src, source_type).parse_expression().unwrap();
    /// ```
    ///
    /// # Errors
    /// If the source code being parsed has syntax errors.
    #[inline]
    pub fn parse_expression(self) -> Result<Expression<'a>, Diagnostics> {
        self.source.parse_expression_with_config(self.options, self.config)
    }
}

impl<'a> Source<'a> {
    /// Parse this [`Source`], with default [`ParseOptions`].
    ///
    /// Returns an empty `Program` on unrecoverable error.
    /// Recoverable errors are stored inside [`ParserReturn::diagnostics`].
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_allocator::Allocator;
    /// use oxc_parser::Source;
    /// use oxc_span::SourceType;
    ///
    /// let allocator = Allocator::default();
    /// let source = Source::from_str("let x = 1;", SourceType::mjs(), &allocator);
    /// let ret = source.parse();
    /// assert!(ret.diagnostics.is_empty());
    /// assert_eq!(ret.program.body.len(), 1);
    /// ```
    pub fn parse(self) -> ParserReturn<'a> {
        self.parse_with_options(ParseOptions::default())
    }

    /// Parse this [`Source`], with the provided [`ParseOptions`].
    ///
    /// Returns an empty `Program` on unrecoverable error.
    /// Recoverable errors are stored inside [`ParserReturn::diagnostics`].
    pub fn parse_with_options(self, options: ParseOptions) -> ParserReturn<'a> {
        self.parse_with_config(options, NoTokensParserConfig)
    }

    /// Parse this [`Source`], with the provided [`ParseOptions`] and [`ParserConfig`].
    ///
    /// Returns an empty `Program` on unrecoverable error.
    /// Recoverable errors are stored inside [`ParserReturn::diagnostics`].
    ///
    /// See [`ParserConfig`] for more details.
    //
    // # Implementation note
    //
    // The parser is generic over `C: ParserConfig`. Rust monomorphizes generic functions
    // per consuming crate (the legacy mangled name encodes the instantiating crate's
    // disambiguator, and `share-generics` is off at `opt-level >= 2`).
    // For a parser that's pulled in by ~15 crates in a real workspace, that means ~15 private
    // copies of every parser method in the final cdylib. None of them can be deduped by COMDAT
    // (different names) or fat LTO (slightly different inlining contexts).
    //
    // To avoid that, this method and `parse_expression_with_config` dispatch via `Any`
    // to a non-generic helper for each of the 3 known configs.
    //
    // - Each helper is non-generic, so it's emitted exactly once in `oxc_parser`'s rlib,
    //   and shared by all consumers.
    // - The helpers are `#[inline(never)]`, to prevent fat LTO from re-inlining the parser body
    //   across the rlib boundary, which would defeat the purpose.
    // - The `Any::is` / `downcast_ref` checks const-fold when `C` is concrete. The trait object
    //   is built from a known concrete type at every monomorphization site, so LLVM devirtualises
    //   the vtable call to `Any::type_id`, and folds the comparison. That leaves
    //   each monomorphization of the dispatch as a single call into the matching helper.
    // - A user-defined `ParserConfig` (rare) falls through to a generic body, which is
    //   monomorphized per consuming crate. That's the same cost the parser had without
    //   this pattern. It's just contained to the rare custom-config case.
    #[inline]
    pub fn parse_with_config<C: ParserConfig>(
        self,
        options: ParseOptions,
        config: C,
    ) -> ParserReturn<'a> {
        let config_any: &dyn Any = &config;
        if config_any.is::<NoTokensParserConfig>() {
            #[inline(never)]
            fn parse(source: Source<'_>, options: ParseOptions) -> ParserReturn<'_> {
                ParserImpl::parse(source, options, NoTokensParserConfig)
            }
            parse(self, options)
        } else if config_any.is::<TokensParserConfig>() {
            #[inline(never)]
            fn parse(source: Source<'_>, options: ParseOptions) -> ParserReturn<'_> {
                ParserImpl::parse(source, options, TokensParserConfig)
            }
            parse(self, options)
        } else if let Some(&config) = config_any.downcast_ref::<RuntimeParserConfig>() {
            #[inline(never)]
            fn parse(
                source: Source<'_>,
                options: ParseOptions,
                config: RuntimeParserConfig,
            ) -> ParserReturn<'_> {
                ParserImpl::parse(source, options, config)
            }
            parse(self, options, config)
        } else {
            // User-defined `ParserConfig`. Generic codegen here, monomorphized per consuming crate.
            // Users using custom configs would need to perform the monomorphization themselves.
            ParserImpl::parse(self, options, config)
        }
    }

    /// Parse this [`Source`] as a single [`Expression`], with default [`ParseOptions`].
    ///
    /// Every node receives a unique, nonzero node ID. IDs follow construction
    /// order and may have gaps from discarded nodes.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_allocator::Allocator;
    /// use oxc_ast::ast::Expression;
    /// use oxc_parser::Source;
    /// use oxc_span::SourceType;
    ///
    /// let allocator = Allocator::default();
    /// let source = Source::from_str("1 + 2", SourceType::default(), &allocator);
    /// let expr: Expression<'_> = source.parse_expression().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the source has syntax errors, or is not valid UTF-8.
    pub fn parse_expression(self) -> Result<Expression<'a>, Diagnostics> {
        self.parse_expression_with_options(ParseOptions::default())
    }

    /// Parse this [`Source`] as a single [`Expression`], with the provided [`ParseOptions`].
    ///
    /// # Errors
    ///
    /// Returns `Err` if the source has syntax errors, or is not valid UTF-8.
    pub fn parse_expression_with_options(
        self,
        options: ParseOptions,
    ) -> Result<Expression<'a>, Diagnostics> {
        self.parse_expression_with_config(options, NoTokensParserConfig)
    }

    /// Parse this [`Source`] as a single [`Expression`], with the provided [`ParseOptions`]
    /// and [`ParserConfig`].
    ///
    /// See [`ParserConfig`] for more details.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the source has syntax errors, or is not valid UTF-8.
    //
    // # Implementation note
    // Dispatches via `Any`, same as `parse_with_config` does.
    #[inline]
    pub fn parse_expression_with_config<C: ParserConfig>(
        self,
        options: ParseOptions,
        config: C,
    ) -> Result<Expression<'a>, Diagnostics> {
        let config_any: &dyn Any = &config;
        if config_any.is::<NoTokensParserConfig>() {
            #[inline(never)]
            fn parse(
                source: Source<'_>,
                options: ParseOptions,
            ) -> Result<Expression<'_>, Diagnostics> {
                ParserImpl::parse_expression(source, options, NoTokensParserConfig)
            }
            parse(self, options)
        } else if config_any.is::<TokensParserConfig>() {
            #[inline(never)]
            fn parse(
                source: Source<'_>,
                options: ParseOptions,
            ) -> Result<Expression<'_>, Diagnostics> {
                ParserImpl::parse_expression(source, options, TokensParserConfig)
            }
            parse(self, options)
        } else if let Some(&config) = config_any.downcast_ref::<RuntimeParserConfig>() {
            #[inline(never)]
            fn parse(
                source: Source<'_>,
                options: ParseOptions,
                config: RuntimeParserConfig,
            ) -> Result<Expression<'_>, Diagnostics> {
                ParserImpl::parse_expression(source, options, config)
            }
            parse(self, options, config)
        } else {
            ParserImpl::parse_expression(self, options, config)
        }
    }
}

/// Implementation of parser.
/// `Parser` is just a public wrapper, the guts of the implementation is in this type.
struct ParserImpl<'a, C: ParserConfig> {
    /// Options
    options: ParseOptions,

    pub(crate) lexer: Lexer<'a, C::LexerConfig>,

    /// SourceType: JavaScript or TypeScript, Script or Module, jsx support?
    source_type: SourceType,

    /// Source Code
    source_text: &'a str,

    /// All syntax errors from parser and lexer
    /// Note: favor adding to `Diagnostics` instead of raising Err
    ///
    /// Stored in deferred (unmaterialized) form and materialized once at parse exit.
    errors: Vec<ParserDiagnostic<'a>>,

    /// Errors that are only valid if the file is determined to be a Script (not a Module).
    /// For `ModuleKind::Unambiguous`, we defer ESM-only errors (like top-level await)
    /// until we know whether the file is ESM or Script.
    /// If resolved to Module → discard these errors.
    /// If resolved to Script → emit these errors.
    deferred_script_errors: Vec<ParserDiagnostic<'a>>,

    fatal_error: Option<FatalError<'a>>,

    /// The current parsing token
    token: Token,

    /// The end range of the previous token
    prev_token_end: u32,

    /// Parser state
    state: ParserState<'a>,

    /// Parsing context
    ctx: Context,

    /// Ast builder for creating AST nodes
    ast: ParserAstBuilder<'a>,

    /// Module Record Builder
    module_record_builder: ModuleRecordBuilder<'a>,

    /// Precomputed typescript detection
    is_ts: bool,
}

impl<'a, C: ParserConfig> ParserImpl<'a, C> {
    /// Create a new [`ParserImpl`].
    ///
    /// `brand_origin` is for the `LexerSource` in the `ParserImpl`'s [`Lexer`] (see [`brands`]).
    ///
    /// [`brands`]: oxc_data_structures::brands
    #[inline]
    #[expect(clippy::needless_pass_by_value)]
    fn new(
        source: ValidSource<'a>,
        options: ParseOptions,
        config: C,
        brand_origin: BrandOrigin<'a>,
    ) -> Self {
        let allocator = source.allocator();
        let source_text = source.source_text();
        let source_type = source.source_type();

        Self {
            options,
            lexer: Lexer::new(&source, config.lexer_config(), brand_origin),
            source_type,
            source_text,
            errors: vec![],
            deferred_script_errors: vec![],
            fatal_error: None,
            token: Token::default(),
            prev_token_end: 0,
            state: ParserState::new(allocator),
            ctx: Self::default_context(source_type, options),
            ast: ParserAstBuilder::new(allocator),
            module_record_builder: ModuleRecordBuilder::new(allocator, source_type),
            is_ts: source_type.is_typescript(),
        }
    }

    /// Main entry point
    ///
    /// Returns an empty `Program` on unrecoverable error,
    /// Recoverable errors are stored inside `errors`.
    fn parse(source: Source<'a>, options: ParseOptions, config: C) -> ParserReturn<'a> {
        // The `ParserImpl` is created inside the closure, with the closure's lifetime `'brand`.
        // The closure must accept any `'brand`, so code in it can't treat `'brand` as the same
        // as any other lifetime. That makes `'brand` a brand (see `oxc_data_structures::brands`).
        //
        // `parse_impl` creates the `ParserImpl` and consumes it, all within this one closure.
        // Passing it by value into a 2nd closure would copy the whole `ParserImpl`, which is large.
        BrandOrigin::scope_within(|brand_origin, brand_scope: BrandScope<'_, 'a>| {
            // `source` is shortened from `'a` to `'brand` here
            let ret = ParserImpl::parse_impl(source, options, config, brand_origin);

            // Change lifetime of the output from `'brand` back to `'a`
            brand_scope.extend_lifetime::<ParserReturn>(ret)
        })
    }

    /// Parse `source` as a program.
    ///
    /// `brand_origin` is for the `LexerSource` in the `ParserImpl`'s [`Lexer`] (see [`brands`]).
    ///
    /// [`brands`]: oxc_data_structures::brands
    #[inline]
    fn parse_impl(
        source: Source<'a>,
        options: ParseOptions,
        config: C,
        brand_origin: BrandOrigin<'a>,
    ) -> ParserReturn<'a> {
        let allocator = source.allocator();
        let source_type = source.source_type();
        let source = match ValidSource::new(source) {
            Ok(source) => source,
            Err(error) => return Self::invalid_parser_return(error, source_type, allocator),
        };

        let mut parser = Self::new(source, options, config, brand_origin);

        let mut program = parser.parse_program();
        let mut has_fatal_error = false;

        if let Some(fatal_error) = parser.fatal_error.take() {
            has_fatal_error = true;
            parser.errors.truncate(fatal_error.errors_len);
            if !parser.lexer.errors.is_empty() && parser.cur_kind().is_eof() {
                // Noop
            } else {
                parser.error(fatal_error.error);
            }

            program = Program::dummy(parser.allocator());
            program.source_type = parser.source_type;
            program.source_text = parser.source_text;
        }

        parser.check_unfinished_errors();

        let mut is_flow_language = false;
        let mut errors = Diagnostics::new();
        // only check for `@flow` if the file failed to parse.
        if (!parser.lexer.errors.is_empty() || !parser.errors.is_empty())
            && let Some(error) = parser.flow_error()
        {
            is_flow_language = true;
            errors.push(error.into_diagnostic());
        }
        let (module_record, mut module_record_errors) = parser.module_record_builder.build();
        if errors.len() != 1 {
            errors.reserve(
                parser.lexer.errors.len() + parser.errors.len() + module_record_errors.len(),
            );
            errors.extend(parser.lexer.errors.drain(..).map(ParserDiagnostic::into_diagnostic));
            errors.extend(parser.errors.drain(..).map(ParserDiagnostic::into_diagnostic));
            errors.append(&mut module_record_errors);
        }
        let irregular_whitespaces =
            std::mem::take(&mut parser.lexer.trivia_builder.irregular_whitespaces)
                .into_boxed_slice();

        let source_type = program.source_type;
        if source_type.is_unambiguous() {
            if module_record.has_module_syntax {
                // Resolved to Module - discard deferred script errors (TLA is valid in ESM)
                // but emit deferred module errors (HTML comments are invalid in ESM)
                program.source_type = source_type.with_module(true);
                errors.extend(
                    parser
                        .lexer
                        .deferred_module_errors
                        .drain(..)
                        .map(ParserDiagnostic::into_diagnostic),
                );
            } else {
                // Resolved to Script - emit deferred script errors
                // discard deferred module errors (HTML comments are valid in scripts)
                program.source_type = source_type.with_script(true);
                errors.extend(
                    parser
                        .deferred_script_errors
                        .into_iter()
                        .map(ParserDiagnostic::into_diagnostic),
                );
            }
        }

        let tokens = if has_fatal_error {
            ArenaVec::new_in(&parser.ast)
        } else {
            parser.lexer.finalize_tokens()
        };

        program.comments = parser.lexer.trivia_builder.comments.take_vec(&parser.ast);

        ParserReturn {
            program,
            module_record,
            diagnostics: errors,
            irregular_whitespaces,
            tokens,
            fatal_error: has_fatal_error,
            is_flow_language,
        }
    }

    fn parse_expression(
        source: Source<'a>,
        options: ParseOptions,
        config: C,
    ) -> Result<Expression<'a>, Diagnostics> {
        /// Wrapper around `Result<Expression<'a>, Diagnostics>`.
        /// `Result<T, E>` is only [`Unbranded`] if `T` and `E` are. `Expression` and `Diagnostics`
        /// are from other crates, so this crate can't implement `Unbranded` for them.
        struct ExpressionParserReturn<'a>(Result<Expression<'a>, Diagnostics>);

        // SAFETY: `ExpressionParserReturn` meets `Unbranded`'s contract.
        // `Expression` and `Diagnostics` don't name `BrandOrigin` or `Brand`,
        // or any type which does, and they don't contain a `dyn Trait + 'a`.
        // `Expression`'s lifetime is the lifetime of the arena. Nothing uses it as a brand.
        unsafe impl Unbranded for ExpressionParserReturn<'_> {
            type WithLifetime<'a> = ExpressionParserReturn<'a>;
        }

        // See comments in `parse`
        BrandOrigin::scope_within(|brand_origin, brand_scope: BrandScope<'_, 'a>| {
            let ret = ParserImpl::parse_expression_impl(source, options, config, brand_origin);

            // Change lifetime of the output from `'brand` back to `'a`
            brand_scope.extend_lifetime::<ExpressionParserReturn>(ExpressionParserReturn(ret)).0
        })
    }

    /// Parse `source` as a single expression.
    ///
    /// `brand_origin` is for the `LexerSource` in the `ParserImpl`'s [`Lexer`] (see [`brands`]).
    ///
    /// [`brands`]: oxc_data_structures::brands
    fn parse_expression_impl(
        source: Source<'a>,
        options: ParseOptions,
        config: C,
        brand_origin: BrandOrigin<'a>,
    ) -> Result<Expression<'a>, Diagnostics> {
        let source = ValidSource::new(source).map_err(ParserDiagnostic::into_diagnostic)?;

        let mut parser = Self::new(source, options, config, brand_origin);

        // Initialize by moving onto the first token
        parser.token = parser.lexer.first_token_without_hashbang();
        let expr = parser.parse_expr();
        if !parser.at(Kind::Eof) {
            parser.set_unexpected();
        }
        if let Some(FatalError { error, .. }) = parser.fatal_error.take() {
            return Err(error.into_diagnostic().into());
        }
        parser.check_unfinished_errors();
        let errors = parser
            .lexer
            .errors
            .into_iter()
            .chain(parser.errors)
            .map(ParserDiagnostic::into_diagnostic)
            .collect::<Diagnostics>();
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(expr)
    }

    #[expect(clippy::cast_possible_truncation)]
    fn parse_program(&mut self) -> Program<'a> {
        // Initialize by moving onto the first token.
        // Checks for hashbang comment.
        self.token = self.lexer.first_token();

        let hashbang = self.parse_hashbang();
        self.ctx |= Context::TopLevel;
        let (directives, statements) =
            self.parse_directives_and_statements(/* in_ts_namespace_body */ false);

        let span = Span::new(0, self.source_text.len() as u32);
        Program::new(
            span,
            self.source_type,
            self.source_text,
            // Populated at the end of `parse` after `flow_error` has read from `trivia_builder.comments`
            [],
            hashbang,
            directives,
            statements,
            // The default builder assigns ID zero (NodeId::ROOT) to Program.
            // ParserAstBuilder starts at one, keeping every child ID distinct.
            &AstBuilder::new(self.allocator()),
        )
    }

    fn default_context(source_type: SourceType, options: ParseOptions) -> Context {
        let mut ctx = Context::default().and_ambient(source_type.is_typescript_definition());
        if source_type.is_module() {
            // for [top-level-await](https://tc39.es/proposal-top-level-await/)
            ctx = ctx.and_await(true);
        }
        // CommonJS files are wrapped in a function, so return and `new.target`
        // are allowed at top-level
        if options.allow_return_outside_function || source_type.is_commonjs() {
            ctx = ctx.and_return(true);
        }
        if source_type.is_commonjs() {
            ctx = ctx.and_new_target(true);
        }
        ctx
    }

    /// Check for Flow declaration if the file cannot be parsed.
    /// The declaration must be [on the first line before any code](https://flow.org/en/docs/usage/#toc-prepare-your-code-for-flow)
    fn flow_error(&mut self) -> Option<ParserDiagnostic<'a>> {
        if !self.source_type.is_javascript() {
            return None;
        }
        let span = self.lexer.trivia_builder.comments.first()?.span;
        if span.source_text(self.source_text).contains("@flow") {
            self.errors.clear();
            Some(diagnostics::flow(span))
        } else {
            None
        }
    }

    fn check_unfinished_errors(&mut self) {
        // PropertyDefinition : cover_initialized_name
        // It is a Syntax Error if any source text is matched by this production.
        for cover in self.state.cover_initialized_name.values() {
            self.errors.push(diagnostics::cover_initialized_name(cover.span));
        }
    }

    /// Create the [`ParserReturn`] for a source which failed validation (see [`ValidSource`]).
    ///
    /// It has an empty `Program`, and 1 diagnostic.
    #[cold]
    #[inline(never)]
    fn invalid_parser_return(
        error: ParserDiagnostic<'a>,
        source_type: SourceType,
        allocator: &'a Allocator,
    ) -> ParserReturn<'a> {
        let mut program = Program::dummy(allocator);
        // Same as when parsing fails for any other reason, an unambiguous source resolves to a script
        program.source_type =
            if source_type.is_unambiguous() { source_type.with_script(true) } else { source_type };

        ParserReturn {
            program,
            module_record: ModuleRecord::new(allocator),
            diagnostics: error.into_diagnostic().into(),
            irregular_whitespaces: Box::default(),
            tokens: ArenaVec::new_in(&allocator),
            fatal_error: true,
            is_flow_language: false,
        }
    }

    #[inline]
    fn alloc<T>(&self, value: T) -> ArenaBox<'a, T> {
        ArenaBox::new_in(value, self)
    }
}

impl<'a, C: ParserConfig> GetAllocator<'a> for ParserImpl<'a, C> {
    #[inline]
    fn allocator(&self) -> &'a Allocator {
        self.ast.allocator()
    }
}

impl<'a, C: ParserConfig> GetAstBuilder<'a> for ParserImpl<'a, C> {
    type Builder = ParserAstBuilder<'a>;

    #[inline]
    fn builder(&self) -> &ParserAstBuilder<'a> {
        &self.ast
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;
    #[cfg(any(target_pointer_width = "64", not(debug_assertions)))]
    #[cfg(not(miri))]
    use std::{alloc::Layout, ptr::NonNull};

    use oxc_ast::ast::{
        CommentKind, Expression, Statement, TSNamespaceDeclarationBody, TSNamespaceDeclarationKind,
    };
    use oxc_span::GetSpan;

    use super::*;

    #[test]
    fn parse_program_smoke_test() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let source_text = "";
        let ret = Source::from_str(source_text, source_type, &allocator).parse();
        assert!(ret.program.is_empty());
        assert!(ret.diagnostics.is_empty());
        assert!(!ret.is_flow_language);
    }

    #[test]
    fn parse_expression_smoke_test() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let source_text = "a";
        let source = Source::from_str(source_text, source_type, &allocator);
        let expr = source.parse_expression().unwrap();
        assert!(matches!(expr, Expression::Identifier(_)));
    }

    #[test]
    fn parse_expression_rejects_trailing_tokens() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        for source_text in ["a b", "a;", "let x = 1"] {
            let source = Source::from_str(source_text, source_type, &allocator);
            assert!(source.parse_expression().is_err());
        }
    }

    #[test]
    fn asserted_exponentiation_preserve_parens() {
        for preserve_parens in [true, false] {
            for source_text in [
                "(-value) as number ** 2",
                "(-value as number) ** 2",
                "(-value) as any satisfies number ** 2",
                "(<number>value) satisfies number ** 2",
                "async () => (await value) as number ** 2",
            ] {
                let allocator = Allocator::default();
                let source = Source::from_str(source_text, SourceType::ts(), &allocator);
                let result = source.parse_expression_with_options(ParseOptions {
                    preserve_parens,
                    ..ParseOptions::default()
                });
                assert!(result.is_ok(), "{source_text}, preserve_parens={preserve_parens}");
            }

            for (source_text, operand) in [
                ("-value as any satisfies number ** 2", "-value"),
                ("<number>value as number ** 2", "<number>value"),
                ("async () => await value as number ** 2", "await value"),
            ] {
                let allocator = Allocator::default();
                let source = Source::from_str(source_text, SourceType::ts(), &allocator);
                let diagnostics = source
                    .parse_expression_with_options(ParseOptions {
                        preserve_parens,
                        ..ParseOptions::default()
                    })
                    .unwrap_err();
                assert_eq!(
                    diagnostics.len(),
                    1,
                    "{source_text}, preserve_parens={preserve_parens}"
                );
                let label = &diagnostics[0].labels[0];
                let start = label.offset() as usize;
                let end = start + label.len() as usize;
                assert_eq!(&source_text[start..end], operand);
            }
        }
    }

    /// An unparenthesized instantiation gives its type arguments to the call, `new`, tag or superclass.
    /// A parenthesized one stays whole as in TypeScript, also when the parentheses are not preserved.
    #[test]
    fn parenthesized_instantiation_expression() {
        for preserve_parens in [true, false] {
            // (source, keeps instantiation, outer has type arguments)
            for (source_text, expected) in [
                ("a<b>()", (false, true)),
                ("(a<b>)()", (true, false)),
                ("new a<b>()", (false, true)),
                ("new a<b>", (false, true)),
                ("new (a<b>)()", (true, false)),
                ("new (a<b>)", (true, false)),
                ("a<b>``", (false, true)),
                ("(a<b>)``", (true, false)),
                ("class extends B<T> {}", (false, true)),
                ("class extends (B<T>) {}", (true, false)),
                ("class extends (B<T>)<U> {}", (true, true)),
            ] {
                let allocator = Allocator::default();
                let source = Source::from_str(source_text, SourceType::ts(), &allocator);
                let expr = source
                    .parse_expression_with_options(ParseOptions {
                        preserve_parens,
                        ..ParseOptions::default()
                    })
                    .unwrap_or_else(|_| panic!("{source_text}, preserve_parens={preserve_parens}"));
                let (target, type_arguments) = match &expr {
                    Expression::CallExpression(call) => (&call.callee, &call.type_arguments),
                    Expression::NewExpression(new) => (&new.callee, &new.type_arguments),
                    Expression::TaggedTemplateExpression(tagged) => {
                        (&tagged.tag, &tagged.type_arguments)
                    }
                    Expression::ClassExpression(class) => {
                        let heritage = class.heritage.as_ref().unwrap();
                        (&heritage.expression, &heritage.type_arguments)
                    }
                    _ => unreachable!("{source_text}"),
                };
                let actual = (
                    matches!(
                        target.without_parentheses(),
                        Expression::TSInstantiationExpression(_)
                    ),
                    type_arguments.is_some(),
                );
                assert_eq!(actual, expected, "{source_text}, preserve_parens={preserve_parens}");
            }
        }
    }

    #[test]
    fn simple_arrow_no_side_effects_annotations() {
        for source_text in [
            "/*#__NO_SIDE_EFFECTS__*/ value => value",
            "/*#__NO_SIDE_EFFECTS__*/ async value => value",
            "/*#__NO_SIDE_EFFECTS__*/ async of => of",
        ] {
            let allocator = Allocator::default();
            let source = Source::from_str(source_text, SourceType::default(), &allocator);
            let expr = source.parse_expression().unwrap();
            let Expression::ArrowFunctionExpression(arrow) = expr else { panic!("{source_text}") };
            assert!(arrow.pure, "{source_text}");
        }
    }

    #[test]
    fn identifier_continuation_pure_annotations() {
        for source_text in [
            "/*#__PURE__*/ callee()",
            "/*#__PURE__*/ callee<T>()",
            "/*#__PURE__*/ callee!.method()",
        ] {
            let allocator = Allocator::default();
            let source = Source::from_str(source_text, SourceType::ts(), &allocator);
            let expr = source.parse_expression().unwrap();
            let Expression::CallExpression(call) = expr else { panic!("{source_text}") };
            assert!(call.pure, "{source_text}");
        }

        // Check both the assignment-expression entry and a binary RHS continuation.
        for (source_text, annotated_left) in [
            ("/*#__PURE__*/ callee() + other()", true),
            ("other() + /*#__PURE__*/ callee()", false),
        ] {
            let allocator = Allocator::default();
            let source = Source::from_str(source_text, SourceType::default(), &allocator);
            let expr = source.parse_expression().unwrap();
            let Expression::BinaryExpression(binary) = expr else { panic!("{source_text}") };
            let Expression::CallExpression(left) = &binary.left else { panic!("{source_text}") };
            let Expression::CallExpression(right) = &binary.right else { panic!("{source_text}") };
            assert_eq!(left.pure, annotated_left, "{source_text}");
            assert_eq!(right.pure, !annotated_left, "{source_text}");
        }
    }

    #[test]
    fn flow_error() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let sources = [
            "// @flow\nasdf adsf",
            "/* @flow */\n asdf asdf",
            "/**
             * @flow
             */
             asdf asdf
             ",
            "/* @flow */ super;",
        ];
        for source_text in sources {
            let ret = Source::from_str(source_text, source_type, &allocator).parse();
            assert!(ret.is_flow_language);
            assert_eq!(ret.diagnostics.len(), 1);
            assert_eq!(ret.diagnostics.first().unwrap().to_string(), "Flow is not supported");
        }
    }

    #[test]
    fn ts_module_declaration() {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(Path::new("module.ts")).unwrap();
        let source_text =
            "declare module 'test'; namespace Foo.Bar {} module Baz {} declare module 'raw' {}";
        let ret = Source::from_str(source_text, source_type, &allocator).parse();
        assert!(ret.diagnostics.is_empty());

        let Statement::TSExternalModuleDeclaration(external) = &ret.program.body[0] else {
            panic!("expected external module declaration");
        };
        assert_eq!(external.id.value, "test");
        assert!(external.body.is_none());
        assert!(external.declare);

        let Statement::TSNamespaceDeclaration(namespace) = &ret.program.body[1] else {
            panic!("expected namespace declaration");
        };
        assert_eq!(namespace.id.name, "Foo");
        assert_eq!(namespace.kind, TSNamespaceDeclarationKind::Namespace);
        let TSNamespaceDeclarationBody::TSNamespaceDeclaration(inner) = &namespace.body else {
            panic!("expected dotted namespace declaration");
        };
        assert_eq!(inner.id.name, "Bar");

        let Statement::TSNamespaceDeclaration(module) = &ret.program.body[2] else {
            panic!("expected identifier module declaration");
        };
        assert_eq!(module.id.name, "Baz");
        assert_eq!(module.kind, TSNamespaceDeclarationKind::Module);

        let Statement::TSExternalModuleDeclaration(external) = &ret.program.body[3] else {
            panic!("expected external module declaration");
        };
        assert_eq!(external.id.value, "raw");
        assert!(external.body.is_some());
        assert!(external.declare);
    }

    #[test]
    fn surrogate_halves_join_across_escape_forms() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        // Each spelling escapes U+1F600 as its two surrogate halves.
        let escapes = [
            r"\uD83D\uDE00",
            r"\u{D83D}\u{DE00}",
            r"\uD83D\u{DE00}",
            r"\u{D83D}\uDE00",
            // A LineContinuation contributes no code units, so the halves are adjacent.
            "\\uD83D\\\n\\uDE00",
        ];
        for escape in escapes {
            let source_text = format!("'{escape}'");
            let source = Source::from_str(&source_text, source_type, &allocator);
            let expr = source.parse_expression().unwrap();
            let Expression::StringLiteral(literal) = expr else { panic!("{source_text}") };
            assert_eq!(literal.value.as_str(), Some("😀"), "{source_text}");

            let source = format!("`{escape}`");
            let expr =
                Source::from_str(&source, source_type, &allocator).parse_expression().unwrap();
            let Expression::TemplateLiteral(template) = expr else { panic!("{source}") };
            let cooked = template.quasis[0].value.cooked.and_then(oxc_str::JSStr::as_str);
            assert_eq!(cooked, Some("😀"), "{source}");
        }
    }

    #[test]
    fn template_raw_normalizes_line_terminators() {
        // `(raw, cooked)` of each quasi, as `strings.raw` and `strings` in a tag function (V8).
        type Quasis = &'static [(&'static str, Option<&'static str>)];
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let cases: &[(&str, Quasis)] = &[
            ("tag`\\unicode\r\n`", &[("\\unicode\n", None)]),
            ("tag`\\xZ\r`", &[("\\xZ\n", None)]),
            ("tag`\\xZ\r\r\n`", &[("\\xZ\n\n", None)]),
            ("tag`a\r\n\\u{}`", &[("a\n\\u{}", None)]),
            ("tag`\\\r\n\\8`", &[("\\\n\\8", None)]),
            (
                "tag`h\\xZ\r\n${0}m\\u{}\r${1}t\\8\r\n`",
                &[("h\\xZ\n", None), ("m\\u{}\n", None), ("t\\8\n", None)],
            ),
            ("tag`\\n\r\n`", &[("\\n\n", Some("\n\n"))]),
            ("tag`a\\\rb`", &[("a\\\nb", Some("ab"))]),
            ("tag`ok\r\n`", &[("ok\n", Some("ok\n"))]),
            ("tag`ok\rx`", &[("ok\nx", Some("ok\nx"))]),
        ];
        for &(source_text, expected) in cases {
            let source = Source::from_str(source_text, source_type, &allocator);
            let expr = source.parse_expression().unwrap();
            let Expression::TaggedTemplateExpression(tagged) = expr else {
                panic!("{source_text:?}")
            };
            let values = tagged
                .quasi
                .quasis
                .iter()
                .map(|quasi| {
                    let cooked = quasi.value.cooked.map(|cooked| cooked.as_str().unwrap());
                    (quasi.value.raw.as_str(), cooked)
                })
                .collect::<Vec<_>>();
            assert_eq!(values, expected, "{source_text:?}");
        }
    }

    #[test]
    fn directives() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let sources = [
            ("import x from 'foo'; 'use strict';", 2),
            ("export {x} from 'foo'; 'use strict';", 2),
            (";'use strict';", 2),
            ("('use strict'); 'use strict';", 2),
        ];
        for preserve_parens in [true, false] {
            for (source_text, body_length) in sources {
                let source = Source::from_str(source_text, source_type, &allocator);
                let ret = source.parse_with_options(ParseOptions {
                    preserve_parens,
                    ..ParseOptions::default()
                });
                assert!(ret.program.directives.is_empty(), "{source_text}");
                assert_eq!(ret.program.body.len(), body_length, "{source_text}");
            }
        }
    }

    #[test]
    fn v8_intrinsics() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        {
            let source_text = "%DebugPrint('Raging against the Dying Light')";
            let source = Source::from_str(source_text, source_type, &allocator);
            let opts = ParseOptions { allow_v8_intrinsics: true, ..ParseOptions::default() };
            let ret = source.parse_with_options(opts);
            assert!(ret.diagnostics.is_empty());

            if let Some(Statement::ExpressionStatement(expr_stmt)) = ret.program.body.first() {
                if let Expression::V8IntrinsicExpression(expr) = &expr_stmt.expression {
                    assert_eq!(expr.span().source_text(source_text), source_text);
                } else {
                    panic!("Expected V8IntrinsicExpression");
                }
            } else {
                panic!("Expected ExpressionStatement");
            }
        }
        {
            let source_text = "%DebugPrint(...illegalSpread)";
            let source = Source::from_str(source_text, source_type, &allocator);
            let opts = ParseOptions { allow_v8_intrinsics: true, ..ParseOptions::default() };
            let ret = source.parse_with_options(opts);
            assert_eq!(ret.diagnostics.len(), 1);
            assert_eq!(
                ret.diagnostics[0].to_string(),
                "V8 runtime calls cannot have spread elements as arguments"
            );
        }
        {
            let source_text = "%DebugPrint('~~')";
            let ret = Source::from_str(source_text, source_type, &allocator).parse();
            assert_eq!(ret.diagnostics.len(), 1);
            assert_eq!(ret.diagnostics[0].to_string(), "Unexpected token");
        }
        {
            // https://github.com/oxc-project/oxc/issues/12121
            let source_text = "interface Props extends %enuProps {}";
            let source_type = SourceType::default().with_typescript(true);
            let source = Source::from_str(source_text, source_type, &allocator);
            // Should not panic whether `allow_v8_intrinsics` is set or not.
            let opts = ParseOptions { allow_v8_intrinsics: true, ..ParseOptions::default() };
            let ret = source.parse_with_options(opts);
            assert_eq!(ret.diagnostics.len(), 1);
            let ret = Source::from_str(source_text, source_type, &allocator).parse();
            assert_eq!(ret.diagnostics.len(), 1);
        }
    }

    #[test]
    fn comments() {
        let allocator = Allocator::default();
        let source_type = SourceType::default().with_typescript(true);
        let sources = [
            ("// line comment", CommentKind::Line),
            ("/* line comment */", CommentKind::SingleLineBlock),
            (
                "type Foo = ( /* Require properties which are not generated automatically. */ 'bar')",
                CommentKind::SingleLineBlock,
            ),
        ];
        for (source_text, kind) in sources {
            let ret = Source::from_str(source_text, source_type, &allocator).parse();
            let comments = &ret.program.comments;
            assert_eq!(comments.len(), 1, "{source_text}");
            assert_eq!(comments.first().unwrap().kind, kind, "{source_text}");
        }
    }

    #[test]
    fn hashbang() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let source_text = "#!/usr/bin/node\n;";
        let ret = Source::from_str(source_text, source_type, &allocator).parse();
        assert_eq!(ret.program.hashbang.unwrap().value.as_str(), "/usr/bin/node");
    }

    #[test]
    fn unambiguous() {
        let allocator = Allocator::default();
        let source_type = SourceType::unambiguous();
        assert!(source_type.is_unambiguous());
        let sources = ["import x from 'foo';", "export {x} from 'foo';", "import.meta"];
        for source_text in sources {
            let ret = Source::from_str(source_text, source_type, &allocator).parse();
            assert!(ret.program.source_type.is_module());
        }

        let sources = ["", "import('foo')"];
        for source_text in sources {
            let ret = Source::from_str(source_text, source_type, &allocator).parse();
            assert!(ret.program.source_type.is_script());
        }
    }

    #[test]
    fn source_parse() {
        let allocator = Allocator::default();
        let source_text = "let x = (1);";

        // `parse`
        let ret = Source::from_str(source_text, SourceType::mjs(), &allocator).parse();
        assert!(!ret.fatal_error);
        assert!(ret.diagnostics.is_empty());
        assert_eq!(ret.program.body.len(), 1);
        assert_eq!(ret.program.source_text, source_text);
        assert!(ret.tokens.is_empty());

        // `parse_with_options`
        let has_parens = |options: ParseOptions| {
            let source = Source::from_str(source_text, SourceType::mjs(), &allocator);
            let ret = source.parse_with_options(options);
            assert!(ret.diagnostics.is_empty());
            let Statement::VariableDeclaration(decl) = &ret.program.body[0] else { panic!() };
            matches!(decl.declarations[0].init, Some(Expression::ParenthesizedExpression(_)))
        };
        assert!(has_parens(ParseOptions { preserve_parens: true, ..ParseOptions::default() }));
        assert!(!has_parens(ParseOptions { preserve_parens: false, ..ParseOptions::default() }));

        // `parse_with_config`
        let token_count = |config: RuntimeParserConfig| {
            let source = Source::from_str(source_text, SourceType::mjs(), &allocator);
            let ret = source.parse_with_config(ParseOptions::default(), config);
            assert!(ret.diagnostics.is_empty());
            ret.tokens.len()
        };
        assert_eq!(token_count(RuntimeParserConfig::new(false)), 0);
        assert_eq!(token_count(RuntimeParserConfig::new(true)), 7);

        let source = Source::from_str(source_text, SourceType::mjs(), &allocator);
        let ret = source.parse_with_config(ParseOptions::default(), TokensParserConfig);
        assert_eq!(ret.tokens.len(), 7);
    }

    #[test]
    fn source_parse_expression() {
        let allocator = Allocator::default();
        let source_text = "(1 + 2)";

        // `parse_expression`
        let source = Source::from_str(source_text, SourceType::mjs(), &allocator);
        assert!(matches!(source.parse_expression(), Ok(Expression::ParenthesizedExpression(_))));

        // `parse_expression_with_options`
        let source = Source::from_str(source_text, SourceType::mjs(), &allocator);
        let options = ParseOptions { preserve_parens: false, ..ParseOptions::default() };
        let expr = source.parse_expression_with_options(options);
        assert!(matches!(expr, Ok(Expression::BinaryExpression(_))));

        // `parse_expression_with_config`
        for config in [RuntimeParserConfig::new(false), RuntimeParserConfig::new(true)] {
            let source = Source::from_str(source_text, SourceType::mjs(), &allocator);
            let expr = source.parse_expression_with_config(ParseOptions::default(), config);
            assert!(matches!(expr, Ok(Expression::ParenthesizedExpression(_))));
        }

        // Syntax error
        let source = Source::from_str("1 +", SourceType::mjs(), &allocator);
        assert!(source.parse_expression().is_err());
    }

    #[test]
    fn source_from_bytes_parse() {
        let allocator = Allocator::default();

        // Valid UTF-8
        let source_text = "let x = '日本語';";
        let source = Source::from_bytes(source_text.as_bytes(), SourceType::mjs(), &allocator);
        let ret = source.parse();
        assert!(!ret.fatal_error);
        assert!(ret.diagnostics.is_empty());
        assert_eq!(ret.program.body.len(), 1);
        assert_eq!(ret.program.source_text, source_text);

        // Invalid UTF-8
        for source_type in [SourceType::mjs(), SourceType::cjs(), SourceType::unambiguous()] {
            let source = Source::from_bytes(b"let x = '\xFF';", source_type, &allocator);
            let ret = source.parse();
            assert!(ret.fatal_error);
            assert!(ret.program.is_empty());
            assert_eq!(ret.program.source_text, "");
            assert_eq!(ret.program.source_type.is_module(), source_type.is_module());
            assert!(!ret.program.source_type.is_unambiguous());
            assert!(ret.tokens.is_empty());
            assert_eq!(ret.diagnostics.len(), 1);
            assert_eq!(ret.diagnostics[0].to_string(), "File appears to be binary.");
        }
    }

    #[test]
    fn source_from_bytes_parse_expression() {
        let allocator = Allocator::default();
        let parse = |bytes: &[u8]| {
            Source::from_bytes(bytes, SourceType::mjs(), &allocator).parse_expression()
        };

        // Valid UTF-8
        assert!(matches!(parse("'日本語'".as_bytes()), Ok(Expression::StringLiteral(_))));

        // Invalid UTF-8
        let diagnostics = parse(b"'\xFF'").err().unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].to_string(), "File appears to be binary.");
    }

    #[test]
    fn parser_new_copies_source_text() {
        let allocator = Allocator::default();
        let source_text = "let x = 1;";
        let ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
        assert_eq!(ret.program.source_text, source_text);
        assert_ne!(ret.program.source_text.as_ptr(), source_text.as_ptr());
    }

    /// `Parser` passes its options and config on to the parser.
    #[test]
    fn parser_options_and_config() {
        let allocator = Allocator::default();
        let options = ParseOptions { preserve_parens: false, ..ParseOptions::default() };

        let ret = Parser::new(&allocator, "(x);", SourceType::mjs())
            .with_options(options)
            .with_config(config::TokensParserConfig)
            .parse();
        assert!(ret.diagnostics.is_empty());
        let Statement::ExpressionStatement(stmt) = &ret.program.body[0] else { unreachable!() };
        assert!(matches!(stmt.expression, Expression::Identifier(_)));
        assert!(!ret.tokens.is_empty());

        let expr = Parser::new(&allocator, "(x)", SourceType::mjs())
            .with_options(options)
            .with_config(config::TokensParserConfig)
            .parse_expression()
            .unwrap();
        assert!(matches!(expr, Expression::Identifier(_)));
    }

    #[test]
    fn binary_file() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();

        // U+FFFD as a standalone token — file appears to be binary
        let ret = Source::from_str("\u{FFFD}", source_type, &allocator).parse();
        assert!(ret.program.is_empty());
        assert_eq!(ret.diagnostics.len(), 1);
        assert_eq!(ret.diagnostics[0].to_string(), "File appears to be binary.");

        // U+FFFD inside string literals — should parse fine
        let ret = Source::from_str("\"oops \u{FFFD} oops\";", source_type, &allocator).parse();
        assert!(!ret.program.is_empty());
        assert!(ret.diagnostics.is_empty());
    }

    #[test]
    fn memory_leak() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let sources = ["2n", ";'1234567890123456789012345678901234567890'"];
        for source_text in sources {
            let ret = Source::from_str(source_text, source_type, &allocator).parse();
            assert!(!ret.program.body.is_empty());
        }
    }

    /// Create a [`Source`] which is `len` bytes long, directly in `allocator`'s arena.
    ///
    /// It starts with `head`, ends with `foot`, and every byte in between is `fill`.
    ///
    /// Returns the `Source`, and a pointer to the start of its bytes.
    ///
    /// This is much faster than building a `String` that long, and copying it into the arena.
    //
    // `cfg`s match the tests which use this: `overlong_source` and `legal_length_source`
    #[cfg(any(target_pointer_width = "64", not(debug_assertions)))]
    #[cfg(not(miri))]
    fn filled_source<'a>(
        len: usize,
        fill: u8,
        head: &str,
        foot: &str,
        allocator: &'a Allocator,
    ) -> (Source<'a>, NonNull<u8>) {
        assert!(fill.is_ascii());
        assert!(head.len() + foot.len() <= len);

        let layout = Layout::from_size_align(len + Source::PADDING, 1).unwrap();
        let ptr = allocator.alloc_layout(layout);

        // SAFETY: `ptr` points to an allocation of `len + PADDING` bytes.
        // `head.len() + foot.len() <= len`, so all these writes are within its first `len` bytes.
        unsafe {
            ptr.write_bytes(fill, len);
            let head_ptr = NonNull::from(head.as_bytes()).cast::<u8>();
            ptr.copy_from_nonoverlapping(head_ptr, head.len());
            let foot_ptr = NonNull::from(foot.as_bytes()).cast::<u8>();
            ptr.add(len - foot.len()).copy_from_nonoverlapping(foot_ptr, foot.len());
        }

        // SAFETY:
        // - `ptr` points to a new allocation of `len + PADDING` bytes in `allocator`'s arena.
        //   No reference to it exists.
        // - The first `len` bytes were initialized just above.
        // - They are `head`, then ASCII bytes, then `foot`, which together are valid UTF-8.
        let source =
            unsafe { Source::from_raw_parts(ptr, len, true, SourceType::default(), allocator) };

        (source, ptr)
    }

    // Source with length MAX_LEN + 1 fails to parse.
    // Skip this test on 32-bit systems as impossible to allocate a source longer than `isize::MAX`.
    // Also skip running under Miri since it takes so long.
    #[cfg(target_pointer_width = "64")]
    #[cfg(not(miri))]
    #[test]
    fn overlong_source() {
        const LEN: usize = MAX_LEN + 1;

        // Create long source (MAX_LEN + 1 bytes)
        let allocator = Allocator::default();
        let (source, ptr) = filled_source(LEN, 0, "", "", &allocator);

        // Attempt to parse the source as a program - parsing should fail
        let ret = source.parse();

        assert!(ret.program.is_empty());
        assert!(ret.fatal_error);
        assert_eq!(ret.diagnostics.len(), 1);
        assert_eq!(ret.diagnostics.first().unwrap().to_string(), MAX_LEN_ERROR);

        drop(ret);

        // Create a 2nd `Source` from the same bytes, to avoid allocating and filling another 4 GiB.
        // SAFETY: `ptr` and `LEN` are the same as the 1st `Source` was created from.
        // The 1st `Source` has been consumed, and the `ParserReturn` it produced was dropped above,
        // so no reference to the bytes exists.
        // `allocator` has not been reset, so the bytes are still valid.
        let source =
            unsafe { Source::from_raw_parts(ptr, LEN, true, SourceType::default(), &allocator) };

        // Attempt to parse the source as an expression - parsing should fail
        let ret = source.parse_expression();

        assert!(ret.is_err());
        let diagnostics = ret.err().unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics.first().unwrap().to_string(), MAX_LEN_ERROR);
    }

    // Source with length `MAX_LEN` parses OK.
    // This test takes over 1 minute on an M1 Macbook Pro unless compiled in release mode.
    // `not(debug_assertions)` is a proxy for detecting release mode.
    // Also skip running under Miri since it takes so long.
    #[cfg(not(debug_assertions))]
    #[cfg(not(miri))]
    #[test]
    fn legal_length_source() {
        // Build a source MAX_LEN bytes long which doesn't take too long to parse
        let allocator = Allocator::default();
        let (source, ptr) = filled_source(MAX_LEN, b'x', "x(/*", "*/)", &allocator);
        assert_eq!(source.len(), MAX_LEN);

        // Parse as a program
        let ret = source.parse();
        assert!(!ret.fatal_error);
        assert!(ret.diagnostics.is_empty());
        assert_eq!(ret.program.body.len(), 1);
        drop(ret);

        // Create a 2nd `Source` from the same bytes, to avoid allocating and filling another 4 GiB.
        // SAFETY: `ptr` and `MAX_LEN` are the same as the 1st `Source` was created from.
        // The 1st `Source` has been consumed, and the `ParserReturn` it produced was dropped above,
        // so no reference to the bytes exists.
        // `allocator` has not been reset, so the bytes are still valid.
        let source = unsafe {
            Source::from_raw_parts(ptr, MAX_LEN, true, SourceType::default(), &allocator)
        };

        // Parse as an expression
        let ret = source.parse_expression();
        assert!(ret.is_ok());
        let expr = ret.ok().unwrap();
        assert!(matches!(expr, Expression::CallExpression(_)));
    }

    // Where the lexer cannot lex any further it emits `Undetermined`, and no `Eof` follows.
    // That final token ends the input just as `Eof` does, so it is discarded, and discarding it
    // must not take the real token before it with it.
    #[test]
    fn tokens_when_lexing_ends_in_error() {
        let allocator = Allocator::default();
        let source = Source::from_str("foo();\n'unterminated", SourceType::default(), &allocator);
        let ret = source.parse_with_config(ParseOptions::default(), config::TokensParserConfig);

        assert!(!ret.fatal_error);
        assert_eq!(ret.diagnostics.len(), 1);
        assert_eq!(ret.diagnostics.first().unwrap().to_string(), "Unterminated string");

        let tokens = ret
            .tokens
            .iter()
            .map(|token| (token.kind(), token.start(), token.end()))
            .collect::<Vec<_>>();
        assert_eq!(
            tokens,
            [
                (Kind::Ident, 0, 3),
                (Kind::LParen, 3, 4),
                (Kind::RParen, 4, 5),
                (Kind::Semicolon, 5, 6),
            ]
        );
    }

    #[test]
    fn tokens_after_unambiguous_await_reparse() {
        for source_text in [
            "await /x/u; export {};",
            // Reparsing merges `/a/`, but splits `/b/g`, increasing the token count.
            "await /a/ /b/g; export {};",
            "before(); await /a/ /b/g; between(); await /x/u; export {}; tail();",
            "before(); await /x/u; after(); export {}; tail();",
            "before(); await /x/u; between(); await /y/g; export {}; tail();",
            "await /x/u\nawait /y/g\nexport {};",
            "await\nawait /x/u; export {};",
            "before(); await\nawait /x/u; between(); await\nawait /y/g; export {}; tail();",
            "const x = await /x/u; between(); const y = await /y/g; export {};",
            "/before/.test(await /x/u); /after/.test('after'); export {};",
            "await /x/u, import.meta;",
            "#!/usr/bin/env node\n'use strict';\nawait /* regexp */ /x/u; export {};",
            "export {}; await /x/u;",
        ] {
            let allocator = Allocator::default();
            let parse = |source_type| {
                let source = Source::from_str(source_text, source_type, &allocator);
                source.parse_with_config(ParseOptions::default(), config::TokensParserConfig)
            };
            let unambiguous = parse(SourceType::unambiguous());
            let module = parse(SourceType::mjs());
            assert_eq!(unambiguous.tokens.as_slice(), module.tokens.as_slice(), "{source_text}");
        }
    }

    // A fatal error fast-forwards the lexer to end of file. Re-lexing the `}` of a template
    // substitution after that would derive a token position from the moved cursor, and so
    // overwrite an unrelated token in the collected stream.
    #[test]
    fn tokens_when_template_substitution_fails() {
        let allocator = Allocator::default();
        let source = Source::from_str("`${}`", SourceType::default(), &allocator);
        let ret = source.parse_with_config(ParseOptions::default(), config::TokensParserConfig);

        assert!(ret.fatal_error);
        assert!(ret.tokens.is_empty());
    }
}
