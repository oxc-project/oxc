use oxc_allocator::Box;
use oxc_ast::ast::*;
use oxc_span::Span;

use super::FunctionKind;
use crate::{
    Context, ParserImpl, StatementContext, diagnostics,
    lexer::Kind,
    modifiers::{ModifierFlags, ModifierKind, Modifiers},
};

impl FunctionKind {
    pub(crate) fn is_id_required(self) -> bool {
        matches!(self, Self::Declaration)
    }

    pub(crate) fn is_expression(self) -> bool {
        self == Self::Expression
    }
}

impl<'a> ParserImpl<'a> {
    pub(crate) fn at_function_with_async(&mut self) -> bool {
        self.at(Kind::Function)
            || self.at(Kind::Async) && {
                let token = self.lexer.peek_token();
                token.kind() == Kind::Function && !token.is_on_new_line()
            }
    }

    pub(crate) fn parse_function_body(&mut self) -> Box<'a, FunctionBody<'a>> {
        let span = self.start_span();
        self.expect(Kind::LCurly);

        let (directives, statements) = self.context(Context::Return, Context::empty(), |p| {
            p.parse_directives_and_statements(/* is_top_level */ false)
        });

        self.expect(Kind::RCurly);
        self.ast.alloc_function_body(self.end_span(span), directives, statements)
    }

    pub(crate) fn parse_formal_parameters(
        &mut self,
        func_kind: FunctionKind,
        params_kind: FormalParameterKind,
    ) -> (Option<TSThisParameter<'a>>, Box<'a, FormalParameters<'a>>) {
        let span = self.start_span();
        self.expect(Kind::LParen);
        let this_param = if self.is_ts && self.at(Kind::This) {
            let param = self.parse_ts_this_parameter();
            self.bump(Kind::Comma);
            Some(param)
        } else {
            None
        };
        let (list, rest) = self.parse_delimited_list_with_rest(
            Kind::RParen,
            |p| p.parse_formal_parameter(func_kind),
            diagnostics::rest_parameter_last,
        );
        self.expect(Kind::RParen);
        let formal_parameters =
            self.ast.alloc_formal_parameters(self.end_span(span), params_kind, list, rest);
        (this_param, formal_parameters)
    }

