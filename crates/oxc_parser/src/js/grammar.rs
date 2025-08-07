//! Cover Grammar for Destructuring Assignment

use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{ParserImpl, diagnostics};

pub trait CoverGrammar<'a, T>: Sized {
    fn cover(value: T, p: &mut ParserImpl<'a>) -> Self;
}

impl<'a> CoverGrammar<'a, Expression<'a>> for AssignmentTarget<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a>) -> Self {
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

impl<'a> CoverGrammar<'a, Expression<'a>> for SimpleAssignmentTarget<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a>) -> Self {
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
            Expression::TSAsExpression(expr) => {
                if is_valid_ts_assignment_target(expr.expression.get_inner_expression()) {
                    SimpleAssignmentTarget::TSAsExpression(expr)
                } else {
                    p.fatal_error(diagnostics::invalid_assignment(expr.span()))
                }
            }
            Expression::TSSatisfiesExpression(expr) => {
                if is_valid_ts_assignment_target(expr.expression.get_inner_expression()) {
                    SimpleAssignmentTarget::TSSatisfiesExpression(expr)
                } else {
                    p.fatal_error(diagnostics::invalid_assignment(expr.span()))
                }
            }
            Expression::TSNonNullExpression(expr) => {
                if is_valid_ts_assignment_target(expr.expression.get_inner_expression()) {
                    SimpleAssignmentTarget::TSNonNullExpression(expr)
                } else {
                    p.fatal_error(diagnostics::invalid_assignment(expr.span()))
                }
            }
            Expression::TSTypeAssertion(expr) => {
                if is_valid_ts_assignment_target(expr.expression.get_inner_expression()) {
                    SimpleAssignmentTarget::TSTypeAssertion(expr)
                } else {
                    p.fatal_error(diagnostics::invalid_assignment(expr.span()))
                }
            }
            Expression::TSInstantiationExpression(expr) => {
                p.fatal_error(diagnostics::invalid_lhs_assignment(expr.span()))
            }
            expr => p.fatal_error(diagnostics::invalid_assignment(expr.span())),
        }
    }
}

fn is_valid_ts_assignment_target<'a>(expr: &Expression<'a>) -> bool {
    matches!(
        expr,
        Expression::Identifier(_)
            | Expression::StaticMemberExpression(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::PrivateFieldExpression(_)
    )
}

impl<'a> CoverGrammar<'a, ArrayExpression<'a>> for ArrayAssignmentTarget<'a> {
    fn cover(expr: ArrayExpression<'a>, p: &mut ParserImpl<'a>) -> Self {
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
                        
                        if !is_valid_rest_target(&argument) {
                            p.error(diagnostics::invalid_rest_assignment_target(argument.span()));
                        }
                        
                        let target = AssignmentTarget::cover(argument, p);
                        rest = Some(p.ast.alloc_assignment_target_rest(span, target));
                        
                        if let Some(span) = p.state.trailing_commas.get(&expr.span.start) {
                            p.error(diagnostics::rest_element_trailing_comma(*span));
                        }
                    } else {
                        return p.fatal_error(diagnostics::spread_last_element(elem.span));
                    }
                }
                ArrayExpressionElement::Elision(_) => elements.push(None),
            }
        }

        p.ast.array_assignment_target(expr.span, elements, rest)
    }
}

fn is_valid_rest_target<'a>(argument: &Expression<'a>) -> bool {
    matches!(
        argument.get_inner_expression(),
        Expression::Identifier(_)
            | Expression::ArrayExpression(_)
            | Expression::ObjectExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::PrivateFieldExpression(_)
    )
}

impl<'a> CoverGrammar<'a, Expression<'a>> for AssignmentTargetMaybeDefault<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a>) -> Self {
        match expr {
            Expression::AssignmentExpression(assignment_expr) => {
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

impl<'a> CoverGrammar<'a, AssignmentExpression<'a>> for AssignmentTargetWithDefault<'a> {
    fn cover(expr: AssignmentExpression<'a>, p: &mut ParserImpl<'a>) -> Self {
        p.ast.assignment_target_with_default(expr.span, expr.left, expr.right)
    }
}

impl<'a> CoverGrammar<'a, ObjectExpression<'a>> for ObjectAssignmentTarget<'a> {
    fn cover(expr: ObjectExpression<'a>, p: &mut ParserImpl<'a>) -> Self {
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
                        
                        if !is_valid_object_rest_target(&argument) {
                            p.error(diagnostics::invalid_rest_assignment_target(argument.span()));
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

fn is_valid_object_rest_target<'a>(argument: &Expression<'a>) -> bool {
    matches!(
        argument.get_inner_expression(),
        Expression::Identifier(_)
            | Expression::StaticMemberExpression(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::PrivateFieldExpression(_)
    )
}

impl<'a> CoverGrammar<'a, ObjectProperty<'a>> for AssignmentTargetProperty<'a> {
    fn cover(property: ObjectProperty<'a>, p: &mut ParserImpl<'a>) -> Self {
        if property.shorthand {
            handle_shorthand_assignment_property(property, p)
        } else {
            handle_regular_assignment_property(property, p)
        }
    }
}

fn handle_shorthand_assignment_property<'a>(
    property: ObjectProperty<'a>,
    p: &mut ParserImpl<'a>,
) -> AssignmentTargetProperty<'a> {
    let binding = match property.key {
        PropertyKey::StaticIdentifier(ident) => {
            let ident = ident.unbox();
            p.ast.identifier_reference(ident.span, ident.name)
        }
        _ => return p.unexpected(),
    };
    
    let init = p.state.cover_initialized_name.remove(&property.span.start).map(|e| e.right);
    p.ast.assignment_target_property_assignment_target_property_identifier(
        property.span,
        binding,
        init,
    )
}

fn handle_regular_assignment_property<'a>(
    property: ObjectProperty<'a>,
    p: &mut ParserImpl<'a>,
) -> AssignmentTargetProperty<'a> {
    let binding = AssignmentTargetMaybeDefault::cover(property.value, p);
    p.ast.assignment_target_property_assignment_target_property_property(
        property.span,
        property.key,
        binding,
        property.computed,
    )
}
