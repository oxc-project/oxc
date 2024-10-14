mod check_for_state_change;
mod is_literal_value;
mod may_have_side_effects;

use std::borrow::Cow;

use num_bigint::BigInt;
use oxc_ast::ast::*;
use oxc_ecmascript::{StringToBigInt, ToBigInt, ToBoolean, ToJsString, ToNumber};
use oxc_semantic::{IsGlobalReference, ScopeTree, SymbolTable};

pub use self::{is_literal_value::IsLiteralValue, may_have_side_effects::MayHaveSideEffects};

use crate::tri::Tri;

pub fn is_exact_int64(num: f64) -> bool {
    num.fract() == 0.0
}

#[derive(Debug, Eq, PartialEq)]
pub enum ValueType {
    Undetermined,
    Null,
    Void,
    Number,
    Bigint,
    String,
    Boolean,
    Object,
}

pub trait NodeUtil<'a> {
    fn symbols(&self) -> &SymbolTable;

    #[allow(unused)]
    fn scopes(&self) -> &ScopeTree;

    fn is_expression_undefined(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Identifier(ident) if self.is_identifier_undefined(ident) => true,
            Expression::UnaryExpression(e) if e.operator.is_void() && e.argument.is_number() => {
                true
            }
            _ => false,
        }
    }

    fn is_identifier_undefined(&self, ident: &IdentifierReference) -> bool {
        if ident.name == "undefined" && ident.is_global_reference(self.symbols()) {
            return true;
        }
        false
    }

    /// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/AbstractPeepholeOptimization.java#L104-L114)
    /// Returns the number value of the node if it has one and it cannot have side effects.
    fn get_side_free_number_value(&self, expr: &Expression<'a>) -> Option<f64> {
        let value = self.get_number_value(expr);
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

    /// port from [closure compiler](https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/AbstractPeepholeOptimization.java#L121)
    fn get_side_free_bigint_value(&self, expr: &Expression<'a>) -> Option<BigInt> {
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
    fn get_side_free_string_value(&self, expr: &'a Expression) -> Option<Cow<'a, str>> {
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

    // port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/NodeUtil.java#L109)
    // Gets the boolean value of a node that represents an expression, or `None` if no
    // such value can be determined by static analysis.
    // This method does not consider whether the node may have side-effects.
    fn get_boolean_value(&self, expr: &Expression<'a>) -> Tri {
        Tri::from(expr.to_boolean())
    }

    /// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/NodeUtil.java#L348)
    /// Gets the value of a node as a Number, or None if it cannot be converted.
    /// This method does not consider whether `expr` may have side effects.
    fn get_number_value(&self, expr: &Expression<'a>) -> Option<f64> {
        expr.to_number()
    }

    fn get_bigint_value(&self, expr: &Expression<'a>) -> Option<BigInt> {
        expr.to_big_int()
    }

    /// Retrieve the literal value of a string, such as `abc` or "abc".
    fn get_string_literal(&self, expr: &Expression<'a>) -> Option<Cow<'a, str>> {
        match expr {
            Expression::StringLiteral(lit) => Some(Cow::Borrowed(lit.value.as_str())),
            Expression::TemplateLiteral(_) => Some(self.get_string_value(expr)?),
            _ => None,
        }
    }

    /// Port from [closure-compiler](https://github.com/google/closure-compiler/blob/e13f5cd0a5d3d35f2db1e6c03fdf67ef02946009/src/com/google/javascript/jscomp/NodeUtil.java#L234)
    /// Gets the value of a node as a String, or `None` if it cannot be converted. When it returns a
    /// String, this method effectively emulates the `String()` JavaScript cast function.
    /// This method does not consider whether `expr` may have side effects.
    fn get_string_value(&self, expr: &Expression<'a>) -> Option<Cow<'a, str>> {
        expr.to_js_string()
    }

    /// port from [closure compiler](https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/NodeUtil.java#L540)
    fn get_string_bigint_value(&self, raw_string: &str) -> Option<BigInt> {
        raw_string.string_to_big_int()
    }

    /// Evaluate  and attempt to determine which primitive value type it could resolve to.
    /// Without proper type information some assumptions had to be made for operations that could
    /// result in a BigInt or a Number. If there is not enough information available to determine one
    /// or the other then we assume Number in order to maintain historical behavior of the compiler and
    /// avoid breaking projects that relied on this behavior.
    fn get_known_value_type(&self, e: &Expression<'_>) -> ValueType {
        match e {
            Expression::NumericLiteral(_) => ValueType::Number,
            Expression::NullLiteral(_) => ValueType::Null,
            Expression::ArrayExpression(_) | Expression::ObjectExpression(_) => ValueType::Object,
            Expression::BooleanLiteral(_) => ValueType::Boolean,
            Expression::Identifier(ident) if self.is_identifier_undefined(ident) => ValueType::Void,
            Expression::SequenceExpression(e) => e
                .expressions
                .last()
                .map_or(ValueType::Undetermined, |e| self.get_known_value_type(e)),
            Expression::BigIntLiteral(_) => ValueType::Bigint,
            Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => ValueType::String,
            // TODO: complete this
            _ => ValueType::Undetermined,
        }
    }
}
