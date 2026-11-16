//! Arrow functions.
//!
//! `( … )` and `async ( … )` are parsed once with the cover grammars in [`super::cover`],
//! and refined to arrow function parameters if `=>` follows.

use std::{mem, num::NonZeroU32};

use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::ast::*;
use oxc_span::{FileExtension, Span};
use oxc_str::Ident;
use oxc_syntax::precedence::Precedence;

use super::{
    FunctionKind, Tristate,
    cover::{Cover, CoverElement, CoverIdentifier, CoverList},
    operator::kind_to_precedence,
};
use crate::{
    Context, ParserConfig as Config, ParserImpl, cursor::ParserCheckpoint, diagnostics, lexer::Kind,
};

struct ArrowFunctionHead<'a> {
    type_parameters: Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>,
    params: ArenaBox<'a, FormalParameters<'a>>,
    return_type: Option<ArenaBox<'a, TSTypeAnnotation<'a>>>,
    r#async: bool,
    start: u32,
}

impl<'a, C: Config> ParserImpl<'a, C> {
    /// `CoverParenthesizedExpressionAndArrowParameterList`[Yield, Await] :
    ///     ( `Expression`[+In, ?Yield, ?Await] )
    ///     ( `Expression`[+In, ?Yield, ?Await] , )
    ///     ( )
    ///     ( ... `BindingIdentifier`[?Yield, ?Await] )
    ///     ( ... `BindingPattern`[?Yield, ?Await] )
    ///     ( `Expression`[+In, ?Yield, ?Await] , ... `BindingIdentifier`[?Yield, ?Await] )
    ///     ( `Expression`[+In, ?Yield, ?Await] , ... `BindingPattern`[?Yield, ?Await] )
    ///
    /// Returns the expression, and whether it is an arrow function.
    pub(super) fn parse_parenthesized_expression_or_arrow_function(
        &mut self,
        allow_return_type_in_arrow_function: bool,
        no_side_effects_comments: Option<(u32, NonZeroU32)>,
    ) -> (Expression<'a>, bool) {
        debug_assert!(self.at(Kind::LParen));
        let start = self.cur_start();
        self.bump_any(); // `(`
        let first_start = self.cur_start();
        let mut cover = Cover::new(
            start,
            start,
            self.state.cover_elements.len(),
            self.state.cover_parenthesized.len(),
            /* is_async */ false,
        );

        let end = 'list: {
            let kind = self.cur_kind();
            let mut identifier = None;
            if kind == Kind::Ident {
                let ident = self.parse_cover_identifier();
                // `(a)` and `(a) => …`
                if self.at(Kind::RParen) {
                    match self.parse_parenthesized_identifier_or_arrow_function(
                        start,
                        ident,
                        allow_return_type_in_arrow_function,
                    ) {
                        Ok(result) => return result,
                        Err(end) => break 'list end,
                    }
                }
                identifier = Some(ident);
            }

            // A first element that cannot be a `BindingElement` makes the list
            // a parenthesized expression: `(a.b)`, `(a + b)`, `(!a)`, `(function () {})`.
            let first_parsed = if identifier.is_some() {
                Self::continues_identifier_expression(self.cur_kind())
            } else {
                Self::cannot_start_binding_element(kind)
            };
            if first_parsed {
                match self.parse_parenthesized_expression_first_element(
                    cover,
                    identifier.take(),
                    no_side_effects_comments,
                ) {
                    Ok(expression) => return (expression, false),
                    Err(Some(end)) => break 'list end,
                    Err(None) => {}
                }
            }

            match self.parse_cover_list(start, cover.base, false, first_parsed, identifier) {
                CoverList::Parameters(params) => {
                    let arrow = self.parse_cover_parameters_arrow_function(
                        cover,
                        params,
                        allow_return_type_in_arrow_function,
                    );
                    return (arrow, true);
                }
                CoverList::Undecided { trailing_comma, end } => {
                    cover.trailing_comma = trailing_comma;
                    end
                }
            }
        };

