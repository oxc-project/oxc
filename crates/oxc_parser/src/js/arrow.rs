use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::{ast::*, builder::NONE};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{FileExtension, GetSpan, Span};
use oxc_syntax::precedence::Precedence;

use super::{FunctionKind, Tristate, grammar, grammar::CoverGrammar};
use crate::{Context, ParserConfig as Config, ParserImpl, diagnostics, lexer::Kind};

struct ArrowFunctionHead<'a> {
    type_parameters: Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>,
    params: ArenaBox<'a, FormalParameters<'a>>,
    return_type: Option<ArenaBox<'a, TSTypeAnnotation<'a>>>,
    r#async: bool,
    span: u32,
}

pub(super) enum ArrowOrExpression<'a> {
    Arrow(Expression<'a>),
    /// `Tristate::Maybe` refined to a non-arrow: the parenthesized expression.
    /// Member/call/binary continuations have not been parsed yet.
    Expression(Expression<'a>),
    NotArrow,
}

/// [`CoverParenthesizedExpressionAndArrowParameterList`](https://tc39.es/ecma262/#prod-CoverParenthesizedExpressionAndArrowParameterList),
/// parsed once and refined by what follows:
///     `CoverParenthesizedExpressionAndArrowParameterList`[Yield, Await] :
///         ( `Expression`[+In, ?Yield, ?Await] )
///         ( `Expression`[+In, ?Yield, ?Await] , )
///         ( )
///         ( ... `BindingIdentifier`[?Yield, ?Await] )
///         ( ... `BindingPattern`[?Yield, ?Await] )
///         ( `Expression`[+In, ?Yield, ?Await] , ... `BindingIdentifier`[?Yield, ?Await] )
///         ( `Expression`[+In, ?Yield, ?Await] , ... `BindingPattern`[?Yield, ?Await] )
struct CoverParen<'a> {
    /// Span of `( ... )` including parens, for `FormalParameters`.
    params_span: Span,
    /// Span of the items excluding parens, for `SequenceExpression`.
    inner_span: Span,
    items: ArenaVec<'a, Expression<'a>>,
    extras: std::vec::Vec<CoverItemExtras<'a>>,
    trailing_comma: Option<Span>,
    /// First parameter-only TS syntax (`?` / `: Type`) — invalid in a parenthesized expression.
    ts_error: Option<(u32, OxcDiagnostic)>,
    /// First `...` spread — invalid in a parenthesized expression.
    spread_error: Option<(u32, OxcDiagnostic)>,
    has_no_side_effects_comment: bool,
}

/// Parameter-only syntax attached to a cover item, keyed by item index.
struct CoverItemExtras<'a> {
    index: u32,
    spread_span: Option<Span>,
    optional_span: Option<Span>,
    type_annotation: Option<ArenaBox<'a, TSTypeAnnotation<'a>>>,
    /// Initializer following a type annotation or optional marker (`(a: T = 1)`).
    init: Option<Expression<'a>>,
}

