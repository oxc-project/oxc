//! [`CoverParenthesizedExpressionAndArrowParameterList`] and [`CoverCallExpressionAndAsyncArrowHead`].
//!
//! The elements between the parentheses are parsed once, and refined to parameters or
//! expressions once the token after `)` is known. In TypeScript, the first construct only a
//! parameter can have (`a: T`, `a?`, modifiers) decides the list as parameters.
//!
//! [`CoverParenthesizedExpressionAndArrowParameterList`]: https://tc39.es/ecma262/#prod-CoverParenthesizedExpressionAndArrowParameterList
//! [`CoverCallExpressionAndAsyncArrowHead`]: https://tc39.es/ecma262/#prod-CoverCallExpressionAndAsyncArrowHead

use std::{mem, num::NonZeroU32};

use oxc_allocator::{ArenaBox, ArenaVec, Dummy, GetAllocator};
use oxc_ast::ast::*;
use oxc_span::{GetSpan, SPAN, Span};
use oxc_str::Ident;

use super::{FunctionKind, grammar::CoverGrammar};
use crate::{
    Context, ParserConfig as Config, ParserImpl,
    diagnostics::{self, ParserDiagnostic},
    lexer::Kind,
};

/// An element of a cover list whose meaning is not decided yet.
///
/// Identifiers are by far the most common element. They are kept as their span and name, so
/// that the single node they become is only allocated once it is known which node that is.
pub enum CoverElement<'a> {
    /// `a`: a `BindingIdentifier`, or an `IdentifierReference`.
    Identifier { span: Span, name: Ident<'a> },
    /// `a = b`: a parameter with an initializer, or an assignment. `end` is the end of `b`.
    IdentifierWithInitializer { span: Span, name: Ident<'a>, end: u32, initializer: Expression<'a> },
    /// `{ a } = b`, `[a] = b`: a destructuring parameter with an initializer, or a destructuring
    /// assignment. `literal` is the object or array literal, and `end` is the end of `b`.
    LiteralWithInitializer { literal: Expression<'a>, end: u32, initializer: Expression<'a> },
    /// Any other `AssignmentExpression`.
    Expression(Expression<'a>),
    /// `...a` in `async(…)`: an argument, or the rest parameter of an async arrow function.
    Spread(ArenaBox<'a, SpreadElement<'a>>),
}

/// An identifier that starts a cover list element, consumed before the element is known.
#[derive(Clone, Copy)]
pub(super) struct CoverIdentifier<'a> {
    pub(super) span: Span,
    pub(super) name: Ident<'a>,
    pure_comments: Option<(u32, NonZeroU32)>,
    no_side_effects_comments: Option<(u32, NonZeroU32)>,
}

/// A construct that only a parameter can have, found while parsing a cover list.
pub(super) enum CoverParameter<'a> {
    /// At the first token of a parameter: `(...a)`, `(public a)`.
    Start,
    /// After the binding of a parameter that has a type annotation or is optional:
    /// `(a: T)`, `(a?)`, `({ a }: T)`.
    Binding { start: u32, pattern: BindingPattern<'a>, optional: bool },
    /// After the rest parameter of an async arrow function: `async (...a: T[])`.
    Rest(ArenaBox<'a, FormalParameterRest<'a>>),
}

/// How a cover list ended. Its `)` has been consumed.
pub(super) enum CoverList<'a> {
    /// Nothing between the parentheses decided what the elements are.
    /// `end` is the end of the last token before `)`.
    Undecided { trailing_comma: Option<u32>, end: u32 },
    /// The list could only be the parameters of an arrow function.
    Parameters(ArenaBox<'a, FormalParameters<'a>>),
}

/// An undecided cover list, whose elements are in [`ParserState::cover_elements`].
///
/// [`ParserState::cover_elements`]: crate::state::ParserState::cover_elements
#[derive(Clone, Copy)]
pub(super) struct Cover {
    /// Start of the expression: `(`, or `async`.
    pub(super) start: u32,
    /// Start of `(`.
    pub(super) lparen: u32,
    /// Length of `ParserState::cover_elements` before this list.
    pub(super) base: usize,
    /// Length of `ParserState::cover_parenthesized` before this list.
    pub(super) parenthesized_base: usize,
    /// Start of the `,` before `)`, if there is one.
    pub(super) trailing_comma: Option<u32>,
    pub(super) is_async: bool,
}

