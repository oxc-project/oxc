//! Cover Grammar for Destructuring Assignment

use oxc_ast::ast::*;
use oxc_diagnostics::Result;
use oxc_span::GetSpan;

use crate::{diagnostics, Parser};

pub trait CoverGrammar<'a, T>: Sized {
    fn cover(value: T, p: &mut Parser<'a>) -> Result<Self>;
}

impl<'a> CoverGrammar<'a, Expression<'a>> for AssignmentTarget<'a> {
    fn cover(expr: Expression<'a>, p: &mut Parser<'a>) -> Result<Self> {
        match expr {
            Expression::ArrayExpression(array_expr) => {
                ArrayAssignmentTarget::cover(array_expr.unbox(), p)
                    .map(|pat| p.ast.alloc(pat))
                    .map(AssignmentTargetPattern::ArrayAssignmentTarget)
                    .map(AssignmentTarget::AssignmentTargetPattern)
            }
            Expression::ObjectExpression(object_expr) => {
                ObjectAssignmentTarget::cover(object_expr.unbox(), p)
                    .map(|pat| p.ast.alloc(pat))
                    .map(AssignmentTargetPattern::ObjectAssignmentTarget)
                    .map(AssignmentTarget::AssignmentTargetPattern)
            }
            _ => {
                SimpleAssignmentTarget::cover(expr, p).map(AssignmentTarget::SimpleAssignmentTarget)
            }
        }
    }
}

impl<'a> CoverGrammar<'a, Expression<'a>> for SimpleAssignmentTarget<'a> {
    #[allow(clippy::only_used_in_recursion)]
    fn cover(expr: Expression<'a>, p: &mut Parser<'a>) -> Result<Self> {
        match expr {
            Expression::Identifier(ident) => {
                Ok(SimpleAssignmentTarget::AssignmentTargetIdentifier(ident))
            }
            Expression::MemberExpression(expr) => {
                Ok(SimpleAssignmentTarget::MemberAssignmentTarget(expr))
            }
            Expression::ParenthesizedExpression(expr) => {
                let span = expr.span;
                match expr.unbox().expression {
                    Expression::ObjectExpression(_) | Expression::ArrayExpression(_) => {
                        Err(diagnostics::InvalidAssignment(span).into())
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
            expr => Err(diagnostics::InvalidAssignment(expr.span()).into()),
        }
    }
}

impl<'a> CoverGrammar<'a, ArrayExpression<'a>> for ArrayAssignmentTarget<'a> {
    fn cover(expr: ArrayExpression<'a>, p: &mut Parser<'a>) -> Result<Self> {
        let mut elements = p.ast.new_vec();
        let mut rest = None;

        let len = expr.elements.len();
        for (i, elem) in expr.elements.into_iter().enumerate() {
            match elem {
                ArrayExpressionElement::Expression(expr) => {
                    let target = AssignmentTargetMaybeDefault::cover(expr, p)?;
                    elements.push(Some(target));
                }
                ArrayExpressionElement::SpreadElement(elem) => {
                    if i == len - 1 {
                        rest = Some(AssignmentTarget::cover(elem.unbox().argument, p)?);
                        if let Some(span) = expr.trailing_comma {
                            p.error(diagnostics::RestElementTrailingComma(span));
                        }
                    } else {
                        return Err(diagnostics::SpreadLastElement(elem.span).into());
                    }
                }
                ArrayExpressionElement::Elision(_) => elements.push(None),
            }
        }

        Ok(ArrayAssignmentTarget {
            span: expr.span,
            elements,
            rest,
            trailing_comma: expr.trailing_comma,
        })
    }
}

impl<'a> CoverGrammar<'a, Expression<'a>> for AssignmentTargetMaybeDefault<'a> {
    fn cover(expr: Expression<'a>, p: &mut Parser<'a>) -> Result<Self> {
        match expr {
            Expression::AssignmentExpression(assignment_expr) => {
                let target = AssignmentTargetWithDefault::cover(assignment_expr.unbox(), p)?;
                Ok(AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(p.ast.alloc(target)))
            }
            expr => {
                let target = AssignmentTarget::cover(expr, p)?;
                Ok(AssignmentTargetMaybeDefault::AssignmentTarget(target))
            }
        }
    }
}

impl<'a> CoverGrammar<'a, AssignmentExpression<'a>> for AssignmentTargetWithDefault<'a> {
    fn cover(expr: AssignmentExpression<'a>, _p: &mut Parser<'a>) -> Result<Self> {
        Ok(Self { span: expr.span, binding: expr.left, init: expr.right })
    }
}

impl<'a> CoverGrammar<'a, ObjectExpression<'a>> for ObjectAssignmentTarget<'a> {
    fn cover(expr: ObjectExpression<'a>, p: &mut Parser<'a>) -> Result<Self> {
        let mut properties = p.ast.new_vec();
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
                        rest = Some(AssignmentTarget::cover(spread.unbox().argument, p)?);
                    } else {
                        return Err(diagnostics::SpreadLastElement(spread.span).into());
                    }
                }
            }
        }

        Ok(Self { span: expr.span, properties, rest })
    }
}

impl<'a> CoverGrammar<'a, ObjectProperty<'a>> for AssignmentTargetProperty<'a> {
    fn cover(property: ObjectProperty<'a>, p: &mut Parser<'a>) -> Result<Self> {
        if property.shorthand {
            let binding = match property.key {
                PropertyKey::Identifier(ident) => IdentifierReference {
                    span: ident.span,
                    name: ident.unbox().name,
                    reference_id: None,
                },
                _ => return Err(p.unexpected()),
            };
            // convert `CoverInitializedName`
            let init = match property.init {
                Some(Expression::AssignmentExpression(assignment_expr)) => {
                    Some(assignment_expr.unbox().right)
                }
                _ => None,
            };
            let target = AssignmentTargetPropertyIdentifier { span: property.span, binding, init };
            Ok(AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(p.ast.alloc(target)))
        } else {
            let binding = AssignmentTargetMaybeDefault::cover(property.value, p)?;
            let target = AssignmentTargetPropertyProperty {
                span: property.span,
                name: property.key,
                binding,
            };
            Ok(AssignmentTargetProperty::AssignmentTargetPropertyProperty(p.ast.alloc(target)))
        }
    }
}
