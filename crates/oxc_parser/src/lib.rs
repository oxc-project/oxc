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
//! ```rust
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
//! * AST is allocated in a memory arena ([bumpalo](https://docs.rs/bumpalo)) for fast AST drop
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
//! ```rust
#![doc = include_str!("../examples/parser.rs")]
//! ```
//!
//! ### Parsing TSX
//! ```rust
#![doc = include_str!("../examples/parser_tsx.rs")]
//! ```
//!
//! # Visitor
//!
//! See [oxc_ast::Visit] and [oxc_ast::VisitMut]
//!
//! # Visiting without a visitor
//!
//! For ad-hoc tasks, the semantic analyzer can be used to get a parent pointing tree with untyped nodes,
//! the nodes can be iterated through a sequential loop.
//!
//! ```rust
//! for node in semantic.nodes().iter() {
//!     match node.kind() {
//!         // check node
//!     }
//! }
//! ```
//!
//! See [full linter example](https://github.com/Boshen/oxc/blob/ab2ef4f89ba3ca50c68abb2ca43e36b7793f3673/crates/oxc_linter/examples/linter.rs#L38-L39)

#![warn(missing_docs)]

mod context;
mod cursor;
mod modifiers;
mod state;

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

use context::{Context, StatementContext};
use oxc_allocator::{Allocator, Box as ArenaBox};
use oxc_ast::{
    ast::{Expression, Program},
    AstBuilder,
};
use oxc_diagnostics::{OxcDiagnostic, Result};
use oxc_span::{ModuleKind, SourceType, Span};

use crate::{
    lexer::{Kind, Lexer, Token},
    state::ParserState,
};

/// Maximum length of source which can be parsed (in bytes).
/// ~4 GiB on 64-bit systems, ~2 GiB on 32-bit systems.
// Length is constrained by 2 factors:
// 1. `Span`'s `start` and `end` are `u32`s, which limits length to `u32::MAX` bytes.
// 2. Rust's allocator APIs limit allocations to `isize::MAX`.
// https://doc.rust-lang.org/std/alloc/struct.Layout.html#method.from_size_align
pub(crate) const MAX_LEN: usize = if std::mem::size_of::<usize>() >= 8 {
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
/// 1. Check that [`errors`] is empty
/// 2. Run semantic analysis with [syntax error checking
///    enabled](https://docs.rs/oxc_semantic/latest/oxc_semantic/struct.SemanticBuilder.html#method.with_check_syntax_error)
///
/// ## Errors
/// Oxc's [`Parser`] is able to recover from some syntax errors and continue parsing. When this
/// happens,
/// 1. [`errors`] will be non-empty
/// 2. [`program`] will contain a full AST
/// 3. [`panicked`] will be false
///
/// When the parser cannot recover, it will abort and terminate parsing early. [`program`] will
/// be empty and [`panicked`] will be `true`.
///
/// [`program`]: ParserReturn::program
/// [`errors`]: ParserReturn::errors
/// [`panicked`]: ParserReturn::panicked
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
    /// To ensure a valid AST, check that [`errors`](ParserReturn::errors) is empty. Then, run
    /// semantic analysis with syntax error checking enabled.
    pub program: Program<'a>,

    /// Syntax errors encountered while parsing.
    ///
    /// This list is not comprehensive. Oxc offloads more-expensive checks to [semantic
    /// analysis](https://docs.rs/oxc_semantic), which can be enabled using
    /// [`SemanticBuilder::with_check_syntax_error`](https://docs.rs/oxc_semantic/latest/oxc_semantic/struct.SemanticBuilder.html#method.with_check_syntax_error).
    pub errors: Vec<OxcDiagnostic>,

    /// Irregular whitespaces for `Oxlint`
    pub irregular_whitespaces: Box<[Span]>,

    /// Whether the parser panicked and terminated early.
    ///
    /// This will be `false` if parsing was successful, or if parsing was able to recover from a
    /// syntax error. When `true`, [`program`] will be empty and [`errors`] will contain at least
    /// one error.
    ///
    /// [`program`]: ParserReturn::program
    /// [`errors`]: ParserReturn::errors
    pub panicked: bool,
}

