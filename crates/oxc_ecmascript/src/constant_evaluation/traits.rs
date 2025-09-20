use std::borrow::Cow;

use num_bigint::BigInt;
use oxc_ast::AstBuilder;

use crate::{
    ToBoolean, ToJsString, ToNumber,
    side_effects::MayHaveSideEffectsContext,
};

use super::{ConstantValue, ValueType};

pub trait ConstantEvaluationCtx<'a>: MayHaveSideEffectsContext<'a> {
    fn ast(&self) -> AstBuilder<'a>;
}

pub trait ConstantEvaluation<'a>: crate::side_effects::MayHaveSideEffects<'a> {
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