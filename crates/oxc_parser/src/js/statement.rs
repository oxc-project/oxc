use oxc_allocator::{Box, Vec};
use oxc_ast::ast::*;
use oxc_span::{Atom, GetSpan, Span};

use super::{VariableDeclarationParent, grammar::CoverGrammar};
use crate::{
    Context, ParserImpl, StatementContext, diagnostics,
    lexer::Kind,
    modifiers::{Modifier, ModifierFlags, ModifierKind, Modifiers},
};

impl<'a> ParserImpl<'a> {
    // Section 12
    // The InputElementHashbangOrRegExp goal is used at the start of a Script
    // or Module.
    pub(crate) fn parse_hashbang(&mut self) -> Option<Hashbang<'a>> {
        if self.cur_kind() == Kind::HashbangComment {
            let span = self.start_span();
            self.bump_any();
            let span = self.end_span(span);
            let src = &self.source_text[span.start as usize + 2..span.end as usize];
            Some(self.ast.hashbang(span, Atom::from(src)))
        } else {
            None
        }
    }

    /// <https://tc39.es/ecma262/#prod-StatementList>
    /// `StatementList`[Yield, Await, Return] :
    ///     `StatementListItem`[?Yield, ?Await, ?Return]
    ///     `StatementList`[?Yield, ?Await, ?Return] `StatementListItem`[?Yield, ?Await, ?Return]
    pub(crate) fn parse_directives_and_statements(
        &mut self,
        is_top_level: bool,
    ) -> (Vec<'a, Directive<'a>>, Vec<'a, Statement<'a>>) {
        let mut directives = self.ast.vec();
        let mut statements = self.ast.vec();

        let stmt_ctx = if is_top_level {
            StatementContext::TopLevelStatementList
        } else {
            StatementContext::StatementList
        };

        let mut expecting_directives = true;
        while !self.has_fatal_error() {
            if !is_top_level && self.at(Kind::RCurly) {
                break;
            }
            let stmt = self.parse_statement_list_item(stmt_ctx);

            // Section 11.2.1 Directive Prologue
            // The only way to get a correct directive is to parse the statement first and check if it is a string literal.
            // All other method are flawed, see test cases in [babel](https://github.com/babel/babel/blob/v7.26.2/packages/babel-parser/test/fixtures/core/categorized/not-directive/input.js)
            if expecting_directives {
                if let Statement::ExpressionStatement(expr) = &stmt {
                    if let Expression::StringLiteral(string) = &expr.expression {
                        // span start will mismatch if they are parenthesized when `preserve_parens = false`
                        if expr.span.start == string.span.start {
                            let src = &self.source_text
                                [string.span.start as usize + 1..string.span.end as usize - 1];
                            let directive =
                                self.ast.directive(expr.span, (*string).clone(), Atom::from(src));
                            directives.push(directive);
                            continue;
                        }
                    }
                }
                expecting_directives = false;
            }
            statements.push(stmt);
        }

        (directives, statements)
    }

    /// `StatementListItem`[Yield, Await, Return] :
    ///     Statement[?Yield, ?Await, ?Return]
    ///     Declaration[?Yield, ?Await]
    pub(crate) fn parse_statement_list_item(
        &mut self,
        stmt_ctx: StatementContext,
    ) -> Statement<'a> {
        let has_no_side_effects_comment =
            self.lexer.trivia_builder.previous_token_has_no_side_effects_comment();

        let mut stmt = match self.cur_kind() {
            Kind::LCurly => self.parse_block_statement(),
            Kind::Semicolon => self.parse_empty_statement(),
            Kind::If => self.parse_if_statement(),
            Kind::Do => self.parse_do_while_statement(),
            Kind::While => self.parse_while_statement(),
            Kind::For => self.parse_for_statement(),
            Kind::Break | Kind::Continue => self.parse_break_or_continue_statement(),
            Kind::With => self.parse_with_statement(),
            Kind::Switch => self.parse_switch_statement(),
            Kind::Throw => self.parse_throw_statement(),
            Kind::Try => self.parse_try_statement(),
            Kind::Debugger => self.parse_debugger_statement(),
            Kind::Class => self.parse_class_statement(
                self.start_span(),
                stmt_ctx,
                &Modifiers::empty(),
                self.ast.vec(),
            ),
            Kind::Export => {
                self.parse_export_declaration(self.start_span(), self.ast.vec(), stmt_ctx)
            }
            // [+Return] ReturnStatement[?Yield, ?Await]
            Kind::Return => self.parse_return_statement(),
            Kind::Var => {
                let span = self.start_span();
                self.bump_any();
                self.parse_variable_statement(span, VariableDeclarationKind::Var, stmt_ctx)
            }
            // Fast path
            Kind::Function => {
                self.parse_function_declaration(self.start_span(), /* async */ false, stmt_ctx)
            }
            Kind::At => self.parse_decorated_statement(stmt_ctx),
            Kind::Let if !self.cur_token().escaped() => self.parse_let(stmt_ctx),
            Kind::Async => self.parse_async_statement(self.start_span(), stmt_ctx),
            Kind::Import => self.parse_import_statement(stmt_ctx),
            Kind::Const => self.parse_const_statement(stmt_ctx),
            Kind::Using if self.is_using_declaration() => self.parse_using_statement(),
            Kind::Await if self.is_using_statement() => self.parse_using_statement(),
            Kind::Interface
            | Kind::Type
            | Kind::Module
            | Kind::Namespace
            | Kind::Declare
            | Kind::Enum
            | Kind::Private
            | Kind::Protected
            | Kind::Public
            | Kind::Abstract
            | Kind::Accessor
            | Kind::Static
            | Kind::Readonly
            | Kind::Global
                if self.is_ts && self.at_start_of_ts_declaration() =>
            {
                self.parse_ts_declaration_statement(self.start_span())
            }
            _ => self.parse_expression_or_labeled_statement(),
        };

        if has_no_side_effects_comment {
            Self::set_pure_on_function_stmt(&mut stmt);
        }

        stmt
    }

    fn set_pure_on_function_stmt(stmt: &mut Statement<'a>) {
        match stmt {
            Statement::FunctionDeclaration(func) => {
                func.pure = true;
            }
            Statement::ExportDefaultDeclaration(decl) => match &mut decl.declaration {
                ExportDefaultDeclarationKind::FunctionExpression(func)
                | ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                    func.pure = true;
                }
                ExportDefaultDeclarationKind::ArrowFunctionExpression(func) => {
                    func.pure = true;
                }
                _ => {}
            },
            Statement::ExportNamedDeclaration(decl) => match &mut decl.declaration {
                Some(Declaration::FunctionDeclaration(func)) => {
                    func.pure = true;
                }
                Some(Declaration::VariableDeclaration(var_decl)) if var_decl.kind.is_const() => {
                    if let Some(Some(expr)) = var_decl.declarations.first_mut().map(|d| &mut d.init)
                    {
                        Self::set_pure_on_function_expr(expr);
                    }
                }
                _ => {}
            },
            Statement::VariableDeclaration(var_decl) if var_decl.kind.is_const() => {
                if let Some(Some(expr)) = var_decl.declarations.first_mut().map(|d| &mut d.init) {
                    Self::set_pure_on_function_expr(expr);
                }
            }
            _ => {}
        }
    }

    fn parse_expression_or_labeled_statement(&mut self) -> Statement<'a> {
        let span = self.start_span();
        let expr = self.parse_expr();
        if let Expression::Identifier(ident) = &expr {
            // Section 14.13 Labelled Statement
            // Avoids lookahead for a labeled statement, which is on a hot path
            if self.eat(Kind::Colon) {
                let label = self.ast.label_identifier(ident.span, ident.name);
                let body = self.parse_statement_list_item(StatementContext::Label);
                return self.ast.statement_labeled(self.end_span(span), label, body);
            }
        }
        self.parse_expression_statement(span, expr)
    }

    /// Section 14.2 Block Statement
    pub(crate) fn parse_block(&mut self) -> Box<'a, BlockStatement<'a>> {
        let span = self.start_span();
        self.expect(Kind::LCurly);
        let mut body = self.ast.vec();
        while !self.at(Kind::RCurly) && !self.has_fatal_error() {
            let stmt = self.parse_statement_list_item(StatementContext::StatementList);
            body.push(stmt);
        }
        self.expect(Kind::RCurly);
        self.ast.alloc_block_statement(self.end_span(span), body)
    }

    pub(crate) fn parse_block_statement(&mut self) -> Statement<'a> {
        let block = self.parse_block();
        Statement::BlockStatement(block)
    }

    /// Section 14.3.2 Variable Statement
    pub(crate) fn parse_variable_statement(
        &mut self,
        start_span: u32,
        kind: VariableDeclarationKind,
        stmt_ctx: StatementContext,
    ) -> Statement<'a> {
        let decl = self.parse_variable_declaration(
            start_span,
            kind,
            VariableDeclarationParent::Statement,
            &Modifiers::empty(),
        );

        if stmt_ctx.is_single_statement() && decl.kind.is_lexical() {
            self.error(diagnostics::lexical_declaration_single_statement(decl.span));
        }

        Statement::VariableDeclaration(decl)
    }

    /// Section 14.4 Empty Statement
    fn parse_empty_statement(&mut self) -> Statement<'a> {
        let span = self.start_span();
        self.bump_any(); // bump `;`
        self.ast.statement_empty(self.end_span(span))
    }

    /// Section 14.5 Expression Statement
    pub(crate) fn parse_expression_statement(
        &mut self,
        span: u32,
        expression: Expression<'a>,
    ) -> Statement<'a> {
        self.asi();
        self.ast.statement_expression(self.end_span(span), expression)
    }

    /// Section 14.6 If Statement
    fn parse_if_statement(&mut self) -> Statement<'a> {
        let span = self.start_span();
        self.bump_any(); // bump `if`
        let test = self.parse_paren_expression();
        let consequent = self.parse_statement_list_item(StatementContext::If);
        let alternate =
            self.eat(Kind::Else).then(|| self.parse_statement_list_item(StatementContext::If));
        self.ast.statement_if(self.end_span(span), test, consequent, alternate)
    }

    /// Section 14.7.2 Do-While Statement
    fn parse_do_while_statement(&mut self) -> Statement<'a> {
        let span = self.start_span();
        self.bump_any(); // advance `do`
        let body = self.parse_statement_list_item(StatementContext::Do);
        self.expect(Kind::While);
        let test = self.parse_paren_expression();
        self.bump(Kind::Semicolon);
        self.ast.statement_do_while(self.end_span(span), body, test)
    }

    /// Section 14.7.3 While Statement
    fn parse_while_statement(&mut self) -> Statement<'a> {
        let span = self.start_span();
        self.bump_any(); // bump `while`
        let test = self.parse_paren_expression();
        let body = self.parse_statement_list_item(StatementContext::While);
        self.ast.statement_while(self.end_span(span), test, body)
    }

    /// Section 14.7.4 For Statement
    fn parse_for_statement(&mut self) -> Statement<'a> {
        let span = self.start_span();
        self.bump_any(); // bump `for`

        // [+Await]
        let r#await = if self.at(Kind::Await) {
            if !self.ctx.has_await() {
                self.error(diagnostics::await_expression(self.cur_token().span()));
            }
            self.bump_any();
            true
        } else {
            false
        };

        self.expect(Kind::LParen);

        // for (;..
        if self.at(Kind::Semicolon) {
            return self.parse_for_loop(span, None, r#await);
        }

        // `for (const` | `for (var`
        match self.cur_kind() {
            Kind::Const | Kind::Var => {
                let start_span = self.start_span();
                return self
                    .parse_variable_declaration_for_statement(span, start_span, None, r#await);
            }
            Kind::Let => {
                // `for (let`
                let start_span = self.start_span();
                let checkpoint = self.checkpoint();
                self.bump_any(); // bump `let`
                // disallow `for (let in ...`
                if self.cur_kind().is_after_let() {
                    return self.parse_variable_declaration_for_statement(
                        span,
                        start_span,
                        Some(VariableDeclarationKind::Let),
                        r#await,
                    );
                }
                // Should be a relatively cold branch, since most tokens after `let` will be allowed in most files
                self.rewind(checkpoint);
            }
            _ => {}
        }

        // [+Using, +Await] await [no LineTerminator here] using [no LineTerminator here]
        if self.at(Kind::Await)
            && self.lookahead(|p| {
                p.bump_any();
                if !p.at(Kind::Using) || p.cur_token().is_on_new_line() {
                    return false;
                }
                p.bump_any();
                !p.cur_token().is_on_new_line()
            })
        {
            return self.parse_using_declaration_for_statement(span, r#await);
        }

        // [+Using] using [no LineTerminator here] ForBinding[?Yield, ?Await, ~Pattern]
        if self.at(Kind::Using) && {
            let token = self.lexer.peek_token();
            let kind = token.kind();
            !token.is_on_new_line() && kind != Kind::Of && kind.is_binding_identifier()
        } {
            return self.parse_using_declaration_for_statement(span, r#await);
        }

        if self.at(Kind::RParen) {
            return self.parse_for_loop(span, None, r#await);
        }

        let is_let = self.at(Kind::Let);
        // `async` is allowed as `for (async of ...)` if `async` is escaped
        let is_async = self.at(Kind::Async) && !self.cur_token().escaped();
        let expr_span = self.start_span();

        let init_expression = self.context(Context::empty(), Context::In, ParserImpl::parse_expr);

        match self.cur_kind() {
            Kind::In => {
                let target = AssignmentTarget::cover(init_expression, self);
                let for_stmt_left = ForStatementLeft::from(target);
                self.parse_for_in_loop(span, r#await, for_stmt_left)
            }
            Kind::Of => {
                if init_expression.is_identifier_reference() {
                    if !r#await && is_async {
                        // `for (async of ...)` is not allowed
                        self.error(diagnostics::for_loop_async_of(self.end_span(expr_span)));
                    }
                    if is_let {
                        // `for (let of ...)` is not allowed
                        self.error(diagnostics::unexpected_token(self.end_span(expr_span)));
                    }
                }
                let target = AssignmentTarget::cover(init_expression, self);
                let for_stmt_left = ForStatementLeft::from(target);
                self.parse_for_of_loop(span, r#await, for_stmt_left)
            }
            _ => self.parse_for_loop(span, Some(ForStatementInit::from(init_expression)), r#await),
        }
    }

    fn parse_variable_declaration_for_statement(
        &mut self,
        span: u32,
        start_span: u32,
        decl_kind: Option<VariableDeclarationKind>,
        r#await: bool,
    ) -> Statement<'a> {
        let init_declaration = self.context(Context::empty(), Context::In, |p| {
            let decl_ctx = VariableDeclarationParent::For;
            let kind = if let Some(kind) = decl_kind {
                kind
            } else {
                let kind = p.get_variable_declaration_kind();
                p.bump_any();
                kind
            };
            p.parse_variable_declaration(start_span, kind, decl_ctx, &Modifiers::empty())
        });

        self.parse_any_for_loop(span, init_declaration, r#await)
    }

    fn is_using_declaration(&mut self) -> bool {
        self.lookahead(|p| {
            p.is_next_token_binding_identifier_or_start_of_object_destructuring_on_same_line(false)
        })
    }

    fn is_next_token_binding_identifier_or_start_of_object_destructuring_on_same_line(
        &mut self,
        disallow_of: bool,
    ) -> bool {
        self.bump_any();
        if disallow_of && self.at(Kind::Of) {
            return false;
        }
        (self.cur_kind().is_binding_identifier() || self.at(Kind::LCurly))
            && !self.cur_token().is_on_new_line()
    }

    fn parse_using_declaration_for_statement(&mut self, span: u32, r#await: bool) -> Statement<'a> {
        let using_decl = self.parse_using_declaration(StatementContext::For);

        if matches!(self.cur_kind(), Kind::In) {
            if using_decl.kind.is_await() {
                self.error(diagnostics::await_using_declaration_not_allowed_in_for_in_statement(
                    using_decl.span,
                ));
            } else {
                self.error(diagnostics::using_declaration_not_allowed_in_for_in_statement(
                    using_decl.span,
                ));
            }
        }

        let init_declaration = self.alloc(using_decl);
        self.parse_any_for_loop(span, init_declaration, r#await)
    }

    fn parse_any_for_loop(
        &mut self,
        span: u32,
        init_declaration: Box<'a, VariableDeclaration<'a>>,
        r#await: bool,
    ) -> Statement<'a> {
        match self.cur_kind() {
            Kind::In => self.parse_for_in_loop(
                span,
                r#await,
                ForStatementLeft::VariableDeclaration(init_declaration),
            ),
            Kind::Of => self.parse_for_of_loop(
                span,
                r#await,
                ForStatementLeft::VariableDeclaration(init_declaration),
            ),
            _ => self.parse_for_loop(
                span,
                Some(ForStatementInit::VariableDeclaration(init_declaration)),
                r#await,
            ),
        }
    }

    fn parse_for_loop(
        &mut self,
        span: u32,
        init: Option<ForStatementInit<'a>>,
        r#await: bool,
    ) -> Statement<'a> {
        self.expect(Kind::Semicolon);
        if let Some(ForStatementInit::VariableDeclaration(decl)) = &init {
            for d in &decl.declarations {
                self.check_missing_initializer(d);
            }
        }
        let test = if !self.at(Kind::Semicolon) && !self.at(Kind::RParen) {
            Some(self.context(Context::In, Context::empty(), ParserImpl::parse_expr))
        } else {
            None
        };
        self.expect(Kind::Semicolon);
        let update = if self.at(Kind::RParen) {
            None
        } else {
            Some(self.context(Context::In, Context::empty(), ParserImpl::parse_expr))
        };
        self.expect(Kind::RParen);
        if r#await {
            self.error(diagnostics::for_await(self.end_span(span)));
        }
        let body = self.parse_statement_list_item(StatementContext::For);
        self.ast.statement_for(self.end_span(span), init, test, update, body)
    }

    fn parse_for_in_loop(
        &mut self,
        span: u32,
        r#await: bool,
        left: ForStatementLeft<'a>,
    ) -> Statement<'a> {
        self.bump_any(); // bump `in`
        let right = self.parse_expr();
        self.expect(Kind::RParen);

        if r#await {
            self.error(diagnostics::for_await(self.end_span(span)));
        }

        let body = self.parse_statement_list_item(StatementContext::For);
        let span = self.end_span(span);
        self.ast.statement_for_in(span, left, right, body)
    }

    fn parse_for_of_loop(
        &mut self,
        span: u32,
        r#await: bool,
        left: ForStatementLeft<'a>,
    ) -> Statement<'a> {
        self.bump_any(); // bump `of`
        let right = self.parse_assignment_expression_or_higher();
        self.expect(Kind::RParen);

        let body = self.parse_statement_list_item(StatementContext::For);
        let span = self.end_span(span);
        self.ast.statement_for_of(span, r#await, left, right, body)
    }

    /// Section 14.8 Continue Statement
    /// Section 14.9 Break Statement
    fn parse_break_or_continue_statement(&mut self) -> Statement<'a> {
        let span = self.start_span();
        let kind = self.cur_kind();
        self.bump_any(); // bump `break` or `continue`
        let label =
            if self.can_insert_semicolon() { None } else { Some(self.parse_label_identifier()) };
        self.asi();
        let end_span = self.end_span(span);
        match kind {
            Kind::Break => self.ast.statement_break(end_span, label),
            Kind::Continue => self.ast.statement_continue(end_span, label),
            _ => unreachable!(),
        }
    }

    /// Section 14.10 Return Statement
    /// `ReturnStatement`[Yield, Await] :
    ///   return ;
    ///   return [no `LineTerminator` here] Expression[+In, ?Yield, ?Await] ;
    fn parse_return_statement(&mut self) -> Statement<'a> {
        let span = self.start_span();
        self.bump_any(); // advance `return`
        let argument = if self.eat(Kind::Semicolon) || self.can_insert_semicolon() {
            None
        } else {
            let expr = self.context(Context::In, Context::empty(), ParserImpl::parse_expr);
            self.asi();
            Some(expr)
        };
        if !self.ctx.has_return() {
            self.error(diagnostics::return_statement_only_in_function_body(Span::sized(span, 6)));
        }
        self.ast.statement_return(self.end_span(span), argument)
    }

    /// Section 14.11 With Statement
    fn parse_with_statement(&mut self) -> Statement<'a> {
        let span = self.start_span();
        self.bump_any(); // bump `with`
        let object = self.parse_paren_expression();
        let body = self.parse_statement_list_item(StatementContext::With);
        let span = self.end_span(span);
        self.ast.statement_with(span, object, body)
    }

    /// Section 14.12 Switch Statement
    fn parse_switch_statement(&mut self) -> Statement<'a> {
        let span = self.start_span();
        self.bump_any(); // advance `switch`
        let discriminant = self.parse_paren_expression();
        let cases = self.parse_normal_list(Kind::LCurly, Kind::RCurly, Self::parse_switch_case);
        self.ast.statement_switch(self.end_span(span), discriminant, cases)
    }

    pub(crate) fn parse_switch_case(&mut self) -> Option<SwitchCase<'a>> {
        let span = self.start_span();
        let test = match self.cur_kind() {
            Kind::Default => {
                self.bump_any();
                None
            }
            Kind::Case => {
                self.bump_any();
                let expression = self.parse_expr();
                Some(expression)
            }
            _ => return self.unexpected(),
        };
        self.expect(Kind::Colon);
        let mut consequent = self.ast.vec();
        while !matches!(self.cur_kind(), Kind::Case | Kind::Default | Kind::RCurly)
            && !self.has_fatal_error()
        {
            let stmt = self.parse_statement_list_item(StatementContext::StatementList);
            consequent.push(stmt);
        }
        Some(self.ast.switch_case(self.end_span(span), test, consequent))
    }

    /// Section 14.14 Throw Statement
    fn parse_throw_statement(&mut self) -> Statement<'a> {
        let span = self.start_span();
        self.bump_any(); // advance `throw`
        if self.cur_token().is_on_new_line() {
            self.error(diagnostics::illegal_newline(
                "throw",
                self.end_span(span),
                self.cur_token().span(),
            ));
        }
        let argument = self.parse_expr();
        self.asi();
        self.ast.statement_throw(self.end_span(span), argument)
    }

    /// Section 14.15 Try Statement
    fn parse_try_statement(&mut self) -> Statement<'a> {
        let span = self.start_span();
        self.bump_any(); // bump `try`

        let block = self.parse_block();

        let handler = self.at(Kind::Catch).then(|| self.parse_catch_clause());

        let finalizer = self.eat(Kind::Finally).then(|| self.parse_block());

        if handler.is_none() && finalizer.is_none() {
            let range = Span::empty(block.span.end);
            self.error(diagnostics::expect_catch_finally(range));
        }

        self.ast.statement_try(self.end_span(span), block, handler, finalizer)
    }

    fn parse_catch_clause(&mut self) -> Box<'a, CatchClause<'a>> {
        let span = self.start_span();
        self.bump_any(); // advance `catch`
        let pattern = if self.eat(Kind::LParen) {
            let pattern = self.parse_binding_pattern(false);
            self.expect(Kind::RParen);
            Some(pattern)
        } else {
            None
        };
        let body = self.parse_block();
        let param = pattern.map(|pattern| self.ast.catch_parameter(pattern.kind.span(), pattern));
        self.ast.alloc_catch_clause(self.end_span(span), param, body)
    }

    /// Section 14.16 Debugger Statement
    fn parse_debugger_statement(&mut self) -> Statement<'a> {
        let span = self.start_span();
        self.bump_any();
        self.asi();
        self.ast.statement_debugger(self.end_span(span))
    }

    /// Parse const declaration or `const enum`.
    fn parse_const_statement(&mut self, stmt_ctx: StatementContext) -> Statement<'a> {
        let span = self.start_span();
        self.bump_any();
        if self.is_ts && self.at(Kind::Enum) {
            let modifiers = self.ast.vec1(Modifier::new(self.end_span(span), ModifierKind::Const));
            let modifiers = Modifiers::new(Some(modifiers), ModifierFlags::CONST);
            Statement::from(self.parse_ts_enum_declaration(span, &modifiers))
        } else {
            self.parse_variable_statement(span, VariableDeclarationKind::Const, stmt_ctx)
        }
    }

    /// Parse import statement or import expression.
    fn parse_import_statement(&mut self, stmt_ctx: StatementContext) -> Statement<'a> {
        let checkpoint = self.checkpoint();
        let span = self.start_span();
        self.bump_any();
        if matches!(self.cur_kind(), Kind::Dot | Kind::LParen) {
            // Parse the whole expression `import.meta.url` so a rewind is required.
            self.rewind(checkpoint);
            self.parse_expression_or_labeled_statement()
        } else {
            self.parse_import_declaration(span, stmt_ctx.is_top_level())
        }
    }

    /// Parse statements that start with `sync`.
    fn parse_async_statement(&mut self, span: u32, stmt_ctx: StatementContext) -> Statement<'a> {
        let checkpoint = self.checkpoint();
        self.bump_any(); // bump `async`
        let token = self.cur_token();
        if token.kind() == Kind::Function && !token.is_on_new_line() {
            return self.parse_function_declaration(span, /* async */ true, stmt_ctx);
        }
        self.rewind(checkpoint);
        if self.is_ts && self.at_start_of_ts_declaration() {
            return self.parse_ts_declaration_statement(span);
        }
        self.parse_expression_or_labeled_statement()
    }

    /// Parse statements that start with `@`.
    fn parse_decorated_statement(&mut self, stmt_ctx: StatementContext) -> Statement<'a> {
        let span = self.start_span();
        let decorators = self.parse_decorators();
        let kind = self.cur_kind();
        if kind == Kind::Export {
            // Export span.start starts after decorators.
            return self.parse_export_declaration(self.start_span(), decorators, stmt_ctx);
        }
        let modifiers = self.parse_modifiers(false, false);
        if self.at(Kind::Class) {
            // Class span.start starts before decorators.
            return self.parse_class_statement(span, stmt_ctx, &modifiers, decorators);
        }
        self.unexpected()
    }
}
