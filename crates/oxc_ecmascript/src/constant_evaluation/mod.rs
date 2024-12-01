mod is_literal_value;
mod value;
mod value_type;

use std::{borrow::Cow, cmp::Ordering};

use num_bigint::BigInt;
use num_traits::Zero;

use oxc_ast::ast::*;

use crate::{side_effects::MayHaveSideEffects, ToBigInt, ToBoolean, ToInt32, ToJsString, ToNumber};

pub use self::{is_literal_value::IsLiteralValue, value::ConstantValue, value_type::ValueType};

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

    fn get_side_free_string_value(&self, expr: &Expression<'a>) -> Option<Cow<'a, str>> {
        let value = expr.to_js_string();
        if value.is_some() && !expr.may_have_side_effects() {
            return value;
        }
        None
    }

    fn get_side_free_boolean_value(&self, expr: &Expression<'a>) -> Option<bool> {
        let value = expr.to_boolean();
        if value.is_some() && !expr.may_have_side_effects() {
            return value;
        }
        None
    }

    fn get_side_free_bigint_value(&self, expr: &Expression<'a>) -> Option<BigInt> {
        let value = expr.to_big_int();
        if value.is_some() && expr.may_have_side_effects() {
            None
        } else {
            value
        }
    }

    fn get_boolean_value(&self, expr: &Expression<'a>) -> Option<bool> {
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
                        let left = self.get_boolean_value(&logical_expr.left);
                        let right = self.get_boolean_value(&logical_expr.right);
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
                        let left = self.get_boolean_value(&logical_expr.left);
                        let right = self.get_boolean_value(&logical_expr.right);
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
                sequence_expr.expressions.last().and_then(|e| self.get_boolean_value(e))
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
                        self.get_boolean_value(&unary_expr.argument).map(|b| !b)
                    }
                    _ => None,
                }
            }
            Expression::AssignmentExpression(assign_expr) => {
                match assign_expr.operator {
                    AssignmentOperator::LogicalAnd | AssignmentOperator::LogicalOr => None,
                    // For ASSIGN, the value is the value of the RHS.
                    _ => self.get_boolean_value(&assign_expr.right),
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
                    self.get_boolean_value(expr).map(|b| if b { 1_f64 } else { 0_f64 })
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
            Expression::NullLiteral(_) => Some(ConstantValue::Null),
            Expression::BooleanLiteral(lit) => Some(ConstantValue::Boolean(lit.value)),
            Expression::BigIntLiteral(lit) => lit.to_big_int().map(ConstantValue::BigInt),
            Expression::StringLiteral(lit) => {
                Some(ConstantValue::String(Cow::Borrowed(lit.value.as_str())))
            }
            _ => None,
        }
    }

    fn eval_binary_expression(&self, e: &BinaryExpression<'a>) -> Option<ConstantValue<'a>> {
        let left = &e.left;
        let right = &e.right;
        match e.operator {
            BinaryOperator::Addition => {
                if left.may_have_side_effects() || right.may_have_side_effects() {
                    return None;
                }
                let left_type = ValueType::from(left);
                let right_type = ValueType::from(right);
                if left_type.is_string() || right_type.is_string() {
                    let lval = self.eval_expression(left)?;
                    let rval = self.eval_expression(right)?;
                    let lstr = lval.to_js_string()?;
                    let rstr = rval.to_js_string()?;
                    return Some(ConstantValue::String(lstr + rstr));
                }
                if left_type.is_number() || right_type.is_number() {
                    let lval = self.eval_expression(left)?;
                    let rval = self.eval_expression(right)?;
                    let lnum = lval.to_number()?;
                    let rnum = rval.to_number()?;
                    return Some(ConstantValue::Number(lnum + rnum));
                }
                None
            }
            BinaryOperator::Subtraction
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::Multiplication
            | BinaryOperator::Exponential => {
                let lval = self.eval_to_number(left)?;
                let rval = self.eval_to_number(right)?;
                let val = match e.operator {
                    BinaryOperator::Subtraction => lval - rval,
                    BinaryOperator::Division => lval / rval,
                    BinaryOperator::Remainder => {
                        if rval.is_zero() {
                            f64::NAN
                        } else {
                            lval % rval
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
                let left_num = self.get_side_free_number_value(left);
                let right_num = self.get_side_free_number_value(right);
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

                    let result_val: f64 = match e.operator {
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
            BinaryOperator::LessThan => {
                self.is_less_than(left, right, true).map(|value| match value {
                    ConstantValue::Undefined => ConstantValue::Boolean(false),
                    _ => value,
                })
            }
            BinaryOperator::GreaterThan => {
                self.is_less_than(right, left, false).map(|value| match value {
                    ConstantValue::Undefined => ConstantValue::Boolean(false),
                    _ => value,
                })
            }
            BinaryOperator::LessEqualThan => {
                self.is_less_than(right, left, false).map(|value| match value {
                    ConstantValue::Boolean(true) | ConstantValue::Undefined => {
                        ConstantValue::Boolean(false)
                    }
                    ConstantValue::Boolean(false) => ConstantValue::Boolean(true),
                    _ => unreachable!(),
                })
            }
            BinaryOperator::GreaterEqualThan => {
                self.is_less_than(left, right, true).map(|value| match value {
                    ConstantValue::Boolean(true) | ConstantValue::Undefined => {
                        ConstantValue::Boolean(false)
                    }
                    ConstantValue::Boolean(false) => ConstantValue::Boolean(true),
                    _ => unreachable!(),
                })
            }
            _ => None,
        }
    }

    fn eval_logical_expression(&self, expr: &LogicalExpression<'a>) -> Option<ConstantValue<'a>> {
        match expr.operator {
            LogicalOperator::And => {
                if self.get_boolean_value(&expr.left) == Some(true) {
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
                let s = match &expr.argument {
                    Expression::ObjectExpression(_) | Expression::ArrayExpression(_)
                        if expr.argument.is_literal_value(true) =>
                    {
                        "object"
                    }
                    Expression::FunctionExpression(_) => "function",
                    Expression::StringLiteral(_) => "string",
                    Expression::NumericLiteral(_) => "number",
                    Expression::BooleanLiteral(_) => "boolean",
                    Expression::NullLiteral(_) => "object",
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
                self.get_boolean_value(&expr.argument).map(|b| !b).map(ConstantValue::Boolean)
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

    /// <https://tc39.es/ecma262/#sec-abstract-relational-comparison>
    fn is_less_than(
        &self,
        left_expr: &Expression<'a>,
        right_expr: &Expression<'a>,
        _left_first: bool,
    ) -> Option<ConstantValue<'a>> {
        let left = ValueType::from(left_expr);
        let right = ValueType::from(right_expr);

        if left.is_string() && right.is_string() {
            let left_string = self.get_side_free_string_value(left_expr);
            let right_string = self.get_side_free_string_value(right_expr);
            if let (Some(left_string), Some(right_string)) = (left_string, right_string) {
                // In JS, browsers parse \v differently. So do not compare strings if one contains \v.
                if left_string.contains('\u{000B}') || right_string.contains('\u{000B}') {
                    return None;
                }
                return Some(ConstantValue::Boolean(
                    left_string.cmp(&right_string) == Ordering::Less,
                ));
            }
        }

        // TODO: bigint is handled very differently in the spec
        // See <https://tc39.es/ecma262/#sec-islessthan>
        if left.is_bigint() || right.is_bigint() {
            return None;
        }

        let left_num = self.get_side_free_number_value(left_expr)?;
        let right_num = self.get_side_free_number_value(right_expr)?;

        if left_num.is_nan() || right_num.is_nan() {
            return Some(ConstantValue::Undefined);
        }

        Some(ConstantValue::Boolean(left_num < right_num))
    }
}
