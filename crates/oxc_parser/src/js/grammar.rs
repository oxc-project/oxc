//! Cover Grammar for Destructuring Assignment

use oxc_allocator::Box;
use oxc_ast::{NONE, ast::*};
use oxc_span::{GetSpan, Span};

use crate::{ParserConfig as Config, ParserImpl, diagnostics};

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

// ============================================================================
// Cover grammar for `CoverParenthesizedExpressionAndArrowParameterList`:
// refine a parsed (parenthesized) `Expression` into arrow `FormalParameters`.
//
// `is_convertible_to_binding` mirrors the *fatal-error frontier* of
// `parse_binding_pattern`/`parse_formal_parameters`: it returns `true` exactly when the old
// speculative arrow-param parse would have *committed* (not rewound). Non-fatal binding errors
// (e.g. `[...a = 1]`) still count as convertible — the refine step emits them.
//
// NOTE: dead until wired into `parse_assignment_expression_or_higher_impl` (a later step).
// ============================================================================

#[expect(dead_code)]
impl<'a, C: Config> ParserImpl<'a, C> {
    /// Can `expr` be refined into a binding pattern without a *fatal* binding error?
    pub(crate) fn is_convertible_to_binding(&self, expr: &Expression<'a>) -> bool {
        match expr {
            Expression::Identifier(_) => true,
            Expression::ArrayExpression(arr) => self.is_array_expr_convertible(arr),
            Expression::ObjectExpression(obj) => self.is_object_expr_convertible(obj),
            Expression::AssignmentExpression(ae) => {
                ae.operator == AssignmentOperator::Assign
                    && self.is_assignment_target_convertible(&ae.left)
            }
            _ => false,
        }
    }

    fn is_array_expr_convertible(&self, arr: &ArrayExpression<'a>) -> bool {
        let len = arr.elements.len();
        arr.elements.iter().enumerate().all(|(i, el)| match el {
            ArrayExpressionElement::Elision(_) => true,
            ArrayExpressionElement::SpreadElement(spread) => {
                // rest must be last; the rest target may itself carry a (non-fatal) `= init`.
                i + 1 == len && self.is_rest_arg_convertible(&spread.argument)
            }
            _ => self.is_convertible_to_binding(el.as_expression().unwrap()),
        })
    }

    fn is_object_expr_convertible(&self, obj: &ObjectExpression<'a>) -> bool {
        let len = obj.properties.len();
        obj.properties.iter().enumerate().all(|(i, prop)| match prop {
            // object rest must be last AND a plain identifier (`{...{a}}` is fatal).
            ObjectPropertyKind::SpreadProperty(spread) => {
                i + 1 == len && matches!(spread.argument, Expression::Identifier(_))
            }
            ObjectPropertyKind::ObjectProperty(p) => {
                // shorthand `{x}` / `{x = 1}` are always convertible (value is the ident);
                // `{a: V}` requires `V` convertible.
                p.shorthand || self.is_convertible_to_binding(&p.value)
            }
        })
    }

    /// A `...rest` argument: a binding target, optionally with a non-fatal `= init`.
    fn is_rest_arg_convertible(&self, expr: &Expression<'a>) -> bool {
        match expr {
            Expression::AssignmentExpression(ae) => {
                ae.operator == AssignmentOperator::Assign
                    && self.is_assignment_target_convertible(&ae.left)
            }
            other => self.is_convertible_to_binding(other),
        }
    }

