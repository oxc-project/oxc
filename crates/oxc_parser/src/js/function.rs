use std::cell::Cell;

use oxc_allocator::Box;
use oxc_ast::{ast::*, AstBuilder};
use oxc_diagnostics::Result;
use oxc_span::{GetSpan, Span};

use super::list::FormalParameterList;
use crate::{diagnostics, lexer::Kind, list::SeparatedList, Context, ParserImpl, StatementContext};

type ArrowFunctionHead<'a> = (
    Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    Box<'a, FormalParameters<'a>>,
    Option<Box<'a, TSTypeAnnotation<'a>>>,
    bool,
    Span,
);

#[derive(Debug, Clone, Copy)]
pub enum IsParenthesizedArrowFunction {
    True,
    False,
    Maybe,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FunctionKind {
    Declaration { single_statement: bool },
    Expression,
    DefaultExport,
    TSDeclaration,
}

impl FunctionKind {
    pub(crate) fn is_id_required(self) -> bool {
        matches!(self, Self::Declaration { single_statement: true })
    }

    pub(crate) fn is_expression(self) -> bool {
        self == Self::Expression
    }
}

impl<'a> ParserImpl<'a> {
    pub(crate) fn at_function_with_async(&mut self) -> bool {
        self.at(Kind::Function)
            || self.at(Kind::Async)
                && self.peek_at(Kind::Function)
                && !self.peek_token().is_on_new_line
    }

    pub(crate) fn at_async_no_new_line(&mut self) -> bool {
        self.at(Kind::Async) && !self.cur_token().escaped() && !self.peek_token().is_on_new_line
    }

    pub(crate) fn parse_function_body(&mut self) -> Result<Box<'a, FunctionBody<'a>>> {
        let span = self.start_span();
        self.expect(Kind::LCurly)?;

        let (directives, statements) = self.with_context(Context::Return, |p| {
            p.parse_directives_and_statements(/* is_top_level */ false)
        })?;

        self.expect(Kind::RCurly)?;
        Ok(self.ast.function_body(self.end_span(span), directives, statements))
    }

    pub(crate) fn parse_formal_parameters(
        &mut self,
        params_kind: FormalParameterKind,
    ) -> Result<(Option<TSThisParameter<'a>>, Box<'a, FormalParameters<'a>>)> {
        let span = self.start_span();
        let list: FormalParameterList<'_> = FormalParameterList::parse(self)?;
        let formal_parameters =
            self.ast.formal_parameters(self.end_span(span), params_kind, list.elements, list.rest);
        let this_param = list.this_param;
        Ok((this_param, formal_parameters))
    }

    pub(crate) fn parse_function(
        &mut self,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        r#async: bool,
        generator: bool,
        func_kind: FunctionKind,
        modifiers: Modifiers<'a>,
    ) -> Result<Box<'a, Function<'a>>> {
        let ctx = self.ctx;
        self.ctx = self.ctx.and_in(true).and_await(r#async).and_yield(generator);

        let type_parameters = self.parse_ts_type_parameters()?;

        let (this_param, params) =
            self.parse_formal_parameters(FormalParameterKind::FormalParameter)?;

        let return_type = self.parse_ts_return_type_annotation()?;

        let body = if self.at(Kind::LCurly) { Some(self.parse_function_body()?) } else { None };

        self.ctx =
            self.ctx.and_in(ctx.has_in()).and_await(ctx.has_await()).and_yield(ctx.has_yield());

        if !self.ts_enabled() && body.is_none() {
            return Err(self.unexpected());
        }

        let function_type = if body.is_none() {
            FunctionType::TSDeclareFunction
        } else {
            match func_kind {
                FunctionKind::Declaration { .. } | FunctionKind::DefaultExport => {
                    FunctionType::FunctionDeclaration
                }
                FunctionKind::Expression { .. } => FunctionType::FunctionExpression,
                FunctionKind::TSDeclaration { .. } => FunctionType::TSDeclareFunction,
            }
        };

        if FunctionType::TSDeclareFunction == function_type {
            self.asi()?;
        }

        Ok(self.ast.function(
            function_type,
            self.end_span(span),
            id,
            generator,
            r#async,
            this_param,
            params,
            body,
            type_parameters,
            return_type,
            modifiers,
        ))
    }

    /// [Function Declaration](https://tc39.es/ecma262/#prod-FunctionDeclaration)
    pub(crate) fn parse_function_declaration(
        &mut self,
        stmt_ctx: StatementContext,
    ) -> Result<Statement<'a>> {
        let func_kind =
            FunctionKind::Declaration { single_statement: stmt_ctx.is_single_statement() };
        let decl = self.parse_function_impl(func_kind)?;
        if stmt_ctx.is_single_statement() {
            if decl.r#async {
                self.error(diagnostics::AsyncFunctionDeclaration(Span::new(
                    decl.span.start,
                    decl.params.span.end,
                )));
            } else if decl.generator {
                self.error(diagnostics::GeneratorFunctionDeclaration(Span::new(
                    decl.span.start,
                    decl.params.span.end,
                )));
            }
        }

