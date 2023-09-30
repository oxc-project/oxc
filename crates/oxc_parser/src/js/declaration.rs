use oxc_allocator::Box;
use oxc_ast::ast::*;
use oxc_diagnostics::Result;
use oxc_span::{GetSpan, Span};

use crate::{diagnostics, lexer::Kind, Parser, StatementContext};

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum VariableDeclarationParent {
    For,
    Statement,
    Clause,
}

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub struct VariableDeclarationContext {
    pub parent: VariableDeclarationParent,
}

impl VariableDeclarationContext {
    pub(crate) fn new(parent: VariableDeclarationParent) -> Self {
        Self { parent }
    }
}

impl<'a> Parser<'a> {
    pub(crate) fn parse_let(&mut self, stmt_ctx: StatementContext) -> Result<Statement<'a>> {
        let span = self.start_span();
        let peeked = self.peek_kind();
        // let = foo, let instanceof x, let + 1
        if peeked.is_assignment_operator() || peeked.is_binary_operator() {
            let expr = self.parse_assignment_expression_base()?;
            self.parse_expression_statement(span, expr)
        // single statement let declaration: while (0) let
        } else if (stmt_ctx.is_single_statement() && peeked != Kind::LBrack)
            || peeked == Kind::Semicolon
        {
            let expr = self.parse_identifier_expression()?;
            self.parse_expression_statement(span, expr)
        } else {
            self.parse_variable_statement(stmt_ctx)
        }
    }

    pub(crate) fn parse_using(&mut self) -> Result<Statement<'a>> {
        let using_decl = self.parse_using_declaration(StatementContext::StatementList)?;

        self.expect(Kind::Semicolon)?;

        Ok(Statement::Declaration(Declaration::UsingDeclaration(self.ast.alloc(using_decl))))
    }

    pub(crate) fn parse_variable_declaration(
        &mut self,
        start_span: Span,
        decl_ctx: VariableDeclarationContext,
        modifiers: Modifiers<'a>,
    ) -> Result<Box<'a, VariableDeclaration<'a>>> {
        let kind = match self.cur_kind() {
            Kind::Var => VariableDeclarationKind::Var,
            Kind::Const => VariableDeclarationKind::Const,
            Kind::Let => VariableDeclarationKind::Let,
            _ => return Err(self.unexpected()),
        };
        self.bump_any();

        let mut declarations = self.ast.new_vec();
        loop {
            let declaration = self.parse_variable_declarator(decl_ctx, kind)?;
            declarations.push(declaration);
            if !self.eat(Kind::Comma) {
                break;
            }
        }

        if matches!(
            decl_ctx.parent,
            VariableDeclarationParent::Statement | VariableDeclarationParent::Clause
        ) {
            self.asi()?;
        }

        Ok(self.ast.variable_declaration(self.end_span(start_span), kind, declarations, modifiers))
    }

    fn parse_variable_declarator(
        &mut self,
        decl_ctx: VariableDeclarationContext,
        kind: VariableDeclarationKind,
    ) -> Result<VariableDeclarator<'a>> {
        let span = self.start_span();

        let (id, definite) = self.parse_binding()?;

        let init =
            self.eat(Kind::Eq).then(|| self.parse_assignment_expression_base()).transpose()?;

        if init.is_none() && decl_ctx.parent == VariableDeclarationParent::Statement {
            // LexicalBinding[In, Yield, Await] :
            //   BindingIdentifier[?Yield, ?Await] Initializer[?In, ?Yield, ?Await] opt
            //   BindingPattern[?Yield, ?Await] Initializer[?In, ?Yield, ?Await]
            // the grammar forbids `let []`, `let {}`
            if !matches!(id.kind, BindingPatternKind::BindingIdentifier(_)) {
                self.error(diagnostics::InvalidDestrucuringDeclaration(id.span()));
            } else if kind == VariableDeclarationKind::Const && !self.ctx.has_ambient() {
                // It is a Syntax Error if Initializer is not present and IsConstantDeclaration of the LexicalDeclaration containing this LexicalBinding is true.
                self.error(diagnostics::MissinginitializerInConst(id.span()));
            }
        }

        Ok(self.ast.variable_declarator(self.end_span(span), kind, id, init, definite))
    }

    /// Section 14.3.1 Let, Const, and Using Declarations
    /// UsingDeclaration[In, Yield, Await] :
    /// using [no LineTerminator here] [lookahead ≠ await] BindingList[?In, ?Yield, ?Await, ~Pattern] ;
    pub(crate) fn parse_using_declaration(
        &mut self,
        statement_ctx: StatementContext,
    ) -> Result<UsingDeclaration<'a>> {
        let span = self.start_span();

        let is_await = self.eat(Kind::Await);

        self.expect(Kind::Using)?;

        // `[no LineTerminator here]`
        if self.cur_token().is_on_new_line {
            self.error(diagnostics::LineTerminatorBeforeUsingDeclaration(self.cur_token().span()));
        }

        // [lookahead ≠ await]
        if self.cur_kind() == Kind::Await {
            self.error(diagnostics::AwaitInUsingDeclaration(self.cur_token().span()));
            self.eat(Kind::Await);
        }

        // BindingList[?In, ?Yield, ?Await, ~Pattern]
        let mut declarations: oxc_allocator::Vec<'_, VariableDeclarator<'_>> = self.ast.new_vec();
        loop {
            let declaration = self.parse_variable_declarator(
                VariableDeclarationContext::new(VariableDeclarationParent::Statement),
                VariableDeclarationKind::Var,
            )?;

            match declaration.id.kind {
                BindingPatternKind::BindingIdentifier(_) => {}
                _ => {
                    self.error(diagnostics::InvalidIdentifierInUsingDeclaration(
                        declaration.id.span(),
                    ));
                }
            }

            // Excluding `for` loops, an initializer is required in a UsingDeclaration.
            if declaration.init.is_none() && !matches!(statement_ctx, StatementContext::For) {
                self.error(diagnostics::UsingDeclarationsMustBeInitialized(declaration.id.span()));
            }

            declarations.push(declaration);
            if !self.eat(Kind::Comma) {
                break;
            }
        }

        Ok(self.ast.using_declaration(self.end_span(span), declarations, is_await))
    }
}
