use std::{borrow::Cow, cmp::Ordering};

use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive, Zero};

use equality_comparison::{abstract_equality_comparison, strict_equality_comparison};
use oxc_ast::{AstBuilder, ast::*};

use crate::{
    ToBigInt, ToBoolean, ToInt32, ToJsString, ToNumber,
    side_effects::{MayHaveSideEffects, MayHaveSideEffectsContext},
    to_numeric::ToNumeric,
};

mod equality_comparison;
mod is_int32_or_uint32;
mod is_literal_value;
mod value;
mod value_type;
pub use is_int32_or_uint32::IsInt32OrUint32;
pub use is_literal_value::IsLiteralValue;
pub use value::ConstantValue;
pub use value_type::{DetermineValueType, ValueType};

pub trait ConstantEvaluationCtx<'a>: MayHaveSideEffectsContext<'a> {
    fn ast(&self) -> AstBuilder<'a>;
}

pub trait ConstantEvaluation<'a>: MayHaveSideEffects<'a> {
    /// Evaluate the expression to a constant value.
    ///
    /// Use the specific functions (e.g. [`ConstantEvaluation::evaluate_value_to_boolean`], [`ConstantEvaluation::evaluate_value`]).
    ///
    /// - target_ty: How the result will be used.
    ///   For example, if the result will be converted to a boolean,
    ///   passing `Some(ValueType::Boolean)` will allow to utilize that information.
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>>;

    /// Evaluate the expression to a constant value.
    ///
    /// If you know the result will be converted to a specific type, use other functions (e.g. [`ConstantEvaluation::evaluate_value_to_boolean`]).
    fn evaluate_value(&self, ctx: &impl ConstantEvaluationCtx<'a>) -> Option<ConstantValue<'a>> {
        self.evaluate_value_to(ctx, None)
    }

    /// Evaluate the expression to a constant value and convert it to a number.
    fn evaluate_value_to_number(&self, ctx: &impl ConstantEvaluationCtx<'a>) -> Option<f64> {
        self.evaluate_value_to(ctx, Some(ValueType::Number))?.to_number(ctx)
    }

    /// Evaluate the expression to a constant value and convert it to a bigint.
    fn evaluate_value_to_bigint(&self, ctx: &impl ConstantEvaluationCtx<'a>) -> Option<BigInt> {
        self.evaluate_value_to(ctx, Some(ValueType::BigInt))?.into_bigint()
    }

    /// Evaluate the expression to a constant value and convert it to a boolean.
    fn evaluate_value_to_boolean(&self, ctx: &impl ConstantEvaluationCtx<'a>) -> Option<bool> {
        self.evaluate_value_to(ctx, Some(ValueType::Boolean))?.to_boolean(ctx)
    }

    /// Evaluate the expression to a constant value and convert it to a string.
    fn evaluate_value_to_string(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
    ) -> Option<Cow<'a, str>> {
        self.evaluate_value_to(ctx, Some(ValueType::String))?.to_js_string(ctx)
    }

    fn get_side_free_number_value(&self, ctx: &impl ConstantEvaluationCtx<'a>) -> Option<f64> {
        let value = self.evaluate_value_to_number(ctx)?;
        // Calculating the number value, if any, is likely to be faster than calculating side effects,
        // and there are only a very few cases where we can compute a number value, but there could
        // also be side effects. e.g. `void doSomething()` has value NaN, regardless of the behavior
        // of `doSomething()`
        (!self.may_have_side_effects(ctx)).then_some(value)
    }

    fn get_side_free_bigint_value(&self, ctx: &impl ConstantEvaluationCtx<'a>) -> Option<BigInt> {
        let value = self.evaluate_value_to_bigint(ctx)?;
        (!self.may_have_side_effects(ctx)).then_some(value)
    }

    fn get_side_free_boolean_value(&self, ctx: &impl ConstantEvaluationCtx<'a>) -> Option<bool> {
        let value = self.evaluate_value_to_boolean(ctx)?;
        (!self.may_have_side_effects(ctx)).then_some(value)
    }

    fn get_side_free_string_value(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
    ) -> Option<Cow<'a, str>> {
        let value = self.evaluate_value_to_string(ctx)?;
        (!self.may_have_side_effects(ctx)).then_some(value)
    }
}

