//! Recursive Descent Parser for ECMAScript and TypeScript

#![allow(clippy::wildcard_imports)] // allow for use `oxc_ast::ast::*`

mod cursor;
mod list;
mod state;

mod js;
mod jsx;
mod ts;

mod lexer;

use oxc_allocator::Allocator;
use oxc_ast::{ast::Program, context::Context, AstBuilder, Node, SourceType};
use oxc_diagnostics::{Diagnostic, Diagnostics, Result};

use crate::{
    lexer::{Kind, Lexer, Token},
    state::ParserState,
};

#[derive(Debug)]
pub struct ParserReturn<'a> {
    pub program: Program<'a>,
    pub errors: Vec<Diagnostic>,
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,

    /// SourceType: JavaScript or TypeScript, Script or Module, jsx support?
    source_type: SourceType,

    /// Source Code
    source: &'a str,

    /// All syntax errors from parser and lexer
    /// Note: favor adding to `Diagnostics` instead of raising Err
    errors: Diagnostics,

    /// The current parsing token
    token: Token,

    /// The end range of the previous token
    prev_token_end: usize,

    /// Parser state
    state: ParserState<'a>,

    /// Parsing context saved into every AST node
    ctx: Context,

    /// Ast builder for creating AST nodes
    ast: AstBuilder<'a>,
}

impl<'a> Parser<'a> {
    #[must_use]
    pub fn new(allocator: &'a Allocator, source: &'a str, source_type: SourceType) -> Self {
        let errors = Diagnostics::default();
        Self {
            lexer: Lexer::new(allocator, source, errors.clone(), source_type),
            source_type,
            source,
            errors,
            token: Token::default(),
            prev_token_end: 0,
            state: ParserState::default(),
            ctx: source_type.default_context(),
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
        let program = match self.parse_program() {
            Ok(program) => program,
            Err(error) => {
                self.error(self.flow_error().unwrap_or(error));
                let program = self.ast.program(
                    Node::default(),
                    self.ast.new_vec(),
                    self.ast.new_vec(),
                    self.source_type,
                );
                program
            }
        };
        ParserReturn { program, errors: self.errors.borrow().clone() }
    }

    fn parse_program(&mut self) -> Result<Program<'a>> {
        // initialize cur_token and prev_token by moving onto the first token
        self.bump_any();

        let (directives, statements) =
            self.parse_directives_and_statements(/* is_top_level */ true)?;

        let node = Node::new(0, self.source.len(), self.ctx);
        Ok(self.ast.program(node, directives, statements, self.source_type))
    }

    /// Check for Flow declaration if the file cannot be parsed.
    /// The declaration must be [on the first line before any code](https://flow.org/en/docs/usage/#toc-prepare-your-code-for-flow)
    fn flow_error(&self) -> Option<Diagnostic> {
        if self.source_type.is_javascript()
            && (self.source.starts_with("// @flow") || self.source.starts_with("/* @flow */"))
        {
            return Some(Diagnostic::Flow(0..8));
        }
        None
    }

    /// Return error info at current token
    /// # Panics
    ///   * The lexer did not push a diagnostic when `Kind::Undetermined` is returned
    fn unexpected<T>(&self) -> Result<T> {
        // The lexer should have reported a more meaningful diagnostic
        // when it is a undetermined kind.
        if self.cur_kind() == Kind::Undetermined {
            return Err(self.errors.borrow_mut().pop().unwrap());
        }
        Err(Diagnostic::UnexpectedToken(self.current_range()))
    }

    /// Push a Syntax Error
    fn error(&mut self, error: Diagnostic) {
        self.errors.borrow_mut().push(error);
    }

    #[must_use]
    const fn ts_enabled(&self) -> bool {
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
