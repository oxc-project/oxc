use std::borrow::Cow;
use std::ops::Deref;

use num_bigint::BigInt;
use oxc_ast::ast::*;
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, ConstantValue},
    side_effects::MayHaveSideEffects,
};
use oxc_ecmascript::{ToBigInt, ToJsString};
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

impl<'a, 'b> ConstantEvaluation<'a> for Ctx<'a, 'b> {
    fn is_global_reference(&self, ident: &oxc_ast::ast::IdentifierReference<'a>) -> bool {
        ident.is_global_reference(self.0.symbols())
    }
}

pub fn is_exact_int64(num: f64) -> bool {
    num.fract() == 0.0
}

impl<'a, 'b> Ctx<'a, 'b> {
    fn symbols(&self) -> &SymbolTable {
        self.0.symbols()
    }

    pub fn value_to_expr(self, span: Span, value: ConstantValue<'a>) -> Expression<'a> {
        match value {
            ConstantValue::Number(n) => {
                let number_base =
                    if is_exact_int64(n) { NumberBase::Decimal } else { NumberBase::Float };
                self.ast.expression_numeric_literal(span, n, "", number_base)
            }
            ConstantValue::BigInt(n) => {
                self.ast.expression_big_int_literal(span, n.to_string() + "n", BigintBase::Decimal)
            }
            ConstantValue::String(s) => self.ast.expression_string_literal(span, s),
            ConstantValue::Boolean(b) => self.ast.expression_boolean_literal(span, b),
            ConstantValue::Undefined => self.ast.void_0(span),
        }
    }

    /// Gets the boolean value of a node that represents an expression, or `None` if no
    /// such value can be determined by static analysis.
    /// This method does not consider whether the node may have side-effects.
    /// <https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/NodeUtil.java#L109>
    pub fn get_boolean_value(self, expr: &Expression<'a>) -> Option<bool> {
        self.eval_to_boolean(expr)
    }

    /// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/AbstractPeepholeOptimization.java#L104-L114)
    /// Returns the number value of the node if it has one and it cannot have side effects.
    pub fn get_side_free_number_value(self, expr: &Expression<'a>) -> Option<f64> {
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

    pub fn is_expression_undefined(self, expr: &Expression) -> bool {
        match expr {
            Expression::Identifier(ident) if self.is_identifier_undefined(ident) => true,
            Expression::UnaryExpression(e) if e.operator.is_void() && e.argument.is_number() => {
                true
            }
            _ => false,
        }
    }

    pub fn is_identifier_undefined(self, ident: &IdentifierReference) -> bool {
        if ident.name == "undefined" && ident.is_global_reference(self.symbols()) {
            return true;
        }
        false
    }

    /// port from [closure compiler](https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/AbstractPeepholeOptimization.java#L121)
    pub fn get_side_free_bigint_value(self, expr: &Expression<'a>) -> Option<BigInt> {
        let value = self.get_bigint_value(expr);
        // Calculating the bigint value, if any, is likely to be faster than calculating side effects,
        // and there are only a very few cases where we can compute a bigint value, but there could
        // also be side effects. e.g. `void doSomething()` has value NaN, regardless of the behavior
        // of `doSomething()`
        if value.is_some() && expr.may_have_side_effects() {
            None
        } else {
            value
        }
    }

    /// Port from [closure-compiler](https://github.com/google/closure-compiler/blob/e13f5cd0a5d3d35f2db1e6c03fdf67ef02946009/src/com/google/javascript/jscomp/AbstractPeepholeOptimization.java#L139-L149)
    /// Gets the value of a node as a String, or `None` if it cannot be converted.
    /// This method effectively emulates the `String()` JavaScript cast function when
    /// possible and the node has no side effects. Otherwise, it returns `None`.
    pub fn get_side_free_string_value(self, expr: &'a Expression) -> Option<Cow<'a, str>> {
        let value = self.get_string_value(expr);
        // Calculating the string value, if any, is likely to be faster than calculating side effects,
        // and there are only a very few cases where we can compute a string value, but there could
        // also be side effects. e.g. `void doSomething()` has value 'undefined', regardless of the
        // behavior of `doSomething()`
        if value.is_some() && !expr.may_have_side_effects() {
            return value;
        }
        None
    }

    #[expect(clippy::unused_self)]
    pub fn get_bigint_value(self, expr: &Expression<'a>) -> Option<BigInt> {
        expr.to_big_int()
    }

    /// Port from [closure-compiler](https://github.com/google/closure-compiler/blob/e13f5cd0a5d3d35f2db1e6c03fdf67ef02946009/src/com/google/javascript/jscomp/NodeUtil.java#L234)
    /// Gets the value of a node as a String, or `None` if it cannot be converted. When it returns a
    /// String, this method effectively emulates the `String()` JavaScript cast function.
    /// This method does not consider whether `expr` may have side effects.
    #[expect(clippy::unused_self)]
    pub fn get_string_value(self, expr: &Expression<'a>) -> Option<Cow<'a, str>> {
        expr.to_js_string()
    }
}
