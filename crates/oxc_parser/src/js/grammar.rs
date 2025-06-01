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
            Expression::TSAsExpression(expr) => SimpleAssignmentTarget::TSAsExpression(expr),
            Expression::TSSatisfiesExpression(expr) => {
                SimpleAssignmentTarget::TSSatisfiesExpression(expr)
            }
            Expression::TSNonNullExpression(expr) => {
                SimpleAssignmentTarget::TSNonNullExpression(expr)
            }
            Expression::TSTypeAssertion(expr) => SimpleAssignmentTarget::TSTypeAssertion(expr),
            Expression::TSInstantiationExpression(expr) => {
                p.fatal_error(diagnostics::invalid_lhs_assignment(expr.span()))
            }
            expr => p.fatal_error(diagnostics::invalid_assignment(expr.span())),
        }
    }
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
                        rest = Some(p.ast.assignment_target_rest(
                            elem.span,
                            AssignmentTarget::cover(elem.unbox().argument, p),
                        ));
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
                        rest = Some(p.ast.assignment_target_rest(
                            spread.span,
                            AssignmentTarget::cover(spread.unbox().argument, p),
                        ));
                    } else {
                        return p.fatal_error(diagnostics::spread_last_element(spread.span));
                    }
                }
            }
        }

        p.ast.object_assignment_target(expr.span, properties, rest)
    }
}

impl<'a> CoverGrammar<'a, ObjectProperty<'a>> for AssignmentTargetProperty<'a> {
    fn cover(property: ObjectProperty<'a>, p: &mut ParserImpl<'a>) -> Self {
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
