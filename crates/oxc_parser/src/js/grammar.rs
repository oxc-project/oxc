//! Cover Grammar for Destructuring Assignment and Arrow Function Parameters

use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};
use oxc_str::Ident;

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
                // The parens are unwrapped here; remember them in case this target is later
                // refined to an arrow parameter, where they are invalid.
                if p.state.cover_paren_depth != 0 {
                    p.state.cover_invalid_patterns.push(span);
                }
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
    // Destructuring-target conversion is comparatively rare and large. Keeping it out of line
    // stops it being inlined into `AssignmentTarget::cover`, whose common arm (simple targets)
    // would otherwise carry this body's large stack frame + callee-saved spills on every call.
    #[inline(never)]
    fn cover(expr: ArrayExpression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        let mut elements = ArenaVec::new_in(p);
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
                        rest = Some(AssignmentTargetRest::boxed(span, target, p));
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

        ArrayAssignmentTarget::new(expr.span, elements, rest, p)
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
                    // The operator is erased by this conversion; a binding pattern requires a
                    // plain `=` default, so a containing arrow head must not refine.
                    if p.state.cover_paren_depth != 0 {
                        p.state.cover_invalid_patterns.push(assignment_expr.span);
                    }
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
        AssignmentTargetWithDefault::new(expr.span, expr.left, expr.right, p)
    }
}

impl<'a, C: Config> CoverGrammar<'a, ObjectExpression<'a>, C> for ObjectAssignmentTarget<'a> {
    // Kept out of line for the same reason as `ArrayAssignmentTarget::cover` above: avoid
    // inlining this large body into the hot `AssignmentTarget::cover` dispatcher.
    #[inline(never)]
    fn cover(expr: ObjectExpression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        let mut properties = ArenaVec::new_in(p);
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
                        rest = Some(AssignmentTargetRest::boxed(span, target, p));
                    } else {
                        return p.fatal_error(diagnostics::spread_last_element(spread.span));
                    }
                }
            }
        }

        ObjectAssignmentTarget::new(expr.span, properties, rest, p)
    }
}

/// Whether this pattern position overlaps syntax that the cover grammar parse erased from
/// the AST (unwrapped parens, compound-operator defaults), making it invalid as a binding.
fn span_has_invalid_pattern<C: Config>(span: Span, p: &ParserImpl<'_, C>) -> bool {
    !p.state.cover_invalid_patterns.is_empty()
        && p.state
            .cover_invalid_patterns
            .iter()
            .any(|invalid| invalid.start <= span.start && span.end <= invalid.end)
}

/// Read-only check that an expression parsed by the cover grammar can refine to a
/// [`BindingPattern`](https://tc39.es/ecma262/#sec-destructuring-binding-patterns),
/// mirroring the fatal arms of the `cover` conversions below. When this fails, the
/// enclosing `( ... )` refines to a parenthesized expression instead — matching the
/// speculative parser, which rewound and re-parsed as an expression, leaving the stray
/// `=>` to error downstream.
pub(super) fn is_binding_pattern_expression<'a, C: Config>(
    expr: &Expression<'a>,
    p: &ParserImpl<'a, C>,
) -> bool {
    if span_has_invalid_pattern(expr.span(), p) {
        return false;
    }
    match expr {
        Expression::Identifier(_) => true,
        // `(yield, a) => {}` in a generator: the item parses as a bare yield expression, but
        // the speculative parser accepted it as a binding named `yield` (with an error).
        Expression::YieldExpression(e) => e.argument.is_none() && !e.delegate,
        Expression::ObjectExpression(obj) => {
            let len = obj.properties.len();
            obj.properties.iter().enumerate().all(|(i, prop)| match prop {
                ObjectPropertyKind::ObjectProperty(property) => {
                    property.kind == PropertyKind::Init
                        && !property.method
                        && (property.shorthand || is_binding_pattern_expression(&property.value, p))
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    i == len - 1 && matches!(spread.argument, Expression::Identifier(_))
                }
            })
        }
        Expression::ArrayExpression(arr) => {
            let len = arr.elements.len();
            arr.elements.iter().enumerate().all(|(i, elem)| match elem {
                ArrayExpressionElement::Elision(_) => true,
                ArrayExpressionElement::SpreadElement(spread) => {
                    i == len - 1 && is_binding_pattern_expression(&spread.argument, p)
                }
                match_expression!(ArrayExpressionElement) => {
                    is_binding_pattern_expression(elem.to_expression(), p)
                }
            })
        }
        Expression::AssignmentExpression(assign) => {
            assign.operator == AssignmentOperator::Assign
                && is_binding_pattern_target(&assign.left, p)
        }
        _ => false,
    }
}

