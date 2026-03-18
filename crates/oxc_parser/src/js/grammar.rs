//! Cover Grammar for Destructuring Assignment

use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{ParserConfig as Config, ParserImpl, diagnostics};

pub trait CoverGrammar<'a, T, C: Config>: Sized {
    fn cover(value: T, p: &mut ParserImpl<'a, C>) -> Self;
}

impl<'a, C: Config> CoverGrammar<'a, Expression<'a>, C> for AssignmentTarget<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        if expr.is_array_expression() {
            let array_expr = expr.into_array_expression();
            let pat = ArrayAssignmentTarget::cover(array_expr.unbox(), p);
            AssignmentTarget::ArrayAssignmentTarget(p.alloc(pat))
        } else if expr.is_object_expression() {
            let object_expr = expr.into_object_expression();
            let pat = ObjectAssignmentTarget::cover(object_expr.unbox(), p);
            AssignmentTarget::ObjectAssignmentTarget(p.alloc(pat))
        } else {
            AssignmentTarget::from(SimpleAssignmentTarget::cover(expr, p))
        }
    }
}

impl<'a, C: Config> CoverGrammar<'a, Expression<'a>, C> for SimpleAssignmentTarget<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a, C>) -> Self {
        // Helper to check if inner expression is valid for TS assignment target
        fn is_valid_ts_inner(inner: &Expression) -> bool {
            inner.is_identifier()
                || inner.is_static_member_expression()
                || inner.is_computed_member_expression()
                || inner.is_private_field_expression()
        }

        if expr.is_identifier() {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(expr.into_identifier())
        } else if expr.is_member_expression() {
            SimpleAssignmentTarget::from(expr.into_member_expression())
        } else if expr.is_parenthesized_expression() {
            let paren = expr.into_parenthesized_expression();
            let span = paren.span;
            if paren.expression.is_object_expression() || paren.expression.is_array_expression() {
                p.fatal_error(diagnostics::invalid_assignment(span))
            } else {
                SimpleAssignmentTarget::cover(paren.unbox().expression, p)
            }
        } else if expr.is_ts_as_expression() {
            let inner = expr.as_ts_as_expression().unwrap();
            if is_valid_ts_inner(inner.expression.get_inner_expression()) {
                SimpleAssignmentTarget::TSAsExpression(expr.into_ts_as_expression())
            } else {
                p.fatal_error(diagnostics::invalid_assignment(inner.span()))
            }
        } else if expr.is_ts_satisfies_expression() {
            let inner = expr.as_ts_satisfies_expression().unwrap();
            if is_valid_ts_inner(inner.expression.get_inner_expression()) {
                SimpleAssignmentTarget::TSSatisfiesExpression(expr.into_ts_satisfies_expression())
            } else {
                p.fatal_error(diagnostics::invalid_assignment(inner.span()))
            }
        } else if expr.is_ts_non_null_expression() {
            let inner = expr.as_ts_non_null_expression().unwrap();
            if is_valid_ts_inner(inner.expression.get_inner_expression()) {
                SimpleAssignmentTarget::TSNonNullExpression(expr.into_ts_non_null_expression())
            } else {
                p.fatal_error(diagnostics::invalid_assignment(inner.span()))
            }
        } else if expr.is_ts_type_assertion() {
            let inner = expr.as_ts_type_assertion().unwrap();
            if is_valid_ts_inner(inner.expression.get_inner_expression()) {
                SimpleAssignmentTarget::TSTypeAssertion(expr.into_ts_type_assertion())
            } else {
                p.fatal_error(diagnostics::invalid_assignment(inner.span()))
            }
        } else if expr.is_ts_instantiation_expression() {
            p.fatal_error(diagnostics::invalid_lhs_assignment(expr.span()))
        } else {
            p.fatal_error(diagnostics::invalid_assignment(expr.span()))
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
                            argument.get_inner_expression().kind(),
                            ExpressionKind::Identifier(_)
                                | ExpressionKind::ArrayExpression(_)
                                | ExpressionKind::ObjectExpression(_)
                                | ExpressionKind::StaticMemberExpression(_)
                                | ExpressionKind::ComputedMemberExpression(_)
                                | ExpressionKind::PrivateFieldExpression(_)
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
        if expr.is_assignment_expression() {
            let assignment_expr = expr.into_assignment_expression();
            if assignment_expr.operator != AssignmentOperator::Assign {
                p.error(diagnostics::invalid_assignment_target_default_value_operator(
                    assignment_expr.span,
                ));
            }
            let target = AssignmentTargetWithDefault::cover(assignment_expr.unbox(), p);
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(p.alloc(target))
        } else {
            let target = AssignmentTarget::cover(expr, p);
            AssignmentTargetMaybeDefault::from(target)
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
                            argument.get_inner_expression().kind(),
                            ExpressionKind::Identifier(_)
                                | ExpressionKind::StaticMemberExpression(_)
                                | ExpressionKind::ComputedMemberExpression(_)
                                | ExpressionKind::PrivateFieldExpression(_)
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
