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
//!
//! ```rust,ignore
//! let parser_return = Parser::new(&allocator, &source_text, source_type).parse();
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

use std::{
    any::Any,
    path::{Path, PathBuf},
};

pub mod config;
mod context;
mod cursor;
mod error_handler;
mod modifiers;
mod module_record;
mod state;

mod ets;
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
    ast::{Expression, Program, Statement},
    builder::{AstBuilder, GetAstBuilder},
};
use oxc_diagnostics::{Diagnostics, OxcDiagnostic};
use oxc_span::{SourceType, Span};
use oxc_syntax::module_record::ModuleRecord;

pub use crate::lexer::{Kind, Token};
use crate::{
    config::{
        LexerConfig, NoTokensParserConfig, ParserConfig, RuntimeParserConfig, TokensParserConfig,
    },
    context::{Context, StatementContext},
    error_handler::FatalError,
    lexer::Lexer,
    module_record::ModuleRecordBuilder,
    state::ParserState,
};

/// Maximum length of source which can be parsed (in bytes).
/// ~4 GiB on 64-bit systems, ~2 GiB on 32-bit systems.
// Length is constrained by 2 factors:
// 1. `Span`'s `start` and `end` are `u32`s, which limits length to `u32::MAX` bytes.
// 2. Rust's allocator APIs limit allocations to `isize::MAX`.
// https://doc.rust-lang.org/std/alloc/struct.Layout.html#method.from_size_align
pub(crate) const MAX_LEN: usize = if size_of::<usize>() >= 8 {
    // 64-bit systems
    u32::MAX as usize
} else {
    // 32-bit or 16-bit systems
    isize::MAX as usize
};

/// Return value of [`Parser::parse`] consisting of AST, errors and comments
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
/// Oxc's [`Parser`] is able to recover from some syntax errors and continue parsing. When this
/// happens,
/// 1. [`diagnostics`] will be non-empty
/// 2. [`program`] will contain a full AST
/// 3. [`panicked`] will be false
///
/// When the parser cannot recover, it will abort and terminate parsing early. [`program`] will
/// be empty and [`panicked`] will be `true`.
///
/// [`program`]: ParserReturn::program
/// [`diagnostics`]: ParserReturn::diagnostics
/// [`panicked`]: ParserReturn::panicked
#[non_exhaustive]
pub struct ParserReturn<'a> {
    /// The parsed AST.
    ///
    /// Will be empty (e.g. no statements, directives, etc) if the parser panicked.
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

    /// Whether the parser panicked and terminated early.
    ///
    /// This will be `false` if parsing was successful, or if parsing was able to recover from a
    /// syntax error. When `true`, [`program`] will be empty and [`diagnostics`] will contain at least
    /// one error.
    ///
    /// [`program`]: ParserReturn::program
    /// [`diagnostics`]: ParserReturn::diagnostics
    pub panicked: bool,

    /// Whether the file is [flow](https://flow.org).
    pub is_flow_language: bool,
}

/// Parse options
///
/// You may provide options to the [`Parser`] using [`Parser::with_options`].
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

/// Configurable ArkTS grammar hooks corresponding to OpenHarmony's `EtsOptions`.
#[derive(Debug, Clone)]
pub struct ArkTsOptions {
    /// Identifiers that are parsed as ArkUI component calls in render bodies.
    pub components: Vec<String>,
    /// Struct method names whose bodies use ArkUI component syntax.
    pub render_methods: Vec<String>,
    /// Bare decorator identifiers that enable ArkUI syntax in functions/methods.
    pub render_decorators: Vec<String>,
    /// Call-expression decorator identifiers such as `@Extend(Text)`.
    pub extend_decorators: Vec<String>,
    /// Bare decorator identifier used for style functions/methods.
    pub styles_decorator: Option<String>,
    /// Calls whose arguments after the first data-source argument are UI callbacks.
    pub parameter_ui_callbacks: Vec<String>,
    /// Component attributes whose arguments are UI callbacks.
    pub attribute_ui_callbacks: Vec<ArkTsAttributeCallback>,
    /// Enable the ArkTS `@interface` annotation declaration grammar.
    pub annotations: bool,
}

/// An ArkUI component and the attributes that accept UI callback arguments.
#[derive(Debug, Clone)]
pub struct ArkTsAttributeCallback {
    pub component: String,
    pub attributes: Vec<String>,
}

impl Default for ArkTsOptions {
    fn default() -> Self {
        Self {
            components: Vec::new(),
            render_methods: vec!["build".into(), "pageTransition".into()],
            render_decorators: vec!["Builder".into(), "LocalBuilder".into()],
            extend_decorators: vec!["Extend".into(), "AnimatableExtend".into()],
            styles_decorator: Some("Styles".into()),
            parameter_ui_callbacks: vec!["ForEach".into(), "LazyForEach".into()],
            attribute_ui_callbacks: vec![ArkTsAttributeCallback {
                component: "Repeat".into(),
                attributes: vec!["each".into(), "template".into()],
            }],
            annotations: true,
        }
    }
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
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
    source_path: Option<PathBuf>,
    options: ParseOptions,
    arkts_options: Option<ArkTsOptions>,
    config: C,
}

impl<'a> Parser<'a> {
    /// Create a new [`Parser`]
    ///
    /// # Parameters
    /// - `allocator`: [Memory arena](oxc_allocator::Allocator) for allocating AST nodes
    /// - `source_text`: Source code to parse
    /// - `source_type`: Source type (e.g. JavaScript, TypeScript, JSX, ESM Module, Script)
    pub fn new(allocator: &'a Allocator, source_text: &'a str, source_type: SourceType) -> Self {
        let options = ParseOptions::default();
        Self {
            allocator,
            source_text,
            source_type,
            source_path: None,
            options,
            arkts_options: None,
            config: NoTokensParserConfig,
        }
    }
}

impl<'a, C: ParserConfig> Parser<'a, C> {
    /// Set parse options
    #[must_use]
    pub fn with_options(mut self, options: ParseOptions) -> Self {
        self.options = options;
        self
    }

    /// Set the path of the source being parsed.
    ///
    /// Static ETS uses this optional information to reject a module that
    /// re-exports bindings from itself. Other language modes ignore it.
    #[must_use]
    pub fn with_source_path(mut self, source_path: impl AsRef<Path>) -> Self {
        self.source_path = Some(source_path.as_ref().to_path_buf());
        self
    }

    /// Set ArkTS/ArkUI grammar configuration corresponding to OpenHarmony's `EtsOptions`.
    #[must_use]
    pub fn with_arkts_options(mut self, options: ArkTsOptions) -> Self {
        self.arkts_options = Some(options);
        self
    }

    /// Set parser config.
    ///
    /// See [`ParserConfig`] for more details.
    #[must_use]
    pub fn with_config<Config: ParserConfig>(self, config: Config) -> Parser<'a, Config> {
        Parser {
            allocator: self.allocator,
            source_text: self.source_text,
            source_type: self.source_type,
            source_path: self.source_path,
            options: self.options,
            arkts_options: self.arkts_options,
            config,
        }
    }
}

mod parser_parse {
    use super::*;

    /// `UniquePromise` is a way to use the type system to enforce the invariant that only
    /// a single `ParserImpl`, `Lexer` and `lexer::Source` can exist at any time on a thread.
    /// This constraint is required to guarantee the soundness of some methods of these types
    /// e.g. `Source::set_position`.
    ///
    /// `ParserImpl::new`, `Lexer::new` and `lexer::Source::new` all require a `UniquePromise`
    /// to be provided to them. `UniquePromise::new` is not visible outside this module, so only
    /// `Parser::parse` can create one, and it only calls `ParserImpl::new` once.
    /// This enforces the invariant throughout the entire parser.
    ///
    /// `UniquePromise` is a zero-sized type and has no runtime cost. It's purely for the type-checker.
    ///
    /// `UniquePromise::new_for_tests_and_benchmarks` is a backdoor for tests/benchmarks, so they can
    /// create a `ParserImpl` or `Lexer`, and manipulate it directly, for testing/benchmarking purposes.
    pub struct UniquePromise(());

    impl UniquePromise {
        #[inline]
        fn new() -> Self {
            Self(())
        }

        /// Backdoor for tests/benchmarks to create a `UniquePromise` (see above).
        /// This function must NOT be exposed outside of tests and benchmarks,
        /// as it allows circumventing safety invariants of the parser.
        #[cfg(any(test, feature = "benchmarking"))]
        pub fn new_for_tests_and_benchmarks() -> Self {
            Self(())
        }
    }

