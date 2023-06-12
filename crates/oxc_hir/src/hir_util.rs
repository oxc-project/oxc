use num_bigint::BigUint;
use oxc_syntax::operator::{AssignmentOperator, LogicalOperator, UnaryOperator};

use crate::hir::{
    ArrayExpressionElement, Expression, NumberLiteral, ObjectProperty, ObjectPropertyKind,
    PropertyKey, SpreadElement,
};

/// Code ported from [closure-compiler](https://github.com/google/closure-compiler/blob/f3ce5ed8b630428e311fe9aa2e20d36560d975e2/src/com/google/javascript/jscomp/NodeUtil.java#LL836C6-L836C6)
/// Returns true if this is a literal value. We define a literal value as any node that evaluates
/// to the same thing regardless of when or where it is evaluated. So `/xyz/` and `[3, 5]` are
/// literals, but the name a is not.
///
/// Function literals do not meet this definition, because they lexically capture variables. For
/// example, if you have `function() { return a; }`.
/// If it is evaluated in a different scope, then it captures a different variable. Even if
/// the function did not read any captured variables directly, it would still fail this definition,
/// because it affects the lifecycle of variables in the enclosing scope.
///
/// However, a function literal with respect to a particular scope is a literal.
/// If `include_functions` is true, all function expressions will be treated as literals.
pub trait IsLiteralValue<'a, 'b> {
    fn is_literal_value(&self, include_functions: bool) -> bool;
}

impl<'a, 'b> IsLiteralValue<'a, 'b> for Expression<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        match self {
            Self::FunctionExpression(_) | Self::ArrowExpression(_) => include_functions,
            Self::ArrayExpression(expr) => {
                expr.elements.iter().all(|element| element.is_literal_value(include_functions))
            }
            Self::ObjectExpression(expr) => {
                expr.properties.iter().all(|property| property.is_literal_value(include_functions))
            }
            _ => self.is_immutable_value(),
        }
    }
}

impl<'a, 'b> IsLiteralValue<'a, 'b> for ArrayExpressionElement<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        match self {
            Self::SpreadElement(element) => element.is_literal_value(include_functions),
            Self::Expression(expr) => expr.is_literal_value(include_functions),
            Self::Elision(_) => true,
        }
    }
}

impl<'a, 'b> IsLiteralValue<'a, 'b> for SpreadElement<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        self.argument.is_literal_value(include_functions)
    }
}

impl<'a, 'b> IsLiteralValue<'a, 'b> for ObjectPropertyKind<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        match self {
            Self::ObjectProperty(method) => method.is_literal_value(include_functions),
            Self::SpreadProperty(property) => property.is_literal_value(include_functions),
        }
    }
}

impl<'a, 'b> IsLiteralValue<'a, 'b> for ObjectProperty<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        self.key.is_literal_value(include_functions)
            && self.value.is_literal_value(include_functions)
    }
}

impl<'a, 'b> IsLiteralValue<'a, 'b> for PropertyKey<'a> {
    fn is_literal_value(&self, include_functions: bool) -> bool {
        match self {
            Self::Identifier(_) | Self::PrivateIdentifier(_) => false,
            Self::Expression(expr) => expr.is_literal_value(include_functions),
        }
    }
}

/// port from [closure-compiler](https://github.com/google/closure-compiler/blob/f3ce5ed8b630428e311fe9aa2e20d36560d975e2/src/com/google/javascript/jscomp/AstAnalyzer.java#L94)
/// Returns true if the node which may have side effects when executed.
/// This version default to the "safe" assumptions when the compiler object
/// is not provided (RegExp have side-effects, etc).
pub trait MayHaveSideEffects<'a, 'b>
where
    Self: CheckForStateChange<'a, 'b>,
{
    fn may_have_side_effects(&self) -> bool {
        self.check_for_state_change(false)
    }
}

/// port from [closure-compiler](https://github.com/google/closure-compiler/blob/f3ce5ed8b630428e311fe9aa2e20d36560d975e2/src/com/google/javascript/jscomp/AstAnalyzer.java#L241)
/// Returns true if some node in n's subtree changes application state. If
/// `check_for_new_objects` is true, we assume that newly created mutable objects (like object
/// literals) change state. Otherwise, we assume that they have no side effects.
pub trait CheckForStateChange<'a, 'b> {
    fn check_for_state_change(&self, check_for_new_objects: bool) -> bool;
}

impl<'a, 'b> CheckForStateChange<'a, 'b> for Expression<'a> {
    fn check_for_state_change(&self, _check_for_new_objects: bool) -> bool {
        match self {
            Self::NumberLiteral(_)
            | Self::BooleanLiteral(_)
            | Self::StringLiteral(_)
            | Self::BigintLiteral(_)
            | Self::NullLiteral(_)
            | Self::RegExpLiteral(_) => false,
            Self::Identifier(ident) => {
                !matches!(ident.name.as_str(), "undefined" | "Infinity" | "NaN")
            }
            Self::UnaryExpression(unary_expr) => {
                if is_simple_unary_operator(unary_expr.operator) {
                    return unary_expr.argument.check_for_state_change(_check_for_new_objects);
                }

                true
            }
            _ => true,
        }
    }
}