impl<'a> ConstantEvaluation<'a> for IdentifierReference<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        _target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>> {
        match self.name.as_str() {
            "undefined" if ctx.is_global_reference(self)? => Some(ConstantValue::Undefined),
            "NaN" if ctx.is_global_reference(self)? => Some(ConstantValue::Number(f64::NAN)),
            "Infinity" if ctx.is_global_reference(self)? => {
                Some(ConstantValue::Number(f64::INFINITY))
            }
            _ => self
                .reference_id
                .get()
                .and_then(|reference_id| ctx.get_constant_value_for_reference_id(reference_id)),
        }
    }
}

impl<'a> ConstantEvaluation<'a> for Expression<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>> {
        let result = match target_ty {
            Some(ValueType::Boolean) => self.to_boolean(ctx).map(ConstantValue::Boolean),
            Some(ValueType::Number) => self.to_number(ctx).map(ConstantValue::Number),
            Some(ValueType::BigInt) => self.to_big_int(ctx).map(ConstantValue::BigInt),
            Some(ValueType::String) => self.to_js_string(ctx).map(ConstantValue::String),
            _ => None,
        };
        if result.is_some() {
            return result;
        }

        match self {
            Expression::BinaryExpression(e) => e.evaluate_value_to(ctx, target_ty),
            Expression::LogicalExpression(e) => e.evaluate_value_to(ctx, target_ty),
            Expression::UnaryExpression(e) => e.evaluate_value_to(ctx, target_ty),
            Expression::Identifier(ident) => ident.evaluate_value_to(ctx, target_ty),
            Expression::NumericLiteral(lit) => Some(ConstantValue::Number(lit.value)),
            Expression::NullLiteral(_) => Some(ConstantValue::Null),
            Expression::BooleanLiteral(lit) => Some(ConstantValue::Boolean(lit.value)),
            Expression::BigIntLiteral(lit) => lit.to_big_int(ctx).map(ConstantValue::BigInt),
            Expression::StringLiteral(lit) => {
                Some(ConstantValue::String(Cow::Borrowed(lit.value.as_str())))
            }
            Expression::StaticMemberExpression(e) => e.evaluate_value_to(ctx, target_ty),
            Expression::ComputedMemberExpression(e) => e.evaluate_value_to(ctx, target_ty),
            Expression::SequenceExpression(e) => {
                // For sequence expression, the value is the value of the RHS.
                e.expressions.last().and_then(|e| e.evaluate_value_to(ctx, target_ty))
            }
            _ => None,
        }
    }
}

impl<'a> ConstantEvaluation<'a> for BinaryExpression<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>> {
        // FIXME: skipped for now to avoid performance regression, can be removed
        if target_ty == Some(ValueType::Boolean) {
            return None;
        }

        binary_operation_evaluate_value_to(self.operator, &self.left, &self.right, ctx, target_ty)
    }
}

pub fn binary_operation_evaluate_value<'a, Ctx: ConstantEvaluationCtx<'a>>(
    operator: BinaryOperator,
    left: &Expression<'a>,
    right: &Expression<'a>,
    ctx: &Ctx,
) -> Option<ConstantValue<'a>> {
    binary_operation_evaluate_value_to(operator, left, right, ctx, None)
}

