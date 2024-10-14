use core::f64;
use std::borrow::Cow;

use num_traits::Zero;
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

pub enum ConstantValue<'a> {
    Number(f64),
    String(Cow<'a, str>),
    Identifier,
    Undefined,
}

// impl<'a> ConstantValue<'a> {
// fn to_boolean(&self) -> Option<bool> {
// match self {
// Self::Number(n) => Some(!n.is_zero()),
// Self::String(s) => Some(!s.is_empty()),
// Self::Identifier => None,
// Self::Undefined => Some(false),
// }
// }
// }

pub trait ConstantEvaluation<'a> {
    fn is_global_reference(&self, ident: &IdentifierReference<'a>) -> bool {
        matches!(ident.name.as_str(), "undefined" | "NaN" | "Infinity")
    }

    fn resolve_binding(&self, ident: &IdentifierReference<'a>) -> Option<ConstantValue> {
        match ident.name.as_str() {
            "undefined" if self.is_global_reference(ident) => Some(ConstantValue::Undefined),
            "NaN" if self.is_global_reference(ident) => Some(ConstantValue::Number(f64::NAN)),
            "Infinity" if self.is_global_reference(ident) => {
                Some(ConstantValue::Number(f64::INFINITY))
            }
            _ => None,
        }
    }

    fn eval_to_boolean(&self, expr: &Expression<'a>) -> Option<bool> {
        match expr {
            Expression::Identifier(ident) => match ident.name.as_str() {
                "undefined" | "NaN" if self.is_global_reference(ident) => Some(false),
                "Infinity" if self.is_global_reference(ident) => Some(true),
                _ => None,
            },
            Expression::LogicalExpression(logical_expr) => {
                match logical_expr.operator {
                    // true && true -> true
                    // true && false -> false
                    // a && true -> None
                    LogicalOperator::And => {
                        let left = self.eval_to_boolean(&logical_expr.left);
                        let right = self.eval_to_boolean(&logical_expr.right);
                        match (left, right) {
                            (Some(true), Some(true)) => Some(true),
                            (Some(false), _) | (_, Some(false)) => Some(false),
                            (None, _) | (_, None) => None,
                        }
                    }
                    // true || false -> true
                    // false || false -> false
                    // a || b -> None
                    LogicalOperator::Or => {
                        let left = self.eval_to_boolean(&logical_expr.left);
                        let right = self.eval_to_boolean(&logical_expr.right);
                        match (left, right) {
                            (Some(true), _) | (_, Some(true)) => Some(true),
                            (Some(false), Some(false)) => Some(false),
                            (None, _) | (_, None) => None,
                        }
                    }
                    LogicalOperator::Coalesce => None,
                }
            }
            Expression::SequenceExpression(sequence_expr) => {
                // For sequence expression, the value is the value of the RHS.
                sequence_expr.expressions.last().and_then(|e| self.eval_to_boolean(e))
            }
            Expression::UnaryExpression(unary_expr) => {
                match unary_expr.operator {
                    UnaryOperator::Void => Some(false),

                    UnaryOperator::BitwiseNot
                    | UnaryOperator::UnaryPlus
                    | UnaryOperator::UnaryNegation => {
                        // `~0 -> true` `+1 -> true` `+0 -> false` `-0 -> false`
                        self.eval_to_number(expr).map(|value| !value.is_zero())
                    }
                    UnaryOperator::LogicalNot => {
                        // !true -> false
                        self.eval_to_boolean(&unary_expr.argument).map(|b| !b)
                    }
                    _ => None,
                }
            }
            Expression::AssignmentExpression(assign_expr) => {
                match assign_expr.operator {
                    AssignmentOperator::LogicalAnd | AssignmentOperator::LogicalOr => None,
                    // For ASSIGN, the value is the value of the RHS.
                    _ => self.eval_to_boolean(&assign_expr.right),
                }
            }
            expr => {
                use crate::ToBoolean;
                expr.to_boolean()
            }
        }
    }

    fn eval_to_number(&self, expr: &Expression<'a>) -> Option<f64> {
        match expr {
            Expression::Identifier(ident) => match ident.name.as_str() {
                "undefined" | "NaN" if self.is_global_reference(ident) => Some(f64::NAN),
                "Infinity" if self.is_global_reference(ident) => Some(f64::INFINITY),
                _ => None,
            },
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::UnaryPlus => self.eval_to_number(&unary_expr.argument),
                UnaryOperator::UnaryNegation => {
                    self.eval_to_number(&unary_expr.argument).map(|v| -v)
                }
                // UnaryOperator::BitwiseNot => {
                // unary_expr.argument.to_number().map(|value| {
                // match value {
                // NumberValue::Number(num) => NumberValue::Number(f64::from(
                // !NumericLiteral::ecmascript_to_int32(num),
                // )),
                // // ~Infinity -> -1
                // // ~-Infinity -> -1
                // // ~NaN -> -1
                // _ => NumberValue::Number(-1_f64),
                // }
                // })
                // }
                UnaryOperator::LogicalNot => {
                    self.eval_to_boolean(expr).map(|b| if b { 1_f64 } else { 0_f64 })
                }
                UnaryOperator::Void => Some(f64::NAN),
                _ => None,
            },
            expr => {
                use crate::ToNumber;
                expr.to_number()
            }
        }
    }

    fn eval_expression(&self, expr: &Expression<'a>) -> Option<ConstantValue> {
        match expr {
            Expression::LogicalExpression(e) => self.eval_logical_expression(e),
            Expression::Identifier(ident) => self.resolve_binding(ident),
            _ => None,
        }
    }

    fn eval_logical_expression(&self, expr: &LogicalExpression<'a>) -> Option<ConstantValue> {
        match expr.operator {
            LogicalOperator::And => {
                if self.eval_to_boolean(&expr.left) == Some(true) {
                    self.eval_expression(&expr.right)
                } else {
                    self.eval_expression(&expr.left)
                }
            }
            _ => None,
        }
    }
}