impl Cover {
    pub(super) fn new(
        start: u32,
        lparen: u32,
        base: usize,
        parenthesized_base: usize,
        is_async: bool,
    ) -> Self {
        Self { start, lparen, base, parenthesized_base, trailing_comma: None, is_async }
    }
}

pub(super) type CoverCheck<'a> = Result<(), ParserDiagnostic<'a>>;

impl<'a, C: Config> ParserImpl<'a, C> {
    /// Parse the elements of a cover list up to and including its `)`.
    ///
    /// `CoverParenthesizedExpressionAndArrowParameterList`[Yield, Await] :
    ///     ( `Expression`[+In, ?Yield, ?Await] )
    ///     ( `Expression`[+In, ?Yield, ?Await] , )
    ///     ( )
    ///     ( ... `BindingIdentifier`[?Yield, ?Await] )
    ///     ( ... `BindingPattern`[?Yield, ?Await] )
    ///     ( `Expression`[+In, ?Yield, ?Await] , ... `BindingIdentifier`[?Yield, ?Await] )
    ///     ( `Expression`[+In, ?Yield, ?Await] , ... `BindingPattern`[?Yield, ?Await] )
    ///
    /// The elements are pushed to `ParserState::cover_elements`, unless the list turns out to
    /// be a parameter list, which is then completed.
    ///
    /// If `first_parsed`, the first element is already on the stack and the cursor is after it.
    /// If `first_identifier`, the identifier that starts the first element has been consumed.
    pub(super) fn parse_cover_list(
        &mut self,
        lparen: u32,
        base: usize,
        is_async: bool,
        first_parsed: bool,
        mut first_identifier: Option<CoverIdentifier<'a>>,
    ) -> CoverList<'a> {
        let ctx = self.ctx;
        self.ctx = ctx.and_in(true).and_decorator(false);
        self.state.cover_depth += 1;