/// Parse options
///
/// You may provide options to the [`Parser`] using [`Parser::with_options`].
#[derive(Debug, Clone, Copy)]
pub struct ParseOptions {
    /// Whether to parse regular expressions or not.
    ///
    /// Default: `false`
    pub parse_regular_expression: bool,

    /// Allow [`return`] statements outside of functions.
    ///
    /// By default, a return statement at the top level raises an error (`false`).
    /// Set this to `true` to accept such code.
    ///
    /// [`return`]: oxc_ast::ast::ReturnStatement
    pub allow_return_outside_function: bool,

    /// Emit [`ParenthesizedExpression`]s in AST.
    ///
    /// If this option is `true`, parenthesized expressions are represented by
    /// (non-standard) [`ParenthesizedExpression`] nodes that have a single `expression` property
    /// containing the expression inside parentheses.
    ///
    /// Default: `true`
    ///
    /// [`ParenthesizedExpression`]: oxc_ast::ast::ParenthesizedExpression
    pub preserve_parens: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            parse_regular_expression: false,
            allow_return_outside_function: false,
            preserve_parens: true,
        }
    }
}

/// Recursive Descent Parser for ECMAScript and TypeScript
///
/// See [`Parser::parse`] for entry function.
pub struct Parser<'a> {
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
    options: ParseOptions,
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
        Self { allocator, source_text, source_type, options }
    }

    /// Set parse options
    #[must_use]
    pub fn with_options(mut self, options: ParseOptions) -> Self {
        self.options = options;
        self
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
    /// `UniquePromise::new_for_benchmarks` is a backdoor for benchmarks, so they can create a
    /// `ParserImpl` or `Lexer`, and manipulate it directly, for testing/benchmarking purposes.
    pub(crate) struct UniquePromise(());

    impl UniquePromise {
        #[inline]
        fn new() -> Self {
            Self(())
        }

        /// Backdoor for tests/benchmarks to create a `UniquePromise` (see above).
        /// This function must NOT be exposed outside of tests and benchmarks,
        /// as it allows circumventing safety invariants of the parser.
        #[cfg(feature = "benchmarking")]
        pub fn new_for_benchmarks() -> Self {
            Self(())
        }
    }

    impl<'a> Parser<'a> {
        /// Main entry point
        ///
        /// Returns an empty `Program` on unrecoverable error,
        /// Recoverable errors are stored inside `errors`.
        ///
        /// See the [module-level documentation](crate) for examples and more information.
        pub fn parse(self) -> ParserReturn<'a> {
            let unique = UniquePromise::new();
            let parser = ParserImpl::new(
                self.allocator,
                self.source_text,
                self.source_type,
                self.options,
                unique,
            );
            parser.parse()
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
        pub fn parse_expression(self) -> std::result::Result<Expression<'a>, Vec<OxcDiagnostic>> {
            let unique = UniquePromise::new();
            let parser = ParserImpl::new(
                self.allocator,
                self.source_text,
                self.source_type,
                self.options,
                unique,
            );
            parser.parse_expression()
        }
    }
}
use parser_parse::UniquePromise;

/// Implementation of parser.
/// `Parser` is just a public wrapper, the guts of the implementation is in this type.
struct ParserImpl<'a> {
    options: ParseOptions,

    lexer: Lexer<'a>,

    /// SourceType: JavaScript or TypeScript, Script or Module, jsx support?
    source_type: SourceType,

    /// Source Code
    source_text: &'a str,

    /// All syntax errors from parser and lexer
    /// Note: favor adding to `Diagnostics` instead of raising Err
    errors: Vec<OxcDiagnostic>,

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

    /// Precomputed typescript detection
    is_ts: bool,
}

