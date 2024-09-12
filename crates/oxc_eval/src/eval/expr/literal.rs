#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, BigInt};
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

// impl<'a> ConstEval<'a> for