        let mut trailing_comma = None;
        let parameter = 'list: {
            if !first_parsed && first_identifier.is_none() && self.at(Kind::RParen) {
                break 'list None;
            }
            let mut is_first = !first_parsed;
            let mut parse_element = !first_parsed;
            loop {
                if parse_element {
                    let parameter = if let Some(identifier) = first_identifier.take() {
                        self.parse_cover_identifier_element_rest(identifier)
                    } else if self.at(Kind::Ident) {
                        let identifier = self.parse_cover_identifier();
                        self.parse_cover_identifier_element_rest(identifier)
                    } else {
                        self.parse_cover_element(is_async, is_first)
                    };
                    if parameter.is_some() {
                        break 'list parameter;
                    }
                }
                parse_element = true;
                if !self.at(Kind::Comma) || self.fatal_error.is_some() {
                    break 'list None;
                }
                let comma = self.cur_start();
                self.bump_any(); // `,`
                if self.at(Kind::RParen) {
                    trailing_comma = Some(comma);
                    break 'list None;
                }
                is_first = false;
            }
        };

        self.state.cover_depth -= 1;
        let list = if let Some(parameter) = parameter {
            CoverList::Parameters(self.parse_cover_parameters(lparen, base, parameter))
        } else {
            let end = self.prev_token_end;
            if self.at(Kind::RParen) {
                self.bump_any();
            } else {
                self.expect_cover_closing(lparen);
            }
            CoverList::Undecided { trailing_comma, end }
        };
        self.ctx = ctx;
        list
    }

    #[cold]
    #[inline(never)]
    fn expect_cover_closing(&mut self, lparen: u32) {
        let error = diagnostics::expect_closing_or_separator(
            Kind::RParen.to_str(),
            Kind::Comma.to_str(),
            self.cur_kind().to_str(),
            self.cur_token().span(),
            Span::sized(lparen, 1),
        );
        self.set_fatal_error(error);
        // Like `expect`, leave the cursor at the end of the file after a fatal error.
        self.bump_any();
    }

    /// Parse a cover list element that starts with an identifier which is not a keyword in
    /// any context.
    ///
    /// The token after the identifier usually decides what the element is, without looking
    /// ahead: `,` and `)` end it, `=` starts an initializer or assignment, `=>` makes it the
    /// parameter of a nested arrow function, and in TypeScript `:` and `?` can only follow
    /// a parameter.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn parse_cover_identifier_element_rest(
        &mut self,
        identifier: CoverIdentifier<'a>,
    ) -> Option<CoverParameter<'a>> {
        let CoverIdentifier { span, name, pure_comments, no_side_effects_comments } = identifier;
        let start = span.start;

        let element = match self.cur_kind() {
            Kind::Comma | Kind::RParen => CoverElement::Identifier { span, name },
            Kind::Eq => {
                self.bump_any(); // `=`
                let initializer = self.parse_assignment_expression_or_higher();
                let end = self.prev_token_end;
                CoverElement::IdentifierWithInitializer { span, name, end, initializer }
            }
            Kind::Arrow => {
                let binding = BindingIdentifier::new(span, name, self);
                let mut arrow = self.parse_simple_arrow_function_expression(
                    start, binding, /* async */ false,
                    /* allow_return_type_in_arrow_function */ true,
                );
                if let Some(comments) = no_side_effects_comments
                    && let Expression::ArrowFunctionExpression(function) = &mut arrow
                {
                    function.pure = true;
                    self.lexer.trivia_builder.mark_no_side_effects_comments_applied(comments);
                }
                CoverElement::Expression(arrow)
            }
            Kind::Colon if self.is_ts => {
                let pattern = BindingPattern::new_binding_identifier(span, name, self);
                return Some(CoverParameter::Binding { start, pattern, optional: false });
            }
            Kind::Question if self.is_ts => {
                let question_span = self.cur_token().span();
                self.bump_any(); // `?`
                // `(a?: T)`, `(a?, b)`, `(a?)`, `(a? = b)`. A conditional expression cannot
                // continue with any of these tokens.
                if matches!(self.cur_kind(), Kind::Colon | Kind::Comma | Kind::RParen | Kind::Eq) {
                    let pattern = BindingPattern::new_binding_identifier(span, name, self);
                    return Some(CoverParameter::Binding { start, pattern, optional: true });
                }
                let test = Expression::new_identifier(span, name, self);
                CoverElement::Expression(self.parse_cover_conditional_expression(
                    start,
                    test,
                    question_span,
                ))
            }
            _ => {
                let primary = Expression::new_identifier(span, name, self);
                CoverElement::Expression(self.parse_assignment_expression_rest(
                    start,
                    primary,
                    pure_comments,
                ))
            }
        };
        self.state.cover_elements.push(element);
        None
    }

    /// Consume the current identifier, which is not a keyword in any context.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(super) fn parse_cover_identifier(&mut self) -> CoverIdentifier<'a> {
        // Read the annotations before consuming the identifier, which resets them.
        let pure_comments = self.lexer.trivia_builder.previous_token_pure_comments();
        let no_side_effects_comments =
            self.lexer.trivia_builder.previous_token_no_side_effects_comments();
        let (span, name) = self.parse_identifier_kind(Kind::Ident);
        CoverIdentifier { span, name, pure_comments, no_side_effects_comments }
    }

    /// Parse the rest of the expression that starts with `identifier`, which cannot be
    /// a `BindingElement`: `(a.b)`, `(a + b)`, `(a(b), c)`.
    pub(super) fn parse_cover_identifier_expression(
        &mut self,
        identifier: CoverIdentifier<'a>,
    ) -> Expression<'a> {
        let primary = Expression::new_identifier(identifier.span, identifier.name, self);
        self.parse_assignment_expression_rest(
            identifier.span.start,
            primary,
            identifier.pure_comments,
        )
    }

    #[inline(never)]
    fn parse_cover_conditional_expression(
        &mut self,
        start: u32,
        test: Expression<'a>,
        question_span: Span,
    ) -> Expression<'a> {
        self.parse_conditional_expression_after_question(
            start,
            test,
            question_span,
            /* allow_return_type_in_arrow_function */ true,
        )
    }

    /// Parse a cover list element that does not start with a plain identifier.
    fn parse_cover_element(
        &mut self,
        is_async: bool,
        is_first: bool,
    ) -> Option<CoverParameter<'a>> {
        let kind = self.cur_kind();
        match kind {
            Kind::LCurly | Kind::LBrack => return self.parse_cover_literal_element(),
            Kind::Dot3 => return self.parse_cover_spread_element(is_async),
            // `async`, `await` and `yield` start expressions that the code below does not
            // parse, but they can also name a parameter. Only an optional parameter has to
            // be recognized up front: `(async?: T) => …`.
            Kind::Async | Kind::Await | Kind::Yield => {
                if self.is_ts
                    && self.lexer.peek_token().kind() == Kind::Question
                    && self.lookahead(|p| {
                        p.bump_any(); // the identifier
                        p.bump_any(); // `?`
                        matches!(p.cur_kind(), Kind::Colon | Kind::Comma | Kind::RParen | Kind::Eq)
                    })
                {
                    let start = self.cur_start();
                    let ident = self.parse_binding_identifier();
                    self.bump_any(); // `?`
                    let pattern = BindingPattern::BindingIdentifier(self.alloc(ident));
                    return Some(CoverParameter::Binding { start, pattern, optional: true });
                }
            }
            _ => {
                // `(public a) => …`. Arrow function parameters cannot have modifiers, but two
                // adjacent identifiers are not an expression either, and the modifier is what
                // an error should be about. `as` and `satisfies` are the only identifiers that
                // may follow another one in an expression.
                if kind.is_modifier_kind() {
                    let next = self.lexer.peek_token().kind();
                    if next.is_binding_identifier() && !matches!(next, Kind::As | Kind::Satisfies) {
                        return Some(CoverParameter::Start);
                    }
                }
                if kind.is_identifier() {
                    let identifier = self.parse_cover_identifier();
                    return self.parse_cover_identifier_element_rest(identifier);
                }
            }
        }

        let start = self.cur_start();
        let expression = self.parse_assignment_expression_or_higher();
        if self.is_ts && self.at(Kind::Colon) {
            match &expression {
                // `(this: T) => …`. Arrow functions cannot have a `this` parameter.
                Expression::ThisExpression(this) if is_first && this.span.start == start => {
                    self.parse_ts_type_annotation();
                    self.error(diagnostics::ts_arrow_function_this_parameter(self.end_span(start)));
                    if !self.at(Kind::RParen) {
                        self.expect(Kind::Comma);
                    }
                    return Some(CoverParameter::Start);
                }
                // `(async: T) => …`
                Expression::Identifier(ident) if ident.span.start == start => {
                    let pattern =
                        BindingPattern::new_binding_identifier(ident.span, ident.name, self);
                    return Some(CoverParameter::Binding { start, pattern, optional: false });
                }
                _ => {}
            }
        }
        self.state.cover_elements.push(CoverElement::Expression(expression));
        None
    }

    /// Parse a cover list element that starts with an object or array literal, which is
    /// a destructuring parameter if a type annotation or `?` follows it.
    fn parse_cover_literal_element(&mut self) -> Option<CoverParameter<'a>> {
        let pure_comments = self.lexer.trivia_builder.previous_token_pure_comments();
        let start = self.cur_start();
        let literal = if self.at(Kind::LCurly) {
            Expression::ObjectExpression(self.parse_object_expression())
        } else {
            self.parse_array_expression()
        };

        let expression = match self.cur_kind() {
            Kind::Comma | Kind::RParen => literal,
            Kind::Eq => {
                // `({ a } = b)`. The literal is only refined once it is known whether this is
                // a parameter or an assignment, instead of to an assignment target first.
                self.bump_any(); // `=`
                let initializer = self.parse_assignment_expression_or_higher();
                let end = self.prev_token_end;
                let element = CoverElement::LiteralWithInitializer { literal, end, initializer };
                self.state.cover_elements.push(element);
                return None;
            }
            Kind::Colon if self.is_ts => {
                // `({ a }: T) => …`
                let pattern = self.cover_literal_to_binding_pattern(literal);
                return Some(CoverParameter::Binding { start, pattern, optional: false });
            }
            Kind::Question if self.is_ts => {
                let question_span = self.cur_token().span();
                self.bump_any(); // `?`
                if matches!(self.cur_kind(), Kind::Colon | Kind::Comma | Kind::RParen | Kind::Eq) {
                    // `({ a }?: T) => …`
                    let pattern = self.cover_literal_to_binding_pattern(literal);
                    return Some(CoverParameter::Binding { start, pattern, optional: true });
                }
                self.parse_cover_conditional_expression(start, literal, question_span)
            }
            _ => self.parse_assignment_expression_rest(start, literal, pure_comments),
        };
        self.state.cover_elements.push(CoverElement::Expression(expression));
        None
    }

    fn cover_literal_to_binding_pattern(&mut self, literal: Expression<'a>) -> BindingPattern<'a> {
        if let Err(error) = self.check_cover_binding(&literal) {
            return self.fatal_error(error);
        }
        self.cover_expression_to_binding_pattern(literal)
    }

    /// Parse a cover list element that starts with `...`.
    fn parse_cover_spread_element(&mut self, is_async: bool) -> Option<CoverParameter<'a>> {
        if !is_async {
            // A parenthesized expression has no spread elements: this is a rest parameter.
            return Some(CoverParameter::Start);
        }
        if self.is_ts && self.lookahead(Self::is_optional_rest_parameter) {
            // `async (...a?: T[]) => …` is an error, reported on the rest parameter.
            return Some(CoverParameter::Start);
        }
        let spread = self.parse_spread_element();
        if self.is_ts && self.at(Kind::Colon) && self.fatal_error.is_none() {
            // `async (...a: T[]) => …`
            return Some(CoverParameter::Rest(self.cover_spread_to_parameter_rest(spread, true)));
        }
        self.state.cover_elements.push(CoverElement::Spread(spread));
        None
    }

    /// Whether the tokens from `...` on are `...a?` followed by `:`, `,` or `)`.
    fn is_optional_rest_parameter(&mut self) -> bool {
        self.bump_any(); // `...`
        if !self.cur_kind().is_binding_identifier() {
            return false;
        }
        self.bump_any();
        if !self.at(Kind::Question) {
            return false;
        }
        self.bump_any();
        matches!(self.cur_kind(), Kind::Colon | Kind::Comma | Kind::RParen)
    }

    /// Complete a cover list as the parameters of an arrow function, up to and including `)`.
    ///
    /// The elements before `parameter` are converted to parameters.
    fn parse_cover_parameters(
        &mut self,
        lparen: u32,
        base: usize,
        parameter: CoverParameter<'a>,
    ) -> ArenaBox<'a, FormalParameters<'a>> {
        let (mut current, rest, at_parameter_start) = match parameter {
            CoverParameter::Start => (None, None, true),
            CoverParameter::Binding { start, pattern, optional } => {
                (Some(self.parse_cover_parameter_rest(start, pattern, optional)), None, false)
            }
            CoverParameter::Rest(rest) => (None, Some(rest), false),
        };

        // Nothing is parsed while the elements are converted, so they stay where they are.
        let mut index = base;
        let (items, rest) = self.parse_formal_parameters_list_after::<true>(
            FunctionKind::Expression,
            Span::sized(lparen, 1),
            at_parameter_start,
            rest,
            |p| {
                while index < p.state.cover_elements.len() {
                    let element = p.take_cover_element(index);
                    let is_first = index == base;
                    index += 1;
                    if let Some(param) = p.cover_element_to_parameter(element, is_first) {
                        return Some(param);
                    }
                }
                current.take()
            },
        );
        self.state.cover_elements.truncate(base);
        self.expect(Kind::RParen);
        FormalParameters::boxed(
            self.end_span(lparen),
            FormalParameterKind::ArrowFormalParameters,
            items,
            rest,
            self,
        )
    }

    /// Parse the type annotation and initializer of a parameter whose binding has been parsed.
    fn parse_cover_parameter_rest(
        &mut self,
        start: u32,
        pattern: BindingPattern<'a>,
        optional: bool,
    ) -> FormalParameter<'a> {
        let type_annotation = self.parse_ts_type_annotation();
        let initializer = if self.eat(Kind::Eq) {
            let initializer =
                self.context_add(Context::In, Self::parse_assignment_expression_or_higher);
            if optional {
                self.error(diagnostics::a_parameter_cannot_have_question_mark_and_initializer(
                    pattern.span(),
                ));
            }
            Some(ArenaBox::new_in(initializer, self))
        } else {
            None
        };
        FormalParameter::new(
            self.end_span(start),
            [],
            pattern,
            type_annotation,
            initializer,
            optional,
            None,
            false,
            false,
            self,
        )
    }

    /// Drop the elements of a cover list after a fatal error.
    #[cold]
    #[inline(never)]
    pub(super) fn abandon_cover(&mut self, cover: Cover) -> Expression<'a> {
        self.state.cover_elements.truncate(cover.base);
        self.state.cover_parenthesized.truncate(cover.parenthesized_base);
        Dummy::dummy(self.allocator())
    }

    /// Take an element of a cover list to convert it. Nothing is parsed during a conversion,
    /// so the elements stay on the stack until the list is truncated.
    #[inline]
    fn take_cover_element(&mut self, index: usize) -> CoverElement<'a> {
        let placeholder = CoverElement::Identifier { span: SPAN, name: Ident::empty() };
        mem::replace(&mut self.state.cover_elements[index], placeholder)
    }

    /// Whether the expression that starts at `start` was directly inside parentheses.
    #[inline]
    pub(super) fn is_cover_parenthesized(&self, start: u32) -> bool {
        // Entries below this list's are before its `(`, so they cannot match.
        let parenthesized = &self.state.cover_parenthesized;
        !parenthesized.is_empty() && parenthesized.binary_search(&start).is_ok()
    }

    /// Refine the elements of a cover list to `ArrowFormalParameters`.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(super) fn cover_elements_to_parameters(
        &mut self,
        cover: Cover,
        span: Span,
    ) -> ArenaBox<'a, FormalParameters<'a>> {
        // `() => …` and `(a) => …` are the common cases. Keep them out of the general
        // conversion, which needs a much larger stack frame.
        let (items, rest) = match &self.state.cover_elements[cover.base..] {
            [] => (ArenaVec::new_in(self), None),
            &[CoverElement::Identifier { span, name }] => {
                self.state.cover_elements.truncate(cover.base);
                let pattern = BindingPattern::new_binding_identifier(span, name, self);
                let param = FormalParameter::new_plain(span, pattern, self);
                (ArenaVec::from_value_in(param, self), None)
            }
            _ => self.cover_elements_to_parameter_list(cover),
        };
        self.state.cover_parenthesized.truncate(cover.parenthesized_base);
        FormalParameters::boxed(span, FormalParameterKind::ArrowFormalParameters, items, rest, self)
    }

    /// Refine the elements of a cover list that is not empty to parameters.
    #[inline(never)]
    fn cover_elements_to_parameter_list(
        &mut self,
        cover: Cover,
    ) -> (ArenaVec<'a, FormalParameter<'a>>, Option<ArenaBox<'a, FormalParameterRest<'a>>>) {
        let mut rest = None;
        if matches!(self.state.cover_elements.last(), Some(CoverElement::Spread(_)))
            && let Some(CoverElement::Spread(spread)) = self.state.cover_elements.pop()
        {
            rest = Some(self.cover_spread_to_parameter_rest(spread, false));
            if let Some(comma) = cover.trailing_comma
                && !self.ctx.has_ambient()
            {
                self.error(diagnostics::rest_element_trailing_comma(Span::sized(comma, 1)));
            }
        }

        let end = self.state.cover_elements.len();
        let mut items = ArenaVec::with_capacity_in(end - cover.base, self);
        for index in cover.base..end {
            // `(a, b) => …` is the common case.
            if let CoverElement::Identifier { span, name } = self.state.cover_elements[index] {
                let pattern = BindingPattern::new_binding_identifier(span, name, self);
                items.push(FormalParameter::new_plain(span, pattern, self));
                continue;
            }
            let element = self.take_cover_element(index);
            if let Some(param) = self.cover_element_to_parameter(element, index == cover.base) {
                items.push(param);
            }
        }
        self.state.cover_elements.truncate(cover.base);
        (items, rest)
    }

    /// Refine an element of a cover list to a parameter.
    ///
    /// Returns `None` for a TypeScript `this` parameter, which arrow functions cannot have.
    fn cover_element_to_parameter(
        &mut self,
        element: CoverElement<'a>,
        is_first: bool,
    ) -> Option<FormalParameter<'a>> {
        let param = match element {
            CoverElement::Identifier { span, name } => {
                let pattern = BindingPattern::new_binding_identifier(span, name, self);
                FormalParameter::new_plain(span, pattern, self)
            }
            CoverElement::IdentifierWithInitializer { span, name, end, initializer } => {
                let pattern = BindingPattern::new_binding_identifier(span, name, self);
                self.cover_parameter_with_initializer(
                    Span::new(span.start, end),
                    pattern,
                    initializer,
                )
            }
            CoverElement::LiteralWithInitializer { literal, end, initializer } => {
                if let Err(error) = self.check_cover_binding(&literal) {
                    return Some(self.fatal_error(error));
                }
                let span = Span::new(literal.span().start, end);
                let pattern = self.cover_expression_to_binding_pattern(literal);
                self.cover_parameter_with_initializer(span, pattern, initializer)
            }
            CoverElement::Expression(expression) => {
                if let Err(error) = self.check_cover_binding_element(&expression) {
                    if self.is_ts
                        && is_first
                        && let Expression::ThisExpression(this) = &expression
                    {
                        // `(this) => …`
                        self.error(diagnostics::ts_arrow_function_this_parameter(this.span));
                        return None;
                    }
                    return Some(self.fatal_error(error));
                }
                match expression {
                    Expression::AssignmentExpression(assignment) => {
                        let AssignmentExpression { span, left, right, .. } = assignment.unbox();
                        let pattern = self.cover_assignment_target_to_binding_pattern(left);
                        self.cover_parameter_with_initializer(span, pattern, right)
                    }
                    expression => {
                        let span = expression.span();
                        let pattern = self.cover_expression_to_binding_pattern(expression);
                        FormalParameter::new_plain(span, pattern, self)
                    }
                }
            }
            CoverElement::Spread(spread) => {
                return Some(self.fatal_error(diagnostics::rest_parameter_last(spread.span)));
            }
        };
        Some(param)
    }

    fn cover_parameter_with_initializer(
        &self,
        span: Span,
        pattern: BindingPattern<'a>,
        initializer: Expression<'a>,
    ) -> FormalParameter<'a> {
        let initializer = Some(ArenaBox::new_in(initializer, self));
        FormalParameter::new(span, [], pattern, None, initializer, false, None, false, false, self)
    }

    /// Refine `...a` in the arguments of `async(…)` to the rest parameter of an arrow function,
    /// and parse its type annotation if `typed`.
    fn cover_spread_to_parameter_rest(
        &mut self,
        spread: ArenaBox<'a, SpreadElement<'a>>,
        typed: bool,
    ) -> ArenaBox<'a, FormalParameterRest<'a>> {
        let SpreadElement { span, argument, .. } = spread.unbox();
        if let Err(error) = self.check_cover_binding_element(&argument) {
            return self.fatal_error(error);
        }
        let argument = self.cover_expression_to_binding_element(argument);
        if let BindingPattern::AssignmentPattern(pattern) = &argument {
            self.error(diagnostics::a_rest_parameter_cannot_have_an_initializer(pattern.span));
        }
        let rest = BindingRestElement::new(span, argument, self);
        let type_annotation = if typed { self.parse_ts_type_annotation() } else { None };
        let span = if typed { self.end_span(span.start) } else { span };
        FormalParameterRest::boxed(span, [], rest, type_annotation, self)
    }

    /// Refine the elements of a cover list to a parenthesized expression.
    ///
    /// `first_start` is the start of the first token after `(`, and `end` is the end of the
    /// last token before `)`.
    #[inline]
    pub(super) fn cover_elements_to_parenthesized_expression(
        &mut self,
        cover: Cover,
        first_start: u32,
        end: u32,
        no_side_effects_comments: Option<(u32, NonZeroU32)>,
    ) -> Expression<'a> {
        let len = self.state.cover_elements.len() - cover.base;
        let expression = match self.state.cover_elements.pop() {
            // `(a)` and `(a + b)` are the common cases. Keep them out of the general
            // conversion, which needs a much larger stack frame.
            Some(element) if len == 1 && cover.trailing_comma.is_none() => match element {
                CoverElement::Identifier { span, name } => {
                    Expression::new_identifier(span, name, self)
                }
                CoverElement::Expression(expression) => expression,
                element => self.cover_element_to_expression(element),
            },
            Some(element) if len > 1 && cover.trailing_comma.is_none() => {
                self.state.cover_elements.push(element);
                self.cover_elements_to_sequence_expression(cover, Span::new(first_start, end))
            }
            element => {
                if let Some(element) = element {
                    self.state.cover_elements.push(element);
                }
                return self.invalid_parenthesized_expression(cover);
            }
        };

        self.finish_parenthesized_expression(
            cover.start,
            cover.parenthesized_base,
            expression,
            no_side_effects_comments,
        )
    }

    /// Complete `( expression )`, whose `)` has been consumed.
    #[inline]
    pub(super) fn finish_parenthesized_expression(
        &mut self,
        start: u32,
        parenthesized_base: usize,
        mut expression: Expression<'a>,
        no_side_effects_comments: Option<(u32, NonZeroU32)>,
    ) -> Expression<'a> {
        match &mut expression {
            Expression::ArrowFunctionExpression(arrow_expr) => {
                arrow_expr.pife = true;
                if let Some(comments) = no_side_effects_comments {
                    arrow_expr.pure = true;
                    self.lexer.trivia_builder.mark_no_side_effects_comments_applied(comments);
                }
            }
            Expression::FunctionExpression(func_expr) => {
                func_expr.pife = true;
                if let Some(comments) = no_side_effects_comments {
                    func_expr.pure = true;
                    self.lexer.trivia_builder.mark_no_side_effects_comments_applied(comments);
                }
            }
            _ => {}
        }

        // An enclosing cover list may become a binding pattern, which cannot contain
        // parentheses. What is inside these parentheses no longer matters to it.
        // Record where the innermost expression starts, which is what an enclosing check
        // finds: inner parentheses leave no node without `preserve_parens`, and an assignment
        // target drops them in `((a)) = b`.
        self.state.cover_parenthesized.truncate(parenthesized_base);
        if self.state.cover_depth > 0 {
            self.state.cover_parenthesized.push(expression.without_parentheses().span().start);
        }

        if self.options.preserve_parens {
            Expression::new_parenthesized_expression(self.end_span(start), expression, self)
        } else {
            expression
        }
    }

    /// Refine the elements of a cover list to the sequence expression `a, b`.
    #[inline(never)]
    fn cover_elements_to_sequence_expression(
        &mut self,
        cover: Cover,
        span: Span,
    ) -> Expression<'a> {
        let end = self.state.cover_elements.len();
        let mut expressions = ArenaVec::with_capacity_in(end - cover.base, self);
        for index in cover.base..end {
            if let CoverElement::Identifier { span, name } = self.state.cover_elements[index] {
                expressions.push(Expression::new_identifier(span, name, self));
                continue;
            }
            expressions.push(match self.take_cover_element(index) {
                CoverElement::Expression(expression) => expression,
                element => self.cover_element_to_expression(element),
            });
        }
        self.state.cover_elements.truncate(cover.base);
        Expression::new_sequence_expression(span, expressions, self)
    }

    /// Report `()` and `(a,)`, which are only valid as arrow function parameters.
    #[cold]
    #[inline(never)]
    fn invalid_parenthesized_expression(&mut self, cover: Cover) -> Expression<'a> {
        let is_empty = self.state.cover_elements.len() == cover.base;
        self.state.cover_elements.truncate(cover.base);
        self.state.cover_parenthesized.truncate(cover.parenthesized_base);
        if let Some(comma) = cover.trailing_comma {
            let error = diagnostics::unexpected_trailing_comma(
                "Parenthesized expressions",
                Span::sized(comma, 1),
            );
            return self.fatal_error(error);
        }
        debug_assert!(is_empty);
        if self.at(Kind::LCurly) {
            // `() {}`: most likely an arrow function that is missing its `=>`.
            self.expect(Kind::Arrow);
            return Dummy::dummy(self.allocator());
        }
        self.fatal_error(diagnostics::empty_parenthesized_expression(self.end_span(cover.start)))
    }

    /// Refine the elements of a cover list to the arguments of a call to `async`.
    pub(super) fn cover_elements_to_call_expression(
        &mut self,
        cover: Cover,
        callee: Expression<'a>,
    ) -> Expression<'a> {
        let end = self.state.cover_elements.len();
        let mut arguments = ArenaVec::with_capacity_in(end - cover.base, self);
        for index in cover.base..end {
            arguments.push(match self.take_cover_element(index) {
                CoverElement::Spread(spread) => Argument::SpreadElement(spread),
                element => Argument::from(self.cover_element_to_expression(element)),
            });
        }
        self.state.cover_elements.truncate(cover.base);
        self.state.cover_parenthesized.truncate(cover.parenthesized_base);
        Expression::new_call_expression(
            self.end_span(cover.start),
            callee,
            None::<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>,
            arguments,
            false,
            self,
        )
    }

    /// Refine an element of a cover list to an expression.
    #[inline(never)]
    fn cover_element_to_expression(&mut self, element: CoverElement<'a>) -> Expression<'a> {
        match element {
            CoverElement::Identifier { span, name } => Expression::new_identifier(span, name, self),
            CoverElement::IdentifierWithInitializer { span, name, end, initializer } => {
                let left = AssignmentTarget::AssignmentTargetIdentifier(
                    IdentifierReference::boxed(span, name, self),
                );
                Expression::new_assignment_expression(
                    Span::new(span.start, end),
                    AssignmentOperator::Assign,
                    left,
                    initializer,
                    self,
                )
            }
            CoverElement::LiteralWithInitializer { literal, end, initializer } => {
                let span = Span::new(literal.span().start, end);
                let left = AssignmentTarget::cover(literal, self);
                Expression::new_assignment_expression(
                    span,
                    AssignmentOperator::Assign,
                    left,
                    initializer,
                    self,
                )
            }
            CoverElement::Expression(expression) => expression,
            // Only the arguments of `async(…)` have spread elements, and they keep them.
            CoverElement::Spread(spread) => {
                self.fatal_error(diagnostics::spread_last_element(spread.span))
            }
        }
    }
}
