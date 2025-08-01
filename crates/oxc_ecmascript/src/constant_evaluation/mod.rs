use std::borrow::Cow;

use cow_utils::CowUtils;
use num_bigint::BigInt;
use num_traits::{ToPrimitive, Zero};
use oxc_syntax::number::ToJsString;

use equality_comparison::{abstract_equality_comparison, strict_equality_comparison};
use oxc_allocator::Vec;
use oxc_ast::{AstBuilder, ast::*};
use oxc_syntax::reference::ReferenceId;

use crate::{
    ToBigInt, ToBoolean, ToInt32, ToJsString as ToJsStringTrait, ToNumber,
    is_less_than::is_less_than,
    side_effects::{MayHaveSideEffects, MayHaveSideEffectsContext},
    to_numeric::ToNumeric,
    StringCharAt, StringCharAtResult, StringCharCodeAt, 
    StringIndexOf, StringLastIndexOf, StringSubstring,
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
    fn is_global_reference_ce(&self, ident: &IdentifierReference<'a>) -> Option<bool>;
    fn get_constant_value_for_reference_id_ce(&self, reference_id: ReferenceId) -> Option<ConstantValue<'a>>;
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

impl<'a, T: ConstantEvaluation<'a>> ConstantEvaluation<'a> for Option<T> {
    fn evaluate_value(&self, ctx: &impl ConstantEvaluationCtx<'a>) -> Option<ConstantValue<'a>> {
        self.as_ref().and_then(|t| t.evaluate_value(ctx))
    }

    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>> {
        self.as_ref().and_then(|t| t.evaluate_value_to(ctx, target_ty))
    }
}

impl<'a> ConstantEvaluation<'a> for IdentifierReference<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        _target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>> {
        match self.name.as_str() {
            "undefined" if ctx.is_global_reference_ce(self)? => Some(ConstantValue::Undefined),
            "NaN" if ctx.is_global_reference_ce(self)? => Some(ConstantValue::Number(f64::NAN)),
            "Infinity" if ctx.is_global_reference_ce(self)? => {
                Some(ConstantValue::Number(f64::INFINITY))
            }
            _ => self
                .reference_id
                .get()
                .and_then(|reference_id| ctx.get_constant_value_for_reference_id_ce(reference_id)),
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
            Expression::CallExpression(e) => e.evaluate_value_to(ctx, target_ty),
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

impl<'a> ConstantEvaluation<'a> for CallExpression<'a> {
    fn evaluate_value_to(
        &self,
        ctx: &impl ConstantEvaluationCtx<'a>,
        _target_ty: Option<ValueType>,
    ) -> Option<ConstantValue<'a>> {
        try_fold_known_global_methods(&self.callee, &self.arguments, ctx)
    }
}

fn try_fold_known_global_methods<'a>(
    callee: &Expression<'a>,
    arguments: &Vec<'a, Argument<'a>>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    let (name, object) = match callee {
        Expression::StaticMemberExpression(member) if !member.optional => {
            (member.property.name.as_str(), &member.object)
        }
        Expression::ComputedMemberExpression(member) if !member.optional => {
            match &member.expression {
                Expression::StringLiteral(s) => (s.value.as_str(), &member.object),
                _ => return None,
            }
        }
        _ => return None,
    };
    match name {
        "toLowerCase" | "toUpperCase" | "trim" | "trimStart" | "trimEnd" => {
            try_fold_string_casing(arguments, name, object, ctx)
        }
        "substring" | "slice" => {
            try_fold_string_substring_or_slice(arguments, object, ctx)
        }
        "indexOf" | "lastIndexOf" => {
            try_fold_string_index_of(arguments, name, object, ctx)
        }
        "charAt" => try_fold_string_char_at(arguments, object, ctx),
        "charCodeAt" => try_fold_string_char_code_at(arguments, object, ctx),
        "startsWith" => try_fold_starts_with(arguments, object, ctx),
        "replace" | "replaceAll" => {
            try_fold_string_replace(arguments, name, object, ctx)
        }
        "fromCharCode" => try_fold_string_from_char_code(arguments, object, ctx),
        "toString" => try_fold_to_string(arguments, object, ctx),
        "isFinite" | "isNaN" | "isInteger" | "isSafeInteger" => {
            try_fold_number_methods(arguments, object, name, ctx)
        }
        "sqrt" | "cbrt" => try_fold_roots(arguments, name, object, ctx),
        "abs" | "ceil" | "floor" | "round" | "fround" | "trunc" | "sign" => {
            try_fold_math_unary(arguments, name, object, ctx)
        }
        "min" | "max" => try_fold_math_variadic(arguments, name, object, ctx),
        "of" => try_fold_array_of(arguments, name, object, ctx),
        _ => None,
    }
}

fn try_fold_string_casing<'a>(
    args: &Vec<'a, Argument<'a>>,
    name: &str,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if !args.is_empty() {
        return None;
    }

    let value = match object {
        Expression::StringLiteral(s) => Cow::Borrowed(s.value.as_str()),
        Expression::Identifier(ident) => {
            ident
                .reference_id
                .get()
                .and_then(|reference_id| ctx.get_constant_value_for_reference_id_ce(reference_id))
                .and_then(ConstantValue::into_string)?
        }
        _ => return None,
    };

    let result = match name {
        "toLowerCase" => value.to_lowercase(),
        "toUpperCase" => value.to_uppercase(), 
        "trim" => value.trim().to_string(),
        "trimStart" => value.trim_start().to_string(),
        "trimEnd" => value.trim_end().to_string(),
        _ => return None,
    };
    Some(ConstantValue::String(Cow::Owned(result)))
}