    fn parse_formal_parameter(&mut self, func_kind: FunctionKind) -> FormalParameter<'a> {
        let span = self.start_span();
        let decorators = self.parse_decorators();
        let modifiers = self.parse_modifiers(false, false);
        if self.is_ts {
            let mut allowed_modifiers = ModifierFlags::READONLY;
            if func_kind == FunctionKind::Constructor {
                allowed_modifiers = allowed_modifiers
                    .union(ModifierFlags::ACCESSIBILITY)
                    .union(ModifierFlags::OVERRIDE);
            }
            self.verify_modifiers(
                &modifiers,
                allowed_modifiers,
                diagnostics::cannot_appear_on_a_parameter,
            );
        } else {
            self.verify_modifiers(
                &modifiers,
                ModifierFlags::empty(),
                diagnostics::parameter_modifiers_in_ts,
            );
        }
        let pattern = self.parse_binding_pattern_with_initializer();
        let are_decorators_allowed =
            matches!(func_kind, FunctionKind::ClassMethod | FunctionKind::Constructor)
                && self.is_ts;
        if !are_decorators_allowed {
            for decorator in &decorators {
                self.error(diagnostics::decorators_are_not_valid_here(decorator.span));
            }
        }
        self.ast.formal_parameter(
            self.end_span(span),
            decorators,
            pattern,
            modifiers.accessibility(),
            modifiers.contains_readonly(),
            modifiers.contains_override(),
        )
    }

    pub(crate) fn parse_function(
        &mut self,
        span: u32,
        id: Option<BindingIdentifier<'a>>,
        r#async: bool,
        generator: bool,
        func_kind: FunctionKind,
        param_kind: FormalParameterKind,
        modifiers: &Modifiers<'a>,
    ) -> Box<'a, Function<'a>> {
        let ctx = self.ctx;
        self.ctx = self.ctx.and_in(true).and_await(r#async).and_yield(generator);
        let type_parameters = self.parse_ts_type_parameters();
        let (this_param, params) = self.parse_formal_parameters(func_kind, param_kind);
        let return_type = if self.is_ts { self.parse_ts_return_type_annotation() } else { None };
        let body = if self.at(Kind::LCurly) { Some(self.parse_function_body()) } else { None };
        self.ctx =
            self.ctx.and_in(ctx.has_in()).and_await(ctx.has_await()).and_yield(ctx.has_yield());
        if !self.is_ts && body.is_none() {
            return self.unexpected();
        }
        let function_type = match func_kind {
            FunctionKind::Declaration | FunctionKind::DefaultExport => {
                if body.is_none() {
                    FunctionType::TSDeclareFunction
                } else {
                    FunctionType::FunctionDeclaration
                }
            }
            FunctionKind::Expression
            | FunctionKind::ClassMethod
            | FunctionKind::Constructor
            | FunctionKind::ObjectMethod => {
                if body.is_none() {
                    FunctionType::TSEmptyBodyFunctionExpression
                } else {
                    FunctionType::FunctionExpression
                }
            }
            FunctionKind::TSDeclaration => FunctionType::TSDeclareFunction,
        };

        if FunctionType::TSDeclareFunction == function_type
            || FunctionType::TSEmptyBodyFunctionExpression == function_type
        {
            self.asi();
        }

        if ctx.has_ambient() && modifiers.contains_declare() {
            if let Some(body) = &body {
                self.error(diagnostics::implementation_in_ambient(Span::empty(body.span.start)));
            }
        }
        self.verify_modifiers(
            modifiers,
            ModifierFlags::DECLARE | ModifierFlags::ASYNC,
            diagnostics::modifier_cannot_be_used_here,
        );

        self.ast.alloc_function(
            self.end_span(span),
            function_type,
            id,
            generator,
            r#async,
            modifiers.contains_declare(),
            type_parameters,
            this_param,
            params,
            return_type,
            body,
        )
    }

    /// [Function Declaration](https://tc39.es/ecma262/#prod-FunctionDeclaration)
    pub(crate) fn parse_function_declaration(
        &mut self,
        span: u32,
        r#async: bool,
        stmt_ctx: StatementContext,
    ) -> Statement<'a> {
        let func_kind = FunctionKind::Declaration;
        let decl = self.parse_function_impl(span, r#async, func_kind);
        if stmt_ctx.is_single_statement() {
            if decl.r#async {
                self.error(diagnostics::async_function_declaration(Span::new(
                    decl.span.start,
                    decl.params.span.end,
                )));
            } else if decl.generator {
                self.error(diagnostics::generator_function_declaration(Span::new(
                    decl.span.start,
                    decl.params.span.end,
                )));
            }
        }
        Statement::FunctionDeclaration(decl)
    }

    /// Parse function implementation in Javascript, cursor
    /// at `function` or `async function`
    pub(crate) fn parse_function_impl(
        &mut self,
        span: u32,
        r#async: bool,
        func_kind: FunctionKind,
    ) -> Box<'a, Function<'a>> {
        self.expect(Kind::Function);
        let generator = self.eat(Kind::Star);
        let id = self.parse_function_id(func_kind, r#async, generator);
        self.parse_function(
            span,
            id,
            r#async,
            generator,
            func_kind,
            FormalParameterKind::FormalParameter,
            &Modifiers::empty(),
        )
    }

    /// Parse function implementation in Typescript, cursor
    /// at `function`
    pub(crate) fn parse_ts_function_impl(
        &mut self,
        start_span: u32,
        func_kind: FunctionKind,
        modifiers: &Modifiers<'a>,
    ) -> Box<'a, Function<'a>> {
        let r#async = modifiers.contains(ModifierKind::Async);
        self.expect(Kind::Function);
        let generator = self.eat(Kind::Star);
        let id = self.parse_function_id(func_kind, r#async, generator);
        self.parse_function(
            start_span,
            id,
            r#async,
            generator,
            func_kind,
            FormalParameterKind::FormalParameter,
            modifiers,
        )
    }

    /// [Function Expression](https://tc39.es/ecma262/#prod-FunctionExpression)
    pub(crate) fn parse_function_expression(&mut self, span: u32, r#async: bool) -> Expression<'a> {
        let func_kind = FunctionKind::Expression;
        self.expect(Kind::Function);

        let generator = self.eat(Kind::Star);
        let id = self.parse_function_id(func_kind, r#async, generator);
        let function = self.parse_function(
            span,
            id,
            r#async,
            generator,
            func_kind,
            FormalParameterKind::FormalParameter,
            &Modifiers::empty(),
        );
        Expression::FunctionExpression(function)
    }

    /// Section 15.4 Method Definitions
    /// `ClassElementName` ( `UniqueFormalParameters` ) { `FunctionBody` }
    /// * `GeneratorMethod`
    ///   * `ClassElementName`
    /// * `AsyncMethod`
    ///   async `ClassElementName`
    /// * `AsyncGeneratorMethod`
    ///   async * `ClassElementName`
    pub(crate) fn parse_method(
        &mut self,
        r#async: bool,
        generator: bool,
        func_kind: FunctionKind,
    ) -> Box<'a, Function<'a>> {
        let span = self.start_span();
        self.parse_function(
            span,
            None,
            r#async,
            generator,
            func_kind,
            FormalParameterKind::UniqueFormalParameters,
            &Modifiers::empty(),
        )
    }

    /// Section 15.5 Yield Expression
    /// yield
    /// yield [no `LineTerminator` here] `AssignmentExpression`
    /// yield [no `LineTerminator` here] * `AssignmentExpression`
    pub(crate) fn parse_yield_expression(&mut self) -> Expression<'a> {
        let span = self.start_span();
        self.bump_any(); // advance `yield`

        let has_yield = self.ctx.has_yield();
        if !has_yield {
            self.error(diagnostics::yield_expression(Span::sized(span, 5)));
        }

        let mut delegate = false;
        let mut argument = None;

        if !self.cur_token().is_on_new_line() {
            delegate = self.eat(Kind::Star);
            let not_assignment_expr = matches!(
                self.cur_kind(),
                Kind::Semicolon
                    | Kind::Eof
                    | Kind::RCurly
                    | Kind::RParen
                    | Kind::RBrack
                    | Kind::Colon
                    | Kind::Comma
            );
            if !not_assignment_expr || delegate {
                self.ctx = self.ctx.union_yield_if(true);
                argument = Some(self.parse_assignment_expression_or_higher());
                self.ctx = self.ctx.and_yield(has_yield);
            }
        }

        self.ast.expression_yield(self.end_span(span), delegate, argument)
    }

    // id: None - for AnonymousDefaultExportedFunctionDeclaration
    pub(crate) fn parse_function_id(
        &mut self,
        func_kind: FunctionKind,
        r#async: bool,
        generator: bool,
    ) -> Option<BindingIdentifier<'a>> {
        let kind = self.cur_kind();
        if kind.is_binding_identifier() {
            let mut ctx = self.ctx;
            if func_kind.is_expression() {
                ctx = ctx.and_await(r#async).and_yield(generator);
            }
            self.check_identifier(kind, ctx);

            let (span, name) = self.parse_identifier_kind(Kind::Ident);
            Some(self.ast.binding_identifier(span, name))
        } else {
            if func_kind.is_id_required() {
                match self.cur_kind() {
                    Kind::LParen => {
                        self.error(diagnostics::expect_function_name(self.cur_token().span()));
                    }
                    kind if kind.is_reserved_keyword() => self.expect_without_advance(Kind::Ident),
                    _ => {}
                }
            }

            None
        }
    }
}
