use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::ast::*;
use oxc_span::{FileExtension, GetSpan, Span};
use oxc_syntax::precedence::Precedence;

use super::{FunctionKind, Tristate, grammar::CoverGrammar};
use crate::{Context, ParserConfig as Config, ParserImpl, diagnostics, lexer::Kind};

struct ArrowFunctionHead<'a> {
    type_parameters: Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>,
    params: ArenaBox<'a, FormalParameters<'a>>,
    return_type: Option<ArenaBox<'a, TSTypeAnnotation<'a>>>,
    r#async: bool,
    start: u32,
}

impl<'a, C: Config> ParserImpl<'a, C> {
    pub(super) fn try_parse_arrow_function_expression<const REUSE_JS_HEAD: bool>(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<Expression<'a>> {
        let kind = match self.cur_kind() {
            Kind::Async => {
                // Both async arrow forms share the token after `async`. Classify them under
                // one checkpoint instead of restarting lookahead for a single parameter.
                let checkpoint = self.checkpoint();
                self.bump_any(); // bump `async`
                if self.cur_token().is_on_new_line() {
                    self.rewind(checkpoint);
                    return None;
                }
                let kind = match self.cur_kind() {
                    Kind::LParen | Kind::LAngle => {
                        self.is_parenthesized_arrow_function_expression_worker(true)
                    }
                    kind if kind.is_binding_identifier() => {
                        self.bump_any();
                        let is_arrow = self.at(Kind::Arrow);
                        self.rewind(checkpoint);
                        return is_arrow.then(|| {
                            self.parse_async_simple_arrow_function_expression(
                                allow_return_type_in_arrow_function,
                            )
                        });
                    }
                    _ => Tristate::False,
                };
                self.rewind(checkpoint);
                kind
            }
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
            Kind::LAngle => {
                self.lookahead(|p| p.is_parenthesized_arrow_function_expression_worker(false))
            }
            _ => Tristate::False,
        };
        match kind {
            Tristate::False => None,
            Tristate::True => Some(self.parse_parenthesized_arrow_function_expression(
                allow_return_type_in_arrow_function,
            )),
            Tristate::Maybe if REUSE_JS_HEAD => {
                self.parse_possible_js_parenthesized_expression(allow_return_type_in_arrow_function)
            }
            Tristate::Maybe => self.parse_possible_parenthesized_arrow_function_expression(
                allow_return_type_in_arrow_function,
            ),
        }
    }

    fn parse_async_simple_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Expression<'a> {
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
        self.parse_simple_arrow_function_expression(
            start,
            ident,
            /* async */ true,
            allow_return_type_in_arrow_function,
        )
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

    fn is_parenthesized_arrow_function_expression_worker(&mut self, is_async: bool) -> Tristate {
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

        self.expect_arrow();
        self.parse_arrow_function_expression_body(
            ArrowFunctionHead { type_parameters: None, params, return_type: None, r#async, start },
            allow_return_type_in_arrow_function,
        )
    }

    fn expect_arrow(&mut self) {
        if self.cur_token().is_on_new_line() {
            self.error(diagnostics::lineterminator_before_arrow(self.cur_token().span()));
        }
        self.expect(Kind::Arrow);
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

    /// Section [Arrow Function](https://tc39.es/ecma262/#sec-arrow-function-definitions)
    /// `ArrowFunction`[In, Yield, Await] :
    ///     `ArrowParameters`[?Yield, ?Await] [no `LineTerminator` here] => `ConciseBody`[?In]
    fn parse_parenthesized_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Expression<'a> {
        let head = self.parse_parenthesized_arrow_function_head();
        self.expect_arrow();
        self.parse_arrow_function_expression_body(head, allow_return_type_in_arrow_function)
    }

    fn expect_arrow_after_head(&mut self) {
        if self.cur_token().is_on_new_line() {
            self.error(diagnostics::lineterminator_before_arrow(self.cur_token().span()));
        }
        self.expect(Kind::Arrow);
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
        // Rejected heads are rewound, so do not construct an error for the missing arrow.
        if !self.at(Kind::Arrow) {
            self.state.not_parenthesized_arrow.insert(pos);
            self.rewind(checkpoint);
            return None;
        }
        self.expect_arrow();
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

    #[inline(never)]
    fn parse_possible_js_parenthesized_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<Expression<'a>> {
        let pos = self.cur_token().start();
        if self.state.not_parenthesized_arrow.contains(&pos) {
            return None;
        }
        let checkpoint = self.checkpoint_with_error_recovery();
        let errors_len = self.errors.len();
        let (params, has_trailing_comma) = self.parse_cover_arrow_parameters();
        let head = ArrowFunctionHead {
            type_parameters: None,
            params,
            return_type: None,
            r#async: false,
            start: pos,
        };
        if !self.at(Kind::Arrow) {
            // Reuse the successful parameter parse, including its initializers and property
            // keys, when it also represents an ordinary parenthesized expression.
            if !self.has_fatal_error()
                && self.errors.len() == errors_len
                && !has_trailing_comma
                && head.params.rest.is_none()
                && !head.params.items.is_empty()
                && let Some(expression) = self.cover_arrow_head_as_expression(head)
            {
                return Some(expression);
            }
            self.state.not_parenthesized_arrow.insert(pos);
            self.rewind(checkpoint);
            return None;
        }
        self.expect_arrow_after_head();
        if self.has_fatal_error() {
            self.state.not_parenthesized_arrow.insert(pos);
            self.rewind(checkpoint);
            return None;
        }
        Some(self.parse_arrow_function_expression_body(head, allow_return_type_in_arrow_function))
    }

    #[inline(never)]
    fn cover_arrow_head_as_expression(
        &mut self,
        head: ArrowFunctionHead<'a>,
    ) -> Option<Expression<'a>> {
        let span = head.params.span;
        let params = head.params.unbox();
        let len = params.items.len();
        let mut items = params.items.into_iter();
        let first = self.cover_parameter_as_expression(items.next().unwrap())?;
        let expression = if len == 1 {
            first
        } else {
            let start = first.span().start;
            let mut expressions = ArenaVec::with_capacity_in(len, self);
            expressions.push(first);
            for item in items {
                expressions.push(self.cover_parameter_as_expression(item)?);
            }
            let end = expressions.last().unwrap().span().end;
            Expression::new_sequence_expression(Span::new(start, end), expressions, self)
        };
        // Keep the wrapper until the caller distinguishes this from an arrow. The caller
        // removes it when `preserve_parens` is disabled.
        Some(Expression::new_parenthesized_expression(span, expression, self))
    }

