//! Cover Grammar for Destructuring Assignment

use oxc_allocator::{Box, Vec};
use oxc_ast::{NONE, ast::*};
use oxc_span::{GetSpan, Span};

use crate::{Context, ParserConfig as Config, ParserImpl, diagnostics, lexer::Kind};

pub trait CoverGrammar<'a, T, C: Config>: Sized {
    fn cover(value: T, p: &mut ParserImpl<'a, C>) -> Self;
}

impl<'a, C: Config> CoverGrammar<'a, Expression<'a>, C> for AssignmentTarget<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        match expr {
            Expression::ArrayExpression(array_expr) => {
                let pat = ArrayAssignmentTarget::cover(array_expr.unbox(), p);
                AssignmentTarget::ArrayAssignmentTarget(p.alloc(pat))
            }
            Expression::ObjectExpression(object_expr) => {
                let pat = ObjectAssignmentTarget::cover(object_expr.unbox(), p);
                AssignmentTarget::ObjectAssignmentTarget(p.alloc(pat))
            }
            _ => AssignmentTarget::from(SimpleAssignmentTarget::cover(expr, p)),
        }
    }
}

impl<'a, C: Config> CoverGrammar<'a, Expression<'a>, C> for SimpleAssignmentTarget<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        match expr {
            Expression::Identifier(ident) => {
                SimpleAssignmentTarget::AssignmentTargetIdentifier(ident)
            }
            match_member_expression!(Expression) => {
                let member_expr = expr.into_member_expression();
                SimpleAssignmentTarget::from(member_expr)
            }
            Expression::ParenthesizedExpression(expr) => {
                let span = expr.span;
                match expr.unbox().expression {
                    Expression::ObjectExpression(_) | Expression::ArrayExpression(_) => {
                        p.fatal_error(diagnostics::invalid_assignment(span))
                    }
                    expr => SimpleAssignmentTarget::cover(expr, p),
                }
            }
            Expression::TSAsExpression(expr) => match expr.expression.get_inner_expression() {
                Expression::Identifier(_)
                | Expression::StaticMemberExpression(_)
                | Expression::ComputedMemberExpression(_)
                | Expression::PrivateFieldExpression(_) => {
                    SimpleAssignmentTarget::TSAsExpression(expr)
                }
                _ => p.fatal_error(diagnostics::invalid_assignment(expr.span())),
            },
            Expression::TSSatisfiesExpression(expr) => {
                match expr.expression.get_inner_expression() {
                    Expression::Identifier(_)
                    | Expression::StaticMemberExpression(_)
                    | Expression::ComputedMemberExpression(_)
                    | Expression::PrivateFieldExpression(_) => {
                        SimpleAssignmentTarget::TSSatisfiesExpression(expr)
                    }
                    _ => p.fatal_error(diagnostics::invalid_assignment(expr.span())),
                }
            }
            Expression::TSNonNullExpression(expr) => match expr.expression.get_inner_expression() {
                Expression::Identifier(_)
                | Expression::StaticMemberExpression(_)
                | Expression::ComputedMemberExpression(_)
                | Expression::PrivateFieldExpression(_) => {
                    SimpleAssignmentTarget::TSNonNullExpression(expr)
                }
                _ => p.fatal_error(diagnostics::invalid_assignment(expr.span())),
            },
            Expression::TSTypeAssertion(expr) => match expr.expression.get_inner_expression() {
                Expression::Identifier(_)
                | Expression::StaticMemberExpression(_)
                | Expression::ComputedMemberExpression(_)
                | Expression::PrivateFieldExpression(_) => {
                    SimpleAssignmentTarget::TSTypeAssertion(expr)
                }
                _ => p.fatal_error(diagnostics::invalid_assignment(expr.span())),
            },
            Expression::TSInstantiationExpression(expr) => {
                p.fatal_error(diagnostics::invalid_lhs_assignment(expr.span()))
            }
            expr => p.fatal_error(diagnostics::invalid_assignment(expr.span())),
        }
    }
}