    impl<'a, C: ParserConfig> Parser<'a, C> {
        /// Main entry point
        ///
        /// Returns an empty `Program` on unrecoverable error,
        /// Recoverable errors are stored inside `errors`.
        ///
        /// See the [module-level documentation](crate) for examples and more information.
        //
        // # Implementation note
        //
        // Dispatches via `Any` to a non-generic helper for each known `ParserConfig`.
        // The dispatch keeps the parser body emitted exactly once in `oxc_parser`'s rlib,
        // so consuming crates don't each get a private copy. The `Any::is` / `downcast_ref`
        // calls const-fold when `C` is concrete (the trait object is built from a known
        // concrete type at every monomorphization site, so LLVM devirtualises the vtable
        // call to `Any::type_id` and folds the comparison), leaving each monomorphization
        // as a single call into the matching helper.
        pub fn parse(self) -> ParserReturn<'a> {
            let config: &dyn Any = &self.config;
            if config.is::<NoTokensParserConfig>() {
                parse_with_no_tokens_config(
                    self.allocator,
                    self.source_text,
                    self.source_type,
                    self.source_path,
                    self.options,
                    self.arkts_options,
                )
            } else if config.is::<TokensParserConfig>() {
                parse_with_tokens_config(
                    self.allocator,
                    self.source_text,
                    self.source_type,
                    self.source_path,
                    self.options,
                    self.arkts_options,
                )
            } else if let Some(&config) = config.downcast_ref::<RuntimeParserConfig>() {
                parse_with_runtime_config(
                    self.allocator,
                    self.source_text,
                    self.source_type,
                    self.source_path,
                    self.options,
                    self.arkts_options,
                    config,
                )
            } else {
                // User-defined `ParserConfig`. Generic codegen here, monomorphized per consuming crate.
                // Users using custom configs would need to perform the monomorphization themselves.
                ParserImpl::<C>::new(
                    self.allocator,
                    self.source_text,
                    self.source_type,
                    self.source_path,
                    self.options,
                    self.arkts_options,
                    self.config,
                    UniquePromise::new(),
                )
                .parse()
            }
        }

        /// Parse a single [`Expression`].
        ///
        /// # Example
        ///
        /// ```rust
        /// use oxc_allocator::Allocator;
        /// use oxc_ast::ast::Expression;
        /// use oxc_parser::Parser;
        /// use oxc_span::SourceType;
        ///
        /// let src = "let x = 1 + 2;";
        /// let allocator = Allocator::new();
        /// let source_type = SourceType::default();
        ///
        /// let expr: Expression<'_> = Parser::new(&allocator, src, source_type).parse_expression().unwrap();
        /// ```
        ///
        /// # Errors
        /// If the source code being parsed has syntax errors.
        //
        // # Implementation note
        // Dispatches via `Any`, same as `parse` does.
        pub fn parse_expression(self) -> Result<Expression<'a>, Diagnostics> {
            let config: &dyn Any = &self.config;
            if config.is::<NoTokensParserConfig>() {
                parse_expression_with_no_tokens_config(
                    self.allocator,
                    self.source_text,
                    self.source_type,
                    self.source_path,
                    self.options,
                    self.arkts_options,
                )
            } else if config.is::<TokensParserConfig>() {
                parse_expression_with_tokens_config(
                    self.allocator,
                    self.source_text,
                    self.source_type,
                    self.source_path,
                    self.options,
                    self.arkts_options,
                )
            } else if let Some(&config) = config.downcast_ref::<RuntimeParserConfig>() {
                parse_expression_with_runtime_config(
                    self.allocator,
                    self.source_text,
                    self.source_type,
                    self.source_path,
                    self.options,
                    self.arkts_options,
                    config,
                )
            } else {
                ParserImpl::<C>::new(
                    self.allocator,
                    self.source_text,
                    self.source_type,
                    self.source_path,
                    self.options,
                    self.arkts_options,
                    self.config,
                    UniquePromise::new(),
                )
                .parse_expression()
            }
        }
    }

    // ===========================================================================
    // Non-generic parse helpers, one per known `ParserConfig`.
    //
    // The parser is generic over `C: ParserConfig`. By default Rust monomorphizes
    // generic functions per consuming crate (the legacy mangled name encodes the
    // instantiating crate's disambiguator, and `share-generics` is off at
    // `opt-level >= 2`). For a parser that's pulled in by ~15 crates in a real
    // workspace, that means ~15 private copies of every parser method in the
    // final cdylib — none of which can be deduped by COMDAT (different names)
    // or fat LTO (slightly different inlining contexts).
    //
    // To avoid that, `Parser<C>::parse` and `Parser<C>::parse_expression` dispatch
    // via `Any` to one of the helpers below for the three known configs.
    // Each helper is non-generic, so it's emitted exactly once in `oxc_parser`'s
    // rlib and shared by all consumers. The `Any::is` / `downcast_ref` checks fold
    // at compile time when `C` is concrete, so each monomorphization of the dispatch
    // shrinks to a single call into the matching helper.
    //
    // The helpers are `#[inline(never)]` to prevent fat LTO from re-inlining the
    // parser body across the rlib boundary, which would defeat the purpose.
    //
    // For user-defined `ParserConfig` impls (rare), the dispatch falls through
    // to a generic body that monomorphizes per consuming crate. That's the same
    // cost the parser had without this pattern; we just keep that cost contained
    // to the rare custom-config case.
    // ===========================================================================

    #[inline(never)]
    fn parse_with_no_tokens_config<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        source_path: Option<PathBuf>,
        options: ParseOptions,
        arkts_options: Option<ArkTsOptions>,
    ) -> ParserReturn<'a> {
        ParserImpl::<NoTokensParserConfig>::new(
            allocator,
            source_text,
            source_type,
            source_path,
            options,
            arkts_options,
            NoTokensParserConfig,
            UniquePromise::new(),
        )
        .parse()
    }

    #[inline(never)]
    fn parse_with_tokens_config<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        source_path: Option<PathBuf>,
        options: ParseOptions,
        arkts_options: Option<ArkTsOptions>,
    ) -> ParserReturn<'a> {
        ParserImpl::<TokensParserConfig>::new(
            allocator,
            source_text,
            source_type,
            source_path,
            options,
            arkts_options,
            TokensParserConfig,
            UniquePromise::new(),
        )
        .parse()
    }

    #[inline(never)]
    fn parse_with_runtime_config<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        source_path: Option<PathBuf>,
        options: ParseOptions,
        arkts_options: Option<ArkTsOptions>,
        config: RuntimeParserConfig,
    ) -> ParserReturn<'a> {
        ParserImpl::<RuntimeParserConfig>::new(
            allocator,
            source_text,
            source_type,
            source_path,
            options,
            arkts_options,
            config,
            UniquePromise::new(),
        )
        .parse()
    }

    #[inline(never)]
    fn parse_expression_with_no_tokens_config<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        source_path: Option<PathBuf>,
        options: ParseOptions,
        arkts_options: Option<ArkTsOptions>,
    ) -> Result<Expression<'a>, Diagnostics> {
        ParserImpl::<NoTokensParserConfig>::new(
            allocator,
            source_text,
            source_type,
            source_path,
            options,
            arkts_options,
            NoTokensParserConfig,
            UniquePromise::new(),
        )
        .parse_expression()
    }

    #[inline(never)]
    fn parse_expression_with_tokens_config<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        source_path: Option<PathBuf>,
        options: ParseOptions,
        arkts_options: Option<ArkTsOptions>,
    ) -> Result<Expression<'a>, Diagnostics> {
        ParserImpl::<TokensParserConfig>::new(
            allocator,
            source_text,
            source_type,
            source_path,
            options,
            arkts_options,
            TokensParserConfig,
            UniquePromise::new(),
        )
        .parse_expression()
    }

    #[inline(never)]
    fn parse_expression_with_runtime_config<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        source_path: Option<PathBuf>,
        options: ParseOptions,
        arkts_options: Option<ArkTsOptions>,
        config: RuntimeParserConfig,
    ) -> Result<Expression<'a>, Diagnostics> {
        ParserImpl::<RuntimeParserConfig>::new(
            allocator,
            source_text,
            source_type,
            source_path,
            options,
            arkts_options,
            config,
            UniquePromise::new(),
        )
        .parse_expression()
    }
}
use parser_parse::UniquePromise;