impl<'a, 'b> MayHaveSideEffects<'a, 'b> for Expression<'a> {}

fn is_simple_unary_operator(operator: UnaryOperator) -> bool {
    operator != UnaryOperator::Delete
}

/// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/NodeUtil.java#L348)
/// Gets the value of a node as a Number, or None if it cannot be converted.
pub fn get_number_value(expr: &Expression) -> Option<f64> {
    match expr {
        Expression::NumberLiteral(number_literal) => Some(number_literal.value),
        Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
            UnaryOperator::UnaryPlus => get_number_value(&unary_expr.argument),
            UnaryOperator::UnaryNegation => get_number_value(&unary_expr.argument).map(|v| -v),
            UnaryOperator::BitwiseNot => get_number_value(&unary_expr.argument)
                .map(|value| f64::from(!NumberLiteral::ecmascript_to_int32(value))),
            UnaryOperator::LogicalNot => {
                get_boolean_value(expr).map(|boolean| if boolean { 1.0 } else { 0.0 })
            }
            _ => None,
        },
        Expression::BooleanLiteral(bool_literal) => {
            if bool_literal.value {
                Some(1.0)
            } else {
                Some(0.0)
            }
        }
        Expression::NullLiteral(_) => Some(0.0),
        _ => None,
    }
}

/// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/NodeUtil.java#L109)
/// Gets the boolean value of a node that represents an expression, or `None` if no
/// such value can be determined by static analysis.
/// This method does not consider whether the node may have side-effects.
pub fn get_boolean_value(expr: &Expression) -> Option<bool> {
    match expr {
        Expression::RegExpLiteral(_) | Expression::ArrayExpression(_)| Expression::ArrowExpression(_)| Expression::ClassExpression(_) | Expression::FunctionExpression(_)| Expression::NewExpression(_) |  Expression::ObjectExpression(_) => Some(true),
        Expression::NullLiteral(_) => Some(false),
        Expression::BooleanLiteral(boolean_literal) =>  Some(boolean_literal.value),
        Expression::NumberLiteral(number_literal) => Some(number_literal.value != 0.0),
        Expression::BigintLiteral(big_int_literal) => Some(big_int_literal.value == BigUint::default()),
        Expression::StringLiteral(string_literal) => Some(!string_literal.value.is_empty()),
        Expression::TemplateLiteral(template_literal) => {
            if let Some(quasi) = template_literal.quasis.get(0) && quasi.tail {
                if quasi.value.cooked.as_ref().map_or(false, |cooked| !cooked.is_empty()) {
                    Some(true)
                } else {
                    Some(false)
                }
            } else {
                None
            }
        },
        Expression::Identifier(ident) => {
            if expr.is_undefined() {
                Some(false)
            } else if  ident.name == "Infinity" {
                Some(true)
            } else if ident.name == "NaN" {
                Some(false)
            } else {
                None
            }
        },
        Expression::AssignmentExpression(assign_expr) => {
            match assign_expr.operator {
                AssignmentOperator::LogicalAnd | AssignmentOperator::LogicalOr => {
                    None
                }
                // For ASSIGN, the value is the value of the RHS.
                _ =>  get_boolean_value(&assign_expr.right)
            }
        },
        Expression::LogicalExpression(logical_expr) => {
            let predict = |expr: &&Expression| get_boolean_value(expr) == Some(true);
            match logical_expr.operator {
                LogicalOperator::And => {
                    Some([&logical_expr.left, &logical_expr.right].iter().all(predict))
                },
                LogicalOperator::Or => {
                    Some([&logical_expr.left, &logical_expr.right].iter().any(predict))
                },
                LogicalOperator::Coalesce => None
            }
        },
        Expression::SequenceExpression(sequence_expr) => {
            // For sequence expression, the value is the value of the RHS.
            sequence_expr.expressions.last().map_or(None, get_boolean_value)
        },
        Expression::UnaryExpression(unary_expr) => {
            if unary_expr.operator == UnaryOperator::Void {
                Some(false)
            } else if matches!(unary_expr.operator, UnaryOperator::BitwiseNot | UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation) {
                match &unary_expr.argument {
                   Expression::NumberLiteral(number_literal) => Some(number_literal.value != 0.0),
                   Expression::BigintLiteral(big_int_literal) => Some(big_int_literal.value == BigUint::default()),
                   _ => None
                }
            } else if unary_expr.operator == UnaryOperator::LogicalNot {
                get_boolean_value(&unary_expr.argument).map(|boolean| !boolean) 
            } else {
                None
            }
        },
        _ => None
    }
}
