use oxc_allocator::Box;
use oxc_ast::{NONE, ast::*};
use oxc_diagnostics::Result;
use oxc_span::{GetSpan, Span};
use oxc_syntax::precedence::Precedence;

use super::Tristate;
use crate::{ParserImpl, diagnostics, lexer::Kind};

struct ArrowFunctionHead<'a> {
    type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    params: Box<'a, FormalParameters<'a>>,
    return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    r#async: bool,
    span: Span,
    has_return_colon: bool,
}

impl<'a> ParserImpl<'a> {
    pub(super) fn try_parse_parenthesized_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Result<Option<Expression<'a>>> {
        match self.is_parenthesized_arrow_function_expression() {
            Tristate::False => Ok(None),
            Tristate::True => self.parse_parenthesized_arrow_function_expression(
                /* allow_return_type_in_arrow_function */ true,
            ),
            Tristate::Maybe => self.parse_possible_parenthesized_arrow_function_expression(
                allow_return_type_in_arrow_function,
            ),
        }
    }

    pub(super) fn try_parse_async_simple_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Result<Option<Expression<'a>>> {
        if self.at(Kind::Async)
            && self.is_un_parenthesized_async_arrow_function_worker() == Tristate::True
        {
            let span = self.start_span();
            self.bump_any(); // bump `async`
            let expr = self.parse_binary_expression_or_higher(Precedence::Comma)?;
            return self
                .parse_simple_arrow_function_expression(
                    span,
                    expr,
                    /* async */ true,
                    allow_return_type_in_arrow_function,
                )
                .map(Some);
        }
        Ok(None)
    }

    fn is_parenthesized_arrow_function_expression(&mut self) -> Tristate {
        match self.cur_kind() {
            Kind::LParen | Kind::LAngle | Kind::Async => {
                self.lookahead(Self::is_parenthesized_arrow_function_expression_worker)
            }
            _ => Tristate::False,
        }
    }

    fn is_parenthesized_arrow_function_expression_worker(&mut self) -> Tristate {
        let mut offset = 0;

        if self.at(Kind::Async) {
            let second_token = self.peek_token();
            let second = second_token.kind;
            if second_token.is_on_new_line {
                return Tristate::False;
            }
            if second != Kind::LParen && second != Kind::LAngle {
                return Tristate::False;
            }
            offset = 1;
        }

        let first = self.nth_kind(offset);
        let second = self.nth_kind(offset + 1);

        match first {
            Kind::LParen => {
                match second {
                    // Simple cases: "() =>", "(): ", and "() {".
                    // This is an arrow function with no parameters.
                    // The last one is not actually an arrow function,
                    // but this is probably what the user intended.
                    Kind::RParen => {
                        let third = self.nth_kind(offset + 2);
                        return match third {
                            Kind::Colon if self.is_ts => Tristate::Maybe,
                            Kind::Arrow | Kind::LCurly => Tristate::True,
                            _ => Tristate::False,
                        };
                    }
                    // If encounter "([" or "({", this could be the start of a binding pattern.
                    // Examples:
                    //      ([ x ]) => { }
                    //      ({ x }) => { }
                    //      ([ x ])
                    //      ({ x })
                    Kind::LBrack | Kind::LCurly => {
                        return Tristate::Maybe;
                    }
                    // Simple case: "(..."
                    // This is an arrow function with a rest parameter.
                    Kind::Dot3 => {
                        return match self.nth_kind(offset + 1) {
                            // '(...ident' is a lambda
                            Kind::Ident => Tristate::True,
                            // '(...null' is not a lambda
                            kind if kind.is_literal() => Tristate::False,
                            _ => Tristate::Maybe,
                        };
                    }
                    _ => {}
                }
                let third = self.nth_kind(offset + 2);

                // Check for "(xxx yyy", where xxx is a modifier and yyy is an identifier. This
                // isn't actually allowed, but we want to treat it as a lambda so we can provide
                // a good error message.
                if second.is_modifier_kind()
                    && second != Kind::Async
                    && third.is_binding_identifier()
                {
                    if third == Kind::As {
                        return Tristate::False; // https://github.com/microsoft/TypeScript/issues/44466
                    }
                    return Tristate::True;
                }

                // If we had "(" followed by something that's not an identifier,
                // then this definitely doesn't look like a lambda.  "this" is not
                // valid, but we want to parse it and then give a semantic error.
                if !second.is_binding_identifier() && second != Kind::This {
                    return Tristate::False;
                }

                match third {
                    // If we have something like "(a:", then we must have a
                    // type-annotated parameter in an arrow function expression.
                    Kind::Colon => Tristate::True,
                    // If we have "(a?:" or "(a?," or "(a?=" or "(a?)" then it is definitely a lambda.
                    Kind::Question => {
                        let fourth = self.nth_kind(offset + 3);
                        if matches!(fourth, Kind::Colon | Kind::Comma | Kind::Eq | Kind::RParen) {
                            return Tristate::True;
                        }
                        Tristate::False
                    }
                    // If we have "(a," or "(a=" or "(a)" this *could* be an arrow function
                    Kind::Comma | Kind::Eq | Kind::RParen => Tristate::Maybe,
                    // It is definitely not an arrow function
                    _ => Tristate::False,
                }
            }
            Kind::LAngle => {
                // If we have "<" not followed by an identifier,
                // then this definitely is not an arrow function.
                if !second.is_binding_identifier() && second != Kind::Const {
                    return Tristate::False;
                }

                // JSX overrides
                if self.source_type.is_jsx() {
                    // <const Ident extends Ident>
                    //  ^^^^^ Optional
                    offset += if second == Kind::Const { 3 } else { 2 };
                    return match self.nth_kind(offset) {
                        Kind::Extends => {
                            let third = self.nth_kind(offset + 1);
                            if matches!(third, Kind::Eq | Kind::RAngle | Kind::Slash) {
                                Tristate::False
                            } else if third.is_binding_identifier() {
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
            _ => unreachable!(),
        }
    }

    fn is_un_parenthesized_async_arrow_function_worker(&mut self) -> Tristate {
        if self.at(Kind::Async) {
            let first_token = self.peek_token();
            let first = first_token.kind;
            // If the "async" is followed by "=>" token then it is not a beginning of an async arrow-function
            // but instead a simple arrow-function which will be parsed inside "parseAssignmentExpressionOrHigher"
            if first_token.is_on_new_line || first == Kind::Arrow {
                return Tristate::False;
            }
            // Check for un-parenthesized AsyncArrowFunction
            if first.is_binding_identifier() {
                // Arrow before newline is checked in `parse_simple_arrow_function_expression`
                if self.nth_at(2, Kind::Arrow) {
                    return Tristate::True;
                }
            }
        }
        Tristate::False
    }

    pub(crate) fn parse_simple_arrow_function_expression(
        &mut self,
        span: Span,
        ident: Expression<'a>,
        r#async: bool,
        allow_return_type_in_arrow_function: bool,
    ) -> Result<Expression<'a>> {
        let has_await = self.ctx.has_await();
        self.ctx = self.ctx.union_await_if(r#async);

        let params = {
            let ident = match ident {
                Expression::Identifier(ident) => {
                    let ident = ident.unbox();
                    self.ast.alloc_binding_identifier(ident.span, ident.name)
                }
                _ => unreachable!(),
            };
            let params_span = self.end_span(ident.span);
            let ident = BindingPatternKind::BindingIdentifier(ident);
            let pattern = self.ast.binding_pattern(ident, NONE, false);
            let formal_parameter = self.ast.plain_formal_parameter(params_span, pattern);
            self.ast.alloc_formal_parameters(
                params_span,
                FormalParameterKind::ArrowFormalParameters,
                self.ast.vec1(formal_parameter),
                NONE,
            )
        };

        self.ctx = self.ctx.and_await(has_await);

        if self.cur_token().is_on_new_line {
            self.error(diagnostics::lineterminator_before_arrow(self.cur_token().span()));
        }

        self.expect(Kind::Arrow)?;

        self.parse_arrow_function_expression_body(
            ArrowFunctionHead {
                type_parameters: None,
                params,
                return_type: None,
                r#async,
                span,
                has_return_colon: false,
            },
            allow_return_type_in_arrow_function,
        )
    }

    fn parse_parenthesized_arrow_function_head(&mut self) -> Result<ArrowFunctionHead<'a>> {
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

        let has_return_colon = self.is_ts && self.at(Kind::Colon);
        let return_type = self.parse_ts_return_type_annotation(Kind::Arrow, false)?;

        self.ctx = self.ctx.and_await(has_await);

        if self.cur_token().is_on_new_line {
            self.error(diagnostics::lineterminator_before_arrow(self.cur_token().span()));
        }

        self.expect(Kind::Arrow)?;

        Ok(ArrowFunctionHead {
            type_parameters,
            params,
            return_type,
            r#async,
            span,
            has_return_colon,
        })
    }

    /// [ConciseBody](https://tc39.es/ecma262/#prod-ConciseBody)
    ///     [lookahead ≠ {] `ExpressionBody`[?In, ~Await]
    ///     { `FunctionBody`[~Yield, ~Await] }
    /// `ExpressionBody`[In, Await] :
    ///     `AssignmentExpression`[?In, ~Yield, ?Await]
    fn parse_arrow_function_expression_body(
        &mut self,
        arrow_function_head: ArrowFunctionHead<'a>,
        allow_return_type_in_arrow_function: bool,
    ) -> Result<Expression<'a>> {
        let ArrowFunctionHead { type_parameters, params, return_type, r#async, span, .. } =
            arrow_function_head;
        let has_await = self.ctx.has_await();
        let has_yield = self.ctx.has_yield();
        self.ctx = self.ctx.and_await(r#async).and_yield(false);

        let expression = !self.at(Kind::LCurly);
        let body = if expression {
            let expr = self
                .parse_assignment_expression_or_higher_impl(allow_return_type_in_arrow_function)?;
            let span = expr.span();
            let expr_stmt = self.ast.statement_expression(span, expr);
            self.ast.alloc_function_body(span, self.ast.vec(), self.ast.vec1(expr_stmt))
        } else {
            self.parse_function_body()?
        };

        self.ctx = self.ctx.and_await(has_await).and_yield(has_yield);

        Ok(self.ast.expression_arrow_function(
            self.end_span(span),
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
        ))
    }

    /// Section [Arrow Function](https://tc39.es/ecma262/#sec-arrow-function-definitions)
    /// `ArrowFunction`[In, Yield, Await] :
    ///     `ArrowParameters`[?Yield, ?Await] [no `LineTerminator` here] => `ConciseBody`[?In]
    fn parse_parenthesized_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Result<Option<Expression<'a>>> {
        let head = self.parse_parenthesized_arrow_function_head()?;
        self.parse_arrow_function_expression_body(head, allow_return_type_in_arrow_function)
            .map(Some)
    }

    fn parse_possible_parenthesized_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Result<Option<Expression<'a>>> {
        let pos = self.cur_token().start;
        if self.state.not_parenthesized_arrow.contains(&pos) {
            return Ok(None);
        }

        let checkpoint = self.checkpoint();

        let Ok(head) = self.parse_parenthesized_arrow_function_head() else {
            self.state.not_parenthesized_arrow.insert(pos);
            self.rewind(checkpoint);
            return Ok(None);
        };

        let has_return_colon = head.has_return_colon;

        let body =
            self.parse_arrow_function_expression_body(head, allow_return_type_in_arrow_function)?;

        // Given:
        //     x ? y => ({ y }) : z => ({ z })
        // We try to parse the body of the first arrow function by looking at:
        //     ({ y }) : z => ({ z })
        // This is a valid arrow function with "z" as the return type.
        //
        // But, if we're in the true side of a conditional expression, this colon
        // terminates the expression, so we cannot allow a return type if we aren't
        // certain whether or not the preceding text was parsed as a parameter list.
        //
        // For example,
        //     a() ? (b: number, c?: string): void => d() : e
        // is determined by isParenthesizedArrowFunctionExpression to unambiguously
        // be an arrow expression, so we allow a return type.
        if !allow_return_type_in_arrow_function && has_return_colon {
            // However, if the arrow function we were able to parse is followed by another colon
            // as in:
            //     a ? (x): string => x : null
            // Then allow the arrow function, and treat the second colon as terminating
            // the conditional expression. It's okay to do this because this code would
            // be a syntax error in JavaScript (as the second colon shouldn't be there).

            if !self.at(Kind::Colon) {
                self.state.not_parenthesized_arrow.insert(pos);
                self.rewind(checkpoint);
                return Ok(None);
            }
        }

        Ok(Some(body))
    }
}
