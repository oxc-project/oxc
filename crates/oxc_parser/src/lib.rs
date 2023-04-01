//! Recursive Descent Parser for ECMAScript and TypeScript

#![allow(clippy::wildcard_imports)] // allow for use `oxc_ast::ast::*`
#![feature(portable_simd)]
#![feature(slice_as_chunks)]

mod context;
mod cursor;
mod list;
mod state;

mod js;
mod jsx;
mod ts;

mod diagnostics;
mod lexer;

use std::rc::Rc;

use context::{Context, StatementContext};
use oxc_allocator::Allocator;
use oxc_ast::{ast::Program, AstBuilder, ModuleKind, SourceType, Span, Trivias};
use oxc_diagnostics::{Error, Result};

use crate::{
    lexer::{Kind, Lexer, Token},
    state::ParserState,
};

/// The parser always return a valid AST.
/// When `panicked = true`, then program will always be empty.
/// When `errors.len() > 0`, then program may or may not be empty due to error recovery.
#[derive(Debug)]
pub struct ParserReturn<'a> {
    pub program: Program<'a>,
    pub errors: Vec<Error>,
    pub trivias: Rc<Trivias>,
    pub panicked: bool,
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,

    /// SourceType: JavaScript or TypeScript, Script or Module, jsx support?
    source_type: SourceType,

    /// Source Code
    source: &'a str,

    /// All syntax errors from parser and lexer
    /// Note: favor adding to `Diagnostics` instead of raising Err
    errors: Vec<Error>,

    /// The current parsing token
    token: Token<'a>,

    /// The end range of the previous token
    prev_token_end: u32,

    /// Parser state
    state: ParserState<'a>,

    /// Parsing context
    ctx: Context,

    /// Ast builder for creating AST spans
    ast: AstBuilder<'a>,
}

impl<'a> Parser<'a> {
    #[must_use]
    pub fn new(allocator: &'a Allocator, source: &'a str, source_type: SourceType) -> Self {
        Self {
            lexer: Lexer::new(allocator, source, source_type),
            source_type,
            source,
            errors: vec![],
            token: Token::default(),
            prev_token_end: 0,
            state: ParserState::new(allocator),
            ctx: Self::default_context(source_type),
            ast: AstBuilder::new(allocator),
        }
    }

    #[must_use]
    pub fn allow_return_outside_function(mut self, allow: bool) -> Self {
        self.ctx = self.ctx.and_return(allow);
        self
    }

    /// Parser main entry point
    /// Returns an empty `Program` on unrecoverable error,
    /// Recoverable errors are stored inside `errors`.
    #[must_use]
    pub fn parse(mut self) -> ParserReturn<'a> {
        let (program, panicked) = match self.parse_program() {
            Ok(program) => (program, false),
            Err(error) => {
                self.error(self.flow_error().unwrap_or(error));
                let program = self.ast.program(
                    Span::default(),
                    self.ast.new_vec(),
                    self.ast.new_vec(),
                    self.source_type,
                );
                (program, true)
            }
        };
        let errors = self.lexer.errors.into_iter().chain(self.errors).collect();
        let trivias = self.lexer.trivia_builder.build();
        ParserReturn { program, errors, trivias, panicked }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn parse_program(&mut self) -> Result<Program<'a>> {
        // initialize cur_token and prev_token by moving onto the first token
        self.bump_any();

        let (directives, statements) =
            self.parse_directives_and_statements(/* is_top_level */ true)?;

        let span = Span::new(0, self.source.len() as u32);
        Ok(self.ast.program(span, directives, statements, self.source_type))
    }

    #[must_use]
    pub fn default_context(source_type: SourceType) -> Context {
        let ctx = Context::default().and_ambient(source_type.is_typescript_definition());
        match source_type.module_kind() {
            ModuleKind::Script => ctx,
            // for [top-level-await](https://tc39.es/proposal-top-level-await/)
            ModuleKind::Module => ctx.and_await(true),
        }
    }

    /// Check for Flow declaration if the file cannot be parsed.
    /// The declaration must be [on the first line before any code](https://flow.org/en/docs/usage/#toc-prepare-your-code-for-flow)
    fn flow_error(&self) -> Option<Error> {
        if self.source_type.is_javascript()
            && (self.source.starts_with("// @flow") || self.source.starts_with("/* @flow */"))
        {
            return Some(diagnostics::Flow(Span::new(0, 8)).into());
        }
        None
    }

    /// Return error info at current token
    /// # Panics
    ///   * The lexer did not push a diagnostic when `Kind::Undetermined` is returned
    fn unexpected(&mut self) -> Error {
        // The lexer should have reported a more meaningful diagnostic
        // when it is a undetermined kind.
        if self.cur_kind() == Kind::Undetermined {
            return self.lexer.errors.pop().unwrap();
        }
        diagnostics::UnexpectedToken(self.cur_token().span()).into()
    }

    /// Push a Syntax Error
    fn error<T: Into<Error>>(&mut self, error: T) {
        self.errors.push(error.into());
    }

    #[must_use]
    fn ts_enabled(&self) -> bool {
        self.source_type.is_typescript()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn smoke_test() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let source = "";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.program.is_empty());
        assert!(ret.errors.is_empty());
    }

    #[test]
    fn flow_error() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let source = "// @flow\nasdf adsf";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.program.is_empty());
        assert_eq!(ret.errors.first().unwrap().to_string(), "Flow is not supported");

        let source = "/* @flow */\n asdf asdf";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.program.is_empty());
        assert_eq!(ret.errors.first().unwrap().to_string(), "Flow is not supported");
    }
}