fn try_fold_string_index_of<'a>(
    args: &Vec<'a, Argument<'a>>,
    name: &str,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.len() >= 3 {
        return None;
    }
    let Expression::StringLiteral(s) = object else { return None };
    let search_value = match args.first() {
        Some(Argument::SpreadElement(_)) => return None,
        Some(arg @ match_expression!(Argument)) => {
            Some(arg.to_expression().get_side_free_string_value(ctx)?)
        }
        None => None,
    };
    let search_start_index = match args.get(1) {
        Some(Argument::SpreadElement(_)) => return None,
        Some(arg @ match_expression!(Argument)) => {
            Some(arg.to_expression().get_side_free_number_value(ctx)?)
        }
        None => None,
    };
    
    let result = match name {
        "indexOf" => s.value.as_str().index_of(search_value.as_deref(), search_start_index),
        "lastIndexOf" => {
            s.value.as_str().last_index_of(search_value.as_deref(), search_start_index)
        }
        _ => unreachable!(),
    };
    Some(ConstantValue::Number(result as f64))
}

fn try_fold_string_substring_or_slice<'a>(
    args: &Vec<'a, Argument<'a>>,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.len() > 2 {
        return None;
    }
    let Expression::StringLiteral(s) = object else { return None };
    let start_idx = match args.first() {
        Some(Argument::SpreadElement(_)) => return None,
        Some(arg @ match_expression!(Argument)) => {
            Some(arg.to_expression().get_side_free_number_value(ctx)?)
        }
        None => None,
    };
    let end_idx = match args.get(1) {
        Some(Argument::SpreadElement(_)) => return None,
        Some(arg @ match_expression!(Argument)) => {
            Some(arg.to_expression().get_side_free_number_value(ctx)?)
        }
        None => None,
    };
    if start_idx.is_some_and(|start| start > s.value.len() as f64 || start < 0.0)
        || end_idx.is_some_and(|end| end > s.value.len() as f64 || end < 0.0)
    {
        return None;
    }
    if let (Some(start), Some(end)) = (start_idx, end_idx) {
        if start > end {
            return None;
        }
    }
    
    Some(ConstantValue::String(Cow::Owned(
        s.value.as_str().substring(start_idx, end_idx)
    )))
}

fn try_fold_string_char_at<'a>(
    args: &Vec<'a, Argument<'a>>,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.len() > 1 {
        return None;
    }
    let Expression::StringLiteral(s) = object else { return None };
    let char_at_index = match args.first() {
        Some(Argument::SpreadElement(_)) => return None,
        Some(arg @ match_expression!(Argument)) => {
            Some(arg.to_expression().get_side_free_number_value(ctx)?)
        }
        None => None,
    };
    
    let result = match s.value.as_str().char_at(char_at_index) {
        StringCharAtResult::Value(c) => c.to_string(),
        StringCharAtResult::InvalidChar(_) => return None,
        StringCharAtResult::OutOfRange => String::new(),
    };
    Some(ConstantValue::String(Cow::Owned(result)))
}

fn try_fold_string_char_code_at<'a>(
    args: &Vec<'a, Argument<'a>>,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    let Expression::StringLiteral(s) = object else { return None };
    let char_at_index = match args.first() {
        Some(Argument::SpreadElement(_)) => return None,
        Some(arg @ match_expression!(Argument)) => {
            Some(arg.to_expression().get_side_free_number_value(ctx)?)
        }
        None => None,
    };
    
    let value = s.value.as_str().char_code_at(char_at_index).map_or(f64::NAN, |n| n as f64);
    Some(ConstantValue::Number(value))
}

fn try_fold_starts_with<'a>(
    args: &Vec<'a, Argument<'a>>,
    object: &Expression<'a>,
    _ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.len() != 1 {
        return None;
    }
    let Argument::StringLiteral(arg) = args.first().unwrap() else { return None };
    let Expression::StringLiteral(s) = object else { return None };
    Some(ConstantValue::Boolean(s.value.starts_with(arg.value.as_str())))
}

