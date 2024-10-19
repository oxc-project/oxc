use num_bigint::BigInt;
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use std::str::FromStr;

use crate::{Eval, EvalContext, EvalResult, Value};

impl<'a> Eval<'a> for BooleanLiteral {
    #[inline]
    fn eval(&self, _ctx: &mut EvalContext<'a>) -> crate::EvalResult<'a> {
        Ok(Value::Boolean(self.value))
    }
}

impl<'a> Eval<'a> for NullLiteral {
    #[inline]
    fn eval(&self, _ctx: &mut EvalContext<'a>) -> crate::EvalResult<'a> {
        Ok(Value::Null)
    }
}

impl<'a> Eval<'a> for StringLiteral<'a> {
    #[inline]
    fn eval(&self, _ctx: &mut EvalContext<'a>) -> crate::EvalResult<'a> {
        Ok(Value::from(self.value.clone()))
    }
}

impl<'a> Eval<'a> for NumericLiteral<'a> {
    #[inline]
    fn eval(&self, _ctx: &mut EvalContext<'a>) -> crate::EvalResult<'a> {
        Ok(self.value.into())
    }
}

impl<'a> Eval<'a> for BigIntLiteral<'a> {
    fn eval(&self, _ctx: &mut EvalContext<'a>) -> EvalResult<'a> {
        BigInt::from_str(self.raw.as_str())
            .map(Value::BigInt)
            .map_err(|e| Some(OxcDiagnostic::error(format!("Invalid BigInt: {e}"))))
    }
}

impl<'a> Eval<'a> for TemplateLiteral<'a> {
    /// [13.2.8.6 Runtime Semantics: Evaluation](https://262.ecma-international.org/15.0/index.html#sec-template-literals-runtime-semantics-evaluation)
    fn eval(&self, ctx: &mut EvalContext<'a>) -> EvalResult<'a> {
        // TemplateLiteral : NoSubstitutionTemplate
        // 1. Return the TV of NoSubstitutionTemplate as defined in 12.9.6.
        if self.is_no_substitution_template() {
            // NOTE: unwrap will never panic because no_substitution_template
            // guarantees that there is only a single quasis element.
            return Ok(Value::from(self.quasi().unwrap()));
        }
        // SubstitutionTemplate : TemplateHead Expression TemplateSpans
        // 1. Let head be the TV of TemplateHead as defined in 12.9.6.
        // 2. Let subRef be ? Evaluation of Expression.
        // 3. Let sub be ? GetValue(subRef).
        // 4. Let middle be ? ToString(sub).
        // 5. Let tail be ? Evaluation of TemplateSpans.
        // 6. Return the string-concatenation of head, middle, and tail.

        todo!("TemplateLiteral::eval for SubstitutionTemplate")
    }
}
