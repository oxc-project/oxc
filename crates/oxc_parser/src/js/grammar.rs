//! Cover Grammar for Destructuring Assignment

use oxc_ast::{ast::*, GetNode};
use oxc_diagnostics::{Diagnostic, Result};

use crate::Parser;

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
                let node = expr.node;
                match expr.unbox().expression {
                    Expression::ObjectExpression(_) | Expression::ArrayExpression(_) => {
                        Err(Diagnostic::InvalidAssignment(node.range()))
                    }
                    expr => SimpleAssignmentTarget::cover(expr, p),
                }
            }
            Expression::TSAsExpression(expr) => Ok(SimpleAssignmentTarget::TSAsExpression(expr)),
            Expression::TSNonNullExpression(expr) => {
                Ok(SimpleAssignmentTarget::TSNonNullExpression(expr))
            }
            Expression::TSTypeAssertion(expr) => Ok(SimpleAssignmentTarget::TSTypeAssertion(expr)),
            expr => Err(Diagnostic::InvalidAssignment(expr.node().range())),
        }
    }
}

impl<'a> CoverGrammar<'a, ArrayExpression<'a>> for ArrayAssignmentTarget<'a> {
    fn cover(expr: ArrayExpression<'a>, p: &mut Parser<'a>) -> Result<Self> {
        let mut elements = p.ast.new_vec();
        let mut rest = None;

        let len = expr.elements.len();
        for (i, elem) in expr.elements.into_iter().enumerate() {
            if let Some(argument) = elem {
                match argument {
                    Argument::Expression(expr) => {
                        let target = AssignmentTargetMaybeDefault::cover(expr, p)?;
                        elements.push(Some(target));
                    }
                    Argument::SpreadElement(elem) => {
                        if i == len - 1 {
                            rest = Some(AssignmentTarget::cover(elem.unbox().argument, p)?);
                            if let Some(node) = expr.trailing_comma {
                                p.error(Diagnostic::RestElementTraillingComma(node.range()));
                            }
                        } else {
                            return Err(Diagnostic::SpreadLastElement(elem.node.range()));
                        }
                    }
                }
            } else {
                elements.push(None);
            }
        }

        Ok(ArrayAssignmentTarget {
            node: expr.node,
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
                Ok(AssignmentTargetMaybeDefault::AssignmentTarget(p.ast.alloc(target)))
            }
        }
    }
}

impl<'a> CoverGrammar<'a, AssignmentExpression<'a>> for AssignmentTargetWithDefault<'a> {
    fn cover(expr: AssignmentExpression<'a>, _p: &mut Parser<'a>) -> Result<Self> {
        Ok(Self { node: expr.node, binding: expr.left, init: expr.right })
    }
}

impl<'a> CoverGrammar<'a, ObjectExpression<'a>> for ObjectAssignmentTarget<'a> {
    fn cover(expr: ObjectExpression<'a>, p: &mut Parser<'a>) -> Result<Self> {
        let mut properties = p.ast.new_vec();
        let mut rest = None;

        let len = expr.properties.len();
        for (i, elem) in expr.properties.into_iter().enumerate() {
            match elem {
                ObjectProperty::Property(property) => {
                    let target = AssignmentTargetProperty::cover(property.unbox(), p)?;
                    properties.push(target);
                }
                ObjectProperty::SpreadProperty(spread) => {
                    if i == len - 1 {
                        rest = Some(AssignmentTarget::cover(spread.unbox().argument, p)?);
                    } else {
                        return Err(Diagnostic::SpreadLastElement(spread.node.range()));
                    }
                }
            }
        }

        Ok(Self { node: expr.node, properties, rest })
    }
}

impl<'a> CoverGrammar<'a, Property<'a>> for AssignmentTargetProperty<'a> {
    fn cover(property: Property<'a>, p: &mut Parser<'a>) -> Result<Self> {
        if property.shorthand {
            let binding = match property.key {
                PropertyKey::Identifier(ident) => {
                    IdentifierReference { node: ident.node, name: ident.name }
                }
                _ => return p.unexpected(),
            };
            let init = match property.value {
                PropertyValue::Expression(Expression::AssignmentExpression(assignment_expr)) => {
                    Some(assignment_expr.unbox().right)
                }
                _ => None,
            };
            let target = AssignmentTargetPropertyIdentifier { node: property.node, binding, init };
            Ok(AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(p.ast.alloc(target)))
        } else {
            let binding = match property.value {
                PropertyValue::Expression(expr) => AssignmentTargetMaybeDefault::cover(expr, p)?,
                PropertyValue::Pattern(_) => {
                    return Err(Diagnostic::InvalidAssignment(property.value.node().range()));
                }
            };
            let target = AssignmentTargetPropertyProperty {
                node: property.node,
                name: property.key,
                binding,
            };
            Ok(AssignmentTargetProperty::AssignmentTargetPropertyProperty(p.ast.alloc(target)))
        }
    }
}