    /// The `left` of a `=` default (already refined to an `AssignmentTarget`) — convertible to a
    /// binding pattern only when it is an identifier or a (recursively convertible) array/object
    /// destructuring target; member / TS targets are not bindings.
    fn is_assignment_target_convertible(&self, target: &AssignmentTarget<'a>) -> bool {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(_) => true,
            AssignmentTarget::ArrayAssignmentTarget(arr) => {
                let len = arr.elements.len();
                arr.elements.iter().enumerate().all(|(i, el)| match el {
                    None => true,
                    Some(t) => {
                        self.is_maybe_default_target_convertible(t)
                            && (i + 1 == len || !matches!(arr.rest, Some(_)))
                    }
                }) && arr
                    .rest
                    .as_ref()
                    .is_none_or(|r| self.is_assignment_target_convertible(&r.target))
            }
            AssignmentTarget::ObjectAssignmentTarget(obj) => {
                obj.properties.iter().all(|p| match p {
                    AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(_) => true,
                    AssignmentTargetProperty::AssignmentTargetPropertyProperty(pp) => {
                        self.is_maybe_default_target_convertible(&pp.binding)
                    }
                }) && obj.rest.as_ref().is_none_or(|r| {
                    matches!(r.target, AssignmentTarget::AssignmentTargetIdentifier(_))
                })
            }
            _ => false,
        }
    }

    fn is_maybe_default_target_convertible(
        &self,
        target: &AssignmentTargetMaybeDefault<'a>,
    ) -> bool {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(wd) => {
                self.is_assignment_target_convertible(&wd.binding)
            }
            _ => {
                // SAFETY of unwrap: `AssignmentTargetMaybeDefault` inherits `AssignmentTarget`.
                target
                    .as_assignment_target()
                    .is_some_and(|t| self.is_assignment_target_convertible(t))
            }
        }
    }

    // --- refinement (only called after `is_convertible_to_binding` returns true) ---

    /// Refine a (single, non-sequence) parenthesized expression into arrow `FormalParameters`.
    /// `params_span` must be the `( .. )` span (matching the direct param-parse path).
    pub(crate) fn refine_arrow_params(
        &mut self,
        expr: Expression<'a>,
        params_span: Span,
    ) -> Box<'a, FormalParameters<'a>> {
        let param = self.refine_formal_parameter(expr);
        let items = self.ast.vec1(param);
        self.ast.alloc_formal_parameters(
            params_span,
            FormalParameterKind::ArrowFormalParameters,
            items,
            None::<Box<FormalParameterRest>>,
        )
    }

    fn refine_formal_parameter(&mut self, expr: Expression<'a>) -> FormalParameter<'a> {
        match expr {
            // top-level `(a = 1)`: the `=` sink is `FormalParameter.initializer` (an `Expression`),
            // NOT a nested `AssignmentPattern`.
            Expression::AssignmentExpression(ae) if ae.operator == AssignmentOperator::Assign => {
                let ae = ae.unbox();
                let span = ae.span;
                let pattern = self.refine_target_to_binding(ae.left);
                let decorators = self.ast.vec();
                self.ast.formal_parameter(
                    span,
                    decorators,
                    pattern,
                    NONE,
                    Some(ae.right),
                    false,
                    None,
                    false,
                    false,
                )
            }
            other => {
                let pattern = self.refine_expr_to_binding(other);
                let span = pattern.span();
                self.ast.plain_formal_parameter(span, pattern)
            }
        }
    }

    fn refine_expr_to_binding(&mut self, expr: Expression<'a>) -> BindingPattern<'a> {
        match expr {
            Expression::Identifier(id) => {
                self.ast.binding_pattern_binding_identifier(id.span, id.name)
            }
            Expression::ArrayExpression(arr) => self.refine_array_expr(arr.unbox()),
            Expression::ObjectExpression(obj) => self.refine_object_expr(obj.unbox()),
            Expression::AssignmentExpression(ae) => {
                // nested default (e.g. array/object element `x = 1`): sink is `AssignmentPattern`.
                let ae = ae.unbox();
                let left = self.refine_target_to_binding(ae.left);
                self.ast.binding_pattern_assignment_pattern(ae.span, left, ae.right)
            }
            _ => self.unexpected(),
        }
    }

    fn refine_array_expr(&mut self, arr: ArrayExpression<'a>) -> BindingPattern<'a> {
        let span = arr.span;
        let mut elements = self.ast.vec();
        let mut rest = None;
        for el in arr.elements {
            match el {
                ArrayExpressionElement::Elision(_) => elements.push(None),
                ArrayExpressionElement::SpreadElement(spread) => {
                    let spread = spread.unbox();
                    let argument = self.refine_expr_to_binding(spread.argument);
                    if let BindingPattern::AssignmentPattern(ap) = &argument {
                        self.error(diagnostics::a_rest_element_cannot_have_an_initializer(ap.span));
                    }
                    rest = Some(self.alloc(self.ast.binding_rest_element(spread.span, argument)));
                    if let Some(comma) = self.state.trailing_commas.get(&span.start) {
                        self.error(diagnostics::rest_element_trailing_comma(*comma));
                    }
                }
                _ => {
                    let e = el.into_expression();
                    elements.push(Some(self.refine_expr_to_binding(e)));
                }
            }
        }
        self.ast.binding_pattern_array_pattern(span, elements, rest)
    }

    fn refine_object_expr(&mut self, obj: ObjectExpression<'a>) -> BindingPattern<'a> {
        let span = obj.span;
        let mut properties = self.ast.vec();
        let mut rest = None;
        for prop in obj.properties {
            match prop {
                ObjectPropertyKind::SpreadProperty(spread) => {
                    let spread = spread.unbox();
                    let argument = self.refine_expr_to_binding(spread.argument);
                    if let Some(comma) = self.state.trailing_commas.get(&span.start) {
                        self.error(diagnostics::rest_element_trailing_comma(*comma));
                    }
                    rest = Some(self.alloc(self.ast.binding_rest_element(spread.span, argument)));
                }
                ObjectPropertyKind::ObjectProperty(prop) => {
                    properties.push(self.refine_binding_property(prop.unbox()));
                }
            }
        }
        self.ast.binding_pattern_object_pattern(span, properties, rest)
    }

    fn refine_binding_property(&mut self, prop: ObjectProperty<'a>) -> BindingProperty<'a> {
        if prop.shorthand {
            let (key_span, key_name) = match &prop.key {
                PropertyKey::StaticIdentifier(ident) => (ident.span, ident.name),
                _ => return self.unexpected(),
            };
            let mut value = self.ast.binding_pattern_binding_identifier(key_span, key_name);
            // `({ x = 1 }) =>`: drain the recorded `CoverInitializedName` into a nested default.
            if let Some(init) = self.state.cover_initialized_name.remove(&prop.span.start) {
                value = self.ast.binding_pattern_assignment_pattern(prop.span, value, init.right);
            }
            self.ast.binding_property(prop.span, prop.key, value, true, false)
        } else {
            let value = self.refine_expr_to_binding(prop.value);
            self.ast.binding_property(prop.span, prop.key, value, false, prop.computed)
        }
    }

    // --- AssignmentTarget -> BindingPattern (for `=` default lefts) ---

    fn refine_target_to_binding(&mut self, target: AssignmentTarget<'a>) -> BindingPattern<'a> {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(id) => {
                self.ast.binding_pattern_binding_identifier(id.span, id.name)
            }
            AssignmentTarget::ArrayAssignmentTarget(arr) => self.refine_array_target(arr.unbox()),
            AssignmentTarget::ObjectAssignmentTarget(obj) => self.refine_object_target(obj.unbox()),
            _ => self.unexpected(),
        }
    }

    fn refine_array_target(&mut self, arr: ArrayAssignmentTarget<'a>) -> BindingPattern<'a> {
        let span = arr.span;
        let mut elements = self.ast.vec();
        for el in arr.elements {
            elements.push(el.map(|t| self.refine_maybe_default_target(t)));
        }
        let rest = arr.rest.map(|r| self.refine_target_rest(r.unbox()));
        self.ast.binding_pattern_array_pattern(span, elements, rest)
    }

    fn refine_object_target(&mut self, obj: ObjectAssignmentTarget<'a>) -> BindingPattern<'a> {
        let span = obj.span;
        let mut properties = self.ast.vec();
        for prop in obj.properties {
            let bp = match prop {
                AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(id) => {
                    let id = id.unbox();
                    let (b_span, b_name) = (id.binding.span, id.binding.name);
                    let mut value = self.ast.binding_pattern_binding_identifier(b_span, b_name);
                    if let Some(init) = id.init {
                        value = self.ast.binding_pattern_assignment_pattern(id.span, value, init);
                    }
                    let key = PropertyKey::StaticIdentifier(
                        self.alloc(self.ast.identifier_name(b_span, b_name)),
                    );
                    self.ast.binding_property(id.span, key, value, true, false)
                }
                AssignmentTargetProperty::AssignmentTargetPropertyProperty(pp) => {
                    let pp = pp.unbox();
                    let value = self.refine_maybe_default_target(pp.binding);
                    self.ast.binding_property(pp.span, pp.name, value, false, pp.computed)
                }
            };
            properties.push(bp);
        }
        let rest = obj.rest.map(|r| self.refine_target_rest(r.unbox()));
        self.ast.binding_pattern_object_pattern(span, properties, rest)
    }

    fn refine_target_rest(
        &mut self,
        rest: AssignmentTargetRest<'a>,
    ) -> Box<'a, BindingRestElement<'a>> {
        let span = rest.span;
        let argument = self.refine_target_to_binding(rest.target);
        self.alloc(self.ast.binding_rest_element(span, argument))
    }

    fn refine_maybe_default_target(
        &mut self,
        target: AssignmentTargetMaybeDefault<'a>,
    ) -> BindingPattern<'a> {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(wd) => {
                let wd = wd.unbox();
                let left = self.refine_target_to_binding(wd.binding);
                self.ast.binding_pattern_assignment_pattern(wd.span, left, wd.init)
            }
            other => {
                // SAFETY: inherited `AssignmentTarget` variant.
                let target = other.try_into().unwrap_or_else(|_| unreachable!());
                self.refine_target_to_binding(target)
            }
        }
    }
}
