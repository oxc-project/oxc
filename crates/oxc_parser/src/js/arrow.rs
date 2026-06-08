use oxc_allocator::Box;
use oxc_ast::{NONE, ast::*};
use oxc_span::{FileExtension, GetSpan};
use oxc_syntax::precedence::Precedence;

use super::{ArrowKind, FunctionKind};
use crate::{Context, ParserConfig as Config, ParserImpl, diagnostics, lexer::Kind};

struct ArrowFunctionHead<'a> {
    type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    params: Box<'a, FormalParameters<'a>>,
    return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
    r#async: bool,
    span: u32,
}

/// Outcome of classifying a `(`/`<`/`async (` head, i.e. an
/// `ArrowParameters : CoverParenthesizedExpressionAndArrowParameterList`.
pub enum ArrowAttempt<'a> {
    /// Parsed an arrow directly (`ArrowKind::Yes`) or speculatively (`ArrowKind::Speculate`).
    Parsed(Expression<'a>),
    /// `ArrowKind::Cover`: the cover production's `( Expression )` alternative. The entry parses
    /// `( a )` once as an expression, then refines it via [`ParserImpl::try_refine_cover_arrow`]
    /// (-> `ArrowFormalParameters` if `=>` follows, else `ParenthesizedExpression`).
    Cover,
    /// Not an arrow.
    NotArrow,
}

