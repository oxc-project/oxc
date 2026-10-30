use oxc_allocator::ArenaBox;
use oxc_ast::ast::*;
use oxc_span::FileExtension;
use oxc_syntax::precedence::Precedence;

use super::{FunctionKind, Tristate};
use crate::{Context, ParserConfig as Config, ParserImpl, diagnostics, lexer::Kind};

pub(super) enum ArrowOrExpression<'a> {
    Arrow(Expression<'a>),
    Expression(Expression<'a>),
}

struct ArrowFunctionHead<'a> {
    type_parameters: Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>,
    params: ArenaBox<'a, FormalParameters<'a>>,
    return_type: Option<ArenaBox<'a, TSTypeAnnotation<'a>>>,
    r#async: bool,
    start: u32,
}

impl<'a, C: Config> ParserImpl<'a, C> {
    pub(super) fn try_parse_parenthesized_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<ArrowOrExpression<'a>> {
        // A simple async generic prefix is shared by a call and an arrow. Intercept it
        // before JSX lookahead commits to either interpretation.
        if let Some(result) =
            self.parse_cover_generic_async_arrow(allow_return_type_in_arrow_function)
        {
            return Some(result);
        }
        match self.is_parenthesized_arrow_function_expression() {
            Tristate::False => None,
            Tristate::True => {
                Some(ArrowOrExpression::Arrow(self.parse_parenthesized_arrow_function_expression(
                    allow_return_type_in_arrow_function,
                )))
            }
            Tristate::Maybe => {
                if let Some(result) =
                    self.parse_cover_parenthesized_arrow(allow_return_type_in_arrow_function)
                {
                    return Some(result);
                }
                self.parse_possible_parenthesized_arrow_function_expression(
                    allow_return_type_in_arrow_function,
                )
                .map(ArrowOrExpression::Arrow)
            }
        }
    }

    pub(super) fn try_parse_async_simple_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<Expression<'a>> {
        if self.at(Kind::Async) && self.is_un_parenthesized_async_arrow_function_worker() {
            let start = self.cur_start();
            self.bump_any(); // bump `async`
            // Lookahead established a single binding identifier followed by `=>`.
            // Build the binding directly instead of allocating a temporary identifier reference.
            let ident = if self.at(Kind::Ident) {
                self.parse_plain_binding_identifier()
            } else if self.cur_kind().is_identifier() {
                self.parse_known_binding_identifier()
            } else {
                let expr = self.parse_binary_expression_or_higher(Precedence::Comma);
                let Expression::Identifier(ident) = &expr else {
                    return self.unexpected();
                };
                BindingIdentifier::new(ident.span, ident.name, self)
            };
            // It is a Syntax Error if ArrowParameters Contains AwaitExpression is true.
            if ident.name == "await" {
                self.error(diagnostics::identifier_async("await", ident.span));
            }
            return Some(self.parse_simple_arrow_function_expression(
                start,
                ident,
                /* async */ true,
                allow_return_type_in_arrow_function,
            ));
        }
        None
    }

    fn is_parenthesized_arrow_function_expression(&mut self) -> Tristate {
        match self.cur_kind() {
            Kind::LParen => {
                // This check only reads lexer tokens. Keep one checkpoint for both the
                // literal fast path and the remaining lookahead, so the first token is read once.
                let checkpoint = self.lexer.checkpoint();
                let second = self.lexer.next_token().kind();
                let result = if second.is_literal() {
                    Tristate::False
                } else {
                    self.is_parenthesized_arrow_function_expression_after_lparen(second, false)
                };
                self.lexer.rewind(checkpoint);
                result
            }
            Kind::LAngle | Kind::Async => {
                self.lookahead(Self::is_parenthesized_arrow_function_expression_worker)
            }
            _ => Tristate::False,
        }
    }

    // `second` follows `(` and the lexer is positioned after it. Only lexer state changes;
    // callers must rewind their checkpoint before parsing the expression.
    fn is_parenthesized_arrow_function_expression_after_lparen(
        &mut self,
        second: Kind,
        is_async: bool,
    ) -> Tristate {
        match second {
            // Simple cases: "() =>", "(): ", and "() {".
            // This is an arrow function with no parameters.
            // The last one is not actually an arrow function,
            // but this is probably what the user intended.
            Kind::RParen => {
                let third = self.lexer.next_token().kind();
                match third {
                    // `async()` can be a call on the true side of a conditional.
                    Kind::Colon if self.is_ts && is_async => Tristate::Maybe,
                    // Bare `()` cannot be a parenthesized expression, so the colon
                    // must start a return type, even inside a conditional expression.
                    Kind::Colon if self.is_ts => Tristate::True,
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
                let third = self.lexer.next_token().kind();
                match third {
                    // '(...ident' is a lambda
                    Kind::Ident => Tristate::True,
                    // '(...null' is not a lambda
                    kind if kind.is_literal() => Tristate::False,
                    _ => Tristate::Maybe,
                }
            }
            _ => {
                let third = self.lexer.next_token().kind();

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
                        let fourth = self.lexer.next_token().kind();
                        if matches!(fourth, Kind::Colon | Kind::Comma | Kind::Eq | Kind::RParen) {
                            return Tristate::True;
                        }
                        Tristate::False
                    }
                    // A single identifier needs no speculative parameter parse. Keep `await`,
                    // `yield`, and `this` on the existing contextual/error-recovery paths.
                    // A TS return type can still be ambiguous with a conditional's colon.
                    Kind::RParen if second.is_identifier() => {
                        match self.lexer.next_token().kind() {
                            Kind::Arrow => Tristate::True,
                            Kind::Colon if self.is_ts => Tristate::Maybe,
                            _ => Tristate::False,
                        }
                    }
                    // If we have "(a," or "(a=" or a context-sensitive keyword in "(a)",
                    // this *could* be an arrow function.
                    Kind::Comma | Kind::Eq | Kind::RParen => Tristate::Maybe,
                    // It is definitely not an arrow function
                    _ => Tristate::False,
                }
            }
        }
    }

    fn is_parenthesized_arrow_function_expression_worker(&mut self) -> Tristate {
        let is_async = self.eat(Kind::Async);
        if is_async {
            if self.cur_token().is_on_new_line() {
                return Tristate::False;
            }
            let kind = self.cur_kind();
            if kind != Kind::LParen && kind != Kind::LAngle {
                return Tristate::False;
            }
        }

        let first = self.cur_kind();
        self.bump_any();
        let second = self.cur_kind();

        match first {
            Kind::LParen => {
                self.is_parenthesized_arrow_function_expression_after_lparen(second, is_async)
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
        // Use lookahead to avoid checkpoint/rewind
        self.lookahead(|parser| {
            parser.bump(Kind::Async);
            // If the "async" is followed by "=>" token then it is not a beginning of an async arrow-function
            // but instead a simple arrow-function which will be parsed inside "parseAssignmentExpressionOrHigher"
            if !parser.cur_token().is_on_new_line() && parser.cur_kind().is_binding_identifier() {
                // Arrow before newline is checked in `parse_simple_arrow_function_expression`
                parser.bump_any();
                parser.at(Kind::Arrow)
            } else {
                false
            }
        })
    }

    pub(crate) fn parse_simple_arrow_function_expression(
        &mut self,
        start: u32,
        ident: BindingIdentifier<'a>,
        r#async: bool,
        allow_return_type_in_arrow_function: bool,
    ) -> Expression<'a> {
        let span = ident.span;
        let pattern = BindingPattern::BindingIdentifier(self.alloc(ident));
        let formal_parameter = FormalParameter::new_plain(span, pattern, self);
        let params = FormalParameters::boxed(
            span,
            FormalParameterKind::ArrowFormalParameters,
            [formal_parameter],
            None,
            self,
        );

        if self.cur_token().is_on_new_line() {
            self.error(diagnostics::lineterminator_before_arrow(self.cur_token().span()));
        }

        self.expect(Kind::Arrow);

        self.parse_arrow_function_expression_body(
            ArrowFunctionHead { type_parameters: None, params, return_type: None, r#async, start },
            allow_return_type_in_arrow_function,
        )
    }

    fn parse_parenthesized_arrow_function_head(&mut self) -> ArrowFunctionHead<'a> {
        let start = self.cur_start();
        let r#async = self.eat(Kind::Async);

        let has_await = self.ctx.has_await();
        self.ctx = self.ctx.union_await_if(r#async);

        let (type_parameters, has_trailing_comma) =
            self.parse_ts_type_parameters_with_trailing_comma();

        if let Some(type_params) = &type_parameters
            && matches!(self.source_type.extension(), Some(FileExtension::Mts | FileExtension::Cts))
            && type_params.params.len() == 1
            && type_params.params[0].constraint.is_none()
            && !has_trailing_comma
        {
            self.error(diagnostics::jsx_type_parameter_in_mts_cts(type_params.params[0].name.span));
        }

        let (this_param, params) = self.parse_formal_parameters(
            FunctionKind::Expression,
            FormalParameterKind::ArrowFormalParameters,
        );

        if let Some(this_param) = this_param {
            // const x = (this: number) => {};
            self.error(diagnostics::ts_arrow_function_this_parameter(this_param.span));
        }

        let return_type = if self.is_ts { self.parse_ts_return_type_annotation() } else { None };

        self.ctx = self.ctx.and_await(has_await);

        if self.cur_token().is_on_new_line() {
            self.error(diagnostics::lineterminator_before_arrow(self.cur_token().span()));
        }

        self.expect(Kind::Arrow);

        ArrowFunctionHead { type_parameters, params, return_type, r#async, start }
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
        let ArrowFunctionHead { type_parameters, params, return_type, r#async, start } =
            arrow_function_head;
        let body = self.parse_cover_arrow_body(r#async, allow_return_type_in_arrow_function);
        Expression::new_arrow_function_expression(
            self.end_span(start),
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            self,
        )
    }

    pub(super) fn parse_cover_arrow_body(
        &mut self,
        r#async: bool,
        allow_return_type_in_arrow_function: bool,
    ) -> ArrowFunctionBody<'a> {
        let has_await = self.ctx.has_await();
        let has_yield = self.ctx.has_yield();
        self.ctx = self.ctx.and_await(r#async).and_yield(false);

        let body = if self.at(Kind::LCurly) {
            ArrowFunctionBody::FunctionBody(
                self.context_add(Context::In, Self::parse_function_body),
            )
        } else {
            let expr = self.context_remove(Context::TopLevel, |p| {
                p.parse_assignment_expression_or_higher_impl(allow_return_type_in_arrow_function)
            });
            ArrowFunctionBody::from(expr)
        };

        self.ctx = self.ctx.and_await(has_await).and_yield(has_yield);
        body
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

        let checkpoint = self.checkpoint_with_error_recovery();

        let head = self.parse_parenthesized_arrow_function_head();
        if self.has_fatal_error() {
            self.state.not_parenthesized_arrow.insert(pos);
            self.rewind(checkpoint);
            return None;
        }

        let has_return_type = head.return_type.is_some();

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
        if !allow_return_type_in_arrow_function && has_return_type {
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
