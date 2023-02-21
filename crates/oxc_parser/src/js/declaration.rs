use oxc_allocator::Box;
use oxc_ast::{ast::*, context::StatementContext, GetNode};
use oxc_diagnostics::{Diagnostic, Result};

use crate::lexer::Kind;
use crate::Parser;

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
    pub const fn new(parent: VariableDeclarationParent) -> Self {
        Self { parent }
    }
}

impl<'a> Parser<'a> {
    pub fn parse_let(&mut self, stmt_ctx: StatementContext) -> Result<Statement<'a>> {
        let node = self.start_node();
        let peeked = self.peek_kind();
        // let = foo, let instanceof x, let + 1
        if peeked.is_assignment_operator() || peeked.is_binary_operator() {
            let expr = self.parse_assignment_expression_base()?;
            self.parse_expression_statement(node, expr)
        // single statement let declaration: while (0) let
        } else if (stmt_ctx.is_single_statement() && peeked != Kind::LBrack)
            || peeked == Kind::Semicolon
        {
            let expr = self.parse_identifier_expression()?;
            self.parse_expression_statement(node, expr)
        } else {
            self.parse_variable_statement(stmt_ctx)
        }
    }

    pub fn parse_variable_declaration(
        &mut self,
        decl_ctx: VariableDeclarationContext,
    ) -> Result<Box<'a, VariableDeclaration<'a>>> {
        let node = self.start_node();
        let kind = match self.cur_kind() {
            Kind::Var => VariableDeclarationKind::Var,
            Kind::Const => VariableDeclarationKind::Const,
            Kind::Let => VariableDeclarationKind::Let,
            _ => return self.unexpected(),
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

        Ok(self.ast.variable_declaration(self.end_node(node), kind, declarations))
    }

    fn parse_variable_declarator(
        &mut self,
        decl_ctx: VariableDeclarationContext,
        kind: VariableDeclarationKind,
    ) -> Result<VariableDeclarator<'a>> {
        let node = self.start_node();

        let (id, definite) = self.parse_binding_pattern()?;

        let init =
            self.eat(Kind::Eq).then(|| self.parse_assignment_expression_base()).transpose()?;

        if init.is_none() && decl_ctx.parent == VariableDeclarationParent::Statement {
            // LexicalBinding[In, Yield, Await] :
            //   BindingIdentifier[?Yield, ?Await] Initializer[?In, ?Yield, ?Await] opt
            //   BindingPattern[?Yield, ?Await] Initializer[?In, ?Yield, ?Await]
            // the grammar forbids `let []`, `let {}`
            if !matches!(id.kind, BindingPatternKind::BindingIdentifier(_)) {
                self.error(Diagnostic::InvalidDestrucuringDeclaration(id.node()));
            } else if kind == VariableDeclarationKind::Const && !self.ctx.has_ambient() {
                // It is a Syntax Error if Initializer is not present and IsConstantDeclaration of the LexicalDeclaration containing this LexicalBinding is true.
                self.error(Diagnostic::MissinginitializerInConst(id.node()));
            }
        }

        Ok(self.ast.variable_declarator(self.end_node(node), kind, id, init, definite))
    }
}