fn try_fold_string_replace<'a>(
    args: &Vec<'a, Argument<'a>>,
    name: &str,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if args.len() != 2 {
        return None;
    }
    let Expression::StringLiteral(s) = object else { return None };
    let search_value = args.first().unwrap();
    let search_value = match search_value {
        Argument::SpreadElement(_) => return None,
        match_expression!(Argument) => {
            let value = search_value.to_expression();
            if value.may_have_side_effects(ctx) {
                return None;
            }
            value.evaluate_value(ctx)?.into_string()?
        }
    };
    let replace_value = args.get(1).unwrap();
    let replace_value = match replace_value {
        Argument::SpreadElement(_) => return None,
        match_expression!(Argument) => {
            replace_value.to_expression().get_side_free_string_value(ctx)?
        }
    };
    if replace_value.contains('$') {
        return None;
    }
    let result = match name {
        "replace" => s.value.as_str().cow_replacen(search_value.as_ref(), &replace_value, 1),
        "replaceAll" => s.value.as_str().cow_replace(search_value.as_ref(), &replace_value),
        _ => unreachable!(),
    };
    Some(ConstantValue::String(result))
}

fn try_fold_string_from_char_code<'a>(
    args: &Vec<'a, Argument<'a>>,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    let Expression::Identifier(ident) = object else { return None };
    if ident.name != "String" || ctx.is_global_reference_ce(ident) != Some(true) {
        return None;
    }
    let mut s = String::with_capacity(args.len());
    for arg in args {
        let expr = arg.as_expression()?;
        let v = expr.get_side_free_number_value(ctx)?;
        let v = v.to_int_32() as u16 as u32;
        let c = char::try_from(v).ok()?;
        s.push(c);
    }
    Some(ConstantValue::String(Cow::Owned(s)))
}

fn try_fold_to_string<'a>(
    args: &Vec<'a, Argument<'a>>,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    match object {
        // Number.prototype.toString()
        // Number.prototype.toString(radix)
        Expression::NumericLiteral(lit) if args.len() <= 1 => {
            let mut radix: u32 = 0;
            if args.is_empty() {
                radix = 10;
            }
            if let Some(Argument::NumericLiteral(n)) = args.first() {
                if n.value >= 2.0 && n.value <= 36.0 && n.value.fract() == 0.0 {
                    radix = n.value as u32;
                }
            }
            if radix == 0 {
                return None;
            }
            if radix == 10 {
                let s = lit.value.to_js_string();
                return Some(ConstantValue::String(Cow::Owned(s)));
            }
            // Only convert integers for other radix values.
            let value = lit.value;
            if value.is_infinite() {
                let s = if value.is_sign_negative() { "-Infinity" } else { "Infinity" };
                return Some(ConstantValue::String(Cow::Borrowed(s)));
            }
            if value.is_nan() {
                return Some(ConstantValue::String(Cow::Borrowed("NaN")));
            }
            if value >= 0.0 && value.fract() != 0.0 {
                return None;
            }
            let i = value as u32;
            if i as f64 != value {
                return None;
            }
            let result = format_radix(i, radix);
            Some(ConstantValue::String(Cow::Owned(result)))
        }
        // `null` returns type errors
        Expression::BooleanLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::StringLiteral(_)
            if args.is_empty() =>
        {
            object.to_js_string(ctx).map(ConstantValue::String)
        }
        _ => None,
    }
}

fn format_radix(mut x: u32, radix: u32) -> String {
    debug_assert!((2..=36).contains(&radix));
    let mut result = vec![];
    loop {
        let m = x % radix;
        x /= radix;
        result.push(std::char::from_digit(m, radix).unwrap());
        if x == 0 {
            break;
        }
    }
    result.into_iter().rev().collect()
}

fn validate_global_reference<'a>(
    expr: &Expression<'a>,
    target: &str,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> bool {
    let Expression::Identifier(ident) = expr else { return false };
    ctx.is_global_reference_ce(ident) == Some(true) && ident.name == target
}

fn validate_arguments(args: &Vec<'_, Argument<'_>>, expected_len: usize) -> bool {
    (args.len() == expected_len) && args.iter().all(Argument::is_expression)
}

fn try_fold_number_methods<'a>(
    args: &Vec<'a, Argument<'a>>,
    object: &Expression<'a>,
    name: &str,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if !validate_global_reference(object, "Number", ctx) {
        return None;
    }
    if args.len() != 1 {
        return None;
    }
    let extracted_expr = args.first()?.as_expression()?;
    if !extracted_expr.is_number_literal() {
        return None;
    }
    let extracted = extracted_expr.get_side_free_number_value(ctx)?;
    let result = match name {
        "isFinite" => Some(extracted.is_finite()),
        "isInteger" => Some(extracted.fract().abs() < f64::EPSILON),
        "isNaN" => Some(extracted.is_nan()),
        "isSafeInteger" => {
            let integer = extracted.fract().abs() < f64::EPSILON;
            let safe = extracted.abs() <= 2f64.powi(53) - 1.0;
            Some(safe && integer)
        }
        _ => None,
    };
    result.map(ConstantValue::Boolean)
}