impl<'a, C: Config> ParserImpl<'a, C> {
    pub(super) fn try_parse_parenthesized_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> ArrowAttempt<'a> {
        match self.is_parenthesized_arrow_function_expression() {
            ArrowKind::No => ArrowAttempt::NotArrow,
            ArrowKind::Yes => {
                ArrowAttempt::Parsed(self.parse_parenthesized_arrow_function_expression(
                    allow_return_type_in_arrow_function,
                ))
            }
            ArrowKind::Speculate => match self
                .parse_possible_parenthesized_arrow_function_expression(
                    allow_return_type_in_arrow_function,
                ) {
                Some(expr) => ArrowAttempt::Parsed(expr),
                None => ArrowAttempt::NotArrow,
            },
            ArrowKind::Cover => ArrowAttempt::Cover,
        }
    }

    /// Refine the cover production for a `Cover`-classified `( a )` (a single parenthesized
    /// identifier already parsed into `lhs`):
    /// ```text
    /// ArrowFunction : ArrowParameters [no LineTerminator here] => ConciseBody
    /// // CoverParenthesizedExpressionAndArrowParameterList : ( Expression ) , with Expression == `a`,
    /// // refined to ArrowFormalParameters : ( UniqueFormalParameters )
    /// ```
    /// when `=>` follows (in TS, an optional `: ReturnType` may sit between `)` and `=>`). Returns
    /// `Err(lhs)` unchanged otherwise (no token consumed) — `lhs` is then the other refinement,
    /// `ParenthesizedExpression : ( Expression )`, and parsing continues from it.
    ///
    /// `span` is the `(` start; the current `prev_token_end` is the `)` end, so `( .. )` is the
    /// `FormalParameters` span — matching the direct param-parse path.
    pub(super) fn try_refine_cover_arrow(
        &mut self,
        span: u32,
        lhs: Expression<'a>,
        allow_return_type_in_arrow_function: bool,
    ) -> Result<Expression<'a>, Expression<'a>> {
        // A TS return type may sit between `)` and `=>` (`(a): T => ...`). Only treat a `:` as a
        // return type where arrow return types are allowed; otherwise (e.g. a conditional
        // consequent `cond ? (a) : b`) the `:` belongs to the enclosing conditional.
        let has_return_type =
            self.is_ts && self.at(Kind::Colon) && allow_return_type_in_arrow_function;
        if !has_return_type && !self.at(Kind::Arrow) {
            return Err(lhs);
        }
        // `( a )` parses to `ParenthesizedExpression(Identifier)` (with `preserve_parens`) or a bare
        // `Identifier`. Anything else (e.g. `(a).b =>`) is not a refinable arrow head.
        let is_parenthesized_identifier = match &lhs {
            Expression::Identifier(_) => true,
            Expression::ParenthesizedExpression(p) => {
                matches!(&p.expression, Expression::Identifier(_))
            }
            _ => false,
        };
        if !is_parenthesized_identifier {
            return Err(lhs);
        }
        // `( .. )` span (prev_token_end is the `)` end — before any return type), matching the
        // direct param-parse path.
        let params_span = self.end_span(span);
        if has_return_type {
            // The TS `(a): T =>` case is rare; speculate the return-type + `=>` tail so a malformed
            // `(a): =>` errors exactly like the old rewind path (falls back to `(a)` as an
            // expression, then a stray `:`). For the common no-return-type arrow there is no
            // speculation.
            let checkpoint = self.checkpoint();
            self.parse_ts_return_type_annotation();
            let is_arrow = self.fatal_error.is_none() && self.at(Kind::Arrow);
            self.rewind(checkpoint);
            if !is_arrow {
                return Err(lhs);
            }
        }
        let inner = match lhs {
            Expression::ParenthesizedExpression(p) => p.unbox().expression,
            other => other,
        };
        let params = self.refine_arrow_params(inner, params_span);
        let return_type =
            if has_return_type { self.parse_ts_return_type_annotation() } else { None };
        if self.cur_token().is_on_new_line() {
            self.error(diagnostics::lineterminator_before_arrow(self.cur_token().span()));
        }
        self.expect(Kind::Arrow);
        let head =
            ArrowFunctionHead { type_parameters: None, params, return_type, r#async: false, span };
        Ok(self.parse_arrow_function_expression_body(head, allow_return_type_in_arrow_function))
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

    fn is_parenthesized_arrow_function_expression(&mut self) -> ArrowKind {
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
            _ => ArrowKind::No,
        }
    }

    fn is_parenthesized_arrow_function_expression_worker(&mut self) -> ArrowKind {
        let saw_async = self.eat(Kind::Async);
        if saw_async {
            if self.cur_token().is_on_new_line() {
                return ArrowKind::No;
            }
            let kind = self.cur_kind();
            if kind != Kind::LParen && kind != Kind::LAngle {
                return ArrowKind::No;
            }
        }

        let kind = self.classify_paren_or_angle_arrow();
        // `async (...)` never goes through the single-parse cover path: the head sets await context
        // before parsing params, so `await` inside must not be parsed as an identifier. Keep it on
        // the speculate-and-rewind path.
        if saw_async && kind == ArrowKind::Cover { ArrowKind::Speculate } else { kind }
    }

    /// Classify a `(...)` / `<...>` head by its first few tokens against the cover production:
    /// ```text
    /// CoverParenthesizedExpressionAndArrowParameterList[Yield, Await] :
    ///     ( Expression )                          // `(a)` -> Cover ; `([`/`({`/`(a,`/`(a=` -> Speculate/No
    ///     ( Expression , )                        // trailing comma -> Speculate
    ///     ( )                                     // -> Yes/Speculate (no ParenthesizedExpression form)
    ///     ( ... BindingIdentifier )               // `(...ident` -> Yes
    ///     ( ... BindingPattern )                  // `(...[`/`(...{` -> Speculate
    ///     ( Expression , ... BindingIdentifier )  // a later `...rest` is invisible here -> Speculate
    ///     ( Expression , ... BindingPattern )     // ditto
    /// ```
    /// (plus TS `( a : T )` / generic `<...>` extensions). Only `( a )` — a single identifier with
    /// `)` as the third token — is `Cover`: that alternative is provably a valid `Expression` with no
    /// hidden comma/rest/trailing-comma, so it can be parsed once and refined.
    fn classify_paren_or_angle_arrow(&mut self) -> ArrowKind {
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
                            // `(): T` — empty parens have no expression form, keep speculating.
                            Kind::Colon if self.is_ts => ArrowKind::Speculate,
                            Kind::Arrow | Kind::LCurly => ArrowKind::Yes,
                            _ => ArrowKind::No,
                        }
                    }
                    // "([" or "({": destructuring params or an array/object literal. These can
                    // still hide a top-level `...rest` or trailing comma the worker can't see
                    // (`([x], ...r) =>`, `([x],) =>`), neither of which has an expression form, so
                    // keep them on the speculate path.
                    Kind::LBrack | Kind::LCurly => ArrowKind::Speculate,
                    // Simple case: "(..." — an arrow with a rest parameter; leading rest has no
                    // expression form, keep speculating.
                    Kind::Dot3 => {
                        self.bump_any();
                        let third = self.cur_kind();
                        match third {
                            // '(...ident' is a lambda
                            Kind::Ident => ArrowKind::Yes,
                            // '(...null' is not a lambda
                            kind if kind.is_literal() => ArrowKind::No,
                            _ => ArrowKind::Speculate,
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
                                return ArrowKind::No; // https://github.com/microsoft/TypeScript/issues/44466
                            }
                            return ArrowKind::Yes;
                        }

                        // If we had "(" followed by something that's not an identifier,
                        // then this definitely doesn't look like a lambda.  "this" is not
                        // valid, but we want to parse it and then give a semantic error.
                        if !second.is_binding_identifier() && second != Kind::This {
                            return ArrowKind::No;
                        }

                        match third {
                            // If we have something like "(a:", then we must have a
                            // type-annotated parameter in an arrow function expression.
                            Kind::Colon => ArrowKind::Yes,
                            // If we have "(a?:" or "(a?," or "(a?=" or "(a?)" then it is definitely a lambda.
                            Kind::Question => {
                                self.bump_any();
                                let fourth = self.cur_kind();
                                if matches!(
                                    fourth,
                                    Kind::Colon | Kind::Comma | Kind::Eq | Kind::RParen
                                ) {
                                    return ArrowKind::Yes;
                                }
                                ArrowKind::No
                            }
                            // "(a)" — exactly a single parenthesized identifier. No comma, rest, or
                            // trailing comma is possible (the `)` is the third token), so this is
                            // always a valid expression: parse once and refine.
                            // EXCEPT `(await)`/`(yield)`: in an await/yield context these parse as
                            // `AwaitExpression`/`YieldExpression`, not a parameter name, so keep them
                            // on the speculate path (which parses them as binding identifiers).
                            Kind::RParen if matches!(second, Kind::Await | Kind::Yield) => {
                                ArrowKind::Speculate
                            }
                            Kind::RParen => ArrowKind::Cover,
                            // "(a =" / "(a," may hide a later top-level `...rest` or more params the
                            // worker can't see (`(a = 1, ...r) =>`), with no expression form. Keep
                            // them on the speculate path.
                            Kind::Eq | Kind::Comma => ArrowKind::Speculate,
                            // It is definitely not an arrow function
                            _ => ArrowKind::No,
                        }
                    }
                }
            }
            Kind::LAngle => {
                // If we have "<" not followed by an identifier,
                // then this definitely is not an arrow function.
                if !second.is_binding_identifier() && second != Kind::Const {
                    return ArrowKind::No;
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
                                ArrowKind::No
                            } else if fourth.is_binding_identifier() {
                                ArrowKind::Speculate
                            } else {
                                ArrowKind::Yes
                            }
                        }
                        Kind::Eq | Kind::Comma => ArrowKind::Yes,
                        _ => ArrowKind::No,
                    };
                }
                // generic arrow `<T>(...) =>` — resolved by the speculate path.
                ArrowKind::Speculate
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
        let pattern = BindingPattern::BindingIdentifier(
            self.ast.alloc_binding_identifier(ident.span, ident.name),
        );
        let formal_parameter = self.ast.plain_formal_parameter(ident.span, pattern);
        let params = self.ast.alloc_formal_parameters(
            ident.span,
            FormalParameterKind::ArrowFormalParameters,
            self.ast.vec1(formal_parameter),
            NONE,
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
        let has_await = self.ctx.has_await();
        let has_yield = self.ctx.has_yield();
        self.ctx = self.ctx.and_await(r#async).and_yield(false);

        let expression = !self.at(Kind::LCurly);
        let body = if expression {
            // Remove TopLevel context for arrow function expression body
            let expr = self.context_remove(Context::TopLevel, |p| {
                p.parse_assignment_expression_or_higher_impl(allow_return_type_in_arrow_function)
            });
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
