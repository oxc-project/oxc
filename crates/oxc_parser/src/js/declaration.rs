use oxc_allocator::Box;
use oxc_ast::{NONE, ast::*};
use oxc_span::GetSpan;

use super::VariableDeclarationParent;
use crate::{
    ParserImpl, StatementContext, diagnostics,
    lexer::Kind,
    modifiers::{ModifierFlags, Modifiers},
};

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
        // let.a = 1, let()[a] = 1
        } else if matches!(peeked, Kind::Dot | Kind::LParen) {
            let expr = self.parse_expr();
            self.ast.statement_expression(self.end_span(span), expr)
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

    pub(crate) fn parse_using_statement(&mut self) -> Statement<'a> {
        let mut decl = self.parse_using_declaration(StatementContext::StatementList);
        self.asi();
        decl.span = self.end_span(decl.span.start);
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
        modifiers: &Modifiers<'a>,
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

        self.verify_modifiers(
            modifiers,
            ModifierFlags::DECLARE,
            diagnostics::modifier_cannot_be_used_here,
        );

        self.ast.alloc_variable_declaration(
            self.end_span(start_span),
            kind,
            declarations,
            modifiers.contains_declare(),
        )
    }

    fn parse_variable_declarator(
        &mut self,
        decl_parent: VariableDeclarationParent,
        kind: VariableDeclarationKind,
    ) -> VariableDeclarator<'a> {
        let span = self.start_span();
        let mut binding_kind = self.parse_binding_pattern_kind();

        let (id, definite) = if self.is_ts {
            let definite = self.parse_definite_assignment_assertion(&binding_kind);
            let optional = self.parse_optional_declarator_modifier();
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
        
        self.validate_variable_declarator(decl_parent, &decl, definite);
        decl
    }

    fn parse_definite_assignment_assertion(
        &mut self,
        binding_kind: &BindingPatternKind<'a>,
    ) -> Option<oxc_span::Span> {
        if binding_kind.is_binding_identifier()
            && !self.cur_token().is_on_new_line()
            && self.at(Kind::Bang)
        {
            let span = self.cur_token().span();
            self.bump_any();
            Some(span)
        } else {
            None
        }
    }

    fn parse_optional_declarator_modifier(&mut self) -> bool {
        if self.at(Kind::Question) {
            self.error(diagnostics::unexpected_token(self.cur_token().span()));
            self.bump_any();
            true
        } else {
            false
        }
    }

    fn validate_variable_declarator(
        &mut self,
        decl_parent: VariableDeclarationParent,
        decl: &VariableDeclarator<'a>,
        definite: Option<oxc_span::Span>,
    ) {
        if decl_parent == VariableDeclarationParent::Statement {
            self.check_missing_initializer(decl);
        }
        
        if let Some(span) = definite {
            self.validate_definite_assignment_assertion(decl, span);
        }
    }

    fn validate_definite_assignment_assertion(
        &mut self,
        decl: &VariableDeclarator<'a>,
        span: oxc_span::Span,
    ) {
        if decl.init.is_some() {
            self.error(diagnostics::variable_declarator_definite(span));
        } else if decl.id.type_annotation.is_none() {
            self.error(diagnostics::variable_declarator_definite_type_assertion(span));
        }
    }

    pub(crate) fn check_missing_initializer(&mut self, decl: &VariableDeclarator<'a>) {
        if decl.init.is_none() && !self.ctx.has_ambient() {
            if !matches!(decl.id.kind, BindingPatternKind::BindingIdentifier(_)) {
                self.error(diagnostics::invalid_destrucuring_declaration(decl.id.span()));
            } else if decl.kind == VariableDeclarationKind::Const {
                // It is a Syntax Error if Initializer is not present and IsConstantDeclaration of the LexicalDeclaration containing this LexicalBinding is true.
                self.error(diagnostics::missinginitializer_in_const(decl.id.span()));
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
        self.expect(Kind::Using);

        let kind = if is_await {
            VariableDeclarationKind::AwaitUsing
        } else {
            VariableDeclarationKind::Using
        };

        let declarations = self.parse_using_binding_list(statement_ctx, kind);
        self.ast.variable_declaration(self.end_span(span), kind, declarations, false)
    }

    fn parse_using_binding_list(
        &mut self,
        statement_ctx: StatementContext,
        kind: VariableDeclarationKind,
    ) -> oxc_allocator::Vec<'a, VariableDeclarator<'a>> {
        let mut declarations = self.ast.vec();
        
        loop {
            let declaration = self.parse_variable_declarator(
                VariableDeclarationParent::Statement,
                kind,
            );

            self.validate_using_declaration(&declaration, statement_ctx);
            declarations.push(declaration);
            
            if !self.eat(Kind::Comma) {
                break;
            }
        }
        
        declarations
    }

    fn validate_using_declaration(
        &mut self,
        declaration: &VariableDeclarator<'a>,
        statement_ctx: StatementContext,
    ) {
        match declaration.id.kind {
            BindingPatternKind::BindingIdentifier(_) => {}
            _ => {
                self.error(diagnostics::invalid_identifier_in_using_declaration(
                    declaration.id.span(),
                ));
            }
        }

        if declaration.init.is_none() && !matches!(statement_ctx, StatementContext::For) {
            self.error(diagnostics::using_declarations_must_be_initialized(
                declaration.id.span(),
            ));
        }
    }
}