fn is_binding_pattern_target<'a, C: Config>(
    target: &AssignmentTarget<'a>,
    p: &ParserImpl<'a, C>,
) -> bool {
    if span_has_invalid_pattern(target.span(), p) {
        return false;
    }
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(_) => true,
        AssignmentTarget::ArrayAssignmentTarget(t) => {
            t.elements.iter().flatten().all(|elem| is_binding_pattern_maybe_default(elem, p))
                && t.rest.as_ref().is_none_or(|rest| is_binding_pattern_target(&rest.target, p))
        }
        AssignmentTarget::ObjectAssignmentTarget(t) => {
            t.properties.iter().all(|prop| match prop {
                AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(_) => true,
                AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
                    is_binding_pattern_maybe_default(&prop.binding, p)
                }
            }) && t.rest.as_ref().is_none_or(|rest| {
                matches!(rest.target, AssignmentTarget::AssignmentTargetIdentifier(_))
            })
        }
        _ => false,
    }
}

fn is_binding_pattern_maybe_default<'a, C: Config>(
    target: &AssignmentTargetMaybeDefault<'a>,
    p: &ParserImpl<'a, C>,
) -> bool {
    match target {
        AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(t) => {
            is_binding_pattern_target(&t.binding, p)
        }
        match_assignment_target!(AssignmentTargetMaybeDefault) => {
            is_binding_pattern_target(target.to_assignment_target(), p)
        }
    }
}

/// [`BindingIdentifier`](https://tc39.es/ecma262/#prod-BindingIdentifier) refinement of an
/// identifier parsed as an expression.
/// [Early errors](https://tc39.es/ecma262/#sec-identifiers-static-semantics-early-errors):
/// `await` is a syntax error with an `[Await]` parameter, `yield` with a `[Yield]` parameter.
fn cover_binding_identifier<'a, C: Config>(
    span: Span,
    name: Ident<'a>,
    p: &mut ParserImpl<'a, C>,
) -> BindingPattern<'a> {
    if p.ctx.has_await() && name == "await" {
        p.error(diagnostics::identifier_async("await", span));
    } else if p.ctx.has_yield() && name == "yield" {
        p.error(diagnostics::identifier_generator("yield", span, false));
    }
    BindingPattern::new_binding_identifier(span, name, p)
}

/// [`BindingElement`](https://tc39.es/ecma262/#prod-BindingElement) refinement of an
/// expression parsed by the cover grammar:
///     `BindingElement`[Yield, Await] :
///         `SingleNameBinding`[?Yield, ?Await]
///         `BindingPattern`[?Yield, ?Await] `Initializer`[+In, ?Yield, ?Await]opt
///     `SingleNameBinding`[Yield, Await] :
///         `BindingIdentifier`[?Yield, ?Await] `Initializer`[+In, ?Yield, ?Await]opt
impl<'a, C: Config> CoverGrammar<'a, Expression<'a>, C> for BindingPattern<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        match expr {
            Expression::Identifier(ident) => {
                let ident = ident.unbox();
                cover_binding_identifier(ident.span, ident.name, p)
            }
            // Bare `yield` in a generator parsed as a yield expression; refine to a binding
            // named `yield`, with the same error the binding identifier parser reports.
            Expression::YieldExpression(e) if e.argument.is_none() && !e.delegate => {
                let span = e.span;
                p.error(diagnostics::identifier_generator("yield", span, false));
                BindingPattern::new_binding_identifier(span, "yield", p)
            }
            Expression::ObjectExpression(obj) => {
                let pat = ObjectPattern::cover(obj.unbox(), p);
                BindingPattern::ObjectPattern(p.alloc(pat))
            }
            Expression::ArrayExpression(arr) => {
                let pat = ArrayPattern::cover(arr.unbox(), p);
                BindingPattern::ArrayPattern(p.alloc(pat))
            }
            Expression::AssignmentExpression(assign) => {
                let assign = assign.unbox();
                if assign.operator != AssignmentOperator::Assign {
                    return p.fatal_error(
                        diagnostics::invalid_assignment_target_default_value_operator(assign.span),
                    );
                }
                let left = BindingPattern::cover(assign.left, p);
                BindingPattern::new_assignment_pattern(assign.span, left, assign.right, p)
            }
            expr => p.fatal_error(diagnostics::invalid_binding_pattern(expr.span())),
        }
    }
}