        Ok(self.ast.function_declaration(decl))
    }

    /// Parse function implementation in Javascript, cursor
    /// at `function` or `async function`
    pub(crate) fn parse_function_impl(
        &mut self,
        func_kind: FunctionKind,
    ) -> Result<Box<'a, Function<'a>>> {
        let span = self.start_span();
        let r#async = self.eat(Kind::Async);
        self.expect(Kind::Function)?;
        let generator = self.eat(Kind::Star);
        let id = self.parse_function_id(func_kind, r#async, generator);
        self.parse_function(span, id, r#async, generator, func_kind, Modifiers::empty())
    }

    /// Parse function implementation in Typescript, cursor
    /// at `function`
    pub(crate) fn parse_ts_function_impl(
        &mut self,
        start_span: Span,
        func_kind: FunctionKind,
        modifiers: Modifiers<'a>,
    ) -> Result<Box<'a, Function<'a>>> {
        let r#async = modifiers.contains(ModifierKind::Async);
        self.expect(Kind::Function)?;
        let generator = self.eat(Kind::Star);
        let id = self.parse_function_id(func_kind, r#async, generator);
        self.parse_function(start_span, id, r#async, generator, func_kind, modifiers)
    }

    /// [Function Expression](https://tc39.es/ecma262/#prod-FunctionExpression)
    pub(crate) fn parse_function_expression(
        &mut self,
        span: Span,
        r#async: bool,
    ) -> Result<Expression<'a>> {
        let func_kind = FunctionKind::Expression;
        self.expect(Kind::Function)?;

        let generator = self.eat(Kind::Star);
        let id = self.parse_function_id(func_kind, r#async, generator);
        let function =
            self.parse_function(span, id, r#async, generator, func_kind, Modifiers::empty())?;

        Ok(self.ast.function_expression(function))
    }

    pub(crate) fn parse_single_param_function_expression(
        &mut self,
        span: Span,
        r#async: bool,
        generator: bool,
    ) -> Result<Expression<'a>> {
        let has_await = self.ctx.has_await();
        let has_yield = self.ctx.has_yield();

        self.ctx = self.ctx.union_await_if(r#async).union_yield_if(generator);
        let params_span = self.start_span();
        let param = self.parse_binding_identifier()?;
        let ident = self.ast.binding_pattern_identifier(param);
        let pattern = self.ast.binding_pattern(ident, None, false);
        let params_span = self.end_span(params_span);
        let formal_parameter = self.ast.formal_parameter(
            params_span,
            pattern,
            None,
            false,
            false,
            AstBuilder::new_vec(&self.ast),
        );
        let params = self.ast.formal_parameters(
            params_span,
            FormalParameterKind::ArrowFormalParameters,
            self.ast.new_vec_single(formal_parameter),
            None,
        );

        self.expect(Kind::Arrow)?;

        self.ctx = self.ctx.and_await(r#async).and_yield(generator);
        let expression = !self.at(Kind::LCurly);
        let body = if expression {
            let expr = self.parse_assignment_expression_base()?;
            let span = expr.span();
            let expr_stmt = self.ast.expression_statement(span, expr);
            self.ast.function_body(span, self.ast.new_vec(), self.ast.new_vec_single(expr_stmt))
        } else {
            self.parse_function_body()?
        };
        self.ctx = self.ctx.and_await(has_await).and_yield(has_yield);

        Ok(self.ast.arrow_function_expression(
            self.end_span(span),
            expression,
            r#async,
            params,
            body,
            None,
            None,
        ))
    }

    /// Section 15.4 Method Definitions
    /// `ClassElementName` ( `UniqueFormalParameters` ) { `FunctionBody` }
    /// `GeneratorMethod`
    ///   * `ClassElementName`
    /// `AsyncMethod`
    ///   async `ClassElementName`
    /// `AsyncGeneratorMethod`
    ///   async * `ClassElementName`
    pub(crate) fn parse_method(
        &mut self,
        r#async: bool,
        generator: bool,
    ) -> Result<Box<'a, Function<'a>>> {
        let span = self.start_span();
        self.parse_function(
            span,
            None,
            r#async,
            generator,
            FunctionKind::Expression,
            Modifiers::empty(),
        )
    }

    /// Section 15.5 Yield Expression
    /// yield
    /// yield [no `LineTerminator` here] `AssignmentExpression`
    /// yield [no `LineTerminator` here] * `AssignmentExpression`
    pub(crate) fn parse_yield_expression(&mut self) -> Result<Expression<'a>> {
        let span = self.start_span();
        self.bump_any(); // advance `yield`

        let has_yield = self.ctx.has_yield();
        if !has_yield {
            self.error(diagnostics::YieldExpression(Span::new(span.start, span.start + 5)));
        }

        let mut delegate = false;
        let mut argument = None;

        if !self.cur_token().is_on_new_line {
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
                argument = Some(self.parse_assignment_expression_base()?);
                self.ctx = self.ctx.and_yield(has_yield);
            }
        }

        Ok(self.ast.yield_expression(self.end_span(span), delegate, argument))
    }

    // id: None - for AnonymousDefaultExportedFunctionDeclaration
    pub(crate) fn parse_function_id(
        &mut self,
        kind: FunctionKind,
        r#async: bool,
        generator: bool,
    ) -> Option<BindingIdentifier<'a>> {
        let ctx = self.ctx;
        if kind.is_expression() {
            self.ctx = self.ctx.and_await(r#async).and_yield(generator);
        }
        let id = self.cur_kind().is_binding_identifier().then(|| {
            let (span, name) = self.parse_identifier_kind(Kind::Ident);
            self.check_identifier(span, &name);
            BindingIdentifier { span, name, symbol_id: Cell::default() }
        });
        self.ctx = ctx;

        if kind.is_id_required() && id.is_none() {
            self.error(diagnostics::ExpectFunctionName(self.cur_token().span()));
        }

        id
    }

    pub(crate) fn is_parenthesized_arrow_function_expression(
        &mut self,
        r#async: bool,
    ) -> IsParenthesizedArrowFunction {
        let offset = u8::from(r#async);

        match self.nth_kind(offset) {
            Kind::LParen => match self.nth_kind(offset + 1) {
                // '()' is an arrow expression if followed by an '=>', a type annotation or body.
                // Otherwise, a parenthesized expression with a missing inner expression
                Kind::RParen => {
                    let kind = self.nth_kind(offset + 2);
                    if self.ts_enabled() && kind == Kind::Colon {
                        IsParenthesizedArrowFunction::Maybe
                    } else if matches!(kind, Kind::Arrow | Kind::LCurly) {
                        IsParenthesizedArrowFunction::True
                    } else {
                        IsParenthesizedArrowFunction::False
                    }
                }
                // Rest parameter
                // '(...ident' is not a parenthesized expression
                // '(...null' is a parenthesized expression
                Kind::Dot3 => match self.nth_kind(offset + 1) {
                    Kind::Ident => IsParenthesizedArrowFunction::True,
                    kind if kind.is_literal() => IsParenthesizedArrowFunction::False,
                    _ => IsParenthesizedArrowFunction::Maybe,
                },
                // '([ ...', '({ ... } can either be a parenthesized object or array expression or a destructing parameter
                Kind::LBrack | Kind::LCurly => IsParenthesizedArrowFunction::Maybe,
                _ if self.nth_kind(offset + 1).is_binding_identifier()
                    || self.nth_at(offset + 1, Kind::This) =>
                {
                    match self.nth_kind(offset + 2) {
                        // '(a: ' must be a type annotation
                        Kind::Colon => IsParenthesizedArrowFunction::True,
                        // * '(a = ': an initializer or a parenthesized assignment expression
                        // * '(a, ': separator to next parameter or a parenthesized sequence expression
                        // * '(a)': a single parameter OR a parenthesized expression
                        Kind::Eq | Kind::Comma | Kind::RParen => {
                            IsParenthesizedArrowFunction::Maybe
                        }
                        // '(a?:' | '(a?,' | '(a?=' | '(a?)'
                        Kind::Question
                            if matches!(
                                self.nth_kind(offset + 3),
                                Kind::Colon | Kind::Comma | Kind::Eq | Kind::RParen
                            ) =>
                        {
                            IsParenthesizedArrowFunction::True
                        }
                        _ => IsParenthesizedArrowFunction::False,
                    }
                }
                _ => IsParenthesizedArrowFunction::False,
            },
            Kind::LAngle => {
                let kind = self.nth_kind(offset + 1);

                // `<const` for const type parameter from TypeScript 5.0
                if kind == Kind::Const {
                    return IsParenthesizedArrowFunction::Maybe;
                }

                if !kind.is_identifier() {
                    return IsParenthesizedArrowFunction::False;
                }

                if self.source_type.is_jsx() {
                    return match self.nth_kind(offset + 2) {
                        Kind::Extends => {
                            let third_kind = self.nth_kind(offset + 3);
                            if matches!(third_kind, Kind::Eq | Kind::RAngle) {
                                IsParenthesizedArrowFunction::False
                            } else if third_kind.is_identifier() {
                                IsParenthesizedArrowFunction::Maybe
                            } else {
                                IsParenthesizedArrowFunction::True
                            }
                        }
                        Kind::Eq | Kind::Comma => IsParenthesizedArrowFunction::True,
                        _ => IsParenthesizedArrowFunction::False,
                    };
                }

                IsParenthesizedArrowFunction::Maybe
            }
            _ => IsParenthesizedArrowFunction::False,
        }
    }

    pub(crate) fn is_parenthesized_arrow_function(&mut self) -> IsParenthesizedArrowFunction {
        match self.cur_kind() {
            Kind::LAngle | Kind::LParen => self.is_parenthesized_arrow_function_expression(false),
            Kind::Async => {
                let peeked = self.peek_token();
                if !peeked.is_on_new_line && matches!(peeked.kind, Kind::LAngle | Kind::LParen) {
                    self.is_parenthesized_arrow_function_expression(true)
                } else {
                    IsParenthesizedArrowFunction::False
                }
            }
            _ => IsParenthesizedArrowFunction::False,
        }
    }

    pub(crate) fn parse_parenthesized_arrow_function_head(
        &mut self,
    ) -> Result<ArrowFunctionHead<'a>> {
        let span = self.start_span();
        let r#async = self.eat(Kind::Async);

        let has_await = self.ctx.has_await();
        self.ctx = self.ctx.union_await_if(r#async);

        let type_parameters = self.parse_ts_type_parameters()?;

        let (this_param, params) =
            self.parse_formal_parameters(FormalParameterKind::ArrowFormalParameters)?;

        if let Some(this_param) = this_param {
            // const x = (this: number) => {};
            self.error(diagnostics::TSArrowFunctionThisParameter(this_param.span));
        }

        let return_type = self.parse_ts_return_type_annotation()?;

        self.ctx = self.ctx.and_await(has_await);

        if self.cur_token().is_on_new_line {
            self.error(diagnostics::LineterminatorBeforeArrow(self.cur_token().span()));
        }

        self.expect(Kind::Arrow)?;

        Ok((type_parameters, params, return_type, r#async, span))
    }

    /// [ConciseBody](https://tc39.es/ecma262/#prod-ConciseBody)
    ///     [lookahead ≠ {] `ExpressionBody`[?In, ~Await]
    ///     { `FunctionBody`[~Yield, ~Await] }
    /// `ExpressionBody`[In, Await] :
    ///     `AssignmentExpression`[?In, ~Yield, ?Await]
    pub(crate) fn parse_arrow_function_body(
        &mut self,
        span: Span,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
        r#async: bool,
    ) -> Result<Expression<'a>> {
        let has_await = self.ctx.has_await();
        let has_yield = self.ctx.has_yield();
        self.ctx = self.ctx.and_await(r#async).and_yield(false);

        let expression = !self.at(Kind::LCurly);
        let body = if expression {
            let expr = self.parse_assignment_expression_base()?;
            let span = expr.span();
            let expr_stmt = self.ast.expression_statement(span, expr);
            self.ast.function_body(span, self.ast.new_vec(), self.ast.new_vec_single(expr_stmt))
        } else {
            self.parse_function_body()?
        };

        self.ctx = self.ctx.and_await(has_await).and_yield(has_yield);

        Ok(self.ast.arrow_function_expression(
            self.end_span(span),
            expression,
            r#async,
            params,
            body,
            type_parameters,
            return_type,
        ))
    }

    /// Section [Arrow Function](https://tc39.es/ecma262/#sec-arrow-function-definitions)
    /// `ArrowFunction`[In, Yield, Await] :
    ///     `ArrowParameters`[?Yield, ?Await] [no `LineTerminator` here] => `ConciseBody`[?In]
    pub(crate) fn parse_parenthesized_arrow_function(&mut self) -> Result<Expression<'a>> {
        let (type_parameters, params, return_type, r#async, span) =
            self.parse_parenthesized_arrow_function_head()?;
        self.parse_arrow_function_body(span, type_parameters, params, return_type, r#async)
    }
}