        if self.fatal_error.is_some() {
            return (self.abandon_cover(cover), false);
        }
        if let Some(arrow) =
            self.try_parse_cover_arrow_function(cover, allow_return_type_in_arrow_function)
        {
            return (arrow, true);
        }
        let expression = self.cover_elements_to_parenthesized_expression(
            cover,
            first_start,
            end,
            no_side_effects_comments,
        );
        (expression, false)
    }

    /// Whether `kind` after an identifier continues an expression that is not a `BindingElement`.
    #[inline]
    fn continues_identifier_expression(kind: Kind) -> bool {
        matches!(kind, Kind::Dot | Kind::QuestionDot | Kind::LParen | Kind::LBrack)
            || kind.is_template_start_of_tagged_template()
            || kind_to_precedence(kind).is_some()
    }

    /// Parse `(a)` or `(a) => …` after `a`.
    ///
    /// Returns `Err` with the end of `a`, pushed as a cover element, if a TypeScript `:` follows,
    /// which may start a return type or end the consequent of a conditional expression.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn parse_parenthesized_identifier_or_arrow_function(
        &mut self,
        start: u32,
        identifier: CoverIdentifier<'a>,
        allow_return_type_in_arrow_function: bool,
    ) -> Result<(Expression<'a>, bool), u32> {
        let CoverIdentifier { span, name, .. } = identifier;
        self.bump_any(); // `)`

        match self.cur_kind() {
            // `(a) => …`
            Kind::Arrow => {
                let pattern = BindingPattern::new_binding_identifier(span, name, self);
                let param = FormalParameter::new_plain(span, pattern, self);
                let params = FormalParameters::boxed(
                    self.end_span(start),
                    FormalParameterKind::ArrowFormalParameters,
                    ArenaVec::from_value_in(param, self),
                    None,
                    self,
                );
                let arrow = self.parse_arrow_function_rest(
                    start,
                    false,
                    params,
                    None,
                    allow_return_type_in_arrow_function,
                );
                Ok((arrow, true))
            }
            // `(a): T => …` or `c ? (a) : b`
            Kind::Colon if self.is_ts => {
                self.state.cover_elements.push(CoverElement::Identifier { span, name });
                Err(span.end)
            }
            // `(a)`
            _ => {
                // An enclosing cover list cannot become a binding pattern with `(a)` in it.
                if self.state.cover_depth > 0 {
                    self.state.cover_parenthesized.push(span.start);
                }
                let expression = Expression::new_identifier(span, name, self);
                let expression = if self.options.preserve_parens {
                    Expression::new_parenthesized_expression(self.end_span(start), expression, self)
                } else {
                    expression
                };
                Ok((expression, false))
            }
        }
    }

    /// Whether an element that starts with `kind` cannot be a `BindingElement`.
    #[inline]
    fn cannot_start_binding_element(kind: Kind) -> bool {
        kind.is_literal()
            || kind.is_unary_operator()
            || kind.is_update_operator()
            || kind.is_template_start_of_tagged_template()
            || matches!(kind, Kind::Function | Kind::Class | Kind::New | Kind::LParen | Kind::Slash)
    }

    /// Parse the first element of a list that cannot be arrow function parameters, and
    /// `( Expression )` if it is the only element.
    ///
    /// Otherwise the element is pushed as a cover element, and returns `Err` with its end if
    /// `)` follows it, or `Err(None)` if more elements follow.
    fn parse_parenthesized_expression_first_element(
        &mut self,
        cover: Cover,
        identifier: Option<CoverIdentifier<'a>>,
        no_side_effects_comments: Option<(u32, NonZeroU32)>,
    ) -> Result<Expression<'a>, Option<u32>> {
        let ctx = self.ctx;
        self.ctx = ctx.and_in(true).and_decorator(false);
        // Record parentheses in it, as in `((a)) => {}`.
        self.state.cover_depth += 1;
        let expression = match identifier {
            Some(identifier) => self.parse_cover_identifier_expression(identifier),
            None => self.parse_assignment_expression_or_higher(),
        };
        self.state.cover_depth -= 1;
        self.ctx = ctx;

        if !self.at(Kind::RParen) || self.fatal_error.is_some() {
            self.state.cover_elements.push(CoverElement::Expression(expression));
            return Err(None);
        }
        let end = self.prev_token_end;
        self.bump_any(); // `)`

        // `(1) => {}` is reported by refining the list to parameters.
        if self.at(Kind::Arrow) {
            self.state.cover_elements.push(CoverElement::Expression(expression));
            return Err(Some(end));
        }
        Ok(self.finish_parenthesized_expression(
            cover.start,
            cover.parenthesized_base,
            expression,
            no_side_effects_comments,
        ))
    }

    /// `AsyncArrowFunction`[In, Yield, Await] :
    ///     async [no `LineTerminator` here] `AsyncArrowBindingIdentifier`[?Yield] [no `LineTerminator` here] => `AsyncConciseBody`[?In]
    ///     `CoverCallExpressionAndAsyncArrowHead`[?Yield, ?Await] [no `LineTerminator` here] => `AsyncConciseBody`[?In]
    ///
    /// Returns the expression, and whether it is an arrow function. If it is not, it is the
    /// call `async(…)`. Returns `None` with the cursor at `async` if `async` is an identifier.
    pub(super) fn try_parse_async_arrow_function_or_call(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<(Expression<'a>, bool)> {
        debug_assert!(self.at(Kind::Async));
        if self.fatal_error.is_some() {
            return None;
        }
        let start = self.cur_start();
        let checkpoint = self.checkpoint();
        // An escaped `async` is only an error in an arrow function.
        let is_escaped = self.cur_token().escaped();
        let (async_span, async_name) = self.parse_identifier_kind(Kind::Ident);

        let token = self.cur_token();
        if !token.is_on_new_line() {
            let kind = token.kind();
            if kind == Kind::LParen {
                return self.parse_async_call_or_arrow_function(
                    checkpoint,
                    async_span,
                    async_name,
                    is_escaped,
                    allow_return_type_in_arrow_function,
                );
            }
            if kind == Kind::Ident {
                // `async a => …`
                let ident = self.parse_plain_binding_identifier();
                if self.at(Kind::Arrow) {
                    if is_escaped {
                        self.error(diagnostics::escaped_keyword(async_span));
                    }
                    let arrow = self.parse_simple_arrow_function_expression(
                        start,
                        ident,
                        /* async */ true,
                        allow_return_type_in_arrow_function,
                    );
                    return Some((arrow, true));
                }
            } else if kind == Kind::LAngle {
                self.rewind(checkpoint);
                return self
                    .try_parse_generic_arrow_function_expression(
                        allow_return_type_in_arrow_function,
                    )
                    .map(|arrow| (arrow, true));
            } else if kind.is_binding_identifier() {
                // `async yield => …`
                self.bump_any();
                if self.at(Kind::Arrow) {
                    self.rewind(checkpoint);
                    let arrow = self.parse_async_simple_arrow_function_expression(
                        allow_return_type_in_arrow_function,
                    );
                    return Some((arrow, true));
                }
            }
        }
        self.rewind(checkpoint);
        None
    }

    /// `CoverCallExpressionAndAsyncArrowHead`[Yield, Await] :
    ///     `MemberExpression`[?Yield, ?Await] `Arguments`[?Yield, ?Await]
    ///
    /// The cursor is at `(`, and `checkpoint` is at `async`.
    fn parse_async_call_or_arrow_function(
        &mut self,
        checkpoint: ParserCheckpoint<'a>,
        async_span: Span,
        async_name: Ident<'a>,
        is_escaped: bool,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<(Expression<'a>, bool)> {
        let start = async_span.start;
        // `await` is reserved in the parameters of an async arrow function, but not in the
        // arguments of a call outside of async code: the list may have to be parsed again.
        let may_parse_again = !self.ctx.has_await();
        let had_module_syntax = self.module_record_builder.has_module_syntax();
        let deferred_script_errors_len = self.deferred_script_errors.len();
        let lparen = self.cur_start();
        self.bump_any(); // `(`
        let mut cover = Cover::new(
            start,
            lparen,
            self.state.cover_elements.len(),
            self.state.cover_parenthesized.len(),
            /* is_async */ true,
        );
        let base = cover.base;

        let outer_mentions_await = mem::replace(&mut self.state.mentions_await, false);
        let list = self.parse_cover_list(lparen, base, true, false, None);
        let mentions_await = self.state.mentions_await;
        self.state.mentions_await |= outer_mentions_await;

        if may_parse_again
            && mentions_await
            && (matches!(list, CoverList::Parameters(_))
                || self.fatal_error.is_some()
                || self.at(Kind::Arrow)
                || (self.is_ts && self.at(Kind::Colon)))
        {
            // Discard what `rewind` does not restore, and parse the list as parameters.
            self.state.cover_elements.truncate(base);
            self.state.cover_initialized_name.retain(|&start, _| start < lparen);
            self.state.trailing_commas.retain(|&start, _| start < lparen);
            self.deferred_script_errors.truncate(deferred_script_errors_len);
            if !had_module_syntax {
                self.module_record_builder.unset_module_syntax();
            }
            self.rewind(checkpoint);
            return self
                .parse_possible_parenthesized_arrow_function_expression(
                    allow_return_type_in_arrow_function,
                )
                .map(|arrow| (arrow, true));
        }

        match list {
            CoverList::Parameters(params) => {
                if is_escaped {
                    self.error(diagnostics::escaped_keyword(async_span));
                }
                let arrow = self.parse_cover_parameters_arrow_function(
                    cover,
                    params,
                    allow_return_type_in_arrow_function,
                );
                return Some((arrow, true));
            }
            CoverList::Undecided { trailing_comma, .. } => cover.trailing_comma = trailing_comma,
        }

        if self.fatal_error.is_some() {
            return Some((self.abandon_cover(cover), false));
        }
        if let Some(arrow) =
            self.try_parse_cover_arrow_function(cover, allow_return_type_in_arrow_function)
        {
            if is_escaped {
                self.error(diagnostics::escaped_keyword(async_span));
            }
            return Some((arrow, true));
        }
        let callee = Expression::new_identifier(async_span, async_name, self);
        Some((self.cover_elements_to_call_expression(cover, callee), false))
    }

    /// Parse the rest of an arrow function after an undecided cover list, if `=>` or
    /// a TypeScript return type follows.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn try_parse_cover_arrow_function(
        &mut self,
        cover: Cover,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<Expression<'a>> {
        let kind = self.cur_kind();
        if kind == Kind::Arrow {
            let params = self.cover_elements_to_parameters(cover, self.end_span(cover.lparen));
            return Some(self.parse_arrow_function_rest(
                cover.start,
                cover.is_async,
                params,
                None,
                allow_return_type_in_arrow_function,
            ));
        }
        if kind == Kind::Colon && self.is_ts {
            return self.try_parse_cover_arrow_function_with_return_type(
                cover,
                allow_return_type_in_arrow_function,
            );
        }
        None
    }

    /// Parse `: Type => body` after an undecided cover list, if it is not the `:` of
    /// a conditional expression.
    fn try_parse_cover_arrow_function_with_return_type(
        &mut self,
        cover: Cover,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<Expression<'a>> {
        let params_span = self.end_span(cover.lparen);

        if !cover.is_async && self.state.cover_elements.len() == cover.base {
            // `(): T => …`: `()` is not an expression, so `:` starts a return type.
            let params = self.cover_elements_to_parameters(cover, params_span);
            let return_type = self.parse_ts_return_type_annotation();
            return Some(self.parse_arrow_function_rest(
                cover.start,
                false,
                params,
                return_type,
                allow_return_type_in_arrow_function,
            ));
        }

        // `case (a.b): c => d` and `a ? (b + 1) : c => d` are not arrow functions.
        if self.state.not_parenthesized_arrow.contains(&cover.start)
            || !self.is_cover_parameter_list(cover)
        {
            return None;
        }

        let checkpoint = self.checkpoint();
        let return_type = self.parse_cover_return_type(cover.is_async);
        'reject: {
            if self.fatal_error.is_some() || !self.at(Kind::Arrow) {
                break 'reject;
            }
            self.expect_arrow();

            if allow_return_type_in_arrow_function {
                let params = self.cover_elements_to_parameters(cover, params_span);
                let head = ArrowFunctionHead {
                    type_parameters: None,
                    params,
                    return_type,
                    r#async: cover.is_async,
                    start: cover.start,
                };
                return Some(self.parse_arrow_function_expression_body(
                    head,
                    allow_return_type_in_arrow_function,
                ));
            }

            // In the consequent of a conditional expression, a return type needs another `:`
            // after the body, see `parse_possible_parenthesized_arrow_function_expression`.
            let body =
                self.parse_arrow_function_body(cover.is_async, allow_return_type_in_arrow_function);
            if !self.at(Kind::Colon) {
                break 'reject;
            }
            let params = self.cover_elements_to_parameters(cover, params_span);
            return Some(Expression::new_arrow_function_expression(
                self.end_span(cover.start),
                cover.is_async,
                None,
                params,
                return_type,
                body,
                self,
            ));
        }

        self.state.not_parenthesized_arrow.insert(cover.start);
        self.rewind(checkpoint);
        None
    }

    /// Parse the rest of an arrow function whose parameters were decided inside `( … )`.
    fn parse_cover_parameters_arrow_function(
        &mut self,
        cover: Cover,
        params: ArenaBox<'a, FormalParameters<'a>>,
        allow_return_type_in_arrow_function: bool,
    ) -> Expression<'a> {
        self.state.cover_parenthesized.truncate(cover.parenthesized_base);
        let return_type =
            if self.is_ts { self.parse_cover_return_type(cover.is_async) } else { None };
        self.parse_arrow_function_rest(
            cover.start,
            cover.is_async,
            params,
            return_type,
            allow_return_type_in_arrow_function,
        )
    }

    /// Parse the return type of an arrow function at `:`.
    fn parse_cover_return_type(
        &mut self,
        is_async: bool,
    ) -> Option<ArenaBox<'a, TSTypeAnnotation<'a>>> {
        let has_await = self.ctx.has_await();
        self.ctx = self.ctx.union_await_if(is_async);
        let return_type = self.parse_ts_return_type_annotation();
        self.ctx = self.ctx.and_await(has_await);
        return_type
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

        self.parse_arrow_function_rest(
            start,
            r#async,
            params,
            None,
            allow_return_type_in_arrow_function,
        )
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

    /// Parse an arrow function with type parameters, if `<` starts one: `<T>(a: T) => a`.
    ///
    /// The cursor is at `<`, or at an `async` that is followed by `<` on the same line.
    pub(super) fn try_parse_generic_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<Expression<'a>> {
        let kind = self.lookahead(|p| {
            p.bump(Kind::Async);
            p.is_generic_arrow_function_expression()
        });
        match kind {
            Tristate::False => None,
            Tristate::True => Some(self.parse_parenthesized_arrow_function_expression(
                allow_return_type_in_arrow_function,
            )),
            Tristate::Maybe => self.parse_possible_parenthesized_arrow_function_expression(
                allow_return_type_in_arrow_function,
            ),
        }
    }

    /// Whether the `<` at the cursor starts the type parameters of an arrow function.
    fn is_generic_arrow_function_expression(&mut self) -> Tristate {
        debug_assert!(self.at(Kind::LAngle));
        self.bump_any(); // `<`
        let second = self.cur_kind();

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

        // `<T>` may start an arrow function, which is then tried first. The exception is
        // a type assertion that the next tokens give away, because trying to parse them as
        // type parameters and parameters is certain to fail: the name of a type parameter is
        // followed by `,`, `=`, `extends` or `>`, not by the rest of a type as in `<A.B>a`,
        // `<A[]>a`, `<A | B>a` and `<A<B>>a`, and the `>` of type parameters is followed by
        // `(`, not by an operand as in `<T>a` and `<any>{}`.
        if !self.is_ts || second == Kind::Const {
            return Tristate::Maybe;
        }
        self.bump_any(); // the name
        match self.cur_kind() {
            // `<T,` is not a type, so it can only start type parameters.
            Kind::Comma => Tristate::True,
            Kind::Dot | Kind::LBrack | Kind::Pipe | Kind::Amp | Kind::LAngle => Tristate::False,
            Kind::RAngle => {
                self.bump_any(); // `>`
                let operand = self.cur_kind();
                if operand.is_identifier_name()
                    || operand.is_literal()
                    || operand.is_template_start_of_tagged_template()
                    || matches!(operand, Kind::LCurly | Kind::LBrack)
                {
                    Tristate::False
                } else {
                    Tristate::Maybe
                }
            }
            _ => Tristate::Maybe,
        }
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

    fn expect_arrow(&mut self) {
        if self.at(Kind::Arrow) && self.cur_token().is_on_new_line() {
            self.error(diagnostics::lineterminator_before_arrow(self.cur_token().span()));
        }
        self.expect(Kind::Arrow);
    }

    /// [ConciseBody](https://tc39.es/ecma262/#prod-ConciseBody)
    ///     [lookahead ≠ {] `ExpressionBody`[?In, ~Await]
    ///     { `FunctionBody`[~Yield, ~Await] }
    /// `ExpressionBody`[In, Await] :
    ///     `AssignmentExpression`[?In, ~Yield, ?Await]
    #[inline]
    fn parse_arrow_function_body(
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

    /// Parse `=> ConciseBody` after the parameters and return type.
    fn parse_arrow_function_rest(
        &mut self,
        start: u32,
        r#async: bool,
        params: ArenaBox<'a, FormalParameters<'a>>,
        return_type: Option<ArenaBox<'a, TSTypeAnnotation<'a>>>,
        allow_return_type_in_arrow_function: bool,
    ) -> Expression<'a> {
        self.expect_arrow();
        let head = ArrowFunctionHead { type_parameters: None, params, return_type, r#async, start };
        self.parse_arrow_function_expression_body(head, allow_return_type_in_arrow_function)
    }

    fn parse_arrow_function_expression_body(
        &mut self,
        arrow_function_head: ArrowFunctionHead<'a>,
        allow_return_type_in_arrow_function: bool,
    ) -> Expression<'a> {
        let ArrowFunctionHead { type_parameters, params, return_type, r#async, start } =
            arrow_function_head;
        let body = self.parse_arrow_function_body(r#async, allow_return_type_in_arrow_function);
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
}
