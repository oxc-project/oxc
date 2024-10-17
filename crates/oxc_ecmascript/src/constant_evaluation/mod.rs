mod is_litral_value;
mod r#type;
mod value;

use std::borrow::Cow;

use num_bigint::BigInt;
use num_traits::{One, Zero};

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{side_effects::MayHaveSideEffects, ToInt32, ToJsString};

pub use self::{is_litral_value::IsLiteralValue, r#type::ValueType, value::ConstantValue};

pub trait ConstantEvaluation<'a> {
    fn is_global_reference(&self, ident: &IdentifierReference<'a>) -> bool {
        matches!(ident.name.as_str(), "undefined" | "NaN" | "Infinity")
    }

    fn resolve_binding(&self, ident: &IdentifierReference<'a>) -> Option<ConstantValue<'a>> {
        match ident.name.as_str() {
            "undefined" if self.is_global_reference(ident) => Some(ConstantValue::Undefined),
            "NaN" if self.is_global_reference(ident) => Some(ConstantValue::Number(f64::NAN)),
            "Infinity" if self.is_global_reference(ident) => {
                Some(ConstantValue::Number(f64::INFINITY))
            }
            _ => None,
        }
    }

    fn get_side_free_number_value(&self, expr: &Expression<'a>) -> Option<f64> {
        let value = self.eval_to_number(expr);
        // Calculating the number value, if any, is likely to be faster than calculating side effects,
        // and there are only a very few cases where we can compute a number value, but there could
        // also be side effects. e.g. `void doSomething()` has value NaN, regardless of the behavior
        // of `doSomething()`
        if value.is_some() && expr.may_have_side_effects() {
            None
        } else {
            value
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

    fn eval_to_big_int(&self, expr: &Expression<'a>) -> Option<BigInt> {
        match expr {
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::UnaryPlus => self.eval_to_big_int(&unary_expr.argument),
                UnaryOperator::UnaryNegation => {
                    self.eval_to_big_int(&unary_expr.argument).map(|v| -v)
                }
                _ => None,
            },
            Expression::BigIntLiteral(_) => {
                use crate::ToBigInt;
                expr.to_big_int()
            }
            _ => None,
        }
    }

    fn eval_expression(&self, expr: &Expression<'a>) -> Option<ConstantValue<'a>> {
        match expr {
            Expression::BinaryExpression(e) => self.eval_binary_expression(e),
            Expression::LogicalExpression(e) => self.eval_logical_expression(e),
            Expression::UnaryExpression(e) => self.eval_unary_expression(e),
            Expression::Identifier(ident) => self.resolve_binding(ident),
            Expression::NumericLiteral(lit) => Some(ConstantValue::Number(lit.value)),
            Expression::StringLiteral(lit) => {
                Some(ConstantValue::String(Cow::Borrowed(lit.value.as_str())))
            }
            _ => None,
        }
    }

    fn eval_binary_expression(&self, expr: &BinaryExpression<'a>) -> Option<ConstantValue<'a>> {
        match expr.operator {
            BinaryOperator::Addition => {
                let left = &expr.left;
                let right = &expr.right;
                if left.may_have_side_effects() || right.may_have_side_effects() {
                    return None;
                }
                let left_type = ValueType::from(left);
                let right_type = ValueType::from(right);
                if left_type.is_string() || right_type.is_string() {
                    let lval = self.eval_expression(&expr.left)?;
                    let rval = self.eval_expression(&expr.right)?;
                    let lstr = lval.to_js_string()?;
                    let rstr = rval.to_js_string()?;
                    return Some(ConstantValue::String(lstr + rstr));
                }
                if left_type.is_number() || right_type.is_number() {
                    let lval = self.eval_expression(&expr.left)?;
                    let rval = self.eval_expression(&expr.right)?;
                    let lnum = lval.into_number()?;
                    let rnum = rval.into_number()?;
                    return Some(ConstantValue::Number(lnum + rnum));
                }
                None
            }
            BinaryOperator::Subtraction
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::Multiplication
            | BinaryOperator::Exponential => {
                let lval = self.eval_to_number(&expr.left)?;
                let rval = self.eval_to_number(&expr.right)?;
                let val = match expr.operator {
                    BinaryOperator::Subtraction => lval - rval,
                    BinaryOperator::Division => {
                        if rval.is_zero() {
                            if lval.is_sign_positive() {
                                f64::INFINITY
                            } else {
                                f64::NEG_INFINITY
                            }
                        } else {
                            lval / rval
                        }
                    }
                    BinaryOperator::Remainder => {
                        if !rval.is_zero() && rval.is_finite() {
                            lval % rval
                        } else if rval.is_infinite() {
                            f64::NAN
                        } else {
                            return None;
                        }
                    }
                    BinaryOperator::Multiplication => lval * rval,
                    BinaryOperator::Exponential => lval.powf(rval),
                    _ => unreachable!(),
                };
                Some(ConstantValue::Number(val))
            }
            #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftRightZeroFill => {
                let left_num = self.get_side_free_number_value(&expr.left);
                let right_num = self.get_side_free_number_value(&expr.right);
                if let (Some(left_val), Some(right_val)) = (left_num, right_num) {
                    if left_val.fract() != 0.0 || right_val.fract() != 0.0 {
                        return None;
                    }
                    // only the lower 5 bits are used when shifting, so don't do anything
                    // if the shift amount is outside [0,32)
                    if !(0.0..32.0).contains(&right_val) {
                        return None;
                    }
                    let right_val_int = right_val as u32;
                    let bits = left_val.to_int_32();

                    let result_val: f64 = match expr.operator {
                        BinaryOperator::ShiftLeft => f64::from(bits.wrapping_shl(right_val_int)),
                        BinaryOperator::ShiftRight => f64::from(bits.wrapping_shr(right_val_int)),
                        BinaryOperator::ShiftRightZeroFill => {
                            // JavaScript always treats the result of >>> as unsigned.
                            // We must force Rust to do the same here.
                            let bits = bits as u32;
                            let res = bits.wrapping_shr(right_val_int);
                            f64::from(res)
                        }
                        _ => unreachable!(),
                    };
                    return Some(ConstantValue::Number(result_val));
                }
                None
            }
            _ => None,
        }
    }

    fn eval_logical_expression(&self, expr: &LogicalExpression<'a>) -> Option<ConstantValue<'a>> {
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

    fn eval_unary_expression(&self, expr: &UnaryExpression<'a>) -> Option<ConstantValue<'a>> {
        match expr.operator {
            UnaryOperator::Typeof => {
                if !expr.argument.is_literal_value(true) {
                    return None;
                }
                let s = match &expr.argument {
                    Expression::FunctionExpression(_) => "function",
                    Expression::StringLiteral(_) => "string",
                    Expression::NumericLiteral(_) => "number",
                    Expression::BooleanLiteral(_) => "boolean",
                    Expression::NullLiteral(_)
                    | Expression::ObjectExpression(_)
                    | Expression::ArrayExpression(_) => "object",
                    Expression::UnaryExpression(e) if e.operator == UnaryOperator::Void => {
                        "undefined"
                    }
                    Expression::BigIntLiteral(_) => "bigint",
                    Expression::Identifier(ident) => match ident.name.as_str() {
                        "undefined" if self.is_global_reference(ident) => "undefined",
                        "NaN" | "Infinity" if self.is_global_reference(ident) => "number",
                        _ => return None,
                    },
                    _ => return None,
                };
                Some(ConstantValue::String(Cow::Borrowed(s)))
            }
            UnaryOperator::Void => {
                if (!expr.argument.is_number() || !expr.argument.is_number_0())
                    && !expr.may_have_side_effects()
                {
                    return Some(ConstantValue::Undefined);
                }
                None
            }
            UnaryOperator::LogicalNot => {
                // Don't fold !0 and !1 back to false.
                if let Expression::NumericLiteral(n) = &expr.argument {
                    if n.value.is_zero() || n.value.is_one() {
                        return None;
                    }
                }
                self.eval_to_boolean(&expr.argument).map(|b| !b).map(ConstantValue::Boolean)
            }
            UnaryOperator::UnaryPlus => {
                self.eval_to_number(&expr.argument).map(ConstantValue::Number)
            }
            UnaryOperator::UnaryNegation => {
                let ty = ValueType::from(&expr.argument);
                match ty {
                    ValueType::BigInt => {
                        self.eval_to_big_int(&expr.argument).map(|v| -v).map(ConstantValue::BigInt)
                    }
                    ValueType::Number => self
                        .eval_to_number(&expr.argument)
                        .map(|v| if v.is_nan() { v } else { -v })
                        .map(ConstantValue::Number),
                    _ => None,
                }
            }
            UnaryOperator::BitwiseNot => {
                let ty = ValueType::from(&expr.argument);
                match ty {
                    ValueType::BigInt => {
                        self.eval_to_big_int(&expr.argument).map(|v| !v).map(ConstantValue::BigInt)
                    }
                    #[expect(clippy::cast_lossless)]
                    ValueType::Number => self
                        .eval_to_number(&expr.argument)
                        .map(|v| !v.to_int_32())
                        .map(|v| v as f64)
                        .map(ConstantValue::Number),
                    _ => None,
                }
            }
            UnaryOperator::Delete => None,
        }
    }
}