fn binary_operation_evaluate_value_to<'a>(
    operator: BinaryOperator,
    left: &Expression<'a>,
    right: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
    _target_ty: Option<ValueType>,
) -> Option<ConstantValue<'a>> {
    match operator {
        BinaryOperator::Addition => {
            use crate::to_primitive::ToPrimitive;
            let left_to_primitive = left.to_primitive(ctx);
            let right_to_primitive = right.to_primitive(ctx);
            if left_to_primitive.is_string() == Some(true)
                || right_to_primitive.is_string() == Some(true)
            {
                let lval = left.evaluate_value_to_string(ctx)?;
                let rval = right.evaluate_value_to_string(ctx)?;
                return Some(ConstantValue::String(lval + rval));
            }
            let left_to_numeric_type = left_to_primitive.to_numeric(ctx);
            let right_to_numeric_type = right_to_primitive.to_numeric(ctx);
            if left_to_numeric_type.is_number() || right_to_numeric_type.is_number() {
                let lval = left.evaluate_value_to_number(ctx)?;
                let rval = right.evaluate_value_to_number(ctx)?;
                return Some(ConstantValue::Number(lval + rval));
            }
            if left_to_numeric_type.is_bigint() && right_to_numeric_type.is_bigint() {
                let lval = left.evaluate_value_to_bigint(ctx)?;
                let rval = right.evaluate_value_to_bigint(ctx)?;
                return Some(ConstantValue::BigInt(lval + rval));
            }
            None
        }
        BinaryOperator::Subtraction
        | BinaryOperator::Division
        | BinaryOperator::Remainder
        | BinaryOperator::Multiplication
        | BinaryOperator::Exponential => {
            let lval = left.evaluate_value_to_number(ctx)?;
            let rval = right.evaluate_value_to_number(ctx)?;
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
                BinaryOperator::Exponential => {
                    let result = lval.powf(rval);
                    // For now, ignore the result if it large or has a decimal part
                    // so that the output does not become bigger than the input.
                    if result.is_finite() && (result.fract() != 0.0 || result.log10() > 4.0) {
                        return None;
                    }
                    result
                }
                _ => unreachable!(),
            };
            Some(ConstantValue::Number(val))
        }
        #[expect(clippy::cast_sign_loss)]
        BinaryOperator::ShiftLeft
        | BinaryOperator::ShiftRight
        | BinaryOperator::ShiftRightZeroFill => {
            let left = left.evaluate_value_to_number(ctx)?;
            let right = right.evaluate_value_to_number(ctx)?;
            let left = left.to_int_32();
            let right = (right.to_int_32() as u32) & 31;
            Some(ConstantValue::Number(match operator {
                BinaryOperator::ShiftLeft => f64::from(left << right),
                BinaryOperator::ShiftRight => f64::from(left >> right),
                BinaryOperator::ShiftRightZeroFill => f64::from((left as u32) >> right),
                _ => unreachable!(),
            }))
        }
        BinaryOperator::LessThan => is_less_than(ctx, left, right).map(|value| match value {
            ConstantValue::Undefined => ConstantValue::Boolean(false),
            _ => value,
        }),
        BinaryOperator::GreaterThan => is_less_than(ctx, right, left).map(|value| match value {
            ConstantValue::Undefined => ConstantValue::Boolean(false),
            _ => value,
        }),
        BinaryOperator::LessEqualThan => is_less_than(ctx, right, left).map(|value| match value {
            ConstantValue::Boolean(true) | ConstantValue::Undefined => {
                ConstantValue::Boolean(false)
            }
            ConstantValue::Boolean(false) => ConstantValue::Boolean(true),
            _ => unreachable!(),
        }),
        BinaryOperator::GreaterEqualThan => {
            is_less_than(ctx, left, right).map(|value| match value {
                ConstantValue::Boolean(true) | ConstantValue::Undefined => {
                    ConstantValue::Boolean(false)
                }
                ConstantValue::Boolean(false) => ConstantValue::Boolean(true),
                _ => unreachable!(),
            })
        }
        BinaryOperator::BitwiseAnd | BinaryOperator::BitwiseOR | BinaryOperator::BitwiseXOR => {
            if left.value_type(ctx).is_bigint() && right.value_type(ctx).is_bigint() {
                let left_val = left.evaluate_value_to_bigint(ctx)?;
                let right_val = right.evaluate_value_to_bigint(ctx)?;
                let result_val: BigInt = match operator {
                    BinaryOperator::BitwiseAnd => left_val & right_val,
                    BinaryOperator::BitwiseOR => left_val | right_val,
                    BinaryOperator::BitwiseXOR => left_val ^ right_val,
                    _ => unreachable!(),
                };
                return Some(ConstantValue::BigInt(result_val));
            }
            let left_num = left.evaluate_value_to_number(ctx);
            let right_num = right.evaluate_value_to_number(ctx);
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
            if let Expression::Identifier(right_ident) = right {
                let name = right_ident.name.as_str();
                if matches!(name, "Object" | "Number" | "Boolean" | "String")
                    && ctx.is_global_reference(right_ident) == Some(true)
                {
                    let left_ty = left.value_type(ctx);
                    if left_ty.is_undetermined() {
                        return None;
                    }
                    return Some(ConstantValue::Boolean(
                        name == "Object" && left.value_type(ctx).is_object(),
                    ));
                }
            }
            None
        }
        BinaryOperator::StrictEquality
        | BinaryOperator::StrictInequality
        | BinaryOperator::Equality
        | BinaryOperator::Inequality => {
            let value = match operator {
                BinaryOperator::StrictEquality | BinaryOperator::StrictInequality => {
                    strict_equality_comparison(ctx, left, right)?
                }
                BinaryOperator::Equality | BinaryOperator::Inequality => {
                    abstract_equality_comparison(ctx, left, right)?
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

impl<'a> ConstantEvaluation<'a> for LogicalExpression<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>> {
        match self.operator {
            LogicalOperator::And => match self.left.evaluate_value_to_boolean(ctx) {
                Some(true) => self.right.evaluate_value(ctx),
                Some(false) => self.left.evaluate_value(ctx),
                None => {
                    // ToBoolean(a && false) -> false
                    if target_ty == Some(ValueType::Boolean)
                        && self.right.evaluate_value_to_boolean(ctx) == Some(false)
                    {
                        Some(ConstantValue::Boolean(false))
                    } else {
                        None
                    }
                }
            },
            LogicalOperator::Or => match self.left.evaluate_value_to_boolean(ctx) {
                Some(true) => self.left.evaluate_value(ctx),
                Some(false) => self.right.evaluate_value(ctx),
                None => {
                    // ToBoolean(a || true) -> true
                    if target_ty == Some(ValueType::Boolean)
                        && self.right.evaluate_value_to_boolean(ctx) == Some(true)
                    {
                        Some(ConstantValue::Boolean(true))
                    } else {
                        None
                    }
                }
            },
            LogicalOperator::Coalesce => None,
        }
    }
}

impl<'a> ConstantEvaluation<'a> for UnaryExpression<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        _target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>> {
        match self.operator {
            UnaryOperator::Typeof => {
                let arg_ty = self.argument.value_type(ctx);
                let s = match arg_ty {
                    ValueType::BigInt => "bigint",
                    ValueType::Number => "number",
                    ValueType::String => "string",
                    ValueType::Boolean => "boolean",
                    ValueType::Undefined => "undefined",
                    ValueType::Null => "object",
                    _ => match &self.argument {
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
            UnaryOperator::Void => Some(ConstantValue::Undefined),
            UnaryOperator::LogicalNot => {
                self.argument.evaluate_value_to_boolean(ctx).map(|b| !b).map(ConstantValue::Boolean)
            }
            UnaryOperator::UnaryPlus => {
                self.argument.evaluate_value_to_number(ctx).map(ConstantValue::Number)
            }
            UnaryOperator::UnaryNegation => match self.argument.value_type(ctx) {
                ValueType::BigInt => self
                    .argument
                    .evaluate_value_to_bigint(ctx)
                    .map(|v| -v)
                    .map(ConstantValue::BigInt),
                ValueType::Number => self
                    .argument
                    .evaluate_value_to_number(ctx)
                    .map(|v| if v.is_nan() { v } else { -v })
                    .map(ConstantValue::Number),
                ValueType::Undefined => Some(ConstantValue::Number(f64::NAN)),
                ValueType::Null => Some(ConstantValue::Number(-0.0)),
                _ => None,
            },
            UnaryOperator::BitwiseNot => match self.argument.value_type(ctx) {
                ValueType::BigInt => self
                    .argument
                    .evaluate_value_to_bigint(ctx)
                    .map(|v| !v)
                    .map(ConstantValue::BigInt),
                #[expect(clippy::cast_lossless)]
                _ => self
                    .argument
                    .evaluate_value_to_number(ctx)
                    .map(|v| (!v.to_int_32()) as f64)
                    .map(ConstantValue::Number),
            },
            UnaryOperator::Delete => None,
        }
    }
}

impl<'a> ConstantEvaluation<'a> for StaticMemberExpression<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        _target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>> {
        match self.property.name.as_str() {
            "length" => {
                if let Some(ConstantValue::String(s)) = self.object.evaluate_value(ctx) {
                    Some(ConstantValue::Number(s.encode_utf16().count().to_f64().unwrap()))
                } else if let Expression::ArrayExpression(arr) = &self.object {
                    Some(ConstantValue::Number(arr.elements.len().to_f64().unwrap()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl<'a> ConstantEvaluation<'a> for ComputedMemberExpression<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        _target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>> {
        match &self.expression {
            Expression::StringLiteral(s) if s.value == "length" => {
                if let Some(ConstantValue::String(s)) = self.object.evaluate_value(ctx) {
                    Some(ConstantValue::Number(s.encode_utf16().count().to_f64().unwrap()))
                } else if let Expression::ArrayExpression(arr) = &self.object {
                    Some(ConstantValue::Number(arr.elements.len().to_f64().unwrap()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

fn is_less_than<'a>(
    ctx: &impl ConstantEvaluationCtx<'a>,
    x: &Expression<'a>,
    y: &Expression<'a>,
) -> Option<ConstantValue<'a>> {
    // a. Let px be ? ToPrimitive(x, NUMBER).
    // b. Let py be ? ToPrimitive(y, NUMBER).
    let px = x.value_type(ctx);
    let py = y.value_type(ctx);

    // If the operands are not primitives, `ToPrimitive` is *not* a noop.
    if px.is_undetermined() || px.is_object() || py.is_undetermined() || py.is_object() {
        return None;
    }

    // 3. If px is a String and py is a String, then
    if px.is_string() && py.is_string() {
        let left_string = x.to_js_string(ctx)?;
        let right_string = y.to_js_string(ctx)?;
        return Some(ConstantValue::Boolean(
            left_string.encode_utf16().cmp(right_string.encode_utf16()) == Ordering::Less,
        ));
    }

    // a. If px is a BigInt and py is a String, then
    if px.is_bigint() && py.is_string() {
        use crate::StringToBigInt;
        let ny = y.to_js_string(ctx)?.as_ref().string_to_big_int();
        let Some(ny) = ny else { return Some(ConstantValue::Undefined) };
        return Some(ConstantValue::Boolean(x.to_big_int(ctx)? < ny));
    }
    // b. If px is a String and py is a BigInt, then
    if px.is_string() && py.is_bigint() {
        use crate::StringToBigInt;
        let nx = x.to_js_string(ctx)?.as_ref().string_to_big_int();
        let Some(nx) = nx else { return Some(ConstantValue::Undefined) };
        return Some(ConstantValue::Boolean(nx < y.to_big_int(ctx)?));
    }

    // Both operands are primitives here.
    // ToNumeric returns a BigInt if the operand is a BigInt. Otherwise, it returns a Number.
    let nx_is_number = !px.is_bigint();
    let ny_is_number = !py.is_bigint();

    // f. If SameType(nx, ny) is true, then
    //   i. If nx is a Number, then
    if nx_is_number && ny_is_number {
        let left_num = x.evaluate_value_to_number(ctx)?;
        if left_num.is_nan() {
            return Some(ConstantValue::Undefined);
        }
        let right_num = y.evaluate_value_to_number(ctx)?;
        if right_num.is_nan() {
            return Some(ConstantValue::Undefined);
        }
        return Some(ConstantValue::Boolean(left_num < right_num));
    }
    //   ii. Else,
    if px.is_bigint() && py.is_bigint() {
        return Some(ConstantValue::Boolean(x.to_big_int(ctx)? < y.to_big_int(ctx)?));
    }

    let nx = x.evaluate_value_to_number(ctx);
    let ny = y.evaluate_value_to_number(ctx);

    // h. If nx or ny is NaN, return undefined.
    if nx_is_number && nx.is_some_and(f64::is_nan) || ny_is_number && ny.is_some_and(f64::is_nan) {
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
        let nx = x.to_big_int(ctx)?;
        let ny = y.evaluate_value_to_number(ctx)?;
        return compare_bigint_and_f64(&nx, ny)
            .map(|ord| ConstantValue::Boolean(ord == Ordering::Less));
    }
    if py.is_bigint() {
        let ny = y.to_big_int(ctx)?;
        let nx = x.evaluate_value_to_number(ctx)?;
        return compare_bigint_and_f64(&ny, nx)
            .map(|ord| ConstantValue::Boolean(ord.reverse() == Ordering::Less));
    }

    None
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
