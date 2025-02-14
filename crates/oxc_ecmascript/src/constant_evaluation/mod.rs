use std::{borrow::Cow, cmp::Ordering};

use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive, Zero};

use equality_comparison::{abstract_equality_comparison, strict_equality_comparison};
use oxc_ast::{ast::*, AstBuilder};

use crate::{
    is_global_reference::IsGlobalReference, side_effects::MayHaveSideEffects, ToBigInt, ToBoolean,
    ToInt32, ToJsString, ToNumber,
};

mod equality_comparison;
mod is_literal_value;
mod value;
mod value_type;
pub use is_literal_value::IsLiteralValue;
pub use value::ConstantValue;
pub use value_type::{DetermineValueType, ValueType};

pub trait ConstantEvaluation<'a>: IsGlobalReference {
    fn ast(&self) -> AstBuilder<'a>;

    fn resolve_binding(&self, ident: &IdentifierReference<'a>) -> Option<ConstantValue<'a>> {
        match ident.name.as_str() {
            "undefined" if self.is_global_reference(ident)? => Some(ConstantValue::Undefined),
            "NaN" if self.is_global_reference(ident)? => Some(ConstantValue::Number(f64::NAN)),
            "Infinity" if self.is_global_reference(ident)? => {
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
        if value.is_some() && expr.may_have_side_effects(self) {
            None
        } else {
            value
        }
    }

    fn get_side_free_string_value(&self, expr: &Expression<'a>) -> Option<Cow<'a, str>> {
        let value = expr.to_js_string(self);
        if value.is_some() && !expr.may_have_side_effects(self) {
            return value;
        }
        None
    }

    fn get_side_free_boolean_value(&self, expr: &Expression<'a>) -> Option<bool> {
        let value = self.get_boolean_value(expr);
        if value.is_some() && !expr.may_have_side_effects(self) {
            return value;
        }
        None
    }

    fn get_side_free_bigint_value(&self, expr: &Expression<'a>) -> Option<BigInt> {
        let value = expr.to_big_int(self);
        if value.is_some() && expr.may_have_side_effects(self) {
            None
        } else {
            value
        }
    }

    fn get_boolean_value(&self, expr: &Expression<'a>) -> Option<bool> {
        match expr {
            Expression::Identifier(ident) => match ident.name.as_str() {
                "undefined" | "NaN" if self.is_global_reference(ident)? => Some(false),
                "Infinity" if self.is_global_reference(ident)? => Some(true),
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
            expr => expr.to_boolean(self),
        }
    }

    fn eval_to_number(&self, expr: &Expression<'a>) -> Option<f64> {
        match expr {
            Expression::Identifier(ident) => match ident.name.as_str() {
                "undefined" | "NaN" if self.is_global_reference(ident)? => Some(f64::NAN),
                "Infinity" if self.is_global_reference(ident)? => Some(f64::INFINITY),
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
            Expression::SequenceExpression(s) => {
                s.expressions.last().and_then(|e| self.eval_to_number(e))
            }
            // If the object is empty, `toString` / `valueOf` / `Symbol.toPrimitive` is not overridden.
            // (assuming that those methods in Object.prototype are not modified)
            // In that case, `ToPrimitive` returns `"[object Object]"`
            Expression::ObjectExpression(e) if e.properties.is_empty() => Some(f64::NAN),
            // `ToPrimitive` for RegExp object returns `"/regexp/"`
            Expression::RegExpLiteral(_) => Some(f64::NAN),
            Expression::ArrayExpression(arr) => {
                // If the array is empty, `ToPrimitive` returns `""`
                if arr.elements.is_empty() {
                    return Some(0.0);
                }
                if arr.elements.len() == 1 {
                    let first_element = arr.elements.first().unwrap();
                    return match first_element {
                        ArrayExpressionElement::SpreadElement(_) => None,
                        // `ToPrimitive` returns `""` for `[,]`
                        ArrayExpressionElement::Elision(_) => Some(0.0),
                        match_expression!(ArrayExpressionElement) => {
                            self.eval_to_number(first_element.to_expression())
                        }
                    };
                }

                let non_spread_element_count = arr
                    .elements
                    .iter()
                    .filter(|e| !matches!(e, ArrayExpressionElement::SpreadElement(_)))
                    .count();
                // If the array has at least 2 elements, `ToPrimitive` returns a string containing
                // `,` which is not included in `StringNumericLiteral`
                // So `ToNumber` returns `NaN`
                if non_spread_element_count >= 2 {
                    Some(f64::NAN)
                } else {
                    None
                }
            }
            expr => {
                use crate::ToNumber;
                expr.to_number(self)
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
                expr.to_big_int(self)
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
            Expression::BigIntLiteral(lit) => lit.to_big_int(self).map(ConstantValue::BigInt),
            Expression::StringLiteral(lit) => {
                Some(ConstantValue::String(Cow::Borrowed(lit.value.as_str())))
            }
            Expression::StaticMemberExpression(e) => self.eval_static_member_expression(e),
            Expression::ComputedMemberExpression(e) => self.eval_computed_member_expression(e),
            _ => None,
        }
    }

    fn eval_binary_expression(&self, e: &BinaryExpression<'a>) -> Option<ConstantValue<'a>> {
        self.eval_binary_operation(e.operator, &e.left, &e.right)
    }

    fn eval_binary_operation(
        &self,
        operator: BinaryOperator,
        left: &Expression<'a>,
        right: &Expression<'a>,
    ) -> Option<ConstantValue<'a>> {
        match operator {
            BinaryOperator::Addition => {
                if left.may_have_side_effects(self) || right.may_have_side_effects(self) {
                    return None;
                }
                let left_type = left.value_type(self);
                let right_type = right.value_type(self);
                if left_type.is_string() || right_type.is_string() {
                    let lval = self.eval_expression(left)?;
                    let rval = self.eval_expression(right)?;
                    let lstr = lval.to_js_string(self)?;
                    let rstr = rval.to_js_string(self)?;
                    return Some(ConstantValue::String(lstr + rstr));
                }
                if left_type.is_number() || right_type.is_number() {
                    let lval = self.eval_expression(left)?;
                    let rval = self.eval_expression(right)?;
                    let lnum = lval.to_number(self)?;
                    let rnum = rval.to_number(self)?;
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
                let val = match operator {
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
            #[expect(clippy::cast_sign_loss)]
            BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftRightZeroFill => {
                let left = self.get_side_free_number_value(left)?;
                let right = self.get_side_free_number_value(right)?;
                let left = left.to_int_32();
                let right = (right.to_int_32() as u32) & 31;
                Some(ConstantValue::Number(match operator {
                    BinaryOperator::ShiftLeft => f64::from(left << right),
                    BinaryOperator::ShiftRight => f64::from(left >> right),
                    BinaryOperator::ShiftRightZeroFill => f64::from((left as u32) >> right),
                    _ => unreachable!(),
                }))
            }
            BinaryOperator::LessThan => self.is_less_than(left, right).map(|value| match value {
                ConstantValue::Undefined => ConstantValue::Boolean(false),
                _ => value,
            }),
            BinaryOperator::GreaterThan => {
                self.is_less_than(right, left).map(|value| match value {
                    ConstantValue::Undefined => ConstantValue::Boolean(false),
                    _ => value,
                })
            }
            BinaryOperator::LessEqualThan => {
                self.is_less_than(right, left).map(|value| match value {
                    ConstantValue::Boolean(true) | ConstantValue::Undefined => {
                        ConstantValue::Boolean(false)
                    }
                    ConstantValue::Boolean(false) => ConstantValue::Boolean(true),
                    _ => unreachable!(),
                })
            }
            BinaryOperator::GreaterEqualThan => {
                self.is_less_than(left, right).map(|value| match value {
                    ConstantValue::Boolean(true) | ConstantValue::Undefined => {
                        ConstantValue::Boolean(false)
                    }
                    ConstantValue::Boolean(false) => ConstantValue::Boolean(true),
                    _ => unreachable!(),
                })
            }
            BinaryOperator::BitwiseAnd | BinaryOperator::BitwiseOR | BinaryOperator::BitwiseXOR => {
                if left.value_type(self).is_bigint() && right.value_type(self).is_bigint() {
                    let left_val = self.get_side_free_bigint_value(left)?;
                    let right_val = self.get_side_free_bigint_value(right)?;
                    let result_val: BigInt = match operator {
                        BinaryOperator::BitwiseAnd => left_val & right_val,
                        BinaryOperator::BitwiseOR => left_val | right_val,
                        BinaryOperator::BitwiseXOR => left_val ^ right_val,
                        _ => unreachable!(),
                    };
                    return Some(ConstantValue::BigInt(result_val));
                }
                let left_num = self.get_side_free_number_value(left);
                let right_num = self.get_side_free_number_value(right);
                if let (Some(left_val), Some(right_val)) = (left_num, right_num) {
                    let left_val_int = left_val.to_int_32();
                    let right_val_int = right_val.to_int_32();

                    let result_val: f64 = match operator {
                        BinaryOperator::BitwiseAnd => f64::from(left_val_int & right_val_int),
                        BinaryOperator::BitwiseOR => f64::from(left_val_int | right_val_int),
                        BinaryOperator::BitwiseXOR => f64::from(left_val_int ^ right_val_int),
                        _ => unreachable!(),
                    };
                    return Some(ConstantValue::Number(result_val));
                }
                None
            }
            BinaryOperator::Instanceof => {
                if left.may_have_side_effects(self) {
                    return None;
                }
                if let Expression::Identifier(right_ident) = right {
                    let name = right_ident.name.as_str();
                    if matches!(name, "Object" | "Number" | "Boolean" | "String")
                        && self.is_global_reference(right_ident) == Some(true)
                    {
                        let left_ty = left.value_type(self);
                        if left_ty.is_undetermined() {
                            return None;
                        }
                        return Some(ConstantValue::Boolean(
                            name == "Object" && left.value_type(self).is_object(),
                        ));
                    }
                }
                None
            }
            BinaryOperator::StrictEquality
            | BinaryOperator::StrictInequality
            | BinaryOperator::Equality
            | BinaryOperator::Inequality => {
                if left.may_have_side_effects(self) || right.may_have_side_effects(self) {
                    return None;
                }
                let value = match operator {
                    BinaryOperator::StrictEquality | BinaryOperator::StrictInequality => {
                        strict_equality_comparison(self, left, right)?
                    }
                    BinaryOperator::Equality | BinaryOperator::Inequality => {
                        abstract_equality_comparison(self, left, right)?
                    }
                    _ => unreachable!(),
                };
                Some(ConstantValue::Boolean(match operator {
                    BinaryOperator::StrictEquality | BinaryOperator::Equality => value,
                    BinaryOperator::StrictInequality | BinaryOperator::Inequality => !value,
                    _ => unreachable!(),
                }))
            }
            BinaryOperator::In => None,
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
                if expr.argument.may_have_side_effects(self) {
                    return None;
                }
                let arg_ty = expr.argument.value_type(self);
                let s = match arg_ty {
                    ValueType::BigInt => "bigint",
                    ValueType::Number => "number",
                    ValueType::String => "string",
                    ValueType::Boolean => "boolean",
                    ValueType::Undefined => "undefined",
                    ValueType::Null => "object",
                    _ => match &expr.argument {
                        Expression::ObjectExpression(_) | Expression::ArrayExpression(_) => {
                            "object"
                        }
                        Expression::ClassExpression(_)
                        | Expression::FunctionExpression(_)
                        | Expression::ArrowFunctionExpression(_) => "function",
                        _ => return None,
                    },
                };
                Some(ConstantValue::String(Cow::Borrowed(s)))
            }
            UnaryOperator::Void => {
                (!expr.argument.may_have_side_effects(self)).then_some(ConstantValue::Undefined)
            }
            UnaryOperator::LogicalNot => self
                .get_side_free_boolean_value(&expr.argument)
                .map(|b| !b)
                .map(ConstantValue::Boolean),
            UnaryOperator::UnaryPlus => {
                self.get_side_free_number_value(&expr.argument).map(ConstantValue::Number)
            }
            UnaryOperator::UnaryNegation => match expr.argument.value_type(self) {
                ValueType::BigInt => self
                    .get_side_free_bigint_value(&expr.argument)
                    .map(|v| -v)
                    .map(ConstantValue::BigInt),
                ValueType::Number => self
                    .get_side_free_number_value(&expr.argument)
                    .map(|v| if v.is_nan() { v } else { -v })
                    .map(ConstantValue::Number),
                ValueType::Undefined => Some(ConstantValue::Number(f64::NAN)),
                ValueType::Null => Some(ConstantValue::Number(-0.0)),
                _ => None,
            },
            UnaryOperator::BitwiseNot => match expr.argument.value_type(self) {
                ValueType::BigInt => self
                    .get_side_free_bigint_value(&expr.argument)
                    .map(|v| !v)
                    .map(ConstantValue::BigInt),
                #[expect(clippy::cast_lossless)]
                _ => self
                    .get_side_free_number_value(&expr.argument)
                    .map(|v| (!v.to_int_32()) as f64)
                    .map(ConstantValue::Number),
            },
            UnaryOperator::Delete => None,
        }
    }

    fn eval_static_member_expression(
        &self,
        expr: &StaticMemberExpression<'a>,
    ) -> Option<ConstantValue<'a>> {
        match expr.property.name.as_str() {
            "length" => {
                if let Some(ConstantValue::String(s)) = self.eval_expression(&expr.object) {
                    Some(ConstantValue::Number(s.encode_utf16().count().to_f64().unwrap()))
                } else {
                    if expr.object.may_have_side_effects(self) {
                        return None;
                    }
                    if let Expression::ArrayExpression(arr) = &expr.object {
                        Some(ConstantValue::Number(arr.elements.len().to_f64().unwrap()))
                    } else {
                        None
                    }
                }
            }
            _ => None,
        }
    }

    fn eval_computed_member_expression(
        &self,
        expr: &ComputedMemberExpression<'a>,
    ) -> Option<ConstantValue<'a>> {
        match &expr.expression {
            Expression::StringLiteral(s) if s.value == "length" => {
                if let Some(ConstantValue::String(s)) = self.eval_expression(&expr.object) {
                    Some(ConstantValue::Number(s.encode_utf16().count().to_f64().unwrap()))
                } else {
                    if expr.object.may_have_side_effects(self) {
                        return None;
                    }
                    if let Expression::ArrayExpression(arr) = &expr.object {
                        Some(ConstantValue::Number(arr.elements.len().to_f64().unwrap()))
                    } else {
                        None
                    }
                }
            }
            _ => None,
        }
    }

    /// <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-islessthan>
    fn is_less_than(&self, x: &Expression<'a>, y: &Expression<'a>) -> Option<ConstantValue<'a>> {
        if x.may_have_side_effects(self) || y.may_have_side_effects(self) {
            return None;
        }

        // a. Let px be ? ToPrimitive(x, NUMBER).
        // b. Let py be ? ToPrimitive(y, NUMBER).
        let px = x.value_type(self);
        let py = y.value_type(self);

        // If the operands are not primitives, `ToPrimitive` is *not* a noop.
        if px.is_undetermined() || px.is_object() || py.is_undetermined() || py.is_object() {
            return None;
        }

        // 3. If px is a String and py is a String, then
        if px.is_string() && py.is_string() {
            let left_string = x.to_js_string(self)?;
            let right_string = y.to_js_string(self)?;
            return Some(ConstantValue::Boolean(
                left_string.encode_utf16().cmp(right_string.encode_utf16()) == Ordering::Less,
            ));
        }

        // a. If px is a BigInt and py is a String, then
        if px.is_bigint() && py.is_string() {
            use crate::StringToBigInt;
            let ny = y.to_js_string(self)?.as_ref().string_to_big_int();
            let Some(ny) = ny else { return Some(ConstantValue::Undefined) };
            return Some(ConstantValue::Boolean(x.to_big_int(self)? < ny));
        }
        // b. If px is a String and py is a BigInt, then
        if px.is_string() && py.is_bigint() {
            use crate::StringToBigInt;
            let nx = x.to_js_string(self)?.as_ref().string_to_big_int();
            let Some(nx) = nx else { return Some(ConstantValue::Undefined) };
            return Some(ConstantValue::Boolean(nx < y.to_big_int(self)?));
        }

        // Both operands are primitives here.
        // ToNumeric returns a BigInt if the operand is a BigInt. Otherwise, it returns a Number.
        let nx_is_number = !px.is_bigint();
        let ny_is_number = !py.is_bigint();

        // f. If SameType(nx, ny) is true, then
        //   i. If nx is a Number, then
        if nx_is_number && ny_is_number {
            let left_num = self.eval_to_number(x)?;
            if left_num.is_nan() {
                return Some(ConstantValue::Undefined);
            }
            let right_num = self.eval_to_number(y)?;
            if right_num.is_nan() {
                return Some(ConstantValue::Undefined);
            }
            return Some(ConstantValue::Boolean(left_num < right_num));
        }
        //   ii. Else,
        if px.is_bigint() && py.is_bigint() {
            return Some(ConstantValue::Boolean(x.to_big_int(self)? < y.to_big_int(self)?));
        }

        let nx = self.eval_to_number(x);
        let ny = self.eval_to_number(y);

        // h. If nx or ny is NaN, return undefined.
        if nx_is_number && nx.is_some_and(f64::is_nan)
            || ny_is_number && ny.is_some_and(f64::is_nan)
        {
            return Some(ConstantValue::Undefined);
        }

        // i. If nx is -∞𝔽 or ny is +∞𝔽, return true.
        if nx_is_number && nx.is_some_and(|n| n == f64::NEG_INFINITY)
            || ny_is_number && ny.is_some_and(|n| n == f64::INFINITY)
        {
            return Some(ConstantValue::Boolean(true));
        }
        // j. If nx is +∞𝔽 or ny is -∞𝔽, return false.
        if nx_is_number && nx.is_some_and(|n| n == f64::INFINITY)
            || ny_is_number && ny.is_some_and(|n| n == f64::NEG_INFINITY)
        {
            return Some(ConstantValue::Boolean(false));
        }

        // k. If ℝ(nx) < ℝ(ny), return true; otherwise return false.
        if px.is_bigint() {
            let nx = x.to_big_int(self)?;
            let ny = self.eval_to_number(y)?;
            return compare_bigint_and_f64(&nx, ny)
                .map(|ord| ConstantValue::Boolean(ord == Ordering::Less));
        }
        if py.is_bigint() {
            let ny = y.to_big_int(self)?;
            let nx = self.eval_to_number(x)?;
            return compare_bigint_and_f64(&ny, nx)
                .map(|ord| ConstantValue::Boolean(ord.reverse() == Ordering::Less));
        }

        None
    }
}

fn compare_bigint_and_f64(x: &BigInt, y: f64) -> Option<Ordering> {
    let ny = BigInt::from_f64(y)?;

    let raw_ord = x.cmp(&ny);
    if raw_ord == Ordering::Equal {
        let fract_ord = 0.0f64.partial_cmp(&y.fract()).expect("both should be finite");
        Some(fract_ord)
    } else {
        Some(raw_ord)
    }
}

#[cfg(test)]
mod test {
    use super::compare_bigint_and_f64;
    use num_bigint::BigInt;
    use std::cmp::Ordering;

    #[test]
    fn test_compare_bigint_and_f64() {
        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), f64::NAN), None);
        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), f64::INFINITY), None);
        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), f64::NEG_INFINITY), None);

        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), 0.0), Some(Ordering::Greater));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(0), 0.0), Some(Ordering::Equal));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(-1), 0.0), Some(Ordering::Less));

        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), 0.9), Some(Ordering::Greater));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), 1.0), Some(Ordering::Equal));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(1), 1.1), Some(Ordering::Less));

        assert_eq!(compare_bigint_and_f64(&BigInt::from(-1), -1.1), Some(Ordering::Greater));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(-1), -1.0), Some(Ordering::Equal));
        assert_eq!(compare_bigint_and_f64(&BigInt::from(-1), -0.9), Some(Ordering::Less));
    }
}