/// Implementation of parser.
/// `Parser` is just a public wrapper, the guts of the implementation is in this type.
struct ParserImpl<'a, C: ParserConfig> {
    /// Options
    options: ParseOptions,

    /// Optional ArkTS/ArkUI grammar configuration.
    arkts_options: Option<ArkTsOptions>,

    pub(crate) lexer: Lexer<'a, C::LexerConfig>,

    /// SourceType: JavaScript or TypeScript, Script or Module, jsx support?
    source_type: SourceType,

    /// Source Code
    source_text: &'a str,

    /// All syntax errors from parser and lexer
    /// Note: favor adding to `Diagnostics` instead of raising Err
    errors: Vec<OxcDiagnostic>,

    /// Errors that are only valid if the file is determined to be a Script (not a Module).
    /// For `ModuleKind::Unambiguous`, we defer ESM-only errors (like top-level await)
    /// until we know whether the file is ESM or Script.
    /// If resolved to Module → discard these errors.
    /// If resolved to Script → emit these errors.
    deferred_script_errors: Vec<OxcDiagnostic>,

    fatal_error: Option<FatalError>,

    /// The current parsing token
    token: Token,

    /// The end range of the previous token
    prev_token_end: u32,

    /// Parser state
    state: ParserState<'a>,

    /// Parsing context
    ctx: Context,

    /// Ast builder for creating AST nodes
    ast: AstBuilder<'a>,

    /// Module Record Builder
    module_record_builder: ModuleRecordBuilder<'a>,

    /// Precomputed typescript detection
    is_ts: bool,
}

impl<'a, C: ParserConfig> ParserImpl<'a, C> {
    /// Create a new `ParserImpl`.
    ///
    /// Requiring a `UniquePromise` to be provided guarantees only 1 `ParserImpl` can exist
    /// on a single thread at one time.
    #[inline]
    #[expect(clippy::needless_pass_by_value)]
    pub fn new(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        source_path: Option<PathBuf>,
        options: ParseOptions,
        arkts_options: Option<ArkTsOptions>,
        config: C,
        unique: UniquePromise,
    ) -> Self {
        let ctx = Self::default_context(source_type, &options);
        Self {
            options,
            arkts_options,
            lexer: Lexer::new(allocator, source_text, source_type, config.lexer_config(), unique),
            source_type,
            source_text,
            errors: vec![],
            deferred_script_errors: vec![],
            fatal_error: None,
            token: Token::default(),
            prev_token_end: 0,
            state: ParserState::new(),
            ctx,
            ast: AstBuilder::new(allocator),
            module_record_builder: ModuleRecordBuilder::new(allocator, source_type, source_path),
            is_ts: source_type.is_typescript(),
        }
    }

    /// Main entry point
    ///
    /// Returns an empty `Program` on unrecoverable error,
    /// Recoverable errors are stored inside `errors`.
    #[inline]
    pub fn parse(mut self) -> ParserReturn<'a> {
        let mut program = self.parse_program();
        let mut panicked = false;

        if let Some(fatal_error) = self.fatal_error.take() {
            panicked = true;
            self.errors.truncate(fatal_error.errors_len);
            if !self.lexer.errors.is_empty() && self.cur_kind().is_eof() {
                // Noop
            } else {
                self.error(fatal_error.error);
            }

            program = Program::dummy(self.allocator());
            program.source_type = self.source_type;
            program.source_text = self.source_text;
        }

        self.check_unfinished_errors();

        if let Some(overlong_error) = self.overlong_error() {
            panicked = true;
            self.lexer.errors.clear();
            self.errors.clear();
            self.error(overlong_error);
        }

        let mut is_flow_language = false;
        let mut errors = Diagnostics::new();
        // only check for `@flow` if the file failed to parse.
        if (!self.lexer.errors.is_empty() || !self.errors.is_empty())
            && let Some(error) = self.flow_error()
        {
            is_flow_language = true;
            errors.push(error);
        }
        let (module_record, mut module_record_errors) = self.module_record_builder.build();
        if errors.len() != 1 {
            errors
                .reserve(self.lexer.errors.len() + self.errors.len() + module_record_errors.len());
            errors.append(&mut self.lexer.errors);
            errors.append(&mut self.errors);
            errors.append(&mut module_record_errors);
        }
        let irregular_whitespaces =
            std::mem::take(&mut self.lexer.trivia_builder.irregular_whitespaces).into_boxed_slice();

        let source_type = program.source_type;
        if source_type.is_unambiguous() {
            if module_record.has_module_syntax {
                // Resolved to Module - discard deferred script errors (TLA is valid in ESM)
                // but emit deferred module errors (HTML comments are invalid in ESM)
                program.source_type = source_type.with_module(true);
                errors.append(&mut self.lexer.deferred_module_errors);
            } else {
                // Resolved to Script - emit deferred script errors
                // discard deferred module errors (HTML comments are valid in scripts)
                program.source_type = source_type.with_script(true);
                errors.extend(self.deferred_script_errors);
            }
        }

        let tokens =
            if panicked { ArenaVec::new_in(&self.ast) } else { self.lexer.finalize_tokens() };

        program.comments = self.lexer.trivia_builder.comments;

        ParserReturn {
            program,
            module_record,
            diagnostics: errors,
            irregular_whitespaces,
            tokens,
            panicked,
            is_flow_language,
        }
    }

    pub fn parse_expression(mut self) -> Result<Expression<'a>, Diagnostics> {
        // initialize cur_token and prev_token by moving onto the first token
        self.bump_any();
        let expr = self.parse_expr();
        if let Some(FatalError { error, .. }) = self.fatal_error.take() {
            return Err(error.into());
        }
        self.check_unfinished_errors();
        let errors = self.lexer.errors.into_iter().chain(self.errors).collect::<Diagnostics>();
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
        let (directives, mut statements) =
            self.parse_directives_and_statements(/* in_ts_namespace_body */ false);

        // In unambiguous mode, if ESM syntax was detected (import/export/import.meta),
        // we need to reparse statements that were originally parsed with `await` as identifier.
        // TypeScript's behavior: initially parse `await /x/` as division, then reparse as
        // await expression with regex when ESM is detected.
        if self.source_type.is_unambiguous()
            && self.module_record_builder.has_module_syntax()
            && !self.state.potential_await_reparse.is_empty()
        {
            self.reparse_potential_top_level_awaits(&mut statements);
        }

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
            self,
        )
    }

    /// Reparse statements that may contain top-level await expressions.
    ///
    /// In unambiguous mode, statements like `await /x/u` are initially parsed as
    /// `await / x / u` (identifier with divisions). If ESM syntax is detected,
    /// we need to reparse them with the await context enabled.
    fn reparse_potential_top_level_awaits(&mut self, statements: &mut ArenaVec<'a, Statement<'a>>) {
        // Token stream is already complete from the first parse.
        // Reparsing here is only to patch AST nodes, so keep the original token stream.
        let original_tokens =
            if self.lexer.config.tokens() { Some(self.lexer.take_tokens()) } else { None };

        let checkpoints = std::mem::take(&mut self.state.potential_await_reparse);
        for (stmt_index, checkpoint) in checkpoints {
            // Rewind to the checkpoint
            self.rewind(checkpoint);

            // Parse the statement with await context enabled (TopLevel context is already set)
            let stmt = self.context_add(Context::Await, |p| {
                p.parse_statement_list_item(StatementContext::StatementList)
            });

            // Replace the statement if the index is valid
            if stmt_index < statements.len() {
                statements[stmt_index] = stmt;
            }
        }

        if let Some(original_tokens) = original_tokens {
            self.lexer.set_tokens(original_tokens);
        }
    }

    fn default_context(source_type: SourceType, options: &ParseOptions) -> Context {
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
    fn flow_error(&mut self) -> Option<OxcDiagnostic> {
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
        use oxc_span::GetSpan;
        // PropertyDefinition : cover_initialized_name
        // It is a Syntax Error if any source text is matched by this production.
        for expr in self.state.cover_initialized_name.values() {
            self.errors.push(diagnostics::cover_initialized_name(expr.span()));
        }
    }

    /// Check if source length exceeds MAX_LEN, if the file cannot be parsed.
    /// Original parsing error is not real - `Lexer::new` substituted "\0" as the source text.
    #[cold]
    fn overlong_error(&self) -> Option<OxcDiagnostic> {
        if self.source_text.len() > MAX_LEN {
            return Some(diagnostics::overlong_source());
        }
        None
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
    type Builder = AstBuilder<'a>;

    #[inline]
    fn builder(&self) -> &AstBuilder<'a> {
        &self.ast
    }
}