    fn cover_parameter_as_expression(
        &mut self,
        param: FormalParameter<'a>,
    ) -> Option<Expression<'a>> {
        let left = self.cover_binding_as_expression(param.pattern)?;
        Some(if let Some(initializer) = param.initializer {
            let left = AssignmentTarget::cover(left, self);
            Expression::new_assignment_expression(
                param.span,
                AssignmentOperator::Assign,
                left,
                initializer.unbox(),
                self,
            )
        } else {
            left
        })
    }

    fn cover_binding_as_expression(
        &mut self,
        pattern: BindingPattern<'a>,
    ) -> Option<Expression<'a>> {
        Some(match pattern {
            BindingPattern::BindingIdentifier(ident) => {
                if ident.name == "await" && !self.ctx.has_await() {
                    self.state.encountered_await_identifier = true;
                }
                Expression::new_identifier(ident.span, ident.name, self)
            }
            BindingPattern::AssignmentPattern(pattern) => {
                let pattern = pattern.unbox();
                let left = self.cover_binding_as_expression(pattern.left)?;
                let left = AssignmentTarget::cover(left, self);
                Expression::new_assignment_expression(
                    pattern.span,
                    AssignmentOperator::Assign,
                    left,
                    pattern.right,
                    self,
                )
            }
            BindingPattern::ArrayPattern(pattern) => {
                let pattern = pattern.unbox();
                let mut elements = ArenaVec::with_capacity_in(
                    pattern.elements.len() + usize::from(pattern.rest.is_some()),
                    self,
                );
                for element in pattern.elements {
                    // Binding elisions do not retain their token spans. Reparse those arrays.
                    let expression = self.cover_binding_as_expression(element?)?;
                    elements.push(ArrayExpressionElement::from(expression));
                }
                if let Some(rest) = pattern.rest {
                    let rest = rest.unbox();
                    let argument = self.cover_binding_as_expression(rest.argument)?;
                    elements.push(ArrayExpressionElement::SpreadElement(SpreadElement::boxed(
                        rest.span, argument, self,
                    )));
                }
                Expression::new_array_expression(pattern.span, elements, self)
            }
            BindingPattern::ObjectPattern(pattern) => {
                let pattern = pattern.unbox();
                let mut properties = ArenaVec::with_capacity_in(
                    pattern.properties.len() + usize::from(pattern.rest.is_some()),
                    self,
                );
                for property in pattern.properties {
                    if property.shorthand
                        && matches!(property.value, BindingPattern::AssignmentPattern(_))
                    {
                        // Preserve deferred CoverInitializedName validation on the ordinary path.
                        return None;
                    }
                    let value = if property.shorthand {
                        let BindingPattern::BindingIdentifier(ident) = property.value else {
                            return None;
                        };
                        // As in object-literal parsing, an `await` shorthand does not mark
                        // this statement for unambiguous top-level-await reparsing.
                        Expression::new_identifier(ident.span, ident.name, self)
                    } else {
                        self.cover_binding_as_expression(property.value)?
                    };
                    properties.push(ObjectPropertyKind::ObjectProperty(ObjectProperty::boxed(
                        property.span,
                        PropertyKind::Init,
                        property.key,
                        value,
                        false,
                        property.shorthand,
                        property.computed,
                        self,
                    )));
                }
                if let Some(rest) = pattern.rest {
                    let rest = rest.unbox();
                    let argument = self.cover_binding_as_expression(rest.argument)?;
                    properties.push(ObjectPropertyKind::SpreadProperty(SpreadElement::boxed(
                        rest.span, argument, self,
                    )));
                }
                Expression::new_object_expression(pattern.span, properties, self)
            }
        })
    }
}
