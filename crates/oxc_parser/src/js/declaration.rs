use oxc_allocator::{ArenaBox, ArenaVec, Slot, SlotFilled};
use oxc_ast::{ast::*, builder::builders::traits::SlotBuild};
use oxc_span::{GetSpan, Span};

use super::VariableDeclarationParent;
use crate::{ParserConfig as Config, ParserImpl, StatementContext, diagnostics, lexer::Kind};

impl<'a, C: Config> ParserImpl<'a, C> {
    pub(crate) fn parse_let(&mut self, stmt_ctx: StatementContext) -> Statement<'a> {
        let start = self.cur_start();

        let peeked = self.lexer.peek_token().kind();

        // Fast path: avoid rewind.
        if !stmt_ctx.is_single_statement() && peeked.is_after_let() {
            self.bump_any(); // bump `let`
            return self.parse_variable_statement(start, VariableDeclarationKind::Let, stmt_ctx);
        }

        // let = foo, let instanceof x, let + 1
        if peeked.is_assignment_operator() || peeked.is_binary_operator() {
            let expr = self.parse_assignment_expression_or_higher();
            self.parse_expression_statement(start, expr)
        // let.a = 1, let?.a = 1, let()[a] = 1
        } else if matches!(peeked, Kind::Dot | Kind::QuestionDot | Kind::LParen) {
            let expr = self.parse_expr();
            self.parse_expression_statement(start, expr)
        // single statement let declaration: while (0) let
        } else if (stmt_ctx.is_single_statement() && peeked != Kind::LBrack)
            || peeked == Kind::Semicolon
        {
            let expr = self.parse_identifier_expression();
            self.parse_expression_statement(start, expr)
        } else {
            self.bump_any();
            self.parse_variable_statement(start, VariableDeclarationKind::Let, stmt_ctx)
        }
    }

    pub(crate) fn is_using_statement(&mut self) -> bool {
        // `await using` requires `using` immediately after `await` on the same line. Cheaply peek
        // for it first, so the common `await <expr>` statement avoids the heavier `lookahead`
        // (checkpoint + rewind) and only `await using` pays for the binding-identifier check.
        let next = self.lexer.peek_token();
        next.kind() == Kind::Using
            && !next.is_on_new_line()
            && self.lookahead(Self::is_next_token_using_keyword_then_binding_identifier)
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
        Statement::VariableDeclaration(decl)
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
        start: u32,
        kind: VariableDeclarationKind,
        decl_parent: VariableDeclarationParent,
        declare: bool,
    ) -> ArenaBox<'a, VariableDeclaration<'a>> {
        VariableDeclaration::build(self)
            .span_start(start)
            .kind(kind)
            .declarations_with(|slot| {
                let mut declarations = ArenaVec::new_in(self);
                loop {
                    declarations.push_with(|slot| {
                        self.parse_variable_declarator_into(slot, decl_parent, kind)
                    });
                    if !self.eat(Kind::Comma) {
                        break;
                    }
                }
                slot.fill(declarations)
            })
            .declare(declare)
            .span_end({
                if matches!(decl_parent, VariableDeclarationParent::Statement) {
                    self.asi();
                }
                self.end_span(start).end
            })
            .finish()
    }

    fn parse_variable_declarator_into<'slot>(
        &mut self,
        slot: Slot<'slot, VariableDeclarator<'a>>,
        decl_parent: VariableDeclarationParent,
        kind: VariableDeclarationKind,
    ) -> SlotFilled<'slot> {
        let start = self.cur_start();

        let id = self.parse_binding_pattern();

        let (type_annotation, definite_start) = if self.is_ts {
            // const x!: number = 1
            //        ^ definite
            let definite_start = if id.is_binding_identifier()
                && !self.cur_token().is_on_new_line()
                && self.at(Kind::Bang)
            {
                let definite_start = self.cur_token().start();
                self.bump_any();
                Some(definite_start)
            } else {
                None
            };
            if self.at(Kind::Question) {
                self.error(diagnostics::unexpected_optional_declaration(self.cur_token().span()));
                self.bump_any();
            }
            let type_annotation = self.parse_ts_type_annotation();
            (type_annotation, definite_start)
        } else {
            (None, None)
        };
        // `const foo /* #__PURE__ */ = bar()` - pure comment before `=` cannot be applied
        self.lexer.trivia_builder.mark_current_pure_comment_not_applied();
        let init = self.eat(Kind::Eq).then(|| self.parse_assignment_expression_or_higher());
        let span = self.end_span(start);
        if self.ctx.has_ambient()
            && let Some(init) = &init
            && !kind.is_using()
            && !(kind.is_const() && type_annotation.is_none())
        {
            self.error(diagnostics::initializers_not_allowed_in_ambient_contexts(init.span()));
        }
        if decl_parent == VariableDeclarationParent::Statement {
            self.check_missing_initializer(&id, init.as_ref(), kind);
        }
        if let Some(definite_start) = definite_start {
            let span = Span::sized(definite_start, 1);
            if init.is_some() {
                self.error(diagnostics::variable_declarator_definite(span));
            } else if type_annotation.is_none() {
                self.error(diagnostics::variable_declarator_definite_type_assertion(span));
            } else if self.ctx.has_ambient() {
                self.error(diagnostics::definite_assignment_assertion_not_permitted(span));
            }
        }
        if kind.is_using() && !id.is_binding_identifier() {
            self.error(diagnostics::invalid_identifier_in_using_declaration(id.span()));
        }

        slot.build(self)
            .span(span)
            .id(id)
            .type_annotation(type_annotation)
            .init(init)
            .definite(definite_start.is_some())
            .finish()
    }

    pub(crate) fn check_missing_initializer(
        &mut self,
        id: &BindingPattern<'a>,
        init: Option<&Expression<'a>>,
        kind: VariableDeclarationKind,
    ) {
        if init.is_none() && !self.ctx.has_ambient() {
            if !id.is_binding_identifier() {
                self.error(diagnostics::invalid_destructuring_declaration(id.span()));
            } else if kind == VariableDeclarationKind::Const {
                // It is a Syntax Error if Initializer is not present and IsConstantDeclaration of the LexicalDeclaration containing this LexicalBinding is true.
                self.error(diagnostics::missing_initializer_in_const(id.span()));
            } else if kind.is_using() {
                self.error(diagnostics::using_declarations_must_be_initialized(id.span()));
            }
        }
    }

    /// Section 14.3.1 Let, Const, and Using Declarations
    /// UsingDeclaration[In, Yield, Await] :
    /// using [no LineTerminator here] [lookahead ≠ await] BindingList[?In, ?Yield, ?Await, ~Pattern] ;
    pub(crate) fn parse_using_declaration(
        &mut self,
        statement_ctx: StatementContext,
    ) -> ArenaBox<'a, VariableDeclaration<'a>> {
        let start = self.cur_start();

        let is_await = self.eat(Kind::Await);
        let kind = if is_await {
            VariableDeclarationKind::AwaitUsing
        } else {
            VariableDeclarationKind::Using
        };

        self.expect(Kind::Using);
        if self.ctx.has_ambient() {
            let using_span = self.cur_token().span();
            self.error(if kind.is_await() {
                diagnostics::await_using_declarations_not_allowed_in_ambient_contexts(using_span)
            } else {
                diagnostics::using_declarations_not_allowed_in_ambient_contexts(using_span)
            });
        }

        let decl_parent = if matches!(statement_ctx, StatementContext::For) {
            VariableDeclarationParent::For
        } else {
            VariableDeclarationParent::Statement
        };
        VariableDeclaration::build(self)
            .span_start(start)
            .kind(kind)
            .declarations_with(|slot| {
                let mut declarations = ArenaVec::new_in(self);
                loop {
                    declarations.push_with(|slot| {
                        self.parse_variable_declarator_into(slot, decl_parent, kind)
                    });
                    if !self.eat(Kind::Comma) {
                        break;
                    }
                }
                slot.fill(declarations)
            })
            .declare(false)
            .span_end(self.end_span(start).end)
            .finish()
    }
}
