use oxc_allocator::Box;
use oxc_ast::{NONE, ast::*};
use oxc_span::GetSpan;
use oxc_syntax::precedence::Precedence;

use super::Tristate;
use crate::{ParserImpl, diagnostics, lexer::Kind};

struct ArrowFunctionHead<'a> {
    type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    params: Box<'a, FormalParameters<'a>>,
    return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    r#async: bool,
    span: u32,
    has_return_colon: bool,
}

impl<'a> ParserImpl<'a> {
    pub(super) fn try_parse_parenthesized_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<Expression<'a>> {
        match self.is_parenthesized_arrow_function_expression() {
            Tristate::False => None,
            Tristate::True => Some(self.parse_parenthesized_arrow_function_expression(
                /* allow_return_type_in_arrow_function */ true,
            )),
            Tristate::Maybe => self.parse_possible_parenthesized_arrow_function_expression(
                allow_return_type_in_arrow_function,
            ),
        }
    }

    pub(super) fn try_parse_async_simple_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<Expression<'a>> {
        if self.at(Kind::Async) && self.is_un_parenthesized_async_arrow_function_worker() {
            let span = self.start_span();
            self.bump_any(); // bump `async`
            let expr = self.parse_binary_expression_or_higher(Precedence::Comma);
            return Some(self.parse_simple_arrow_function_expression(
                span,
                expr,
                /* async */ true,
                allow_return_type_in_arrow_function,
            ));
        }
        None
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
        if self.eat(Kind::Async) {
            if self.cur_token().is_on_new_line() {
                return Tristate::False;
            }
            if !self.at(Kind::LParen) && !self.at(Kind::LAngle) {
                return Tristate::False;
            }
        }

        let first = self.cur_kind();
        self.bump_any();
        let second = self.cur_kind();

        match first {
            Kind::LParen => {
                match second {
                    // Simple cases: "() =>", "(): ", and "() {".
                    // This is an arrow function with no parameters.
                    // The last one is not actually an arrow function,
                    // but this is probably what the user intended.
                    Kind::RParen => {
                        self.bump_any();
                        let third = self.cur_kind();
                        match third {
                            Kind::Colon if self.is_ts => Tristate::Maybe,
                            Kind::Arrow | Kind::LCurly => Tristate::True,
                            _ => Tristate::False,
                        }
                    }
                    // If encounter "([" or "({", this could be the start of a binding pattern.
                    // Examples:
                    //      ([ x ]) => { }
                    //      ({ x }) => { }
                    //      ([ x ])
                    //      ({ x })
                    Kind::LBrack | Kind::LCurly => Tristate::Maybe,
                    // Simple case: "(..."
                    // This is an arrow function with a rest parameter.
                    Kind::Dot3 => {
                        self.bump_any();
                        let third = self.cur_kind();
                        match third {
                            // '(...ident' is a lambda
                            Kind::Ident => Tristate::True,
                            // '(...null' is not a lambda
                            kind if kind.is_literal() => Tristate::False,
                            _ => Tristate::Maybe,
                        }
                    }
                    _ => {
                        self.bump_any();
                        let third = self.cur_kind();

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
                                self.bump_any();
                                let fourth = self.cur_kind();
                                if matches!(
                                    fourth,
                                    Kind::Colon | Kind::Comma | Kind::Eq | Kind::RParen
                                ) {
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
                    self.bump(Kind::Const);
                    self.bump_any();
                    let third = self.cur_kind();
                    return match third {
                        Kind::Extends => {
                            self.bump_any();
                            let fourth = self.cur_kind();
                            if matches!(fourth, Kind::Eq | Kind::RAngle | Kind::Slash) {
                                Tristate::False
                            } else if fourth.is_binding_identifier() {
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

    fn is_un_parenthesized_async_arrow_function_worker(&mut self) -> bool {
        let checkpoint = self.checkpoint();
        self.bump(Kind::Async);
        // If the "async" is followed by "=>" token then it is not a beginning of an async arrow-function
        // but instead a simple arrow-function which will be parsed inside "parseAssignmentExpressionOrHigher"
        if !self.cur_token().is_on_new_line() && self.cur_kind().is_binding_identifier() {
            // Arrow before newline is checked in `parse_simple_arrow_function_expression`
            self.bump_any();
            if self.at(Kind::Arrow) {
                self.rewind(checkpoint);
                return true;
            }
        }
        self.rewind(checkpoint);
        false
    }

    pub(crate) fn parse_simple_arrow_function_expression(
        &mut self,
        span: u32,
        ident: Expression<'a>,
        r#async: bool,
        allow_return_type_in_arrow_function: bool,
    ) -> Expression<'a> {
        let has_await = self.ctx.has_await();
        self.ctx = self.ctx.union_await_if(r#async);

        let params = {
            let ident = match ident {
                Expression::Identifier(ident) => {
                    let ident = ident.unbox();
                    self.ast.alloc_binding_identifier(ident.span, ident.name)
                }
                _ => return self.unexpected(),
            };
            let params_span = self.end_span(ident.span.start);
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

        if self.cur_token().is_on_new_line() {
            self.error(diagnostics::lineterminator_before_arrow(self.cur_token().span()));
        }

        self.expect(Kind::Arrow);

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

    fn parse_parenthesized_arrow_function_head(&mut self) -> ArrowFunctionHead<'a> {
        let span = self.start_span();
        let r#async = self.eat(Kind::Async);

        let has_await = self.ctx.has_await();
        self.ctx = self.ctx.union_await_if(r#async);

        let type_parameters = self.parse_ts_type_parameters();

        let (this_param, params) =
            self.parse_formal_parameters(FormalParameterKind::ArrowFormalParameters);

        if let Some(this_param) = this_param {
            // const x = (this: number) => {};
            self.error(diagnostics::ts_arrow_function_this_parameter(this_param.span));
        }

        let has_return_colon = self.is_ts && self.at(Kind::Colon);
        let return_type = self.parse_ts_return_type_annotation(Kind::Arrow, false);

        self.ctx = self.ctx.and_await(has_await);

        if self.cur_token().is_on_new_line() {
            self.error(diagnostics::lineterminator_before_arrow(self.cur_token().span()));
        }

        self.expect(Kind::Arrow);

        ArrowFunctionHead { type_parameters, params, return_type, r#async, span, has_return_colon }
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
    ) -> Expression<'a> {
        let ArrowFunctionHead { type_parameters, params, return_type, r#async, span, .. } =
            arrow_function_head;
        let has_await = self.ctx.has_await();
        let has_yield = self.ctx.has_yield();
        self.ctx = self.ctx.and_await(r#async).and_yield(false);

        let expression = !self.at(Kind::LCurly);
        let body = if expression {
            let expr = self
                .parse_assignment_expression_or_higher_impl(allow_return_type_in_arrow_function);
            let span = expr.span();
            let expr_stmt = self.ast.statement_expression(span, expr);
            self.ast.alloc_function_body(span, self.ast.vec(), self.ast.vec1(expr_stmt))
        } else {
            self.parse_function_body()
        };

        self.ctx = self.ctx.and_await(has_await).and_yield(has_yield);

        self.ast.expression_arrow_function(
            self.end_span(span),
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
        )
    }

    /// Section [Arrow Function](https://tc39.es/ecma262/#sec-arrow-function-definitions)
    /// `ArrowFunction`[In, Yield, Await] :
    ///     `ArrowParameters`[?Yield, ?Await] [no `LineTerminator` here] => `ConciseBody`[?In]
    fn parse_parenthesized_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Expression<'a> {
        let head = self.parse_parenthesized_arrow_function_head();
        self.parse_arrow_function_expression_body(head, allow_return_type_in_arrow_function)
    }

    fn parse_possible_parenthesized_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<Expression<'a>> {
        let pos = self.cur_token().start();
        if self.state.not_parenthesized_arrow.contains(&pos) {
            return None;
        }

        let checkpoint = self.checkpoint();

        let head = self.parse_parenthesized_arrow_function_head();
        if self.has_fatal_error() {
            self.state.not_parenthesized_arrow.insert(pos);
            self.rewind(checkpoint);
            return None;
        }

        let has_return_colon = head.has_return_colon;

        let body =
            self.parse_arrow_function_expression_body(head, allow_return_type_in_arrow_function);

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
                return None;
            }
        }

        Some(body)
    }
}