impl<'a> ParserImpl<'a> {
    /// Create a new `ParserImpl`.
    ///
    /// Requiring a `UniquePromise` to be provided guarantees only 1 `ParserImpl` can exist
    /// on a single thread at one time.
    #[inline]
    pub fn new(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        options: ParseOptions,
        unique: UniquePromise,
    ) -> Self {
        Self {
            options,
            lexer: Lexer::new(allocator, source_text, source_type, unique),
            source_type,
            source_text,
            errors: vec![],
            token: Token::default(),
            prev_token_end: 0,
            state: ParserState::default(),
            ctx: Self::default_context(source_type, options),
            ast: AstBuilder::new(allocator),
            is_ts: source_type.is_typescript(),
        }
    }

    /// Main entry point
    ///
    /// Returns an empty `Program` on unrecoverable error,
    /// Recoverable errors are stored inside `errors`.
    #[inline]
    pub fn parse(mut self) -> ParserReturn<'a> {
        let (program, panicked) = match self.parse_program() {
            Ok(program) => (program, false),
            Err(error) => {
                self.error(self.overlong_error().unwrap_or(error));
                let program = self.ast.program(
                    Span::default(),
                    self.source_type,
                    self.source_text,
                    self.ast.vec(),
                    None,
                    self.ast.vec(),
                    self.ast.vec(),
                );
                (program, true)
            }
        };
        let mut errors = vec![];
        // only check for `@flow` if the file failed to parse.
        if !self.lexer.errors.is_empty() || !self.errors.is_empty() {
            if let Some(error) = self.flow_error() {
                errors.push(error);
            }
        }
        if errors.len() != 1 {
            errors.reserve(self.lexer.errors.len() + self.errors.len());
            errors.extend(self.lexer.errors);
            errors.extend(self.errors);
        }
        let irregular_whitespaces =
            self.lexer.trivia_builder.irregular_whitespaces.into_boxed_slice();
        ParserReturn { program, errors, irregular_whitespaces, panicked }
    }

    pub fn parse_expression(mut self) -> std::result::Result<Expression<'a>, Vec<OxcDiagnostic>> {
        // initialize cur_token and prev_token by moving onto the first token
        self.bump_any();
        let expr = self.parse_expr().map_err(|diagnostic| vec![diagnostic])?;
        let errors = self.lexer.errors.into_iter().chain(self.errors).collect::<Vec<_>>();
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(expr)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn parse_program(&mut self) -> Result<Program<'a>> {
        // initialize cur_token and prev_token by moving onto the first token
        self.bump_any();

        let hashbang = self.parse_hashbang();
        let (directives, statements) =
            self.parse_directives_and_statements(/* is_top_level */ true)?;

        self.set_source_type_to_script_if_unambiguous();

        let span = Span::new(0, self.source_text.len() as u32);
        let comments = self.ast.vec_from_iter(self.lexer.trivia_builder.comments.iter().copied());
        Ok(self.ast.program(
            span,
            self.source_type,
            self.source_text,
            comments,
            hashbang,
            directives,
            statements,
        ))
    }

    fn default_context(source_type: SourceType, options: ParseOptions) -> Context {
        let mut ctx = Context::default().and_ambient(source_type.is_typescript_definition());
        if source_type.module_kind() == ModuleKind::Module {
            // for [top-level-await](https://tc39.es/proposal-top-level-await/)
            ctx = ctx.and_await(true);
        }
        if options.allow_return_outside_function {
            ctx = ctx.and_return(true);
        }
        ctx
    }

    /// Check for Flow declaration if the file cannot be parsed.
    /// The declaration must be [on the first line before any code](https://flow.org/en/docs/usage/#toc-prepare-your-code-for-flow)
    fn flow_error(&mut self) -> Option<OxcDiagnostic> {
        if !self.source_type.is_javascript() {
            return None;
        };
        let span = self.lexer.trivia_builder.comments.first()?.span;
        if span.source_text(self.source_text).contains("@flow") {
            self.errors.clear();
            Some(diagnostics::flow(span))
        } else {
            None
        }
    }

    /// Check if source length exceeds MAX_LEN, if the file cannot be parsed.
    /// Original parsing error is not real - `Lexer::new` substituted "\0" as the source text.
    fn overlong_error(&self) -> Option<OxcDiagnostic> {
        if self.source_text.len() > MAX_LEN {
            return Some(diagnostics::overlong_source());
        }
        None
    }

    /// Return error info at current token
    /// # Panics
    ///   * The lexer did not push a diagnostic when `Kind::Undetermined` is returned
    fn unexpected(&mut self) -> OxcDiagnostic {
        // The lexer should have reported a more meaningful diagnostic
        // when it is a undetermined kind.
        if self.cur_kind() == Kind::Undetermined {
            if let Some(error) = self.lexer.errors.pop() {
                return error;
            }
        }
        diagnostics::unexpected_token(self.cur_token().span())
    }

    /// Push a Syntax Error
    fn error(&mut self, error: OxcDiagnostic) {
        self.errors.push(error);
    }

    fn errors_count(&self) -> usize {
        self.errors.len() + self.lexer.errors.len()
    }

    fn set_source_type_to_module_if_unambiguous(&mut self) {
        if self.source_type.is_unambiguous() {
            self.source_type = self.source_type.with_module(true);
        }
    }

    fn set_source_type_to_script_if_unambiguous(&mut self) {
        if self.source_type.is_unambiguous() {
            self.source_type = self.source_type.with_script(true);
        }
    }

    #[inline]
    fn alloc<T>(&self, value: T) -> ArenaBox<'a, T> {
        self.ast.alloc(value)
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use oxc_ast::ast::{CommentKind, Expression};

    use super::*;

    #[test]
    fn parse_program_smoke_test() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let source = "";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.program.is_empty());
        assert!(ret.errors.is_empty());
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
            assert_eq!(ret.errors.len(), 1);
            assert_eq!(ret.errors.first().unwrap().to_string(), "Flow is not supported");
        }
    }

    #[test]
    fn ts_module_declaration() {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(Path::new("module.ts")).unwrap();
        let source = "declare module 'test'\n";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert_eq!(ret.errors.len(), 0);
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
    fn comments() {
        let allocator = Allocator::default();
        let source_type = SourceType::default().with_typescript(true);
        let sources = [
            ("// line comment", CommentKind::Line),
            ("/* line comment */", CommentKind::Block),
            (
                "type Foo = ( /* Require properties which are not generated automatically. */ 'bar')",
                CommentKind::Block,
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
    #[cfg(target_pointer_width = "64")]
    #[test]
    fn overlong_source() {
        // Build string in 16 KiB chunks for speed
        let mut source = String::with_capacity(MAX_LEN + 1);
        let line = "var x = 123456;\n";
        let chunk = line.repeat(1024);
        while source.len() < MAX_LEN + 1 - chunk.len() {
            source.push_str(&chunk);
        }
        while source.len() < MAX_LEN + 1 - line.len() {
            source.push_str(line);
        }
        while source.len() < MAX_LEN + 1 {
            source.push('\n');
        }
        assert_eq!(source.len(), MAX_LEN + 1);

        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, &source, SourceType::default()).parse();
        assert!(ret.program.is_empty());
        assert!(ret.panicked);
        assert_eq!(ret.errors.len(), 1);
        assert_eq!(ret.errors.first().unwrap().to_string(), "Source length exceeds 4 GiB limit");
    }

    // Source with length MAX_LEN parses OK.
    // This test takes over 1 minute on an M1 Macbook Pro unless compiled in release mode.
    // `not(debug_assertions)` is a proxy for detecting release mode.
    #[cfg(not(debug_assertions))]
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
        assert!(ret.errors.is_empty());
        assert_eq!(ret.program.body.len(), 2);
    }
}
