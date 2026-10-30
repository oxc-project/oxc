//! Shared parenthesized expression, call, and arrow-parameter prefixes.

use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};
use oxc_str::Ident;
use smallvec::SmallVec;

use super::{Tristate, arrow::ArrowOrExpression};
use crate::{Context, ParserConfig as Config, ParserImpl, diagnostics, lexer::Kind};

struct CoverItem<'a> {
    span: Span,
    expression_end: u32,
    expression: CoverExpression<'a>,
    spread: bool,
    optional: bool,
    type_annotation: Option<ArenaBox<'a, TSTypeAnnotation<'a>>>,
    initializer: Option<ArenaBox<'a, Expression<'a>>>,
}

enum CoverExpression<'a> {
    Identifier(Span, Ident<'a>),
    Expression(Expression<'a>),
}

// The shared prefix is temporary; only the selected AST representation belongs in the arena.
type CoverItems<'a> = SmallVec<[CoverItem<'a>; 2]>;

impl<'a, C: Config> ParserImpl<'a, C> {
    pub(super) fn parse_cover_parenthesized_arrow(
        &mut self,
        allow_return_type: bool,
    ) -> Option<ArrowOrExpression<'a>> {
        let is_async = self.at(Kind::Async);
        if !(self.at(Kind::LParen)
            || (is_async
                && !self.cur_token().escaped()
                && self.lexer.peek_token().kind() == Kind::LParen))
            || self.state.not_parenthesized_arrow.contains(&self.cur_start())
        {
            return None;
        }