/// [`ObjectBindingPattern`](https://tc39.es/ecma262/#prod-ObjectBindingPattern) refinement
/// of an object literal:
///     `ObjectBindingPattern`[Yield, Await] :
///         { }
///         { `BindingRestProperty`[?Yield, ?Await] }
///         { `BindingPropertyList`[?Yield, ?Await] }
///         { `BindingPropertyList`[?Yield, ?Await] , `BindingRestProperty`[?Yield, ?Await]opt }
///     `BindingRestProperty`[Yield, Await] :
///         ... `BindingIdentifier`[?Yield, ?Await]
impl<'a, C: Config> CoverGrammar<'a, ObjectExpression<'a>, C> for ObjectPattern<'a> {
    #[inline(never)]
    fn cover(expr: ObjectExpression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        let mut properties = ArenaVec::with_capacity_in(expr.properties.len(), p);
        let mut rest = None;

        let len = expr.properties.len();
        for (i, prop) in expr.properties.into_iter().enumerate() {
            match prop {
                ObjectPropertyKind::ObjectProperty(property) => {
                    properties.push(BindingProperty::cover(property.unbox(), p));
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    if i == len - 1 {
                        let span = spread.span;
                        let argument = spread.unbox().argument;
                        if !matches!(argument, Expression::Identifier(_)) {
                            return p.fatal_error(diagnostics::invalid_binding_rest_element(
                                argument.span(),
                            ));
                        }
                        let target = BindingPattern::cover(argument, p);
                        rest = Some(BindingRestElement::boxed(span, target, p));
                        if let Some(span) = p.state.trailing_commas.get(&expr.span.start) {
                            p.error(diagnostics::rest_element_trailing_comma(*span));
                        }
                    } else {
                        return p.fatal_error(diagnostics::binding_rest_element_last(spread.span));
                    }
                }
            }
        }

        ObjectPattern::new(expr.span, properties, rest, p)
    }
}

/// [`BindingProperty`](https://tc39.es/ecma262/#prod-BindingProperty) refinement of an
/// object literal property:
///     `BindingProperty`[Yield, Await] :
///         `SingleNameBinding`[?Yield, ?Await]
///         `PropertyName`[?Yield, ?Await] : `BindingElement`[?Yield, ?Await]
///
/// A shorthand property with an initializer is a
/// [`CoverInitializedName`](https://tc39.es/ecma262/#prod-CoverInitializedName), recorded by
/// the object literal parser and refined to a `SingleNameBinding` with an initializer here.
impl<'a, C: Config> CoverGrammar<'a, ObjectProperty<'a>, C> for BindingProperty<'a> {
    fn cover(property: ObjectProperty<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        if property.kind != PropertyKind::Init || property.method {
            return p.fatal_error(diagnostics::invalid_binding_pattern(property.span));
        }
        if property.shorthand {
            let (ident_span, ident_name) = match &property.key {
                PropertyKey::StaticIdentifier(ident) => (ident.span, ident.name),
                _ => return p.unexpected(),
            };
            let mut value = cover_binding_identifier(ident_span, ident_name, p);
            // convert `CoverInitializedName`
            if let Some(init) = p.state.cover_initialized_name.remove(&property.span.start) {
                value = BindingPattern::new_assignment_pattern(property.span, value, init.right, p);
            }
            BindingProperty::new(property.span, property.key, value, true, property.computed, p)
        } else {
            let value = BindingPattern::cover(property.value, p);
            BindingProperty::new(property.span, property.key, value, false, property.computed, p)
        }
    }
}

/// [`ArrayBindingPattern`](https://tc39.es/ecma262/#prod-ArrayBindingPattern) refinement
/// of an array literal:
///     `ArrayBindingPattern`[Yield, Await] :
///         [ `Elision`opt `BindingRestElement`[?Yield, ?Await]opt ]
///         [ `BindingElementList`[?Yield, ?Await] ]
///         [ `BindingElementList`[?Yield, ?Await] , `Elision`opt `BindingRestElement`[?Yield, ?Await]opt ]
///     `BindingRestElement`[Yield, Await] :
///         ... `BindingIdentifier`[?Yield, ?Await]
///         ... `BindingPattern`[?Yield, ?Await]
impl<'a, C: Config> CoverGrammar<'a, ArrayExpression<'a>, C> for ArrayPattern<'a> {
    #[inline(never)]
    fn cover(expr: ArrayExpression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        let mut elements = ArenaVec::with_capacity_in(expr.elements.len(), p);
        let mut rest = None;

        let len = expr.elements.len();
        for (i, elem) in expr.elements.into_iter().enumerate() {
            match elem {
                match_expression!(ArrayExpressionElement) => {
                    let expr = elem.into_expression();
                    elements.push(Some(BindingPattern::cover(expr, p)));
                }
                ArrayExpressionElement::SpreadElement(spread) => {
                    if i == len - 1 {
                        let span = spread.span;
                        let argument = spread.unbox().argument;
                        let target = BindingPattern::cover(argument, p);
                        if let BindingPattern::AssignmentPattern(pat) = &target {
                            p.error(diagnostics::a_rest_element_cannot_have_an_initializer(
                                pat.span,
                            ));
                        }
                        rest = Some(BindingRestElement::boxed(span, target, p));
                        if let Some(span) = p.state.trailing_commas.get(&expr.span.start) {
                            p.error(diagnostics::rest_element_trailing_comma(*span));
                        }
                    } else {
                        return p.fatal_error(diagnostics::binding_rest_element_last(spread.span));
                    }
                }
                ArrayExpressionElement::Elision(_) => elements.push(None),
            }
        }

        ArrayPattern::new(expr.span, elements, rest, p)
    }
}