impl<'a, C: Config> ParserImpl<'a, C> {
    pub(super) fn try_parse_parenthesized_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> ArrowOrExpression<'a> {
        match self.is_parenthesized_arrow_function_expression() {
            Tristate::False => ArrowOrExpression::NotArrow,
            Tristate::True => {
                ArrowOrExpression::Arrow(self.parse_parenthesized_arrow_function_expression(
                    allow_return_type_in_arrow_function,
                ))
            }
            Tristate::Maybe => {
                // Only plain `( ... )` is parsed once via the cover grammar. `async (...)`
                // heads re-parse under `[+Await]` and generic `<T>(...)` heads under
                // type-parameter rules — both can parse differently from the expression
                // interpretation, so they keep the speculative parse + rewind.
                if self.cur_kind() == Kind::LParen {
                    self.parse_cover_parenthesized_expression_or_arrow(
                        allow_return_type_in_arrow_function,
                    )
                } else {
                    match self.parse_possible_parenthesized_arrow_function_expression(
                        allow_return_type_in_arrow_function,
                    ) {
                        Some(expr) => ArrowOrExpression::Arrow(expr),
                        None => ArrowOrExpression::NotArrow,
                    }
                }
            }
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
            let Expression::Identifier(ident) = &expr else {
                return self.unexpected();
            };
            // It is a Syntax Error if ArrowParameters Contains AwaitExpression is true.
            if ident.name == "await" {
                self.error(diagnostics::identifier_async("await", ident.span));
            }
            return Some(self.parse_simple_arrow_function_expression(
                span,
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
                // `(1 + a)` can never be arrow parameters: the leading literal is not the start of
                // a `BindingElement`, so the worker would bump past it and return `Tristate::False`
                // (`!second.is_binding_identifier() && second != This`). Skip the lookahead.
                if self.lexer.peek_token().kind().is_literal() {
                    Tristate::False
                } else {
                    self.lookahead(Self::is_parenthesized_arrow_function_expression_worker)
                }
            }
            Kind::LAngle | Kind::Async => {
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
                            // If we have "(a," or "(a=" this *could* be an arrow function
                            Kind::Comma | Kind::Eq => Tristate::Maybe,
                            // "(a)": peek past `)` — a `=>` makes it unambiguously an arrow.
                            // Not for `(this)`, whose head parse fails where the speculative
                            // parser recovered to a parenthesized expression.
                            Kind::RParen => {
                                self.bump_any();
                                if second != Kind::This && self.cur_kind() == Kind::Arrow {
                                    Tristate::True
                                } else {
                                    Tristate::Maybe
                                }
                            }
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
        span: u32,
        ident: &IdentifierReference<'a>,
        r#async: bool,
        allow_return_type_in_arrow_function: bool,
    ) -> Expression<'a> {
        let pattern = BindingPattern::new_binding_identifier(ident.span, ident.name, self);
        let formal_parameter = FormalParameter::new_plain(ident.span, pattern, self);
        let params = FormalParameters::boxed(
            ident.span,
            FormalParameterKind::ArrowFormalParameters,
            ArenaVec::from_value_in(formal_parameter, self),
            NONE,
            self,
        );

        if self.cur_token().is_on_new_line() {
            self.error(diagnostics::lineterminator_before_arrow(self.cur_token().span()));
        }

        self.expect(Kind::Arrow);

        self.parse_arrow_function_expression_body(
            ArrowFunctionHead { type_parameters: None, params, return_type: None, r#async, span },
            allow_return_type_in_arrow_function,
        )
    }

    fn parse_parenthesized_arrow_function_head(&mut self) -> ArrowFunctionHead<'a> {
        let span = self.start_span();
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

        ArrowFunctionHead { type_parameters, params, return_type, r#async, span }
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
        let ArrowFunctionHead { type_parameters, params, return_type, r#async, span } =
            arrow_function_head;
        let (expression, body) =
            self.parse_arrow_function_body_only(r#async, allow_return_type_in_arrow_function);
        Expression::new_arrow_function_expression(
            self.end_span(span),
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            self,
        )
    }

    fn parse_arrow_function_body_only(
        &mut self,
        r#async: bool,
        allow_return_type_in_arrow_function: bool,
    ) -> (bool, ArenaBox<'a, FunctionBody<'a>>) {
        let has_await = self.ctx.has_await();
        let has_yield = self.ctx.has_yield();
        self.ctx = self.ctx.and_await(r#async).and_yield(false);
        // An arrow body is not part of any enclosing cover paren frame.
        let cover_paren_depth = std::mem::replace(&mut self.state.cover_paren_depth, 0);

        let expression = !self.at(Kind::LCurly);
        let body = if expression {
            // Remove TopLevel context for arrow function expression body
            let span = self.start_span();
            let expr = self.context_remove(Context::TopLevel, |p| {
                p.parse_assignment_expression_or_higher_impl(allow_return_type_in_arrow_function)
            });
            let span = self.end_span(span);
            let expr_stmt = Statement::new_expression_statement(span, expr, self);
            FunctionBody::boxed(
                span,
                ArenaVec::new_in(self),
                ArenaVec::from_value_in(expr_stmt, self),
                self,
            )
        } else {
            self.parse_function_body()
        };

        self.state.cover_paren_depth = cover_paren_depth;
        self.ctx = self.ctx.and_await(has_await).and_yield(has_yield);
        (expression, body)
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

    /// Parse `( ... )` once as [`CoverParen`], then apply the
    /// [supplemental syntax](https://tc39.es/ecma262/#sec-primary-expression) refinement:
    /// * `=>` (or `: Type =>`) — `ArrowParameters : CoverParenthesizedExpressionAndArrowParameterList`
    ///   is reinterpreted as its
    ///   [`CoveredFormalsList`](https://tc39.es/ecma262/#sec-static-semantics-coveredformalslist),
    ///   i.e. [`ArrowFormalParameters`](https://tc39.es/ecma262/#prod-ArrowFormalParameters).
    /// * otherwise — `PrimaryExpression : CoverParenthesizedExpressionAndArrowParameterList`
    ///   is reinterpreted as its
    ///   [`ParenthesizedExpression`](https://tc39.es/ecma262/#prod-ParenthesizedExpression).
    fn parse_cover_parenthesized_expression_or_arrow(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> ArrowOrExpression<'a> {
        let result = self.parse_cover_parenthesized_expression_or_arrow_impl(
            allow_return_type_in_arrow_function,
        );
        if self.state.cover_paren_depth == 0 {
            self.state.cover_invalid_patterns.clear();
        }
        result
    }

    fn parse_cover_parenthesized_expression_or_arrow_impl(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> ArrowOrExpression<'a> {
        let span = self.start_span();
        let checkpoint = self.checkpoint_with_error_recovery();
        let frame = self.parse_cover_paren_contents();
        if self.fatal_error.is_some() {
            // An arrow head parses some of these softly where the expression grammar is fatal
            // (binding-only forms like `([...x = []]) => {}`, `await`/`yield` bindings under
            // `[+Await]`/`[+Yield]`). Fall back to the speculative arrow parse, which also
            // reproduces the expression re-parse and its diagnostics when it rejects.
            self.rewind(checkpoint);
            return match self.parse_possible_parenthesized_arrow_function_expression(
                allow_return_type_in_arrow_function,
            ) {
                Some(expr) => ArrowOrExpression::Arrow(expr),
                None => ArrowOrExpression::NotArrow,
            };
        }

        if self.at(Kind::Arrow) {
            if self.cover_refines_to_arrow_params(&frame) {
                return ArrowOrExpression::Arrow(self.cover_to_arrow(
                    span,
                    frame,
                    None,
                    allow_return_type_in_arrow_function,
                ));
            }
        } else if self.is_ts && self.at(Kind::Colon) {
            let checkpoint = self.checkpoint_with_error_recovery();
            let return_type = self.parse_ts_return_type_annotation();
            if self.fatal_error.is_none()
                && self.at(Kind::Arrow)
                && self.cover_refines_to_arrow_params(&frame)
            {
                if allow_return_type_in_arrow_function {
                    return ArrowOrExpression::Arrow(self.cover_to_arrow(
                        span,
                        frame,
                        return_type,
                        allow_return_type_in_arrow_function,
                    ));
                }
                // In the true branch of a conditional, `(a): T => x` is an arrow function only
                // if another `:` follows the body (`cond ? (a): T => x : y`); otherwise the
                // first `:` terminates the conditional. Parse the body before converting the
                // items to parameters so they survive the rewind if the arrow is rejected.
                if self.cur_token().is_on_new_line() {
                    self.error(diagnostics::lineterminator_before_arrow(self.cur_token().span()));
                }
                self.expect(Kind::Arrow);
                let (expression, body) = self.parse_arrow_function_body_only(
                    /* async */ false, /* allow_return_type */ false,
                );
                if self.fatal_error.is_none() && self.at(Kind::Colon) {
                    let params = self.cover_to_formal_parameters(frame);
                    return ArrowOrExpression::Arrow(Expression::new_arrow_function_expression(
                        self.end_span(span),
                        expression,
                        false,
                        NONE,
                        params,
                        return_type,
                        body,
                        self,
                    ));
                }
            }
            // Not an arrow; rewind to the `:` and refine as expression — the caller handles it.
            self.rewind(checkpoint);
        }

        ArrowOrExpression::Expression(self.cover_to_expression(span, frame))
    }

    /// Whether the cover is "covering" an
    /// [`ArrowFormalParameters`](https://tc39.es/ecma262/#prod-ArrowFormalParameters) —
    /// the reparse demanded by
    /// [`CoveredFormalsList`](https://tc39.es/ecma262/#sec-static-semantics-coveredformalslist),
    /// as a read-only check mirroring the speculative head parse's accept/reject decision.
    /// When the items cannot refine to parameters, the cover refines to a parenthesized
    /// expression and the stray `=>` errors downstream, exactly like the speculative
    /// parser's rewind did.
    fn cover_refines_to_arrow_params(&self, frame: &CoverParen<'a>) -> bool {
        let len = frame.items.len();
        let mut extras = frame.extras.iter().peekable();
        frame.items.iter().enumerate().all(|(i, expr)| {
            let extra = if extras.peek().is_some_and(|e| e.index as usize == i) {
                extras.next()
            } else {
                None
            };
            if extra.is_some_and(|e| e.spread_span.is_some()) && i != len - 1 {
                return false;
            }
            if matches!(expr, Expression::ThisExpression(_)) {
                // `(this: T) => {}` parses with an error; anywhere else `this` is fatal.
                return i == 0 && self.is_ts && extra.is_none_or(|e| e.spread_span.is_none());
            }
            grammar::is_binding_pattern_expression(expr, self)
        })
    }

    /// Parse the `( ... )` of [`CoverParen`]: a comma-separated list of cover items,
    /// allowing the trailing-comma and rest productions that only refine to arrows.
    fn parse_cover_paren_contents(&mut self) -> CoverParen<'a> {
        let span = self.start_span();
        let opening_span = self.cur_token().span();
        let has_no_side_effects_comment =
            self.lexer.trivia_builder.previous_token_has_no_side_effects_comment();
        self.bump_any(); // `(`
        let inner_span_start = self.start_span();
        let mut frame = CoverParen {
            params_span: opening_span,
            inner_span: opening_span,
            items: ArenaVec::new_in(self),
            extras: std::vec::Vec::new(),
            trailing_comma: None,
            ts_error: None,
            spread_error: None,
            has_no_side_effects_comment,
        };
        self.state.cover_paren_depth += 1;
        self.context(Context::In, Context::Decorator, |p| {
            loop {
                let kind = p.cur_kind();
                if kind == Kind::RParen
                    || matches!(kind, Kind::Eof | Kind::Undetermined)
                    || p.fatal_error.is_some()
                {
                    break;
                }
                p.parse_cover_item(&mut frame, opening_span);
                let kind = p.cur_kind();
                if kind == Kind::RParen
                    || matches!(kind, Kind::Eof | Kind::Undetermined)
                    || p.fatal_error.is_some()
                {
                    break;
                }
                if kind != Kind::Comma {
                    p.set_fatal_error(diagnostics::expect_closing_or_separator(
                        Kind::RParen.to_str(),
                        Kind::Comma.to_str(),
                        kind.to_str(),
                        p.cur_token().span(),
                        opening_span,
                    ));
                    break;
                }
                p.advance(Kind::Comma);
                if p.cur_kind() == Kind::RParen {
                    let comma_pos = p.prev_token_end - 1;
                    frame.trailing_comma = Some(Span::new(comma_pos, p.prev_token_end));
                    break;
                }
            }
        });
        self.state.cover_paren_depth -= 1;
        frame.inner_span = Span::new(inner_span_start, self.prev_token_end);
        self.expect_closing(Kind::RParen, opening_span);
        frame.params_span = Span::new(span, self.prev_token_end);
        frame
    }

    /// One element of the cover list:
    ///     `AssignmentExpression`[+In, ?Yield, ?Await]
    ///     ... `BindingIdentifier`[?Yield, ?Await]
    ///     ... `BindingPattern`[?Yield, ?Await]
    /// plus TypeScript parameter-only syntax after a pattern-shaped item — `?`, `: Type`,
    /// and `= Initializer` following either. The `...` and TS pieces are recorded in
    /// [`CoverItemExtras`]; they are only valid if the arrow refinement is taken.
    fn parse_cover_item(&mut self, frame: &mut CoverParen<'a>, opening_span: Span) {
        let index = u32::try_from(frame.items.len()).unwrap_or(u32::MAX);
        let mut spread_span = None;
        if self.at(Kind::Dot3) {
            let span = self.cur_token().span();
            if frame.spread_error.is_none() {
                frame.spread_error = Some((span.start, diagnostics::unexpected_token(span)));
            }
            spread_span = Some(span);
            self.bump_any();
        }

        let expr = self.parse_assignment_expression_or_higher();

        let mut optional_span = None;
        let mut type_annotation = None;
        let mut init = None;
        // Parameter-only TS syntax after a pattern-shaped item: `?`, `: Type`, and an
        // initializer following either.
        if self.is_ts
            && self.fatal_error.is_none()
            && matches!(
                expr,
                Expression::Identifier(_)
                    | Expression::ObjectExpression(_)
                    | Expression::ArrayExpression(_)
                    | Expression::ThisExpression(_)
            )
        {
            if self.at(Kind::Question) {
                // Only reachable when `?` is followed by `:`/`,`/`)`/`=`, which
                // `parse_conditional_expression_rest` declines inside a cover frame.
                let question_span = self.cur_token().span();
                if frame.ts_error.is_none() {
                    let next_span = self.lexer.peek_token().span();
                    frame.ts_error =
                        Some((question_span.start, diagnostics::unexpected_token(next_span)));
                }
                optional_span = Some(question_span);
                self.bump_any();
            }
            if self.at(Kind::Colon) {
                let colon_span = self.cur_token().span();
                if frame.ts_error.is_none() {
                    frame.ts_error = Some((
                        colon_span.start,
                        diagnostics::expect_closing_or_separator(
                            Kind::RParen.to_str(),
                            Kind::Comma.to_str(),
                            Kind::Colon.to_str(),
                            colon_span,
                            opening_span,
                        ),
                    ));
                }
                type_annotation = self.parse_ts_type_annotation();
            }
            if (optional_span.is_some() || type_annotation.is_some()) && self.eat(Kind::Eq) {
                init = Some(self.parse_assignment_expression_or_higher());
            }
        }

        if spread_span.is_some()
            || optional_span.is_some()
            || type_annotation.is_some()
            || init.is_some()
        {
            frame.extras.push(CoverItemExtras {
                index,
                spread_span,
                optional_span,
                type_annotation,
                init,
            });
        }
        frame.items.push(expr);
    }

    /// [`ArrowFunction`](https://tc39.es/ecma262/#prod-ArrowFunction) :
    ///     `ArrowParameters`[?Yield, ?Await] [no `LineTerminator` here] => `ConciseBody`[?In]
    fn cover_to_arrow(
        &mut self,
        span: u32,
        frame: CoverParen<'a>,
        return_type: Option<ArenaBox<'a, TSTypeAnnotation<'a>>>,
        allow_return_type_in_arrow_function: bool,
    ) -> Expression<'a> {
        let params = self.cover_to_formal_parameters(frame);
        if self.cur_token().is_on_new_line() {
            self.error(diagnostics::lineterminator_before_arrow(self.cur_token().span()));
        }
        self.expect(Kind::Arrow);
        self.parse_arrow_function_expression_body(
            ArrowFunctionHead { type_parameters: None, params, return_type, r#async: false, span },
            allow_return_type_in_arrow_function,
        )
    }

    /// Refine the cover to
    /// [`ArrowFormalParameters`](https://tc39.es/ecma262/#prod-ArrowFormalParameters) :
    ///     ( `UniqueFormalParameters`[?Yield, ?Await] )
    ///
    /// Items become `BindingElement`s, a trailing `...` item the `BindingRestElement`, and
    /// the recorded TS `?` / `: Type` / initializer extras attach to their parameters.
    fn cover_to_formal_parameters(
        &mut self,
        frame: CoverParen<'a>,
    ) -> ArenaBox<'a, FormalParameters<'a>> {
        let CoverParen { params_span, items, extras, trailing_comma, .. } = frame;
        let mut extras_iter = extras.into_iter().peekable();
        let mut list = ArenaVec::with_capacity_in(items.len(), self);
        let mut rest: Option<ArenaBox<'a, FormalParameterRest<'a>>> = None;

        for (i, expr) in items.into_iter().enumerate() {
            if self.fatal_error.is_some() {
                break;
            }
            let extra = if extras_iter.peek().is_some_and(|e| e.index as usize == i) {
                extras_iter.next()
            } else {
                None
            };
            let (spread_span, optional_span, type_annotation, extra_init) = match extra {
                Some(e) => (e.spread_span, e.optional_span, e.type_annotation, e.init),
                None => (None, None, None, None),
            };

            if let Some(r) = &rest {
                self.set_fatal_error(diagnostics::rest_parameter_last(
                    r.type_annotation.as_ref().map_or(r.rest.span, |type_annotation| {
                        r.rest.span.merge(type_annotation.span())
                    }),
                ));
                break;
            }

            let expr_span = expr.span();

            if let Some(spread_span) = spread_span {
                let argument = BindingPattern::cover(expr, self);
                if let Some(optional_span) = optional_span {
                    self.error(diagnostics::a_rest_parameter_cannot_be_optional(optional_span));
                }
                if let BindingPattern::AssignmentPattern(pat) = &argument {
                    self.error(diagnostics::a_rest_parameter_cannot_have_an_initializer(pat.span));
                }
                let rest_span = Span::new(spread_span.start, expr_span.end);
                let rest_element = BindingRestElement::new(rest_span, argument, self);
                let full_span = type_annotation
                    .as_ref()
                    .map_or(rest_span, |type_annotation| rest_span.merge(type_annotation.span()));
                rest = Some(FormalParameterRest::boxed(
                    full_span,
                    ArenaVec::new_in(self),
                    rest_element,
                    type_annotation,
                    self,
                ));
                continue;
            }

            if matches!(&expr, Expression::ThisExpression(_)) {
                let this_span = type_annotation
                    .as_ref()
                    .map_or(expr_span, |type_annotation| expr_span.merge(type_annotation.span()));
                // `(this: T) => {}` — the speculative head reported and dropped the
                // `this` parameter; do the same for the first item.
                if i == 0 && self.is_ts {
                    self.error(diagnostics::ts_arrow_function_this_parameter(this_span));
                    continue;
                }
                return self.fatal_error(diagnostics::invalid_binding_pattern(this_span));
            }

            let (pattern, init) = match expr {
                Expression::AssignmentExpression(assign) => {
                    let assign = assign.unbox();
                    if assign.operator != AssignmentOperator::Assign {
                        return self.fatal_error(
                            diagnostics::invalid_assignment_target_default_value_operator(
                                assign.span,
                            ),
                        );
                    }
                    (BindingPattern::cover(assign.left, self), Some(assign.right))
                }
                expr => (BindingPattern::cover(expr, self), extra_init),
            };
            if optional_span.is_some() && init.is_some() {
                self.error(diagnostics::a_parameter_cannot_have_question_mark_and_initializer(
                    pattern.span(),
                ));
            }

            let mut param_span = expr_span;
            if let Some(optional_span) = optional_span {
                param_span = param_span.merge(optional_span);
            }
            if let Some(type_annotation) = &type_annotation {
                param_span = param_span.merge(type_annotation.span());
            }
            if let Some(init) = &init {
                param_span = param_span.merge(init.span());
            }
            list.push(FormalParameter::new(
                param_span,
                ArenaVec::new_in(self),
                pattern,
                type_annotation,
                init,
                optional_span.is_some(),
                None,
                false,
                false,
                self,
            ));
        }

        if let Some(comma_span) = trailing_comma
            && rest.is_some()
            && !self.ctx.has_ambient()
        {
            self.error(diagnostics::rest_element_trailing_comma(comma_span));
        }

        FormalParameters::boxed(
            params_span,
            FormalParameterKind::ArrowFormalParameters,
            list,
            rest,
            self,
        )
    }

    /// Refine the cover to
    /// [`ParenthesizedExpression`](https://tc39.es/ecma262/#prod-ParenthesizedExpression) :
    ///     ( `Expression`[+In, ?Yield, ?Await] )
    ///
    /// Per the [grouping operator early errors](https://tc39.es/ecma262/#sec-grouping-operator-static-semantics-early-errors)
    /// the cover must actually cover a `ParenthesizedExpression`: `( )`, a trailing comma,
    /// `...`, and TS parameter syntax exist only in the arrow productions and are syntax
    /// errors here.
    fn cover_to_expression(&mut self, span: u32, frame: CoverParen<'a>) -> Expression<'a> {
        let CoverParen {
            inner_span,
            mut items,
            ts_error,
            spread_error,
            trailing_comma,
            has_no_side_effects_comment,
            ..
        } = frame;

        let error = match (ts_error, spread_error) {
            (Some(a), Some(b)) => Some(if a.0 <= b.0 { a.1 } else { b.1 }),
            (Some(a), None) => Some(a.1),
            (None, Some(b)) => Some(b.1),
            (None, None) => None,
        };
        if let Some(error) = error {
            return self.fatal_error(error);
        }
        if let Some(comma_span) = trailing_comma {
            let error =
                diagnostics::unexpected_trailing_comma("Parenthesized expressions", comma_span);
            return self.fatal_error(error);
        }
        if items.is_empty() {
            let error = diagnostics::empty_parenthesized_expression(self.end_span(span));
            return self.fatal_error(error);
        }

        let mut expression = if items.len() == 1 {
            items.remove(0)
        } else {
            Expression::new_sequence_expression(inner_span, items, self)
        };

        match &mut expression {
            Expression::ArrowFunctionExpression(arrow_expr) => {
                arrow_expr.pife = true;
                if has_no_side_effects_comment {
                    arrow_expr.pure = true;
                }
            }
            Expression::FunctionExpression(func_expr) => {
                func_expr.pife = true;
                if has_no_side_effects_comment {
                    func_expr.pure = true;
                }
            }
            _ => {}
        }

        if self.options.preserve_parens {
            Expression::new_parenthesized_expression(self.end_span(span), expression, self)
        } else {
            if self.state.cover_paren_depth != 0 {
                self.state.cover_invalid_patterns.push(self.end_span(span));
            }
            expression
        }
    }
}
