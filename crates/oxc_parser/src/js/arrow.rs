use oxc_allocator::Box;
use oxc_ast::ast::*;
use oxc_diagnostics::Result;
use oxc_span::{GetSpan, Span};

use crate::{diagnostics, lexer::Kind, AstBuilder, ParserImpl};

use super::Tristate;

type ArrowFunctionHead<'a> = (
    Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    Box<'a, FormalParameters<'a>>,
    Option<Box<'a, TSTypeAnnotation<'a>>>,
    bool,
    Span,
);

impl<'a> ParserImpl<'a> {
    pub(crate) fn is_parenthesized_arrow_function_expression(&mut self, r#async: bool) -> Tristate {
        let offset = u8::from(r#async);

        match self.nth_kind(offset) {
            Kind::LParen => match self.nth_kind(offset + 1) {
                // '()' is an arrow expression if followed by an '=>', a type annotation or body.
                // Otherwise, a parenthesized expression with a missing inner expression
                Kind::RParen => {
                    let kind = self.nth_kind(offset + 2);
                    if self.ts_enabled() && kind == Kind::Colon {
                        Tristate::Maybe
                    } else if matches!(kind, Kind::Arrow | Kind::LCurly) {
                        Tristate::True
                    } else {
                        Tristate::False
                    }
                }
                // Rest parameter
                // '(...ident' is not a parenthesized expression
                // '(...null' is a parenthesized expression
                Kind::Dot3 => match self.nth_kind(offset + 1) {
                    Kind::Ident => Tristate::True,
                    kind if kind.is_literal() => Tristate::False,
                    _ => Tristate::Maybe,
                },
                // '([ ...', '({ ... } can either be a parenthesized object or array expression or a destructing parameter
                Kind::LBrack | Kind::LCurly => Tristate::Maybe,
                _ if self.nth_kind(offset + 1).is_binding_identifier()
                    || self.nth_at(offset + 1, Kind::This) =>
                {
                    match self.nth_kind(offset + 2) {
                        // '(a: ' must be a type annotation
                        Kind::Colon => Tristate::True,
                        // * '(a = ': an initializer or a parenthesized assignment expression
                        // * '(a, ': separator to next parameter or a parenthesized sequence expression
                        // * '(a)': a single parameter OR a parenthesized expression
                        Kind::Eq | Kind::Comma | Kind::RParen => Tristate::Maybe,
                        // '(a?:' | '(a?,' | '(a?=' | '(a?)'
                        Kind::Question
                            if matches!(
                                self.nth_kind(offset + 3),
                                Kind::Colon | Kind::Comma | Kind::Eq | Kind::RParen
                            ) =>
                        {
                            Tristate::True
                        }
                        _ => Tristate::False,
                    }
                }
                _ => Tristate::False,
            },
            Kind::LAngle => {
                let kind = self.nth_kind(offset + 1);

                // `<const` for const type parameter from TypeScript 5.0
                if kind == Kind::Const {
                    return Tristate::Maybe;
                }

                if !kind.is_identifier() {
                    return Tristate::False;
                }

                if self.source_type.is_jsx() {
                    return match self.nth_kind(offset + 2) {
                        Kind::Extends => {
                            let third_kind = self.nth_kind(offset + 3);
                            if matches!(third_kind, Kind::Eq | Kind::RAngle) {
                                Tristate::False
                            } else if third_kind.is_identifier() {
                                Tristate::Maybe
                            } else {
                                Tristate::True
                            }
                        }
                        Kind::Eq | Kind::Comma => Tristate::True,
                        _ => Tristate::False,
                    };
                }

                Tristate::Maybe
            }
            _ => Tristate::False,
        }
    }

    pub(crate) fn is_parenthesized_arrow_function(&mut self) -> Tristate {
        match self.cur_kind() {
            Kind::LAngle | Kind::LParen => self.is_parenthesized_arrow_function_expression(false),
            Kind::Async => {
                let peeked = self.peek_token();
                if !peeked.is_on_new_line && matches!(peeked.kind, Kind::LAngle | Kind::LParen) {
                    self.is_parenthesized_arrow_function_expression(true)
                } else {
                    Tristate::False
                }
            }
            _ => Tristate::False,
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
            self.error(diagnostics::ts_arrow_function_this_parameter(this_param.span));
        }

        let return_type = self.parse_ts_return_type_annotation()?;

        self.ctx = self.ctx.and_await(has_await);

        if self.cur_token().is_on_new_line {
            self.error(diagnostics::lineterminator_before_arrow(self.cur_token().span()));
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
            let expr = self.parse_assignment_expression_or_higher()?;
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
    pub(crate) fn parse_parenthesized_arrow_function(&mut self) -> Result<Option<Expression<'a>>> {
        let (type_parameters, params, return_type, r#async, span) =
            self.parse_parenthesized_arrow_function_head()?;
        self.parse_arrow_function_body(span, type_parameters, params, return_type, r#async)
            .map(Some)
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
            let expr = self.parse_assignment_expression_or_higher()?;
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

    pub(crate) fn parse_possible_parenthesized_arrow_function_expression(
        &mut self,
    ) -> Result<Option<Expression<'a>>> {
        let pos = self.cur_token().start;
        if self.state.not_parenthesized_arrow.contains(&pos) {
            return Ok(None);
        }
        if let Ok((type_parameters, params, return_type, r#async, span)) =
            self.try_parse(ParserImpl::parse_parenthesized_arrow_function_head)
        {
            return self
                .parse_arrow_function_body(span, type_parameters, params, return_type, r#async)
                .map(Some);
        }
        self.state.not_parenthesized_arrow.insert(pos);
        Ok(None)
    }

    pub(crate) fn try_parse_parenthesized_arrow_function_expression(
        &mut self,
    ) -> Result<Option<Expression<'a>>> {
        match self.is_parenthesized_arrow_function() {
            Tristate::False => Ok(None),
            Tristate::True => self.parse_parenthesized_arrow_function(),
            Tristate::Maybe => self.parse_possible_parenthesized_arrow_function_expression(),
        }
    }

    pub(crate) fn try_parse_async_simple_arrow_function_expression(
        &mut self,
    ) -> Result<Option<Expression<'a>>> {
        let span = self.start_span();
        if self.cur_kind().is_binding_identifier()
            && self.peek_at(Kind::Arrow)
            && !self.peek_token().is_on_new_line
        {
            self.parse_single_param_function_expression(span, false, false).map(Some)
        } else if self.at_async_no_new_line()
            && self.peek_kind().is_binding_identifier()
            && !self.peek_token().is_on_new_line
            && self.nth_at(2, Kind::Arrow)
        {
            self.bump_any(); // bump async
            let arrow_token = self.peek_token();
            if arrow_token.is_on_new_line {
                self.error(diagnostics::no_line_break_is_allowed_before_arrow(arrow_token.span()));
            }
            self.parse_single_param_function_expression(span, true, false).map(Some)
        } else {
            Ok(None)
        }
    }
}