/// `BindingPattern` refinement of an assignment target. Inside the cover grammar, the left
/// side of `=` was eagerly covered to a
/// [`DestructuringAssignmentTarget`](https://tc39.es/ecma262/#sec-destructuring-assignment);
/// the arrow refinement reinterprets the same source as binding patterns, which only bind
/// identifiers — member expressions and the other assignment-only targets are errors.
impl<'a, C: Config> CoverGrammar<'a, AssignmentTarget<'a>, C> for BindingPattern<'a> {
    fn cover(target: AssignmentTarget<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                let ident = ident.unbox();
                cover_binding_identifier(ident.span, ident.name, p)
            }
            AssignmentTarget::ArrayAssignmentTarget(target) => {
                let target = target.unbox();
                let mut elements = ArenaVec::with_capacity_in(target.elements.len(), p);
                for elem in target.elements {
                    elements.push(elem.map(|e| BindingPattern::cover(e, p)));
                }
                let rest = target.rest.map(|rest| {
                    let rest = rest.unbox();
                    let argument = BindingPattern::cover(rest.target, p);
                    BindingRestElement::boxed(rest.span, argument, p)
                });
                BindingPattern::new_array_pattern(target.span, elements, rest, p)
            }
            AssignmentTarget::ObjectAssignmentTarget(target) => {
                let target = target.unbox();
                let mut properties = ArenaVec::with_capacity_in(target.properties.len(), p);
                for prop in target.properties {
                    properties.push(BindingProperty::cover(prop, p));
                }
                let rest = target.rest.map(|rest| {
                    let rest = rest.unbox();
                    if !matches!(rest.target, AssignmentTarget::AssignmentTargetIdentifier(_)) {
                        p.error(diagnostics::invalid_binding_rest_element(rest.target.span()));
                    }
                    let argument = BindingPattern::cover(rest.target, p);
                    BindingRestElement::boxed(rest.span, argument, p)
                });
                BindingPattern::new_object_pattern(target.span, properties, rest, p)
            }
            target => p.fatal_error(diagnostics::invalid_binding_pattern(target.span())),
        }
    }
}

impl<'a, C: Config> CoverGrammar<'a, AssignmentTargetMaybeDefault<'a>, C> for BindingPattern<'a> {
    fn cover(target: AssignmentTargetMaybeDefault<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(target) => {
                let target = target.unbox();
                let left = BindingPattern::cover(target.binding, p);
                BindingPattern::new_assignment_pattern(target.span, left, target.init, p)
            }
            match_assignment_target!(AssignmentTargetMaybeDefault) => {
                BindingPattern::cover(target.into_assignment_target(), p)
            }
        }
    }
}

impl<'a, C: Config> CoverGrammar<'a, AssignmentTargetProperty<'a>, C> for BindingProperty<'a> {
    fn cover(property: AssignmentTargetProperty<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        match property {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(prop) => {
                let prop = prop.unbox();
                let ident_span = prop.binding.span;
                let ident_name = prop.binding.name;
                let mut value = cover_binding_identifier(ident_span, ident_name, p);
                if let Some(init) = prop.init {
                    value = BindingPattern::new_assignment_pattern(prop.span, value, init, p);
                }
                let key =
                    PropertyKey::StaticIdentifier(IdentifierName::boxed(ident_span, ident_name, p));
                BindingProperty::new(prop.span, key, value, true, false, p)
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
                let prop = prop.unbox();
                let value = BindingPattern::cover(prop.binding, p);
                BindingProperty::new(prop.span, prop.name, value, false, prop.computed, p)
            }
        }
    }
}

impl<'a, C: Config> CoverGrammar<'a, ObjectProperty<'a>, C> for AssignmentTargetProperty<'a> {
    fn cover(property: ObjectProperty<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        if property.shorthand {
            let binding = match property.key {
                PropertyKey::StaticIdentifier(ident) => {
                    let ident = ident.unbox();
                    IdentifierReference::new(ident.span, ident.name, p)
                }
                _ => return p.unexpected(),
            };
            // convert `CoverInitializedName`
            let init = p.state.cover_initialized_name.remove(&property.span.start).map(|e| e.right);
            AssignmentTargetProperty::new_assignment_target_property_identifier(
                property.span,
                binding,
                init,
                p,
            )
        } else {
            let binding = AssignmentTargetMaybeDefault::cover(property.value, p);
            AssignmentTargetProperty::new_assignment_target_property_property(
                property.span,
                property.key,
                binding,
                property.computed,
                p,
            )
        }
    }
}
