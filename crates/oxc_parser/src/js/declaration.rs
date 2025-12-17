use oxc_allocator::Box;
use oxc_ast::{NONE, ast::*};
use oxc_span::GetSpan;

use super::VariableDeclarationParent;
use crate::{ParserImpl, StatementContext, diagnostics, lexer::Kind};

impl<'a> ParserImpl<'a> {
    pub(crate) fn parse_let(&mut self, stmt_ctx: StatementContext) -> Statement<'a> {
        let span = self.start_span();

        let checkpoint = self.checkpoint();
        self.bump_any(); // bump `let`
        let token = self.cur_token();
        let peeked = token.kind();

        // Fast path: avoid rewind.
        if !stmt_ctx.is_single_statement() && peeked.is_after_let() {
            return self.parse_variable_statement(span, VariableDeclarationKind::Let, stmt_ctx);
        }

        self.rewind(checkpoint);
        // let = foo, let instanceof x, let + 1
        if peeked.is_assignment_operator() || peeked.is_binary_operator() {
            let expr = self.parse_assignment_expression_or_higher();
            self.parse_expression_statement(span, expr)
        // let.a = 1, let?.a = 1, let()[a] = 1
        } else if matches!(peeked, Kind::Dot | Kind::QuestionDot | Kind::LParen) {
            let expr = self.parse_expr();
            self.parse_expression_statement(span, expr)
        // single statement let declaration: while (0) let
        } else if (stmt_ctx.is_single_statement() && peeked != Kind::LBrack)
            || peeked == Kind::Semicolon
        {
            let expr = self.parse_identifier_expression();
            self.parse_expression_statement(span, expr)
        } else {
            self.bump_any();
            self.parse_variable_statement(span, VariableDeclarationKind::Let, stmt_ctx)
        }
    }

    pub(crate) fn is_using_statement(&mut self) -> bool {
        self.lookahead(Self::is_next_token_using_keyword_then_binding_identifier)
    }

    fn is_next_token_using_keyword_then_binding_identifier(&mut self) -> bool {
        self.bump_any();
        if !self.cur_token().is_on_new_line() && self.eat(Kind::Using) {
            self.cur_kind().is_binding_identifier() && !self.cur_token().is_on_new_line()
        } else {
            false
        }
    }

    pub(crate) fn parse_using_statement(&mut self, stmt_ctx: StatementContext) -> Statement<'a> {
        let mut decl = self.parse_using_declaration(stmt_ctx);
        self.asi();
        decl.span = self.end_span(decl.span.start);
        debug_assert!(decl.kind.is_lexical());
        if stmt_ctx.is_single_statement() {
            self.error(diagnostics::lexical_declaration_single_statement(decl.span));
        }
        Statement::VariableDeclaration(self.alloc(decl))
    }

    pub(crate) fn get_variable_declaration_kind(&self) -> VariableDeclarationKind {
        match self.cur_kind() {
            Kind::Var => VariableDeclarationKind::Var,
            Kind::Const => VariableDeclarationKind::Const,
            Kind::Let => VariableDeclarationKind::Let,
            _ => unreachable!(),
        }
    }

    pub(crate) fn parse_variable_declaration(
        &mut self,
        start_span: u32,
        kind: VariableDeclarationKind,
        decl_parent: VariableDeclarationParent,
        declare: bool,
    ) -> Box<'a, VariableDeclaration<'a>> {
        let mut declarations = self.ast.vec();
        loop {
            let declaration = self.parse_variable_declarator(decl_parent, kind);
            declarations.push(declaration);
            if !self.eat(Kind::Comma) {
                break;
            }
        }

        if matches!(decl_parent, VariableDeclarationParent::Statement) {
            self.asi();
        }
        self.ast.alloc_variable_declaration(self.end_span(start_span), kind, declarations, declare)
    }

    fn parse_variable_declarator(
        &mut self,
        decl_parent: VariableDeclarationParent,
        kind: VariableDeclarationKind,
    ) -> VariableDeclarator<'a> {
        let span = self.start_span();

        let mut binding_kind = self.parse_binding_pattern_kind();

        let (id, definite) = if self.is_ts {
            // const x!: number = 1
            //        ^ definite
            let definite = if binding_kind.is_binding_identifier()
                && !self.cur_token().is_on_new_line()
                && self.at(Kind::Bang)
            {
                let span = self.cur_token().span();
                self.bump_any();
                Some(span)
            } else {
                None
            };
            let optional = if self.at(Kind::Question) {
                self.error(diagnostics::unexpected_optional_declaration(self.cur_token().span()));
                self.bump_any();
                true
            } else {
                false
            };
            let type_annotation = self.parse_ts_type_annotation();
            if let Some(type_annotation) = &type_annotation {
                Self::extend_binding_pattern_span_end(type_annotation.span.end, &mut binding_kind);
            }
            (self.ast.binding_pattern(binding_kind, type_annotation, optional), definite)
        } else {
            (self.ast.binding_pattern(binding_kind, NONE, false), None)
        };
        let init = self.eat(Kind::Eq).then(|| self.parse_assignment_expression_or_higher());
        let decl =
            self.ast.variable_declarator(self.end_span(span), kind, id, init, definite.is_some());
        if decl_parent == VariableDeclarationParent::Statement {
            self.check_missing_initializer(&decl);
        }
        if let Some(span) = definite {
            if decl.init.is_some() {
                self.error(diagnostics::variable_declarator_definite(span));
            } else if decl.id.type_annotation.is_none() {
                self.error(diagnostics::variable_declarator_definite_type_assertion(span));
            }
        }
        decl
    }

    pub(crate) fn check_missing_initializer(&mut self, decl: &VariableDeclarator<'a>) {
        if decl.init.is_none() && !self.ctx.has_ambient() {
            if !matches!(decl.id.kind, BindingPatternKind::BindingIdentifier(_)) {
                self.error(diagnostics::invalid_destructuring_declaration(decl.id.span()));
            } else if decl.kind == VariableDeclarationKind::Const {
                // It is a Syntax Error if Initializer is not present and IsConstantDeclaration of the LexicalDeclaration containing this LexicalBinding is true.
                self.error(diagnostics::missing_initializer_in_const(decl.id.span()));
            }
        }
    }

    /// Section 14.3.1 Let, Const, and Using Declarations
    /// UsingDeclaration[In, Yield, Await] :
    /// using [no LineTerminator here] [lookahead ≠ await] BindingList[?In, ?Yield, ?Await, ~Pattern] ;
    pub(crate) fn parse_using_declaration(
        &mut self,
        statement_ctx: StatementContext,
    ) -> VariableDeclaration<'a> {
        let span = self.start_span();

        let is_await = self.eat(Kind::Await);
        let kind = if is_await {
            VariableDeclarationKind::AwaitUsing
        } else {
            VariableDeclarationKind::Using
        };

        self.expect(Kind::Using);

        // BindingList[?In, ?Yield, ?Await, ~Pattern]
        let mut declarations = self.ast.vec();
        loop {
            let declaration =
                self.parse_variable_declarator(VariableDeclarationParent::Statement, kind);

            if !matches!(declaration.id.kind, BindingPatternKind::BindingIdentifier(_)) {
                self.error(diagnostics::invalid_identifier_in_using_declaration(
                    declaration.id.span(),
                ));
            }

            // Excluding `for` loops, an initializer is required in a UsingDeclaration.
            if declaration.init.is_none() && !matches!(statement_ctx, StatementContext::For) {
                self.error(diagnostics::using_declarations_must_be_initialized(
                    declaration.id.span(),
                ));
            }

            declarations.push(declaration);
            if !self.eat(Kind::Comma) {
                break;
            }
        }

        self.ast.variable_declaration(self.end_span(span), kind, declarations, false)
    }
}