impl<'a, C: Config> CoverGrammar<'a, ArrayExpression<'a>, C> for ArrayAssignmentTarget<'a> {
    fn cover(expr: ArrayExpression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        let mut elements = p.ast.vec();
        let mut rest = None;

        let len = expr.elements.len();
        for (i, elem) in expr.elements.into_iter().enumerate() {
            match elem {
                match_expression!(ArrayExpressionElement) => {
                    let expr = elem.into_expression();
                    let target = AssignmentTargetMaybeDefault::cover(expr, p);
                    elements.push(Some(target));
                }
                ArrayExpressionElement::SpreadElement(elem) => {
                    if i == len - 1 {
                        let span = elem.span;
                        let argument = elem.unbox().argument;
                        if !matches!(
                            argument.get_inner_expression(),
                            Expression::Identifier(_)
                                | Expression::ArrayExpression(_)
                                | Expression::ObjectExpression(_)
                                | Expression::StaticMemberExpression(_)
                                | Expression::ComputedMemberExpression(_)
                                | Expression::PrivateFieldExpression(_)
                        ) {
                            p.error(diagnostics::invalid_rest_assignment_target(argument.span()));
                        }
                        let target = AssignmentTarget::cover(argument, p);
                        rest = Some(p.ast.alloc_assignment_target_rest(span, target));
                        if let Some(span) = p.state.trailing_commas.get(&expr.span.start) {
                            p.error(diagnostics::rest_element_trailing_comma(*span));
                        }
                    } else {
                        let error = diagnostics::spread_last_element(elem.span);
                        return p.fatal_error(error);
                    }
                }
                ArrayExpressionElement::Elision(_) => elements.push(None),
            }
        }

        p.ast.array_assignment_target(expr.span, elements, rest)
    }
}

impl<'a, C: Config> CoverGrammar<'a, Expression<'a>, C> for AssignmentTargetMaybeDefault<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        match expr {
            Expression::AssignmentExpression(assignment_expr) => {
                if assignment_expr.operator != AssignmentOperator::Assign {
                    p.error(diagnostics::invalid_assignment_target_default_value_operator(
                        assignment_expr.span,
                    ));
                }
                let target = AssignmentTargetWithDefault::cover(assignment_expr.unbox(), p);
                AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(p.alloc(target))
            }
            expr => {
                let target = AssignmentTarget::cover(expr, p);
                AssignmentTargetMaybeDefault::from(target)
            }
        }
    }
}

impl<'a, C: Config> CoverGrammar<'a, AssignmentExpression<'a>, C>
    for AssignmentTargetWithDefault<'a>
{
    fn cover(expr: AssignmentExpression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        p.ast.assignment_target_with_default(expr.span, expr.left, expr.right)
    }
}

impl<'a, C: Config> CoverGrammar<'a, ObjectExpression<'a>, C> for ObjectAssignmentTarget<'a> {
    fn cover(expr: ObjectExpression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        let mut properties = p.ast.vec();
        let mut rest = None;

        let len = expr.properties.len();
        for (i, elem) in expr.properties.into_iter().enumerate() {
            match elem {
                ObjectPropertyKind::ObjectProperty(property) => {
                    let target = AssignmentTargetProperty::cover(property.unbox(), p);
                    properties.push(target);
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    if i == len - 1 {
                        let span = spread.span;
                        let argument = spread.unbox().argument;
                        if !matches!(
                            argument.get_inner_expression(),
                            Expression::Identifier(_)
                                | Expression::StaticMemberExpression(_)
                                | Expression::ComputedMemberExpression(_)
                                | Expression::PrivateFieldExpression(_)
                        ) {
                            p.error(diagnostics::invalid_rest_assignment_target(argument.span()));
                        }
                        if let Some(span) = p.state.trailing_commas.get(&expr.span.start) {
                            p.error(diagnostics::rest_element_trailing_comma(*span));
                        }
                        let target = AssignmentTarget::cover(argument, p);
                        rest = Some(p.ast.alloc_assignment_target_rest(span, target));
                    } else {
                        return p.fatal_error(diagnostics::spread_last_element(spread.span));
                    }
                }
            }
        }

        p.ast.object_assignment_target(expr.span, properties, rest)
    }
}

