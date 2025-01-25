use std::ops::Deref;

use oxc_ast::ast::*;
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, ConstantValue},
    side_effects::MayHaveSideEffects,
};
use oxc_semantic::{IsGlobalReference, SymbolTable};
use oxc_traverse::TraverseCtx;

#[derive(Clone, Copy)]
pub struct Ctx<'a, 'b>(pub &'b TraverseCtx<'a>);

impl<'a, 'b> Deref for Ctx<'a, 'b> {
    type Target = &'b TraverseCtx<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> ConstantEvaluation<'a> for Ctx<'a, '_> {}

impl MayHaveSideEffects for Ctx<'_, '_> {
    fn is_global_reference(&self, ident: &IdentifierReference<'_>) -> bool {
        ident.is_global_reference(self.0.symbols())
    }
}

pub fn is_exact_int64(num: f64) -> bool {
    num.fract() == 0.0
}

impl<'a> Ctx<'a, '_> {
    fn symbols(&self) -> &SymbolTable {
        self.0.symbols()
    }

    pub fn is_global_reference(self, ident: &IdentifierReference<'a>) -> bool {
        ident.is_global_reference(self.0.symbols())
    }

    pub fn eval_binary(self, e: &BinaryExpression<'a>) -> Option<Expression<'a>> {
        self.eval_binary_expression(e).map(|v| self.value_to_expr(e.span, v))
    }

    pub fn value_to_expr(self, span: Span, value: ConstantValue<'a>) -> Expression<'a> {
        match value {
            ConstantValue::Number(n) => {
                let number_base =
                    if is_exact_int64(n) { NumberBase::Decimal } else { NumberBase::Float };
                self.ast.expression_numeric_literal(span, n, None, number_base)
            }
            ConstantValue::BigInt(n) => {
                self.ast.expression_big_int_literal(span, n.to_string() + "n", BigintBase::Decimal)
            }
            ConstantValue::String(s) => self.ast.expression_string_literal(span, s, None),
            ConstantValue::Boolean(b) => self.ast.expression_boolean_literal(span, b),
            ConstantValue::Undefined => self.ast.void_0(span),
            ConstantValue::Null => self.ast.expression_null_literal(span),
        }
    }

    pub fn is_expression_undefined(self, expr: &Expression) -> bool {
        match expr {
            Expression::Identifier(ident) if self.is_identifier_undefined(ident) => true,
            Expression::UnaryExpression(e) if e.operator.is_void() && e.argument.is_number() => {
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn is_identifier_undefined(self, ident: &IdentifierReference) -> bool {
        if ident.name == "undefined" && ident.is_global_reference(self.symbols()) {
            return true;
        }
        false
    }

    /// If two expressions are equal.
    /// Special case `undefined` == `void 0`
    pub fn expr_eq(self, a: &Expression<'a>, b: &Expression<'a>) -> bool {
        use oxc_span::cmp::ContentEq;
        a.content_eq(b) || (self.is_expression_undefined(a) && self.is_expression_undefined(b))
    }

    // https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_ast_helpers.go#L2641
    pub fn string_to_equivalent_number_value(s: &str) -> Option<f64> {
        if s.is_empty() {
            return None;
        }
        let mut is_negative = false;
        let mut int_value = 0i32;
        let mut start = 0;
        let bytes = s.as_bytes();
        if bytes[0] == b'-' && s.len() > 1 {
            is_negative = true;
            int_value = -int_value;
            start += 1;
        }
        if bytes[start] == b'0' && s.len() > 1 {
            return None;
        }
        for b in &bytes[start..] {
            if !b.is_ascii_digit() {
                return None;
            }
            int_value = int_value.checked_mul(10).and_then(|v| {
                let n = i32::from(b & 15);
                if is_negative {
                    v.checked_sub(n)
                } else {
                    v.checked_add(n)
                }
            })?;
        }
        Some(f64::from(int_value))
    }
}