fn try_fold_roots<'a>(
    args: &Vec<'a, Argument<'a>>,
    name: &str,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if !validate_global_reference(object, "Math", ctx)
        || !validate_arguments(args, 1)
    {
        return None;
    }
    let arg_val = args[0].to_expression().get_side_free_number_value(ctx)?;
    if arg_val == f64::INFINITY || arg_val.is_nan() || arg_val == 0.0 {
        return Some(ConstantValue::Number(arg_val));
    }
    if arg_val < 0.0 {
        return Some(ConstantValue::Number(f64::NAN));
    }
    let calculated_val = match name {
        "sqrt" => arg_val.sqrt(),
        "cbrt" => arg_val.cbrt(),
        _ => unreachable!(),
    };
    (calculated_val.fract() == 0.0).then_some(ConstantValue::Number(calculated_val))
}

fn try_fold_math_unary<'a>(
    args: &Vec<'a, Argument<'a>>,
    name: &str,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if !validate_global_reference(object, "Math", ctx)
        || !validate_arguments(args, 1)
    {
        return None;
    }
    let arg_val = args[0].to_expression().get_side_free_number_value(ctx)?;
    let result = match name {
        "abs" => arg_val.abs(),
        "ceil" => arg_val.ceil(),
        "floor" => arg_val.floor(),
        "round" => {
            // We should be aware that the behavior in JavaScript and Rust towards `round` is different.
            // In Rust, when facing `.5`, it may follow `half-away-from-zero` instead of round to upper bound.
            // So we need to handle it manually.
            let frac_part = arg_val.fract();
            let epsilon = 2f64.powi(-52);
            if (frac_part.abs() - 0.5).abs() < epsilon {
                // We should ceil it.
                arg_val.ceil()
            } else {
                arg_val.round()
            }
        }
        "fround" if arg_val.fract() == 0f64 || arg_val.is_nan() || arg_val.is_infinite() => {
            f64::from(arg_val as f32)
        }
        "fround" => return None,
        "trunc" => arg_val.trunc(),
        "sign" if arg_val.to_bits() == 0f64.to_bits() => 0f64,
        "sign" if arg_val.to_bits() == (-0f64).to_bits() => -0f64,
        "sign" => arg_val.signum(),
        _ => unreachable!(),
    };
    // These results are always shorter to return as a number, so we can just return them as NumericLiteral.
    Some(ConstantValue::Number(result))
}

fn try_fold_math_variadic<'a>(
    args: &Vec<'a, Argument<'a>>,
    name: &str,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if !validate_global_reference(object, "Math", ctx) {
        return None;
    }
    let mut numbers = std::vec::Vec::new();
    for arg in args {
        let expr = arg.as_expression()?;
        let value = expr.get_side_free_number_value(ctx)?;
        numbers.push(value);
    }
    let result = if numbers.iter().any(|n: &f64| n.is_nan()) {
        f64::NAN
    } else {
        match name {
            // TODO
            // see <https://github.com/rust-lang/rust/issues/83984>, we can't use `min` and `max` here due to inconsistency
            "min" => numbers.iter().copied().fold(f64::INFINITY, |a, b| {
                if a < b || ((a == 0f64) && (b == 0f64) && (a.to_bits() > b.to_bits())) {
                    a
                } else {
                    b
                }
            }),
            "max" => numbers.iter().copied().fold(f64::NEG_INFINITY, |a, b| {
                if a > b || ((a == 0f64) && (b == 0f64) && (a.to_bits() < b.to_bits())) {
                    a
                } else {
                    b
                }
            }),
            _ => return None,
        }
    };
    Some(ConstantValue::Number(result))
}

fn try_fold_array_of<'a>(
    _args: &Vec<'a, Argument<'a>>,
    name: &str,
    object: &Expression<'a>,
    ctx: &impl ConstantEvaluationCtx<'a>,
) -> Option<ConstantValue<'a>> {
    if !validate_global_reference(object, "Array", ctx) {
        return None;
    }
    if name != "of" {
        return None;
    }
    // Array.of() creates a new array, but we can't represent that in ConstantValue easily
    // This optimization is better handled by the minifier directly
    None
}

#[cfg(test)]
mod tests {    
    #[test]
    fn test_constant_evaluation_placeholder() {
        // This is a placeholder test to ensure the module compiles
        // Real testing would require complex AST setup
        assert!(true);
    }
}