impl<'a, C: Config> CoverGrammar<'a, ObjectProperty<'a>, C> for AssignmentTargetProperty<'a> {
    fn cover(property: ObjectProperty<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        if property.shorthand {
            let binding = match property.key {
                PropertyKey::StaticIdentifier(ident) => {
                    let ident = ident.unbox();
                    p.ast.identifier_reference(ident.span, ident.name)
                }
                _ => return p.unexpected(),
            };
            // convert `CoverInitializedName`
            let init = p.state.cover_initialized_name.remove(&property.span.start).map(|e| e.right);
            p.ast.assignment_target_property_assignment_target_property_identifier(
                property.span,
                binding,
                init,
            )
        } else {
            let binding = AssignmentTargetMaybeDefault::cover(property.value, p);
            p.ast.assignment_target_property_assignment_target_property_property(
                property.span,
                property.key,
                binding,
                property.computed,
            )
        }
    }
}

// Refinement of the cover grammar `CoverParenthesizedExpressionAndArrowParameterList` into
// `ArrowFormalParameters` — the arrow side of the same refinement that `CoverGrammar` above performs
// for destructuring assignment (`AssignmentTarget`):
// ```text
// // refine `( Expression )` to params when `=>` follows:
// ArrowFormalParameters[Yield, Await] : ( UniqueFormalParameters[?Yield, ?Await] )
// UniqueFormalParameters : FormalParameters
// ```
// <https://tc39.es/ecma262/#sec-arrow-function-definitions>
//
// The whole `( … )` head is parsed once as a `ParenthesizedExpression`/sequence (the
// `PrimaryExpression` refinement), then — when `=>` follows — re-refined here into params. The
// element shapes mirror the `Expression -> AssignmentTarget` `CoverGrammar` above, but target
// `BindingPattern`/`FormalParameter`: a *top-level* `a = init` becomes a parameter with
// `FormalParameter::initializer`, while a *nested* `a = init` (inside `[ … ]` / `{ … }`) becomes a
// `BindingPattern::AssignmentPattern` — the same split as `AssignmentTargetMaybeDefault` vs
// `AssignmentTarget`.
impl<'a, C: Config> ParserImpl<'a, C> {
    /// Refine a parsed cover head `( Expression )` / `( Expr , Expr , … )` into
    /// `ArrowFormalParameters : ( UniqueFormalParameters )`. Each top-level element of the (possibly
    /// sequence) expression becomes one `FormalParameter`. `params_span` is the `( .. )` span
    /// (matching the direct param-parse path).
    pub(crate) fn refine_arrow_params(
        &mut self,
        expr: Expression<'a>,
        params_span: Span,
    ) -> Box<'a, FormalParameters<'a>> {
        let mut items = self.ast.vec();
        match expr {
            Expression::SequenceExpression(seq) => {
                for element in seq.unbox().expressions {
                    let param = self.formal_parameter_from_expression(element);
                    items.push(param);
                }
            }
            expr => {
                let param = self.formal_parameter_from_expression(expr);
                items.push(param);
            }
        }
        // A `...rest` stashed by the cover paren (`(a, ...rest) =>`) is the last parameter.
        let rest = self.state.cover_paren_rest.take();
        self.ast.alloc_formal_parameters(
            params_span,
            FormalParameterKind::ArrowFormalParameters,
            items,
            rest,
        )
    }

