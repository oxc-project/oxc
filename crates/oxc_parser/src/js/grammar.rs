//! Cover Grammar for Destructuring Assignment

use oxc_ast::ast::*;
use oxc_diagnostics::Result;
use oxc_span::GetSpan;

use crate::{diagnostics, ParserImpl};

pub trait CoverGrammar<'a, T>: Sized {
    fn cover(value: T, p: &mut ParserImpl<'a>) -> Result<Self>;
}

impl<'a> CoverGrammar<'a, Expression<'a>> for AssignmentTarget<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a>) -> Result<Self> {
        match expr {
            Expression::ArrayExpression(array_expr) => {
                ArrayAssignmentTarget::cover(array_expr.unbox(), p)
                    .map(|pat| AssignmentTarget::ArrayAssignmentTarget(p.alloc(pat)))
            }
            Expression::ObjectExpression(object_expr) => {
                ObjectAssignmentTarget::cover(object_expr.unbox(), p)
                    .map(|pat| AssignmentTarget::ObjectAssignmentTarget(p.alloc(pat)))
            }
            _ => SimpleAssignmentTarget::cover(expr, p).map(AssignmentTarget::from),
        }
    }
}

impl<'a> CoverGrammar<'a, Expression<'a>> for SimpleAssignmentTarget<'a> {
    #[allow(clippy::only_used_in_recursion)]
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a>) -> Result<Self> {
        match expr {
            Expression::Identifier(ident) => {
                Ok(SimpleAssignmentTarget::AssignmentTargetIdentifier(ident))
            }
            match_member_expression!(Expression) => {
                let member_expr = MemberExpression::try_from(expr).unwrap();
                Ok(SimpleAssignmentTarget::from(member_expr))
            }
            Expression::ParenthesizedExpression(expr) => {
                let span = expr.span;
                match expr.unbox().expression {
                    Expression::ObjectExpression(_) | Expression::ArrayExpression(_) => {
                        Err(diagnostics::invalid_assignment(span))
                    }
                    expr => SimpleAssignmentTarget::cover(expr, p),
                }
            }
            Expression::TSAsExpression(expr) => Ok(SimpleAssignmentTarget::TSAsExpression(expr)),
            Expression::TSSatisfiesExpression(expr) => {
                Ok(SimpleAssignmentTarget::TSSatisfiesExpression(expr))
            }
            Expression::TSNonNullExpression(expr) => {
                Ok(SimpleAssignmentTarget::TSNonNullExpression(expr))
            }
            Expression::TSTypeAssertion(expr) => Ok(SimpleAssignmentTarget::TSTypeAssertion(expr)),
            Expression::TSInstantiationExpression(expr) => {
                Ok(SimpleAssignmentTarget::TSInstantiationExpression(expr))
            }
            expr => Err(diagnostics::invalid_assignment(expr.span())),
        }
    }
}

impl<'a> CoverGrammar<'a, ArrayExpression<'a>> for ArrayAssignmentTarget<'a> {
    fn cover(expr: ArrayExpression<'a>, p: &mut ParserImpl<'a>) -> Result<Self> {
        let mut elements = p.ast.vec();
        let mut rest = None;

        let len = expr.elements.len();
        for (i, elem) in expr.elements.into_iter().enumerate() {
            match elem {
                match_expression!(ArrayExpressionElement) => {
                    let expr = Expression::try_from(elem).unwrap();
                    let target = AssignmentTargetMaybeDefault::cover(expr, p)?;
                    elements.push(Some(target));
                }
                ArrayExpressionElement::SpreadElement(elem) => {
                    if i == len - 1 {
                        rest = Some(p.ast.assignment_target_rest(
                            elem.span,
                            AssignmentTarget::cover(elem.unbox().argument, p)?,
                        ));
                        if let Some(span) = expr.trailing_comma {
                            p.error(diagnostics::binding_rest_element_trailing_comma(span));
                        }
                    } else {
                        return Err(diagnostics::spread_last_element(elem.span));
                    }
                }
                ArrayExpressionElement::Elision(_) => elements.push(None),
            }
        }

        Ok(p.ast.array_assignment_target(expr.span, elements, rest, expr.trailing_comma))
    }
}

impl<'a> CoverGrammar<'a, Expression<'a>> for AssignmentTargetMaybeDefault<'a> {
    fn cover(expr: Expression<'a>, p: &mut ParserImpl<'a>) -> Result<Self> {
        match expr {
            Expression::AssignmentExpression(assignment_expr) => {
                let target = AssignmentTargetWithDefault::cover(assignment_expr.unbox(), p)?;
                Ok(AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(p.alloc(target)))
            }
            expr => {
                let target = AssignmentTarget::cover(expr, p)?;
                Ok(AssignmentTargetMaybeDefault::from(target))
            }
        }
    }
}

impl<'a> CoverGrammar<'a, AssignmentExpression<'a>> for AssignmentTargetWithDefault<'a> {
    fn cover(expr: AssignmentExpression<'a>, p: &mut ParserImpl<'a>) -> Result<Self> {
        Ok(p.ast.assignment_target_with_default(expr.span, expr.left, expr.right))
    }
}

impl<'a> CoverGrammar<'a, ObjectExpression<'a>> for ObjectAssignmentTarget<'a> {
    fn cover(expr: ObjectExpression<'a>, p: &mut ParserImpl<'a>) -> Result<Self> {
        let mut properties = p.ast.vec();
        let mut rest = None;

        let len = expr.properties.len();
        for (i, elem) in expr.properties.into_iter().enumerate() {
            match elem {
                ObjectPropertyKind::ObjectProperty(property) => {
                    let target = AssignmentTargetProperty::cover(property.unbox(), p)?;
                    properties.push(target);
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    if i == len - 1 {
                        rest = Some(p.ast.assignment_target_rest(
                            spread.span,
                            AssignmentTarget::cover(spread.unbox().argument, p)?,
                        ));
                    } else {
                        return Err(diagnostics::spread_last_element(spread.span));
                    }
                }
            }
        }

        Ok(p.ast.object_assignment_target(expr.span, properties, rest))
    }
}

impl<'a> CoverGrammar<'a, ObjectProperty<'a>> for AssignmentTargetProperty<'a> {
    fn cover(property: ObjectProperty<'a>, p: &mut ParserImpl<'a>) -> Result<Self> {
        if property.shorthand {
            let binding = match property.key {
                PropertyKey::StaticIdentifier(ident) => {
                    let ident = ident.unbox();
                    p.ast.identifier_reference(ident.span, ident.name)
                }
                _ => return Err(p.unexpected()),
            };
            // convert `CoverInitializedName`
            let init = p.state.cover_initialized_name.remove(&property.span.start).map(|e| e.right);
            Ok(p.ast.assignment_target_property_assignment_target_property_identifier(
                property.span,
                binding,
                init,
            ))
        } else {
            let binding = AssignmentTargetMaybeDefault::cover(property.value, p)?;
            Ok(p.ast.assignment_target_property_assignment_target_property_property(
                property.span,
                property.key,
                binding,
                property.computed,
            ))
        }
    }
}