        self.parse_cover_parenthesized_arrow_impl(allow_return_type)
    }

    pub(super) fn parse_cover_generic_async_arrow(
        &mut self,
        allow_return_type: bool,
    ) -> Option<ArrowOrExpression<'a>> {
        if !self.is_ts || !self.at(Kind::Async) || self.cur_token().escaped() {
            return None;
        }
        let next = self.lexer.peek_token();
        if next.kind() != Kind::LAngle
            || next.is_on_new_line()
            || self.state.not_parenthesized_arrow.contains(&self.cur_start())
        {
            return None;
        }
        self.parse_cover_parenthesized_arrow_impl(allow_return_type)
    }

    fn parse_cover_parenthesized_arrow_impl(
        &mut self,
        allow_return_type: bool,
    ) -> Option<ArrowOrExpression<'a>> {
        let is_async = self.at(Kind::Async);
        let checkpoint_start = self.cur_start();
        let checkpoint = self.checkpoint_with_error_recovery();
        let errors = self.errors_count();
        let deferred_errors = self.deferred_script_errors.len();
        let module_checkpoint = self.module_record_builder.expression_checkpoint();
        let encountered_await = self.state.encountered_await_identifier;
        let ctx = self.ctx;
        let depth = self.state.cover_depth;
        let parameter_start = self.state.cover_parameter_start;
        let outer_async = self.state.cover_async;
        let outer_await = self.state.cover_async_await;
        self.state.cover_depth += 1;
        self.state.cover_async |= is_async;
        self.state.cover_async_await = false;

        let result = self.parse_cover_parenthesized_prefix(allow_return_type, ctx, outer_async);

        self.ctx = ctx;
        self.state.cover_depth = depth;
        self.state.cover_parameter_start = parameter_start;
        self.state.cover_async = outer_async;
        self.state.cover_async_await |= outer_await;
        if result.is_none() || self.fatal_error.is_some() || self.errors_count() != errors {
            self.deferred_script_errors.truncate(deferred_errors);
            self.module_record_builder.rewind_expression(module_checkpoint);
            self.state.encountered_await_identifier = encountered_await;
            self.state.discard_cover_state_from(checkpoint_start);
            self.rewind(checkpoint);
            return None;
        }
        result
    }

    fn parse_cover_parenthesized_prefix(
        &mut self,
        allow_return_type: bool,
        outer_ctx: Context,
        outer_async: bool,
    ) -> Option<ArrowOrExpression<'a>> {
        let start = self.cur_start();
        let comments = self.lexer.trivia_builder.previous_token_no_side_effects_comments();
        let async_name = self.at(Kind::Async).then(|| self.parse_identifier_kind(Kind::Ident));
        let is_async = async_name.is_some();
        let generic_prefix = if is_async && self.at(Kind::LAngle) {
            Some(self.try_parse_simple_generic_cover_prefix()?)
        } else {
            None
        };
        let generic_arrow = generic_prefix
            .as_ref()
            .map_or(Tristate::Maybe, |prefix| prefix.arrow_kind(self.source_type.is_jsx()));
        let opening = self.cur_token().span();
        self.expect(Kind::LParen);
        let expression_start = self.cur_start();
        let mut items = CoverItems::new();
        let mut trailing_comma = None;
        let mut parameter_syntax = false;
        self.ctx = outer_ctx.union(Context::In).difference(Context::Decorator);
        while !self.at(Kind::RParen) && !self.has_fatal_error() {
            let item_start = self.cur_start();
            let spread = self.eat(Kind::Dot3);
            let previous_parameter = self.state.cover_parameter_start;
            self.state.cover_parameter_start = Some(self.cur_start());
            // Keep a plain identifier neutral until the suffix selects a binding or expression.
            // Defaults can retain their initializer without building an assignment expression.
            let (expression, mut initializer) = if self.at(Kind::Ident)
                && matches!(
                    self.lexer.peek_token().kind(),
                    Kind::Comma | Kind::RParen | Kind::Colon | Kind::Eq
                ) {
                let (span, name) = self.parse_identifier_kind(Kind::Ident);
                let initializer = if self.eat(Kind::Eq) {
                    let initializer = self.parse_assignment_expression_or_higher();
                    Some(self.alloc(initializer))
                } else {
                    None
                };
                (CoverExpression::Identifier(span, name), initializer)
            } else {
                (CoverExpression::Expression(self.parse_assignment_expression_or_higher()), None)
            };
            self.state.cover_parameter_start = previous_parameter;
            if self.fatal_error.is_some() {
                return None;
            }
            let expression_end = self.prev_token_end;
            let optional = self.is_ts && self.eat(Kind::Question);
            let type_annotation = if self.is_ts { self.parse_ts_type_annotation() } else { None };
            let typed = optional || type_annotation.is_some();
            // An annotation must precede a parameter's default initializer.
            if typed
                && (initializer.is_some()
                    || matches!(
                        expression,
                        CoverExpression::Expression(Expression::AssignmentExpression(_))
                    ))
            {
                return None;
            }
            parameter_syntax |= typed;
            if typed && self.eat(Kind::Eq) {
                let expression = self.parse_assignment_expression_or_higher();
                initializer = Some(self.alloc(expression));
            }
            items.push(CoverItem {
                span: self.end_span(item_start),
                expression_end,
                expression,
                spread,
                optional,
                type_annotation,
                initializer,
            });
            if self.at(Kind::RParen) || self.fatal_error.is_some() {
                break;
            }
            let comma = self.cur_token().span();
            self.expect(Kind::Comma);
            if self.at(Kind::RParen) {
                trailing_comma = Some(comma);
                break;
            }
        }
        let expression_end = self.prev_token_end;
        self.expect(Kind::RParen);
        let params_span = self.end_span(opening.start);
        self.ctx = outer_ctx;
        self.state.cover_depth -= 1;
        self.state.cover_async = outer_async;
        // Await can change the grammar of nested parameters, not just their validity.
        // Retain the established parser for these contextual cases during this rollout.
        if is_async && self.state.cover_async_await {
            return None;
        }
        if self.fatal_error.is_some() {
            return None;
        }

        let mut return_type = None;
        let mut parsed_body = None;
        if self.is_ts && self.at(Kind::Colon) && generic_arrow != Tristate::False {
            // Keep the shared prefix. Only the genuinely ambiguous suffix is speculative.
            let suffix_start = self.cur_start();
            let suffix = self.checkpoint_with_error_recovery();
            let suffix_module = self.module_record_builder.expression_checkpoint();
            let suffix_deferred_errors = self.deferred_script_errors.len();
            let suffix_encountered_await = self.state.encountered_await_identifier;
            self.ctx = outer_ctx.union_await_if(is_async);
            let candidate_type = self.parse_ts_return_type_annotation();
            self.ctx = outer_ctx;
            if self.fatal_error.is_none() && self.at(Kind::Arrow) {
                if self.cur_token().is_on_new_line() {
                    self.error(diagnostics::lineterminator_before_arrow(self.cur_token().span()));
                }
                self.bump_any();
                let candidate_body = self.parse_cover_arrow_body(is_async, allow_return_type);
                if allow_return_type || generic_arrow == Tristate::True || self.at(Kind::Colon) {
                    return_type = candidate_type;
                    parsed_body = Some(candidate_body);
                } else {
                    self.module_record_builder.rewind_expression(suffix_module);
                    self.deferred_script_errors.truncate(suffix_deferred_errors);
                    self.state.encountered_await_identifier = suffix_encountered_await;
                    self.state.discard_cover_state_from(suffix_start);
                    self.rewind(suffix);
                }
            } else {
                self.module_record_builder.rewind_expression(suffix_module);
                self.deferred_script_errors.truncate(suffix_deferred_errors);
                self.state.encountered_await_identifier = suffix_encountered_await;
                self.state.discard_cover_state_from(suffix_start);
                self.rewind(suffix);
            }
        }

        if generic_arrow != Tristate::False && (parsed_body.is_some() || self.at(Kind::Arrow)) {
            let type_parameters =
                generic_prefix.map(|prefix| self.simple_generic_cover_type_parameters(&prefix));
            self.ctx = outer_ctx.union_await_if(is_async);
            let params = self.finish_cover_parameters(params_span, items, trailing_comma);
            self.ctx = outer_ctx;
            let body = if let Some(body) = parsed_body {
                body
            } else {
                if self.cur_token().is_on_new_line() {
                    self.error(diagnostics::lineterminator_before_arrow(self.cur_token().span()));
                }
                self.expect(Kind::Arrow);
                self.parse_cover_arrow_body(is_async, allow_return_type)
            };
            return Some(ArrowOrExpression::Arrow(Expression::new_arrow_function_expression(
                self.end_span(start),
                is_async,
                type_parameters,
                params,
                return_type,
                body,
                self,
            )));
        }

        if parameter_syntax || generic_arrow == Tristate::True {
            return None;
        }
        let expression = if let Some((span, name)) = async_name {
            let callee = Expression::new_identifier(span, name, self);
            let type_arguments =
                generic_prefix.map(|prefix| self.simple_generic_cover_type_arguments(&prefix));
            let mut arguments = ArenaVec::with_capacity_in(items.len(), self);
            for item in items {
                let span = item.span;
                let spread = item.spread;
                let expression = self.finish_cover_item_expression(item);
                arguments.push(if spread {
                    Argument::SpreadElement(SpreadElement::boxed(span, expression, self))
                } else {
                    Argument::from(expression)
                });
            }
            Expression::new_call_expression(
                self.end_span(start),
                callee,
                type_arguments,
                arguments,
                false,
                self,
            )
        } else {
            if items.is_empty() || trailing_comma.is_some() || items.iter().any(|item| item.spread)
            {
                return None;
            }
            let expression = if items.len() == 1 {
                self.finish_cover_item_expression(items.remove(0))
            } else {
                let mut expressions = ArenaVec::with_capacity_in(items.len(), self);
                for item in items {
                    expressions.push(self.finish_cover_item_expression(item));
                }
                Expression::new_sequence_expression(
                    Span::new(expression_start, expression_end),
                    expressions,
                    self,
                )
            };
            self.finish_parenthesized_expression(start, expression, comments)
        };
        Some(ArrowOrExpression::Expression(expression))
    }

    fn finish_cover_item_expression(&self, item: CoverItem<'a>) -> Expression<'a> {
        match item.expression {
            CoverExpression::Identifier(span, name) => {
                let identifier = IdentifierReference::boxed(span, name, self);
                if let Some(initializer) = item.initializer {
                    Expression::new_assignment_expression(
                        Span::new(span.start, item.expression_end),
                        AssignmentOperator::Assign,
                        AssignmentTarget::AssignmentTargetIdentifier(identifier),
                        initializer.unbox(),
                        self,
                    )
                } else {
                    Expression::Identifier(identifier)
                }
            }
            CoverExpression::Expression(expression) => expression,
        }
    }

    fn finish_cover_parameters(
        &mut self,
        span: Span,
        items: CoverItems<'a>,
        trailing_comma: Option<Span>,
    ) -> ArenaBox<'a, FormalParameters<'a>> {
        let count = items.len();
        let mut parameters = ArenaVec::with_capacity_in(count, self);
        let mut rest = None;
        let mut has_optional = false;
        for (index, item) in items.into_iter().enumerate() {
            if item.spread {
                if index + 1 != count || item.optional || item.initializer.is_some() {
                    return self.fatal_error(diagnostics::unexpected_token(item.span));
                }
                if let Some(comma) = trailing_comma
                    && !self.ctx.has_ambient()
                {
                    self.error(diagnostics::rest_element_trailing_comma(comma));
                }
                let pattern = match item.expression {
                    CoverExpression::Identifier(span, name) => {
                        BindingPattern::new_binding_identifier(span, name, self)
                    }
                    CoverExpression::Expression(expression) => {
                        self.cover_binding_pattern(expression)
                    }
                };
                if let BindingPattern::AssignmentPattern(pattern) = &pattern {
                    self.error(diagnostics::a_rest_element_cannot_have_an_initializer(
                        pattern.span,
                    ));
                }
                let element = BindingRestElement::new(
                    Span::new(item.span.start, item.expression_end),
                    pattern,
                    self,
                );
                rest = Some(FormalParameterRest::boxed(
                    item.span,
                    ArenaVec::new_in(self),
                    element,
                    item.type_annotation,
                    self,
                ));
            } else {
                let (pattern, initializer) = match item.expression {
                    CoverExpression::Identifier(span, name) => {
                        (BindingPattern::new_binding_identifier(span, name, self), None)
                    }
                    CoverExpression::Expression(expression) => {
                        self.cover_arrow_parameter(expression)
                    }
                };
                let initializer = item.initializer.or(initializer);
                if item.optional {
                    has_optional = true;
                    if initializer.is_some() {
                        self.error(
                            diagnostics::a_parameter_cannot_have_question_mark_and_initializer(
                                pattern.span(),
                            ),
                        );
                    }
                } else if has_optional && initializer.is_none() {
                    self.error(diagnostics::required_parameter_after_optional_parameter(item.span));
                }
                parameters.push(FormalParameter::new(
                    item.span,
                    ArenaVec::new_in(self),
                    pattern,
                    item.type_annotation,
                    initializer,
                    item.optional,
                    None,
                    false,
                    false,
                    self,
                ));
            }
        }
        FormalParameters::boxed(
            span,
            FormalParameterKind::ArrowFormalParameters,
            parameters,
            rest,
            self,
        )
    }
}