    /// A single top-level arrow parameter refined from one cover-grammar element. A top-level
    /// `a = init` (`BindingElement : SingleNameBinding Initializer`) puts `init` on
    /// [`FormalParameter::initializer`], not in a nested [`AssignmentPattern`].
    fn formal_parameter_from_expression(&mut self, expr: Expression<'a>) -> FormalParameter<'a> {
        match expr {
            Expression::AssignmentExpression(assign)
                if assign.operator == AssignmentOperator::Assign =>
            {
                let span = assign.span;
                let assign = assign.unbox();
                let pattern = self.binding_pattern_from_assignment_target(assign.left);
                self.ast.formal_parameter(
                    span,
                    self.ast.vec(),
                    pattern,
                    NONE,
                    Some(self.ast.alloc(assign.right)),
                    false,
                    None,
                    false,
                    false,
                )
            }
            expr => {
                let span = expr.span();
                let pattern = self.binding_pattern_from_expression(expr);
                self.ast.plain_formal_parameter(span, pattern)
            }
        }
    }

    /// Refine an `Expression` element into a `BindingPattern` (nested context: a nested `a = init`
    /// becomes [`BindingPattern::AssignmentPattern`]).
    fn binding_pattern_from_expression(&mut self, expr: Expression<'a>) -> BindingPattern<'a> {
        match expr {
            Expression::Identifier(ident) => {
                self.ast.binding_pattern_binding_identifier(ident.span, ident.name)
            }
            Expression::ArrayExpression(array) => self.binding_pattern_from_array(array.unbox()),
            Expression::ObjectExpression(object) => {
                self.binding_pattern_from_object(object.unbox())
            }
            Expression::AssignmentExpression(assign)
                if assign.operator == AssignmentOperator::Assign =>
            {
                let span = assign.span;
                let assign = assign.unbox();
                let left = self.binding_pattern_from_assignment_target(assign.left);
                self.ast.binding_pattern_assignment_pattern(span, left, assign.right)
            }
            // e.g. `(a, b.c) =>`, `({ a: 1 }) =>` — not a valid binding target.
            expr => {
                self.error(diagnostics::invalid_assignment(expr.span()));
                self.binding_pattern_from_invalid(expr.span())
            }
        }
    }

    /// Refine an `AssignmentTarget` (the eagerly-refined LHS of a cover `a = init`) into a
    /// `BindingPattern`. `a`/`[ … ]`/`{ … }` only; a member/other target is not a binding.
    fn binding_pattern_from_assignment_target(
        &mut self,
        target: AssignmentTarget<'a>,
    ) -> BindingPattern<'a> {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                self.ast.binding_pattern_binding_identifier(ident.span, ident.name)
            }
            AssignmentTarget::ArrayAssignmentTarget(array) => {
                self.binding_pattern_from_array_target(array.unbox())
            }
            AssignmentTarget::ObjectAssignmentTarget(object) => {
                self.binding_pattern_from_object_target(object.unbox())
            }
            target => {
                self.error(diagnostics::invalid_assignment(target.span()));
                self.binding_pattern_from_invalid(target.span())
            }
        }
    }

    fn binding_pattern_from_array(&mut self, array: ArrayExpression<'a>) -> BindingPattern<'a> {
        let span = array.span;
        let mut elements = self.ast.vec();
        let mut rest = None;
        let len = array.elements.len();
        for (i, element) in array.elements.into_iter().enumerate() {
            match element {
                ArrayExpressionElement::Elision(_) => elements.push(None),
                ArrayExpressionElement::SpreadElement(spread) => {
                    let spread = spread.unbox();
                    let argument = self.binding_pattern_from_expression(spread.argument);
                    // `[...x = 1]` — a rest element cannot have an initializer.
                    if let BindingPattern::AssignmentPattern(pat) = &argument {
                        self.error(diagnostics::a_rest_element_cannot_have_an_initializer(
                            pat.span,
                        ));
                    }
                    let rest_element = self.ast.binding_rest_element(spread.span, argument);
                    if i == len - 1 {
                        // `[a, ...b,]` — a trailing comma may not follow a rest element.
                        if let Some(comma) = self.state.trailing_commas.get(&span.start) {
                            self.error(diagnostics::rest_element_trailing_comma(*comma));
                        }
                        rest = Some(self.alloc(rest_element));
                    } else {
                        self.error(diagnostics::binding_rest_element_last(spread.span));
                    }
                }
                match_expression!(ArrayExpressionElement) => {
                    let expr = element.into_expression();
                    let pattern = self.binding_pattern_from_expression(expr);
                    elements.push(Some(pattern));
                }
            }
        }
        self.ast.binding_pattern_array_pattern(span, elements, rest)
    }

    fn binding_pattern_from_array_target(
        &mut self,
        array: ArrayAssignmentTarget<'a>,
    ) -> BindingPattern<'a> {
        let span = array.span;
        let mut elements = self.ast.vec();
        for element in array.elements {
            match element {
                Some(element) => {
                    elements.push(Some(self.binding_pattern_from_maybe_default(element)))
                }
                None => elements.push(None),
            }
        }
        let rest = array.rest.map(|rest| {
            let argument = self.binding_pattern_from_assignment_target(rest.unbox().target);
            self.alloc(self.ast.binding_rest_element(span, argument))
        });
        self.ast.binding_pattern_array_pattern(span, elements, rest)
    }

    fn binding_pattern_from_maybe_default(
        &mut self,
        element: AssignmentTargetMaybeDefault<'a>,
    ) -> BindingPattern<'a> {
        match element {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) => {
                let with_default = with_default.unbox();
                let left = self.binding_pattern_from_assignment_target(with_default.binding);
                self.ast.binding_pattern_assignment_pattern(
                    with_default.span,
                    left,
                    with_default.init,
                )
            }
            // Any non-default variant is a shared `AssignmentTarget` variant.
            target => self.binding_pattern_from_assignment_target(target.into_assignment_target()),
        }
    }

    fn binding_pattern_from_object(&mut self, object: ObjectExpression<'a>) -> BindingPattern<'a> {
        let span = object.span;
        let mut properties = self.ast.vec();
        let mut rest = None;
        let len = object.properties.len();
        for (i, property) in object.properties.into_iter().enumerate() {
            match property {
                ObjectPropertyKind::ObjectProperty(property) => {
                    properties.push(self.binding_property_from_object_property(property.unbox()));
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    let spread = spread.unbox();
                    let argument = self.binding_pattern_from_expression(spread.argument);
                    let rest_element = self.ast.binding_rest_element(spread.span, argument);
                    if i == len - 1 {
                        rest = Some(self.alloc(rest_element));
                    } else {
                        self.error(diagnostics::binding_rest_element_last(spread.span));
                    }
                }
            }
        }
        self.ast.binding_pattern_object_pattern(span, properties, rest)
    }

    fn binding_pattern_from_object_target(
        &mut self,
        object: ObjectAssignmentTarget<'a>,
    ) -> BindingPattern<'a> {
        let span = object.span;
        let mut properties = self.ast.vec();
        for property in object.properties {
            properties.push(self.binding_property_from_target_property(property));
        }
        let rest = object.rest.map(|rest| {
            let argument = self.binding_pattern_from_assignment_target(rest.unbox().target);
            self.alloc(self.ast.binding_rest_element(span, argument))
        });
        self.ast.binding_pattern_object_pattern(span, properties, rest)
    }

    fn binding_property_from_object_property(
        &mut self,
        property: ObjectProperty<'a>,
    ) -> BindingProperty<'a> {
        if property.shorthand {
            // `{ a }` or the `CoverInitializedName` `{ a = init }` (the assignment is held aside in
            // `cover_initialized_name`; consuming it here means it is no longer an early error).
            let span = property.span;
            let key = property.key;
            let (binding_span, name) = match &key {
                PropertyKey::StaticIdentifier(ident) => (ident.span, ident.name),
                _ => {
                    self.error(diagnostics::invalid_assignment(span));
                    return self.ast.binding_property(
                        span,
                        key,
                        self.binding_pattern_from_invalid(span),
                        true,
                        false,
                    );
                }
            };
            let mut value = self.ast.binding_pattern_binding_identifier(binding_span, name);
            if let Some(assign) = self.state.cover_initialized_name.remove(&span.start) {
                value =
                    self.ast.binding_pattern_assignment_pattern(assign.span, value, assign.right);
            }
            self.ast.binding_property(span, key, value, true, false)
        } else {
            let span = property.span;
            let computed = property.computed;
            let key = property.key;
            let value = self.binding_pattern_from_expression(property.value);
            self.ast.binding_property(span, key, value, false, computed)
        }
    }

    fn binding_property_from_target_property(
        &mut self,
        property: AssignmentTargetProperty<'a>,
    ) -> BindingProperty<'a> {
        match property {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident) => {
                let ident = ident.unbox();
                let span = ident.span;
                let name = ident.binding.name;
                let key = PropertyKey::StaticIdentifier(
                    self.ast.alloc_identifier_name(ident.binding.span, name),
                );
                let mut value =
                    self.ast.binding_pattern_binding_identifier(ident.binding.span, name);
                if let Some(init) = ident.init {
                    value = self.ast.binding_pattern_assignment_pattern(span, value, init);
                }
                self.ast.binding_property(span, key, value, true, false)
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                let property = property.unbox();
                let computed = property.computed;
                let value = self.binding_pattern_from_maybe_default(property.binding);
                self.ast.binding_property(property.span, property.name, value, false, computed)
            }
        }
    }

    /// A placeholder binding for an unrefinable element (the real error is already reported).
    fn binding_pattern_from_invalid(&mut self, span: Span) -> BindingPattern<'a> {
        self.ast.binding_pattern_binding_identifier(span, "")
    }

    /// A top-level `:` after a cover element (`(a, b: T) =>`) makes the head unambiguously params:
    /// a `: Type` has no parenthesized-expression form, and any conditional `?:` was already
    /// consumed inside the element. Refine the already-parsed `prefix` (the last element is the one
    /// directly before the `:`) into params and parse the remaining params (`?`/`: T`/`= init`/rest)
    /// directly — no speculation. Returns the params items and an optional trailing rest.
    pub(crate) fn finish_cover_typed_params(
        &mut self,
        prefix: Vec<'a, Expression<'a>>,
    ) -> (Vec<'a, FormalParameter<'a>>, Option<Box<'a, FormalParameterRest<'a>>>) {
        let mut items = self.ast.vec();
        let mut rest = None;
        let last_idx = prefix.len() - 1;
        for (i, expr) in prefix.into_iter().enumerate() {
            if i == last_idx {
                // The element directly before the `:` — convert its pattern, then parse the
                // `?`/`: Type`/`= init` tail that the expression parser stopped short of.
                let start = expr.span().start;
                let pattern = self.binding_pattern_from_expression(expr);
                let param = self.finish_typed_param(start, pattern);
                items.push(param);
            } else {
                items.push(self.formal_parameter_from_expression(expr));
            }
        }
        // The params after the first typed one are not yet parsed; parse them directly.
        loop {
            if !self.at(Kind::Comma) {
                break;
            }
            self.bump_any(); // `,`
            if self.at(Kind::RParen) {
                break; // trailing comma is allowed in params
            }
            if self.at(Kind::Dot3) {
                rest = Some(self.parse_cover_rest_element());
                break;
            }
            let start = self.start_span();
            let pattern = self.parse_binding_pattern();
            let param = self.finish_typed_param(start, pattern);
            items.push(param);
        }
        (items, rest)
    }

    /// Parse the `[?] [: Type] [= init]` tail of a single arrow parameter onto an already-parsed
    /// `pattern`. Mirrors the non-modifier portion of `parse_formal_parameter_with_decorators`.
    fn finish_typed_param(
        &mut self,
        start: u32,
        pattern: BindingPattern<'a>,
    ) -> FormalParameter<'a> {
        let optional = self.is_ts && self.eat(Kind::Question);
        let type_annotation = self.parse_ts_type_annotation();
        let init = if self.eat(Kind::Eq) {
            let init =
                self.context_add(Context::In, ParserImpl::parse_assignment_expression_or_higher);
            if optional {
                self.error(diagnostics::a_parameter_cannot_have_question_mark_and_initializer(
                    pattern.span(),
                ));
            }
            Some(self.ast.alloc(init))
        } else {
            None
        };
        self.ast.formal_parameter(
            self.end_span(start),
            self.ast.vec(),
            pattern,
            type_annotation,
            init,
            optional,
            None,
            false,
            false,
        )
    }
}
