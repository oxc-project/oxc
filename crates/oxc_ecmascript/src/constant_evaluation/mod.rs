mod call_expr;
mod equality_comparison;
mod is_int32_or_uint32;
mod is_literal_value;
mod value;
mod value_type;

pub use is_int32_or_uint32::IsInt32OrUint32;
pub use is_literal_value::IsLiteralValue;
pub use value::ConstantValue;
pub use value_type::{DetermineValueType, ValueType};

use std::borrow::Cow;

use num_bigint::BigInt;
use num_traits::{ToPrimitive, Zero};
use oxc_ast::{AstBuilder, ast::*};
use oxc_syntax::operator::UpdateOperator;

use equality_comparison::{abstract_equality_comparison, strict_equality_comparison};

use crate::{
    ToBigInt, ToBoolean, ToInt32, ToJsString as ToJsStringTrait, ToNumber,
    is_less_than::is_less_than,
    side_effects::{MayHaveSideEffects, MayHaveSideEffectsContext},
    to_numeric::ToNumeric,
};

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
    ///   passing `Some(ValueType::Boolean)` will allow us to utilize that information.
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
            Expression::CallExpression(e) => e.evaluate_value_to(ctx, target_ty),
            Expression::SequenceExpression(e) => {
                // For sequence expression, the value is the value of the RHS.
                e.expressions.last().and_then(|e| e.evaluate_value_to(ctx, target_ty))
            }
            Expression::ParenthesizedExpression(e) => {
                e.expression.evaluate_value_to(ctx, target_ty)
            }
            Expression::ConditionalExpression(e) => {
                // Ternary operator: test ? consequent : alternate
                match e.test.evaluate_value_to_boolean(ctx) {
                    Some(true) => e.consequent.evaluate_value_to(ctx, target_ty),
                    Some(false) => e.alternate.evaluate_value_to(ctx, target_ty),
                    None => None,
                }
            }
            Expression::TemplateLiteral(e) => {
                // Only evaluate template literals with no expressions (plain string templates)
                if !e.expressions.is_empty() {
                    return None;
                }

                // Concatenate all the quasi strings
                let mut result = String::new();
                for quasi in &e.quasis {
                    if let Some(cooked) = &quasi.value.cooked {
                        result.push_str(cooked.as_str());
                    } else {
                        // If cooked is None, the template has invalid escape sequences
                        return None;
                    }
                }
                Some(ConstantValue::String(Cow::Owned(result)))
            }
            Expression::AssignmentExpression(e) => {
                // Assignment expressions return the value being assigned
                match e.operator {
                    AssignmentOperator::Assign => e.right.evaluate_value_to(ctx, target_ty),
                    // For compound assignments, we can only evaluate if both operands are known constants
                    _ => None,
                }
            }
            Expression::ArrayExpression(_) => {
                // For now, we don't evaluate array expressions to constant values
                // as they would create object values, but we could potentially
                // return information about their length in the future
                None
            }
            Expression::UpdateExpression(e) => {
                // For pre/post increment/decrement, we need to know the current value
                if let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = &e.argument {
                    if let Some(current_value) = ident.evaluate_value_to_number(ctx) {
                        let new_value = match e.operator {
                            UpdateOperator::Increment => current_value + 1.0,
                            UpdateOperator::Decrement => current_value - 1.0,
                        };

                        // Return the appropriate value based on prefix/postfix
                        let return_value = if e.prefix {
                            new_value // Pre-increment/decrement returns new value
                        } else {
                            current_value // Post-increment/decrement returns old value
                        };

                        return Some(ConstantValue::Number(return_value));
                    }
                }
                None
            }
            Expression::ThisExpression(_) => Some(ConstantValue::Undefined),
            Expression::ObjectExpression(_) => {
                // For now, we don't evaluate object expressions to constant values
                // as they would create object values
                None
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
            LogicalOperator::Coalesce => {
                // Nullish coalescing: a ?? b
                // Returns the right operand if the left operand is null or undefined
                match self.left.evaluate_value_to(ctx, target_ty) {
                    Some(ConstantValue::Null) | Some(ConstantValue::Undefined) => {
                        self.right.evaluate_value_to(ctx, target_ty)
                    }
                    Some(value) => Some(value),
                    None => None,
                }
            }
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
        call_expr::try_fold_known_global_methods(&self.callee, &self.arguments, ctx)
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use oxc_allocator::Allocator;
    use oxc_ast::{AstBuilder, ast::*};
    use oxc_index::Idx;
    use oxc_span::{Atom, SPAN};
    use oxc_syntax::operator::{AssignmentOperator, LogicalOperator, UpdateOperator};

    use super::{ConstantEvaluation, ConstantEvaluationCtx, ConstantValue};
    use crate::{
        is_global_reference::IsGlobalReference,
        side_effects::{MayHaveSideEffectsContext, PropertyReadSideEffects},
    };

    struct TestCtx<'a> {
        allocator: &'a Allocator,
    }

    impl<'a> TestCtx<'a> {
        fn new(allocator: &'a Allocator) -> Self {
            Self { allocator }
        }
    }

    impl<'a> IsGlobalReference<'a> for TestCtx<'a> {
        fn is_global_reference(&self, ident: &IdentifierReference<'a>) -> Option<bool> {
            match ident.name.as_str() {
                "undefined" | "NaN" | "Infinity" => Some(true),
                _ => Some(false),
            }
        }
    }

    impl<'a> MayHaveSideEffectsContext<'a> for TestCtx<'a> {
        fn annotations(&self) -> bool {
            false
        }

        fn unknown_global_side_effects(&self) -> bool {
            false
        }

        fn manual_pure_functions(&self, _callee: &Expression) -> bool {
            false
        }

        fn property_read_side_effects(&self) -> PropertyReadSideEffects {
            PropertyReadSideEffects::None
        }
    }

    impl<'a> ConstantEvaluationCtx<'a> for TestCtx<'a> {
        fn ast(&self) -> AstBuilder<'a> {
            AstBuilder::new(self.allocator)
        }
    }

    #[test]
    fn test_parenthesized_expression() {
        let allocator = Allocator::default();
        let ctx = TestCtx::new(&allocator);
        let ast = AstBuilder::new(&allocator);

        // (42)
        let inner =
            ast.expression_numeric_literal(SPAN, 42.0, None, oxc_ast::ast::NumberBase::Decimal);
        let expr = ast.expression_parenthesized(SPAN, inner);

        let result = expr.evaluate_value(&ctx);
        assert_eq!(result, Some(ConstantValue::Number(42.0)));
    }

    #[test]
    fn test_conditional_expression() {
        let allocator = Allocator::default();
        let ctx = TestCtx::new(&allocator);
        let ast = AstBuilder::new(&allocator);

        // true ? 1 : 2
        let test = ast.expression_boolean_literal(SPAN, true);
        let consequent =
            ast.expression_numeric_literal(SPAN, 1.0, None, oxc_ast::ast::NumberBase::Decimal);
        let alternate =
            ast.expression_numeric_literal(SPAN, 2.0, None, oxc_ast::ast::NumberBase::Decimal);
        let expr = ast.expression_conditional(SPAN, test, consequent, alternate);

        let result = expr.evaluate_value(&ctx);
        assert_eq!(result, Some(ConstantValue::Number(1.0)));

        // false ? 1 : 2
        let test = ast.expression_boolean_literal(SPAN, false);
        let consequent =
            ast.expression_numeric_literal(SPAN, 1.0, None, oxc_ast::ast::NumberBase::Decimal);
        let alternate =
            ast.expression_numeric_literal(SPAN, 2.0, None, oxc_ast::ast::NumberBase::Decimal);
        let expr = ast.expression_conditional(SPAN, test, consequent, alternate);

        let result = expr.evaluate_value(&ctx);
        assert_eq!(result, Some(ConstantValue::Number(2.0)));
    }

    #[test]
    fn test_template_literal() {
        let allocator = Allocator::default();
        let ctx = TestCtx::new(&allocator);
        let ast = AstBuilder::new(&allocator);

        // `hello world`
        let quasi = ast.template_element(
            SPAN,
            TemplateElementValue {
                raw: Atom::from("hello world"),
                cooked: Some(Atom::from("hello world")),
            },
            false,
        );
        let expr = ast.expression_template_literal(SPAN, ast.vec1(quasi), ast.vec());

        let result = expr.evaluate_value(&ctx);
        assert_eq!(result, Some(ConstantValue::String(Cow::Owned("hello world".to_string()))));

        // Template literal with expressions should return None
        let quasi1 = ast.template_element(
            SPAN,
            TemplateElementValue {
                raw: Atom::from("hello "),
                cooked: Some(Atom::from("hello ")),
            },
            false,
        );
        let quasi2 = ast.template_element(
            SPAN,
            TemplateElementValue {
                raw: Atom::from(""),
                cooked: Some(Atom::from("")),
            },
            true,
        );
        let number_expr =
            ast.expression_numeric_literal(SPAN, 42.0, None, oxc_ast::ast::NumberBase::Decimal);
        let expr_with_placeholder = ast.expression_template_literal(
            SPAN,
            ast.vec_from_iter([quasi1, quasi2]),
            ast.vec1(number_expr),
        );

        let result = expr_with_placeholder.evaluate_value(&ctx);
        assert_eq!(result, None);
    }

    #[test]
    fn test_assignment_expression() {
        let allocator = Allocator::default();
        let ctx = TestCtx::new(&allocator);
        let ast = AstBuilder::new(&allocator);

        // x = 42
        let left = ast.simple_assignment_target_assignment_target_identifier_with_reference_id(
            SPAN,
            Atom::from("x"),
            oxc_syntax::reference::ReferenceId::from_usize(0),
        );
        let right =
            ast.expression_numeric_literal(SPAN, 42.0, None, oxc_ast::ast::NumberBase::Decimal);
        let expr = ast.expression_assignment(SPAN, AssignmentOperator::Assign, left.into(), right);

        let result = expr.evaluate_value(&ctx);
        assert_eq!(result, Some(ConstantValue::Number(42.0)));
    }

    #[test]
    fn test_logical_coalesce() {
        let allocator = Allocator::default();
        let ctx = TestCtx::new(&allocator);
        let ast = AstBuilder::new(&allocator);

        // null ?? 42
        let left = ast.expression_null_literal(SPAN);
        let right =
            ast.expression_numeric_literal(SPAN, 42.0, None, oxc_ast::ast::NumberBase::Decimal);
        let expr = ast.expression_logical(SPAN, left, LogicalOperator::Coalesce, right);

        let result = expr.evaluate_value(&ctx);
        assert_eq!(result, Some(ConstantValue::Number(42.0)));

        // undefined ?? 42
        let left = ast.expression_identifier_with_reference_id(
            SPAN,
            Atom::from("undefined"),
            oxc_syntax::reference::ReferenceId::from_usize(0),
        );
        let right =
            ast.expression_numeric_literal(SPAN, 42.0, None, oxc_ast::ast::NumberBase::Decimal);
        let expr = ast.expression_logical(SPAN, left, LogicalOperator::Coalesce, right);

        let result = expr.evaluate_value(&ctx);
        assert_eq!(result, Some(ConstantValue::Number(42.0)));

        // 1 ?? 42
        let left =
            ast.expression_numeric_literal(SPAN, 1.0, None, oxc_ast::ast::NumberBase::Decimal);
        let right =
            ast.expression_numeric_literal(SPAN, 42.0, None, oxc_ast::ast::NumberBase::Decimal);
        let expr = ast.expression_logical(SPAN, left, LogicalOperator::Coalesce, right);

        let result = expr.evaluate_value(&ctx);
        assert_eq!(result, Some(ConstantValue::Number(1.0)));

        // 0 ?? 42 (0 is not null/undefined, so should return 0)
        let left =
            ast.expression_numeric_literal(SPAN, 0.0, None, oxc_ast::ast::NumberBase::Decimal);
        let right =
            ast.expression_numeric_literal(SPAN, 42.0, None, oxc_ast::ast::NumberBase::Decimal);
        let expr = ast.expression_logical(SPAN, left, LogicalOperator::Coalesce, right);

        let result = expr.evaluate_value(&ctx);
        assert_eq!(result, Some(ConstantValue::Number(0.0)));
    }

    #[test]
    fn test_update_expression() {
        let allocator = Allocator::default();
        let ctx = TestCtx::new(&allocator);
        let ast = AstBuilder::new(&allocator);

        // Test requires identifier to have a constant value, which our test context doesn't provide
        // For now, we'll test that it returns None for unknown identifiers
        let ident = ast.simple_assignment_target_assignment_target_identifier_with_reference_id(
            SPAN,
            Atom::from("x"),
            oxc_syntax::reference::ReferenceId::from_usize(0),
        );
        let expr = ast.expression_update(SPAN, UpdateOperator::Increment, true, ident);

        let result = expr.evaluate_value(&ctx);
        assert_eq!(result, None);
    }

    #[test]
    fn test_this_expression() {
        let allocator = Allocator::default();
        let ctx = TestCtx::new(&allocator);
        let ast = AstBuilder::new(&allocator);

        let expr = ast.expression_this(SPAN);

        let result = expr.evaluate_value(&ctx);
        assert_eq!(result, Some(ConstantValue::Undefined));
    }

    #[test]
    fn test_array_expression() {
        let allocator = Allocator::default();
        let ctx = TestCtx::new(&allocator);
        let ast = AstBuilder::new(&allocator);

        // []
        let expr = ast.expression_array(SPAN, ast.vec());

        let result = expr.evaluate_value(&ctx);
        assert_eq!(result, None); // Arrays are not evaluated to constant values
    }

    #[test]
    fn test_object_expression() {
        let allocator = Allocator::default();
        let ctx = TestCtx::new(&allocator);
        let ast = AstBuilder::new(&allocator);

        // {}
        let expr = ast.expression_object(SPAN, ast.vec());

        let result = expr.evaluate_value(&ctx);
        assert_eq!(result, None); // Objects are not evaluated to constant values
    }
}