#[cfg(test)]
mod test {
    use oxc_ast::ast::AnnotationElement;
    use std::path::Path;

    use oxc_ast::ast::{ClassElement, CommentKind, Expression, Statement, StructElement};
    use oxc_span::GetSpan;

    use super::*;

    #[test]
    fn parse_program_smoke_test() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let source = "";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.program.is_empty());
        assert!(ret.diagnostics.is_empty());
        assert!(!ret.is_flow_language);
    }

    #[test]
    fn parse_expression_smoke_test() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let source = "a";
        let expr = Parser::new(&allocator, source, source_type).parse_expression().unwrap();
        assert!(matches!(expr, Expression::Identifier(_)));
    }

    #[test]
    fn ets_static_must_be_selected_explicitly() {
        let allocator = Allocator::default();
        let source = "let character: char = c'a'; let value: float = 1.25f;";

        let static_ret = Parser::new(&allocator, source, SourceType::ets_static()).parse();
        assert!(static_ret.diagnostics.is_empty(), "Errors: {:?}", static_ret.diagnostics);

        let inferred = SourceType::from_path(Path::new("example.ets")).unwrap();
        assert!(inferred.is_arkui());
        assert!(!inferred.is_ets_static());
        let arkui_ret = Parser::new(&allocator, source, inferred).parse();
        assert!(
            !arkui_ret.diagnostics.is_empty(),
            "A plain .ets path must not activate static ETS literals"
        );
    }

    #[test]
    fn ets_static_estree_metadata_is_conditional() {
        let allocator = Allocator::default();
        let legacy_source = r#"
            const value: number = 1;
            class A {}
            function f(): void {}
            enum E { A }
            export { value };
            ({ key: 1 });
        "#;
        let legacy = Parser::new(&allocator, legacy_source, SourceType::ts()).parse();
        assert!(legacy.diagnostics.is_empty(), "Errors: {:?}", legacy.diagnostics);
        let legacy_json = legacy.program.to_estree_json(true, false);
        assert!(!legacy_json.contains("\"type\":\"VariableDeclaration\",\"decorators\""));
        for field in ["final", "native", "underlyingType", "etsSingle", "etsEquals"] {
            assert!(!legacy_json.contains(&format!("\"{field}\":")), "{legacy_json}");
        }

        let allocator = Allocator::default();
        let static_source = r#"
            @interface Mark {}
            @Mark const value: int = 1;
            final class A {}
            native function f(): void;
            enum E: int { A }
        "#;
        let static_ret = Parser::new(&allocator, static_source, SourceType::ets_static()).parse();
        assert!(static_ret.diagnostics.is_empty(), "Errors: {:?}", static_ret.diagnostics);
        let static_json = static_ret.program.to_estree_json(true, false);
        assert!(static_json.contains("\"type\":\"VariableDeclaration\",\"decorators\""));
        assert!(static_json.contains("\"final\":true"));
        assert!(static_json.contains("\"native\":true"));
        assert!(static_json.contains("\"underlyingType\":"));
    }

    #[test]
    fn ets_static_parses_es2panda_declarations() {
        let allocator = Allocator::default();
        let source = r#"
            package example.parser;

            @interface Mark { value: string = "ok" }
            @Mark("ok")
            enum Color: int { Red, Green }

            class A {}
            class B {}

            function fromA(value: A): A { return value }
            function fromB(value: B): B { return value }
            overload convert { fromA, fromB }

            interface Converter {
                fromA(value: A): A;
                fromB(value: B): B;
                overload convert { fromA, fromB };
            }

            class Factory {
                constructor fromA(value: A) {}
                constructor fromB(value: B) {}
                overload constructor { fromA, fromB }
            }

            struct Point { x: int = 0 }
        "#;
        let ret = Parser::new(&allocator, source, SourceType::ets_static()).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);
        assert!(
            ret.program
                .body
                .iter()
                .any(|statement| matches!(statement, Statement::ETSPackageDeclaration(_)))
        );
        assert!(
            ret.program
                .body
                .iter()
                .any(|statement| matches!(statement, Statement::AnnotationDeclaration(_)))
        );
        assert!(
            ret.program
                .body
                .iter()
                .any(|statement| matches!(statement, Statement::TSEnumDeclaration(_)))
        );
        assert!(
            ret.program
                .body
                .iter()
                .any(|statement| matches!(statement, Statement::ETSOverloadDeclaration(_)))
        );
        assert!(
            ret.program
                .body
                .iter()
                .any(|statement| matches!(statement, Statement::StructStatement(_)))
        );
    }

    #[test]
    fn ets_static_preserves_expression_nodes() {
        let allocator = Allocator::default();
        let source = r#"
            class Value {}
            let value: Value = new Value()
            let values: int[] = new int[3]
            let matches: boolean = value instanceof Value
            function consume(): void {}
            consume() { let nested: int = 1 }
        "#;
        let ret = Parser::new(&allocator, source, SourceType::ets_static()).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);

        let initializers = ret.program.body.iter().filter_map(|statement| {
            let Statement::VariableDeclaration(declaration) = statement else { return None };
            declaration.declarations[0].init.as_ref()
        });
        let expression_kinds = initializers
            .map(|expression| match expression {
                Expression::ETSNewClassInstanceExpression(_) => "class",
                Expression::ETSNewArrayInstanceExpression(_) => "array",
                Expression::ETSInstanceOfExpression(_) => "instanceof",
                _ => "other",
            })
            .collect::<Vec<_>>();
        assert_eq!(expression_kinds, ["class", "array", "instanceof"]);

        let Statement::ExpressionStatement(trailing_block) = &ret.program.body[5] else {
            panic!("Expected a trailing-block expression statement");
        };
        assert!(matches!(trailing_block.expression, Expression::ETSTrailingBlockExpression(_)));
    }

    #[test]
    fn ets_static_matches_es2panda_array_and_await_parsing() {
        let allocator = Allocator::default();
        let source = r#"
            let values: int[] = new int[3]
            let matrix: int[][] = new int[2][3]
            function resolve(promise: Promise<int>): int {
                return await promise
            }
        "#;
        let ret = Parser::new(&allocator, source, SourceType::ets_static()).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);

        let initializers = ret.program.body.iter().take(2).map(|statement| {
            let Statement::VariableDeclaration(declaration) = statement else {
                panic!("Expected a variable declaration");
            };
            declaration.declarations[0].init.as_ref().expect("Expected an initializer")
        });
        assert!(matches!(
            initializers.collect::<Vec<_>>().as_slice(),
            [
                Expression::ETSNewArrayInstanceExpression(_),
                Expression::ETSNewMultiDimArrayInstanceExpression(_)
            ]
        ));
    }

    #[test]
    fn ets_static_validates_annotations_and_exports() {
        let allocator = Allocator::default();
        let invalid_annotation = r#"
            class Value {}
            @interface Mark { value: string }
            @Mark({ value: new Value() })
            class Decorated {}
        "#;
        let ret = Parser::new(&allocator, invalid_annotation, SourceType::ets_static()).parse();
        assert!(!ret.diagnostics.is_empty());

        let forward_export = "export { value }; const value: int = 1;";
        let ret = Parser::new(&allocator, forward_export, SourceType::ets_static()).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);

        let unknown_export = "export { missing }; const value: int = 1;";
        let ret = Parser::new(&allocator, unknown_export, SourceType::ets_static()).parse();
        assert!(!ret.diagnostics.is_empty());

        let self_reexport = "export { value } from './self.ets';";
        let ret = Parser::new(&allocator, self_reexport, SourceType::ets_static())
            .with_source_path("/project/self.ets")
            .parse();
        assert!(!ret.diagnostics.is_empty());
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
        for source in sources {
            let ret = Parser::new(&allocator, source, source_type).parse();
            assert!(ret.is_flow_language);
            assert_eq!(ret.diagnostics.len(), 1);
            assert_eq!(ret.diagnostics.first().unwrap().to_string(), "Flow is not supported");
        }
    }

    #[test]
    fn ts_module_declaration() {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(Path::new("module.ts")).unwrap();
        let source = "declare module 'test'\n";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert_eq!(ret.diagnostics.len(), 0);
    }

    #[test]
    fn directives() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let sources = [
            ("import x from 'foo'; 'use strict';", 2),
            ("export {x} from 'foo'; 'use strict';", 2),
            (";'use strict';", 2),
        ];
        for (source, body_length) in sources {
            let ret = Parser::new(&allocator, source, source_type).parse();
            assert!(ret.program.directives.is_empty(), "{source}");
            assert_eq!(ret.program.body.len(), body_length, "{source}");
        }
    }

    #[test]
    fn v8_intrinsics() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        {
            let source = "%DebugPrint('Raging against the Dying Light')";
            let opts = ParseOptions { allow_v8_intrinsics: true, ..ParseOptions::default() };
            let ret = Parser::new(&allocator, source, source_type).with_options(opts).parse();
            assert!(ret.diagnostics.is_empty());

            if let Some(Statement::ExpressionStatement(expr_stmt)) = ret.program.body.first() {
                if let Expression::V8IntrinsicExpression(expr) = &expr_stmt.expression {
                    assert_eq!(expr.span().source_text(source), source);
                } else {
                    panic!("Expected V8IntrinsicExpression");
                }
            } else {
                panic!("Expected ExpressionStatement");
            }
        }
        {
            let source = "%DebugPrint(...illegalSpread)";
            let opts = ParseOptions { allow_v8_intrinsics: true, ..ParseOptions::default() };
            let ret = Parser::new(&allocator, source, source_type).with_options(opts).parse();
            assert_eq!(ret.diagnostics.len(), 1);
            assert_eq!(
                ret.diagnostics[0].to_string(),
                "V8 runtime calls cannot have spread elements as arguments"
            );
        }
        {
            let source = "%DebugPrint('~~')";
            let ret = Parser::new(&allocator, source, source_type).parse();
            assert_eq!(ret.diagnostics.len(), 1);
            assert_eq!(ret.diagnostics[0].to_string(), "Unexpected token");
        }
        {
            // https://github.com/oxc-project/oxc/issues/12121
            let source = "interface Props extends %enuProps {}";
            let source_type = SourceType::default().with_typescript(true);
            // Should not panic whether `allow_v8_intrinsics` is set or not.
            let opts = ParseOptions { allow_v8_intrinsics: true, ..ParseOptions::default() };
            let ret = Parser::new(&allocator, source, source_type).with_options(opts).parse();
            assert_eq!(ret.diagnostics.len(), 1);
            let ret = Parser::new(&allocator, source, source_type).parse();
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
        for (source, kind) in sources {
            let ret = Parser::new(&allocator, source, source_type).parse();
            let comments = &ret.program.comments;
            assert_eq!(comments.len(), 1, "{source}");
            assert_eq!(comments.first().unwrap().kind, kind, "{source}");
        }
    }

    #[test]
    fn hashbang() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let source = "#!/usr/bin/node\n;";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert_eq!(ret.program.hashbang.unwrap().value.as_str(), "/usr/bin/node");
    }

    #[test]
    fn unambiguous() {
        let allocator = Allocator::default();
        let source_type = SourceType::unambiguous();
        assert!(source_type.is_unambiguous());
        let sources = ["import x from 'foo';", "export {x} from 'foo';", "import.meta"];
        for source in sources {
            let ret = Parser::new(&allocator, source, source_type).parse();
            assert!(ret.program.source_type.is_module());
        }

        let sources = ["", "import('foo')"];
        for source in sources {
            let ret = Parser::new(&allocator, source, source_type).parse();
            assert!(ret.program.source_type.is_script());
        }
    }

    #[test]
    fn binary_file() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();

        // U+FFFD as a standalone token — file appears to be binary
        let ret = Parser::new(&allocator, "\u{FFFD}", source_type).parse();
        assert!(ret.program.is_empty());
        assert_eq!(ret.diagnostics.len(), 1);
        assert_eq!(ret.diagnostics[0].to_string(), "File appears to be binary.");

        // U+FFFD inside string literals — should parse fine
        let ret = Parser::new(&allocator, "\"oops \u{FFFD} oops\";", source_type).parse();
        assert!(!ret.program.is_empty());
        assert!(ret.diagnostics.is_empty());
    }

    #[test]
    fn memory_leak() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let sources = ["2n", ";'1234567890123456789012345678901234567890'"];
        for source in sources {
            let ret = Parser::new(&allocator, source, source_type).parse();
            assert!(!ret.program.body.is_empty());
        }
    }

    // Source with length MAX_LEN + 1 fails to parse.
    // Skip this test on 32-bit systems as impossible to allocate a string longer than `isize::MAX`.
    // Also skip running under Miri since it takes so long.
    #[cfg(target_pointer_width = "64")]
    #[cfg(not(miri))]
    #[test]
    fn overlong_source() {
        use std::{
            alloc::{self, Layout},
            ptr::NonNull,
            slice, str,
        };

        /// A string that has a length of `MAX_LEN + 1`, and is entirely zeros.
        ///
        /// We need to create a `&str` with `MAX_LEN + 1` length, but don't want to write 4 GiB of data,
        /// as it's too slow. This type uses `alloc_zeroed` which on most platforms will just create zeroed pages
        /// without actually writing any data, and so is much faster.
        struct ZeroedString {
            ptr: NonNull<u8>,
        }

        impl ZeroedString {
            const LEN: usize = MAX_LEN + 1;
            const PAGE_SIZE: usize = 4096;
            const LAYOUT: Layout = match Layout::from_size_align(Self::LEN, Self::PAGE_SIZE) {
                Ok(layout) => layout,
                Err(_) => panic!("Failed to create layout"),
            };

            fn new() -> Self {
                // SAFETY: `LAYOUT` is valid and non-zero size.
                let ptr = unsafe { alloc::alloc_zeroed(Self::LAYOUT) };
                let Some(ptr) = NonNull::new(ptr) else {
                    panic!("Failed to allocate {} bytes", Self::LEN);
                };
                Self { ptr }
            }

            fn as_str(&self) -> &str {
                // SAFETY: `self.ptr` is pointer to start of `LEN` initialized and zeroed bytes.
                // A slice consisting entirely of zeros is valid UTF-8.
                unsafe {
                    str::from_utf8_unchecked(slice::from_raw_parts(self.ptr.as_ptr(), Self::LEN))
                }
            }
        }

        impl Drop for ZeroedString {
            fn drop(&mut self) {
                // SAFETY: `self.ptr` is address of an allocation made with `LAYOUT`
                unsafe { alloc::dealloc(self.ptr.as_ptr(), Self::LAYOUT) };
            }
        }

        // Create long source text (MAX_LEN + 1 bytes)
        let zeroed_string = ZeroedString::new();
        let source_text = zeroed_string.as_str();

        // Attempt to parse the source text
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, source_text, SourceType::default()).parse();

        // Parsing should fail
        assert!(ret.program.is_empty());
        assert!(ret.panicked);
        assert_eq!(ret.diagnostics.len(), 1);
        assert_eq!(
            ret.diagnostics.first().unwrap().to_string(),
            "Source length exceeds 4 GiB limit"
        );
    }

    // Source with length MAX_LEN parses OK.
    // This test takes over 1 minute on an M1 Macbook Pro unless compiled in release mode.
    // `not(debug_assertions)` is a proxy for detecting release mode.
    // Also skip running under Miri since it takes so long.
    #[cfg(not(debug_assertions))]
    #[cfg(not(miri))]
    #[test]
    fn legal_length_source() {
        // Build a string MAX_LEN bytes long which doesn't take too long to parse
        let head = "const x = 1;\n/*";
        let foot = "*/\nconst y = 2;\n";
        let mut source = "x".repeat(MAX_LEN);
        source.replace_range(..head.len(), head);
        source.replace_range(MAX_LEN - foot.len().., foot);
        assert_eq!(source.len(), MAX_LEN);

        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, &source, SourceType::default()).parse();
        assert!(!ret.panicked);
        assert!(ret.diagnostics.is_empty());
        assert_eq!(ret.program.body.len(), 2);
    }

    #[test]
    fn arkui_struct_statement() {
        let allocator = Allocator::default();
        // Use ETS source type for ArkUI
        let source_type = SourceType::ets();
        let source = "@Component\nstruct MyComponent {\n  @State message: string = 'Hello';\n  build() {\n    Column() {}\n  }\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);
        assert_eq!(ret.program.body.len(), 1);
        if let Statement::StructStatement(struct_stmt) = &ret.program.body[0] {
            assert_eq!(struct_stmt.id.name.as_str(), "MyComponent");
            assert_eq!(struct_stmt.body.body.len(), 2); // property and method
        } else {
            panic!("Expected StructStatement");
        }
    }

    #[test]
    fn arkui_annotation_declaration() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "@interface MyAnnotation {\n  value: string;\n  count: number;\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);
        assert_eq!(ret.program.body.len(), 1);
        if let Statement::AnnotationDeclaration(annotation) = &ret.program.body[0] {
            assert_eq!(annotation.id.name.as_str(), "MyAnnotation");
            assert_eq!(annotation.body.body.len(), 2); // two properties
        } else {
            panic!("Expected AnnotationDeclaration, got: {:?}", ret.program.body[0]);
        }
    }

    #[test]
    fn arkui_annotation_declaration_with_default_value() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source =
            "@interface MyAnnotation {\n  value: string = 'default';\n  count: number = 10;\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);
        assert_eq!(ret.program.body.len(), 1);
        if let Statement::AnnotationDeclaration(annotation) = &ret.program.body[0] {
            assert_eq!(annotation.id.name.as_str(), "MyAnnotation");
            assert_eq!(annotation.body.body.len(), 2);
            // Check that both properties have default values
            let AnnotationElement::PropertyDefinition(prop1) = &annotation.body.body[0];
            assert!(prop1.value.is_some());
            let AnnotationElement::PropertyDefinition(prop2) = &annotation.body.body[1];
            assert!(prop2.value.is_some());
        } else {
            panic!("Expected AnnotationDeclaration");
        }
    }

    #[test]
    fn arkui_annotation_declaration_empty() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "@interface EmptyAnnotation {}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);
        assert_eq!(ret.program.body.len(), 1);
        if let Statement::AnnotationDeclaration(annotation) = &ret.program.body[0] {
            assert_eq!(annotation.id.name.as_str(), "EmptyAnnotation");
            assert_eq!(annotation.body.body.len(), 0);
        } else {
            panic!("Expected AnnotationDeclaration");
        }
    }

    #[test]
    fn arkui_component_expression_with_newline() {
        let allocator = Allocator::default();
        // Use ETS source type for ArkUI
        let source_type = SourceType::ets();
        // Test case where `{` is on a new line after `Column()`
        let source = "struct MyComponent {\n  build() {\n    Column()\n    {\n      Text('Hello')\n    }\n  }\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);
    }

    #[test]
    fn arkui_component_expression_in_method() {
        let allocator = Allocator::default();
        // Use ETS source type for ArkUI
        let source_type = SourceType::ets();
        // Test case from user's error report
        let source = "struct MyComponent {\n  build() {\n    Column() {\n      Text(this.lastName + ' ' + this.firstName)\n    }\n  }\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);
    }

    #[test]
    fn ets_plain_function_keeps_ts_call_then_block() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "function test() {\n  foo()\n  {\n    bar()\n  }\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);

        let Statement::FunctionDeclaration(func) = &ret.program.body[0] else {
            panic!("Expected FunctionDeclaration");
        };
        let body = func.body.as_ref().expect("function should have a body");
        assert_eq!(body.statements.len(), 2);
        assert!(matches!(body.statements[0], Statement::ExpressionStatement(_)));
        assert!(matches!(body.statements[1], Statement::BlockStatement(_)));
    }

    #[test]
    fn ets_plain_function_keeps_ts_assignment_then_block() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "function test() {\n  MPPerformance.loadBundleHitCache = false\n  {\n    const memoryCachedBundle = load()\n  }\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);

        let Statement::FunctionDeclaration(func) = &ret.program.body[0] else {
            panic!("Expected FunctionDeclaration");
        };
        let body = func.body.as_ref().expect("function should have a body");
        assert_eq!(body.statements.len(), 2);
        assert!(matches!(body.statements[0], Statement::ExpressionStatement(_)));
        assert!(matches!(body.statements[1], Statement::BlockStatement(_)));
    }

    #[test]
    fn ets_plain_function_keeps_ts_return_newline_before_object() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "function test() {\n  return\n  {\n    value: 1\n  }\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);

        let Statement::FunctionDeclaration(func) = &ret.program.body[0] else {
            panic!("Expected FunctionDeclaration");
        };
        let body = func.body.as_ref().expect("function should have a body");
        assert_eq!(body.statements.len(), 2);
        if let Statement::ReturnStatement(return_stmt) = &body.statements[0] {
            assert!(return_stmt.argument.is_none());
        } else {
            panic!("Expected ReturnStatement");
        }
        assert!(matches!(body.statements[1], Statement::BlockStatement(_)));
    }

    #[test]
    fn ets_plain_function_does_not_enable_arkui_leading_dot() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "function test() {\n  .fontSize(16)\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(!ret.diagnostics.is_empty(), "Plain ETS should keep TS syntax errors");
    }

    #[test]
    fn arkui_builder_function_enables_component_dsl() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "@Builder\nfunction test() {\n  Column()\n  {\n    Text('Hello')\n  }\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);

        let Statement::FunctionDeclaration(func) = &ret.program.body[0] else {
            panic!("Expected FunctionDeclaration");
        };
        let body = func.body.as_ref().expect("function should have a body");
        assert_eq!(body.statements.len(), 1);
        if let Statement::ExpressionStatement(stmt) = &body.statements[0] {
            assert!(matches!(stmt.expression, Expression::ArkUIComponentExpression(_)));
        } else {
            panic!("Expected ExpressionStatement");
        }
    }

    #[test]
    fn arkui_local_builder_function_enables_component_dsl() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "@LocalBuilder\nfunction test() {\n  Column() { Text('Hello') }\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);

        let Statement::FunctionDeclaration(func) = &ret.program.body[0] else {
            panic!("Expected FunctionDeclaration");
        };
        let body = func.body.as_ref().expect("function should have a body");
        assert_eq!(body.statements.len(), 1);
        if let Statement::ExpressionStatement(stmt) = &body.statements[0] {
            assert!(matches!(stmt.expression, Expression::ArkUIComponentExpression(_)));
        } else {
            panic!("Expected ExpressionStatement");
        }
    }

    #[test]
    fn arkui_animatable_extend_function_enables_leading_dot_dsl() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "@AnimatableExtend(Text)\nfunction test() {\n  .fontSize(16)\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);

        let Statement::FunctionDeclaration(func) = &ret.program.body[0] else {
            panic!("Expected FunctionDeclaration");
        };
        let body = func.body.as_ref().expect("function should have a body");
        assert_eq!(body.statements.len(), 1);
        if let Statement::ExpressionStatement(stmt) = &body.statements[0] {
            assert!(matches!(stmt.expression, Expression::LeadingDotExpression(_)));
        } else {
            panic!("Expected ExpressionStatement");
        }
    }

    #[test]
    fn arkui_builder_methods_enable_component_dsl() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "struct Test {\n  @Builder\n  itemBuilder() {\n    Column() { Text('Hello') }\n  }\n  @LocalBuilder\n  localItemBuilder() {\n    Row() { Text('World') }\n  }\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);

        let Statement::StructStatement(struct_stmt) = &ret.program.body[0] else {
            panic!("Expected StructStatement");
        };
        assert_eq!(struct_stmt.body.body.len(), 2);
        for element in &struct_stmt.body.body {
            let StructElement::MethodDefinition(method) = element else {
                panic!("Expected MethodDefinition");
            };
            let body = method.value.body.as_ref().expect("method should have a body");
            assert_eq!(body.statements.len(), 1);
            if let Statement::ExpressionStatement(stmt) = &body.statements[0] {
                assert!(matches!(stmt.expression, Expression::ArkUIComponentExpression(_)));
            } else {
                panic!("Expected ExpressionStatement");
            }
        }
    }

    #[test]
    fn arkui_struct_page_transition_enables_component_dsl() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "struct Test {\n  pageTransition() {\n    PageTransitionEnter() { Text('Hello') }\n  }\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);

        let Statement::StructStatement(struct_stmt) = &ret.program.body[0] else {
            panic!("Expected StructStatement");
        };
        let StructElement::MethodDefinition(method) = &struct_stmt.body.body[0] else {
            panic!("Expected MethodDefinition");
        };
        let body = method.value.body.as_ref().expect("method should have a body");
        assert_eq!(body.statements.len(), 1);
        if let Statement::ExpressionStatement(stmt) = &body.statements[0] {
            assert!(matches!(stmt.expression, Expression::ArkUIComponentExpression(_)));
        } else {
            panic!("Expected ExpressionStatement");
        }
    }

    #[test]
    fn arkui_builder_class_method_enables_component_dsl() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source =
            "class Test {\n  @Builder\n  itemBuilder() {\n    Column() { Text('Hello') }\n  }\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);

        let Statement::ClassDeclaration(class_decl) = &ret.program.body[0] else {
            panic!("Expected ClassDeclaration");
        };
        let ClassElement::MethodDefinition(method) = &class_decl.body.body[0] else {
            panic!("Expected MethodDefinition");
        };
        let body = method.value.body.as_ref().expect("method should have a body");
        assert_eq!(body.statements.len(), 1);
        if let Statement::ExpressionStatement(stmt) = &body.statements[0] {
            assert!(matches!(stmt.expression, Expression::ArkUIComponentExpression(_)));
        } else {
            panic!("Expected ExpressionStatement");
        }
    }

    #[test]
    fn arkui_ui_callback_context_does_not_leak_into_arbitrary_callbacks() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = r#"struct Test {
  build() {
    ForEach(items, item => { Text(item) })
    helper(() => {
      Text()
      {}
    })
    Repeat(items).each(item => { Text(item) })
  }
}"#;
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);

        let Statement::StructStatement(structure) = &ret.program.body[0] else {
            panic!("Expected StructStatement");
        };
        let StructElement::MethodDefinition(build) = &structure.body.body[0] else {
            panic!("Expected build method");
        };
        let statements = &build.value.body.as_ref().unwrap().statements;

        let Statement::ExpressionStatement(for_each_statement) = &statements[0] else {
            panic!("Expected ForEach expression statement");
        };
        let Expression::CallExpression(for_each) = &for_each_statement.expression else {
            panic!("Expected ForEach call");
        };
        let Expression::ArrowFunctionExpression(ui_callback) =
            for_each.arguments[1].as_expression().unwrap()
        else {
            panic!("Expected ForEach UI callback");
        };
        let Statement::ExpressionStatement(text_statement) = &ui_callback.body.statements[0] else {
            panic!("Expected Text expression statement");
        };
        assert!(matches!(text_statement.expression, Expression::ArkUIComponentExpression(_)));

        let Statement::ExpressionStatement(helper_statement) = &statements[1] else {
            panic!("Expected helper expression statement");
        };
        let Expression::CallExpression(helper) = &helper_statement.expression else {
            panic!("Expected helper call");
        };
        let Expression::ArrowFunctionExpression(ordinary_callback) =
            helper.arguments[0].as_expression().unwrap()
        else {
            panic!("Expected ordinary callback");
        };
        let Statement::ExpressionStatement(text_statement) = &ordinary_callback.body.statements[0]
        else {
            panic!("Expected ordinary Text call");
        };
        assert!(matches!(text_statement.expression, Expression::CallExpression(_)));
        assert!(matches!(ordinary_callback.body.statements[1], Statement::BlockStatement(_)));

        let Statement::ExpressionStatement(repeat_statement) = &statements[2] else {
            panic!("Expected Repeat.each expression statement");
        };
        let Expression::CallExpression(repeat_each) = &repeat_statement.expression else {
            panic!("Expected Repeat.each call");
        };
        let Expression::ArrowFunctionExpression(repeat_callback) =
            repeat_each.arguments[0].as_expression().unwrap()
        else {
            panic!("Expected Repeat.each UI callback");
        };
        let Statement::ExpressionStatement(text_statement) = &repeat_callback.body.statements[0]
        else {
            panic!("Expected Repeat.each Text statement");
        };
        assert!(matches!(text_statement.expression, Expression::ArkUIComponentExpression(_)));
    }

    #[test]
    fn arkui_attribute_ui_callback_follows_component_call_chain() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let options = ArkTsOptions {
            components: vec!["Text".into(), "Row".into()],
            ..ArkTsOptions::default()
        };
        let source = r"struct Test {
  build() {
    Repeat(items)
      .key(item => item.id)
      .each(item => { Text(item.value) })
      .templateId(item => item.kind)
      .template('card', item => { Row() { Text(item.value) } })
  }
}";
        let ret = Parser::new(&allocator, source, source_type).with_arkts_options(options).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);
    }

    #[test]
    fn arkui_dsl_decorators_require_openharmony_shapes() {
        for decorator in ["@Builder()", "@Namespace.Builder", "@Extend"] {
            let allocator = Allocator::default();
            let source_type = SourceType::ets();
            let source = format!("{decorator}\nfunction test() {{\n  Column()\n  {{}}\n}}");
            let ret = Parser::new(&allocator, &source, source_type).parse();
            assert!(ret.diagnostics.is_empty(), "{decorator}: {:?}", ret.diagnostics);

            let Statement::FunctionDeclaration(function) = &ret.program.body[0] else {
                panic!("Expected FunctionDeclaration for {decorator}");
            };
            let statements = &function.body.as_ref().unwrap().statements;
            let Statement::ExpressionStatement(column) = &statements[0] else {
                panic!("Expected Column call for {decorator}");
            };
            assert!(
                matches!(column.expression, Expression::CallExpression(_)),
                "{decorator} must not enable ArkUI DSL"
            );
            assert!(matches!(statements[1], Statement::BlockStatement(_)));
        }
    }

    #[test]
    fn arkts_options_control_components_render_methods_and_annotations() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let options = ArkTsOptions {
            components: vec!["CustomRoot".into()],
            render_methods: vec!["render".into()],
            annotations: false,
            ..ArkTsOptions::default()
        };
        let source = r#"struct Test {
  render() {
    CustomRoot() {}
    Column()
  }
}"#;
        let ret = Parser::new(&allocator, source, source_type)
            .with_arkts_options(options.clone())
            .parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);
        let Statement::StructStatement(structure) = &ret.program.body[0] else {
            panic!("Expected StructStatement");
        };
        let StructElement::MethodDefinition(render) = &structure.body.body[0] else {
            panic!("Expected render method");
        };
        let statements = &render.value.body.as_ref().unwrap().statements;
        let Statement::ExpressionStatement(custom) = &statements[0] else {
            panic!("Expected custom component statement");
        };
        assert!(matches!(custom.expression, Expression::ArkUIComponentExpression(_)));
        let Statement::ExpressionStatement(column) = &statements[1] else {
            panic!("Expected Column call statement");
        };
        assert!(matches!(column.expression, Expression::CallExpression(_)));

        let annotation = Parser::new(&allocator, "@interface Disabled {}", source_type)
            .with_arkts_options(options)
            .parse();
        assert!(!annotation.diagnostics.is_empty(), "annotation option should disable grammar");
    }

    #[test]
    fn arkui_component_expression() {
        let allocator = Allocator::default();
        // Use ETS source type for ArkUI
        let source_type = SourceType::ets();
        let source = "@Builder\nfunction test() {\n  Column() { Text('Hello') }\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);
        let Statement::FunctionDeclaration(func) = &ret.program.body[0] else {
            panic!("Expected FunctionDeclaration");
        };
        let body = func.body.as_ref().expect("function should have a body");
        if let Statement::ExpressionStatement(stmt) = &body.statements[0] {
            if let Expression::ArkUIComponentExpression(component) = &stmt.expression {
                assert_eq!(component.children.len(), 1);
            } else {
                panic!("Expected ArkUIComponentExpression");
            }
        } else {
            panic!("Expected ExpressionStatement");
        }
    }

    #[test]
    fn arkui_struct_optional_method() {
        let allocator = Allocator::default();
        // Use ETS source type for ArkUI
        let source_type = SourceType::ets();
        let source = "struct A {\n  aboutToAppear?(): void {\n  }\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);
        assert_eq!(ret.program.body.len(), 1);
        if let Statement::StructStatement(struct_stmt) = &ret.program.body[0] {
            assert_eq!(struct_stmt.id.name.as_str(), "A");
            assert_eq!(struct_stmt.body.body.len(), 1);
            if let StructElement::MethodDefinition(method_def) = &struct_stmt.body.body[0] {
                assert!(method_def.optional, "Method should be optional");
                assert_eq!(method_def.key.static_name().unwrap(), "aboutToAppear");
            } else {
                panic!("Expected MethodDefinition");
            }
        } else {
            panic!("Expected StructStatement");
        }
    }

    #[test]
    fn arkui_return_object_literal_with_newlines() {
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        // Test object literal in return statement with commas on new lines
        let source = "struct Test {\n  test(): void {\n    return {\n      x: 1,\n      y: 2\n    };\n  }\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);
    }

    #[test]
    fn arkui_component_with_chain() {
        let allocator = Allocator::default();
        // Use ETS source type for ArkUI
        let source_type = SourceType::ets();
        let source = "Button('Click').onClick(() => {})";
        let expr = Parser::new(&allocator, source, source_type).parse_expression().unwrap();
        // The chain expression should be parsed as a CallExpression wrapping the ArkUIComponentExpression
        if let Expression::CallExpression(_) = expr {
            // This is expected - the chain creates a CallExpression
        } else {
            panic!("Expected CallExpression with chain");
        }
    }

    #[test]
    fn tsx_should_not_parse_as_arkui() {
        // Test that TSX code (function calls followed by {) should NOT be parsed as ArkUI
        // This prevents infinite loops when parsing TSX files
        let allocator = Allocator::default();
        let source_type = SourceType::from_path("test.tsx").unwrap();
        assert!(source_type.is_jsx(), "Source type should be JSX");
        assert!(!source_type.is_arkui(), "TSX should not be ArkUI");

        // Common TSX pattern: function call followed by block
        let source = "function Component() { return <div>{value}</div> }";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(
            ret.diagnostics.is_empty(),
            "Should parse TSX without errors: {:?}",
            ret.diagnostics
        );
        assert!(!ret.panicked, "Should not panic when parsing TSX");

        // Another common pattern: arrow function with block
        let source2 = "const fn = () => { return <div>Hello</div> }";
        let ret2 = Parser::new(&allocator, source2, source_type).parse();
        assert!(
            ret2.diagnostics.is_empty(),
            "Should parse TSX arrow function without errors: {:?}",
            ret2.diagnostics
        );
        assert!(!ret2.panicked, "Should not panic when parsing TSX arrow function");
    }

    #[test]
    fn ets_should_parse_arkui() {
        // Test that ETS files can parse ArkUI syntax
        let allocator = Allocator::default();
        let source_type = SourceType::from_path("test.ets").unwrap();
        assert!(source_type.is_arkui(), "ETS source type should be ArkUI");
        assert!(source_type.is_typescript(), "ETS should be TypeScript");

        // Test ArkUI component expression in a DSL-bearing ETS AST context.
        let source = "struct Test {\n  build() {\n    Column() { Text('Hello') }\n  }\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);
        let Statement::StructStatement(struct_stmt) = &ret.program.body[0] else {
            panic!("Expected StructStatement");
        };
        let StructElement::MethodDefinition(method) = &struct_stmt.body.body[0] else {
            panic!("Expected MethodDefinition");
        };
        let body = method.value.body.as_ref().expect("method should have a body");
        if let Statement::ExpressionStatement(stmt) = &body.statements[0] {
            assert!(matches!(stmt.expression, Expression::ArkUIComponentExpression(_)));
        } else {
            panic!("Expected ExpressionStatement");
        }
    }

    #[test]
    fn arkui_component_span_covers_chain() {
        // Regression: the span must include the trailing chain, otherwise the formatter
        // hides chain-interleaved comments and formatting is not idempotent.
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "struct S {\n  build() {\n    Column() {}.width(100)\n  }\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty(), "Errors: {:?}", ret.diagnostics);
        let Statement::StructStatement(s) = &ret.program.body[0] else { panic!("struct") };
        let StructElement::MethodDefinition(m) = &s.body.body[0] else { panic!("method") };
        let body = m.value.body.as_ref().expect("body");
        let Statement::ExpressionStatement(stmt) = &body.statements[0] else { panic!("expr stmt") };
        let Expression::ArkUIComponentExpression(c) = &stmt.expression else {
            panic!("arkui expr")
        };
        assert!(
            c.span.source_text(source).ends_with(".width(100)"),
            "span must cover the chain, got: {:?}",
            c.span.source_text(source)
        );
    }

    #[test]
    fn arkui_export_function_with_decorator() {
        // Test that ArkUI allows decorators on exported functions (e.g., @Builder)
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "@Builder\nexport function titleContent() {\n  TitleContent();\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(
            ret.diagnostics.is_empty(),
            "Should parse exported function with decorator in ArkUI mode. Errors: {:?}",
            ret.diagnostics
        );
        assert_eq!(ret.program.body.len(), 1);
        // Verify it's an export statement
        let stmt = &ret.program.body[0];
        assert!(stmt.is_module_declaration(), "Expected ModuleDeclaration, got: {:?}", stmt);
    }

    #[test]
    fn arkui_extend_function() {
        // Test @Extend decorator on function declarations
        // In ArkUI, dot expressions chain together across lines
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "@Extend(Text)\nfunction memoTextExpand() {\n  .textOverflow({ overflow: TextOverflow.Ellipsis })\n  .maxLines(Constants.MAX_TEXT_LINES)\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(
            ret.diagnostics.is_empty(),
            "Should parse @Extend function in ArkUI mode. Errors: {:?}",
            ret.diagnostics
        );
        assert_eq!(ret.program.body.len(), 1);
        if let Statement::FunctionDeclaration(func) = &ret.program.body[0] {
            assert_eq!(func.id.as_ref().unwrap().name.as_str(), "memoTextExpand");
            assert!(!func.decorators.is_empty(), "Function should have decorators");
            // Verify function body has statements
            if let Some(body) = &func.body {
                assert!(!body.statements.is_empty(), "Function body should have statements");
                // The dot expressions chain together as a single expression statement
                assert_eq!(body.statements.len(), 1, "Should have 1 chained expression statement");
            }
        } else {
            panic!("Expected FunctionDeclaration");
        }
    }

    #[test]
    fn arkui_extend_function_single_chain() {
        // Test @Extend with single method chain
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "@Extend(Text)\nfunction test() {\n  .fontSize(16)\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(
            ret.diagnostics.is_empty(),
            "Should parse @Extend function with single chain. Errors: {:?}",
            ret.diagnostics
        );
    }

    #[test]
    fn arkui_extend_function_property_access() {
        // Test @Extend with property access (not just method calls)
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "@Extend(Text)\nfunction test() {\n  .fontSize\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(
            ret.diagnostics.is_empty(),
            "Should parse @Extend function with property access. Errors: {:?}",
            ret.diagnostics
        );
    }

    #[test]
    fn arkui_extend_function_multiline_chain() {
        // Test @Extend with multiple dot expressions on separate lines
        // The dot expressions chain together as a single expression statement
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source = "@Extend(Text)\nfunction superFancyText(size: number) {\n  .fontSize(size)\n  .fancy()\n}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(
            ret.diagnostics.is_empty(),
            "Should parse @Extend function with multiline chain. Errors: {:?}",
            ret.diagnostics
        );
        assert_eq!(ret.program.body.len(), 1);
        if let Statement::FunctionDeclaration(func) = &ret.program.body[0] {
            assert_eq!(func.id.as_ref().unwrap().name.as_str(), "superFancyText");
            if let Some(body) = &func.body {
                // Should have 1 chained expression statement
                assert_eq!(body.statements.len(), 1, "Should have 1 chained statement");
                // The statement should be a LeadingDotExpression with chained calls in expression field
                if let Statement::ExpressionStatement(expr_stmt) = &body.statements[0] {
                    assert!(matches!(expr_stmt.expression, Expression::LeadingDotExpression(_)));
                    if let Expression::LeadingDotExpression(leading_dot) = &expr_stmt.expression {
                        // expression field is now required (not Option)
                        assert!(
                            matches!(leading_dot.expression, Expression::CallExpression(_)),
                            "LeadingDotExpression.expression should contain CallExpression"
                        );
                    }
                } else {
                    panic!("Statement should be ExpressionStatement");
                }
            }
        } else {
            panic!("Expected FunctionDeclaration");
        }
    }

    #[test]
    fn arkui_object_literal_with_type_assertion() {
        // Test object literal with type assertion (as Type)
        let allocator = Allocator::default();
        let source_type = SourceType::ets();
        let source =
            "let obj = {\n  toneMapping: {\n    type: 1,\n    exposure: 1.0,\n  } as Type\n};";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(
            ret.diagnostics.is_empty(),
            "Should parse object literal with type assertion. Errors: {:?}",
            ret.diagnostics
        );
    }
}
