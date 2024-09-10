mod check_for_state_change;
mod is_literal_value;
mod may_have_side_effects;
mod number_value;

use std::borrow::Cow;

use num_bigint::BigInt;
use num_traits::{One, Zero};
use oxc_ast::ast::*;
use oxc_semantic::{ScopeTree, SymbolTable};
use oxc_syntax::operator::{AssignmentOperator, LogicalOperator, UnaryOperator};

pub use self::{
    is_literal_value::IsLiteralValue, may_have_side_effects::MayHaveSideEffects,
    number_value::NumberValue,
};

pub fn is_exact_int64(num: f64) -> bool {
    num.fract() == 0.0
}

pub trait NodeUtil {
    fn symbols(&self) -> &SymbolTable;

    #[allow(unused)]
    fn scopes(&self) -> &ScopeTree;

    /// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/AbstractPeepholeOptimization.java#L104-L114)
    /// Returns the number value of the node if it has one and it cannot have side effects.
    fn get_side_free_number_value(&self, expr: &Expression) -> Option<NumberValue> {
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
    fn get_side_free_bigint_value(&self, expr: &Expression) -> Option<BigInt> {
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
    fn get_side_free_string_value<'a>(&self, expr: &'a Expression) -> Option<Cow<'a, str>> {
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

    /// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/NodeUtil.java#L109)
    /// Gets the boolean value of a node that represents an expression, or `None` if no
    /// such value can be determined by static analysis.
    /// This method does not consider whether the node may have side-effects.
    fn get_boolean_value(&self, expr: &Expression) -> Option<bool> {
        match expr {
            Expression::RegExpLiteral(_)
            | Expression::ArrayExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::ClassExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::NewExpression(_)
            | Expression::ObjectExpression(_) => Some(true),
            Expression::NullLiteral(_) => Some(false),
            Expression::BooleanLiteral(boolean_literal) => Some(boolean_literal.value),
            Expression::NumericLiteral(number_literal) => Some(number_literal.value != 0.0),
            Expression::BigIntLiteral(big_int_literal) => Some(!big_int_literal.is_zero()),
            Expression::StringLiteral(string_literal) => Some(!string_literal.value.is_empty()),
            Expression::TemplateLiteral(template_literal) => {
                // only for ``
                template_literal
                    .quasis
                    .first()
                    .filter(|quasi| quasi.tail)
                    .and_then(|quasi| quasi.value.cooked.as_ref())
                    .map(|cooked| !cooked.is_empty())
            }
            Expression::Identifier(ident) => match ident.name.as_str() {
                "NaN" => Some(false),
                "Infinity" => Some(true),
                "undefined" if self.symbols().is_global_identifier_reference(ident) => Some(false),
                _ => None,
            },
            Expression::AssignmentExpression(assign_expr) => {
                match assign_expr.operator {
                    AssignmentOperator::LogicalAnd | AssignmentOperator::LogicalOr => None,
                    // For ASSIGN, the value is the value of the RHS.
                    _ => self.get_boolean_value(&assign_expr.right),
                }
            }
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
                if unary_expr.operator == UnaryOperator::Void {
                    Some(false)
                } else if matches!(
                    unary_expr.operator,
                    UnaryOperator::BitwiseNot
                        | UnaryOperator::UnaryPlus
                        | UnaryOperator::UnaryNegation
                ) {
                    // ~0 -> true
                    // +1 -> true
                    // +0 -> false
                    // -0 -> false
                    self.get_number_value(expr).map(|value| value != NumberValue::Number(0_f64))
                } else if unary_expr.operator == UnaryOperator::LogicalNot {
                    // !true -> false
                    self.get_boolean_value(&unary_expr.argument).map(|boolean| !boolean)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/NodeUtil.java#L348)
    /// Gets the value of a node as a Number, or None if it cannot be converted.
    /// This method does not consider whether `expr` may have side effects.
    fn get_number_value(&self, expr: &Expression) -> Option<NumberValue> {
        match expr {
            Expression::NumericLiteral(number_literal) => {
                Some(NumberValue::Number(number_literal.value))
            }
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::UnaryPlus => self.get_number_value(&unary_expr.argument),
                UnaryOperator::UnaryNegation => {
                    self.get_number_value(&unary_expr.argument).map(|v| v.not())
                }
                UnaryOperator::BitwiseNot => {
                    self.get_number_value(&unary_expr.argument).map(|value| {
                        match value {
                            NumberValue::Number(num) => NumberValue::Number(f64::from(
                                !NumericLiteral::ecmascript_to_int32(num),
                            )),
                            // ~Infinity -> -1
                            // ~-Infinity -> -1
                            // ~NaN -> -1
                            _ => NumberValue::Number(-1_f64),
                        }
                    })
                }
                UnaryOperator::LogicalNot => self
                    .get_boolean_value(expr)
                    .map(|boolean| if boolean { 1_f64 } else { 0_f64 })
                    .map(NumberValue::Number),
                UnaryOperator::Void => Some(NumberValue::NaN),
                _ => None,
            },
            Expression::BooleanLiteral(bool_literal) => {
                if bool_literal.value {
                    Some(NumberValue::Number(1.0))
                } else {
                    Some(NumberValue::Number(0.0))
                }
            }
            Expression::NullLiteral(_) => Some(NumberValue::Number(0.0)),
            Expression::Identifier(ident) => match ident.name.as_str() {
                "Infinity" => Some(NumberValue::PositiveInfinity),
                "NaN" | "undefined" => Some(NumberValue::NaN),
                _ => None,
            },
            // TODO: will be implemented in next PR, just for test pass now.
            Expression::StringLiteral(string_literal) => string_literal
                .value
                .parse::<f64>()
                .map_or(Some(NumberValue::NaN), |num| Some(NumberValue::Number(num))),
            _ => None,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn get_bigint_value(&self, expr: &Expression) -> Option<BigInt> {
        match expr {
            Expression::NumericLiteral(number_literal) => {
                let value = number_literal.value;
                if value.abs() < 2_f64.powi(53) && is_exact_int64(value) {
                    Some(BigInt::from(value as i64))
                } else {
                    None
                }
            }
            Expression::BigIntLiteral(_bigint_literal) => {
                // TODO: evaluate the bigint value
                None
            }
            Expression::BooleanLiteral(bool_literal) => {
                if bool_literal.value {
                    Some(BigInt::one())
                } else {
                    Some(BigInt::zero())
                }
            }
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::LogicalNot => self.get_boolean_value(expr).map(|boolean| {
                    if boolean {
                        BigInt::one()
                    } else {
                        BigInt::zero()
                    }
                }),
                UnaryOperator::UnaryNegation => {
                    self.get_bigint_value(&unary_expr.argument).map(std::ops::Neg::neg)
                }
                UnaryOperator::BitwiseNot => {
                    self.get_bigint_value(&unary_expr.argument).map(std::ops::Not::not)
                }
                UnaryOperator::UnaryPlus => self.get_bigint_value(&unary_expr.argument),
                _ => None,
            },
            Expression::StringLiteral(string_literal) => {
                self.get_string_bigint_value(&string_literal.value)
            }
            Expression::TemplateLiteral(_) => {
                self.get_string_value(expr).and_then(|value| self.get_string_bigint_value(&value))
            }
            _ => None,
        }
    }

    /// Port from [closure-compiler](https://github.com/google/closure-compiler/blob/e13f5cd0a5d3d35f2db1e6c03fdf67ef02946009/src/com/google/javascript/jscomp/NodeUtil.java#L234)
    /// Gets the value of a node as a String, or `None` if it cannot be converted. When it returns a
    /// String, this method effectively emulates the `String()` JavaScript cast function.
    /// This method does not consider whether `expr` may have side effects.
    fn get_string_value<'a>(&self, expr: &'a Expression) -> Option<Cow<'a, str>> {
        match expr {
            Expression::StringLiteral(string_literal) => {
                Some(Cow::Borrowed(string_literal.value.as_str()))
            }
            Expression::TemplateLiteral(template_literal) => {
                // TODO: I don't know how to iterate children of TemplateLiteral in order,so only checkout string like `hi`.
                // Closure-compiler do more: [case TEMPLATELIT](https://github.com/google/closure-compiler/blob/e13f5cd0a5d3d35f2db1e6c03fdf67ef02946009/src/com/google/javascript/jscomp/NodeUtil.java#L241-L256).
                template_literal
                    .quasis
                    .first()
                    .filter(|quasi| quasi.tail)
                    .and_then(|quasi| quasi.value.cooked.as_ref())
                    .map(|cooked| Cow::Borrowed(cooked.as_str()))
            }
            Expression::Identifier(ident) => {
                let name = ident.name.as_str();
                if matches!(name, "undefined" | "Infinity" | "NaN") {
                    Some(Cow::Borrowed(name))
                } else {
                    None
                }
            }
            Expression::NumericLiteral(number_literal) => {
                Some(Cow::Owned(number_literal.value.to_string()))
            }
            Expression::BigIntLiteral(big_int_literal) => {
                Some(Cow::Owned(big_int_literal.raw.to_string()))
            }
            Expression::NullLiteral(_) => Some(Cow::Borrowed("null")),
            Expression::BooleanLiteral(bool_literal) => {
                if bool_literal.value {
                    Some(Cow::Borrowed("true"))
                } else {
                    Some(Cow::Borrowed("false"))
                }
            }
            Expression::UnaryExpression(unary_expr) => {
                match unary_expr.operator {
                    UnaryOperator::Void => Some(Cow::Borrowed("undefined")),
                    UnaryOperator::LogicalNot => {
                        self.get_boolean_value(&unary_expr.argument).map(|boolean| {
                            // need reversed.
                            if boolean {
                                Cow::Borrowed("false")
                            } else {
                                Cow::Borrowed("true")
                            }
                        })
                    }
                    _ => None,
                }
            }
            Expression::ArrayExpression(_) => {
                // TODO: https://github.com/google/closure-compiler/blob/e13f5cd0a5d3d35f2db1e6c03fdf67ef02946009/src/com/google/javascript/jscomp/NodeUtil.java#L302-L303
                None
            }
            Expression::ObjectExpression(_) => Some(Cow::Borrowed("[object Object]")),
            _ => None,
        }
    }

    /// port from [closure compiler](https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/NodeUtil.java#L540)
    fn get_string_bigint_value(&self, raw_string: &str) -> Option<BigInt> {
        if raw_string.contains('\u{000b}') {
            // vertical tab is not always whitespace
            return None;
        }

        let s = raw_string.trim();

        if s.is_empty() {
            return Some(BigInt::zero());
        }

        if s.len() > 2 && s.starts_with('0') {
            let radix: u32 = match s.chars().nth(1) {
                Some('x' | 'X') => 16,
                Some('o' | 'O') => 8,
                Some('b' | 'B') => 2,
                _ => 0,
            };

            if radix == 0 {
                return None;
            }

            return BigInt::parse_bytes(s[2..].as_bytes(), radix);
        }

        return BigInt::parse_bytes(s.as_bytes(), 10);
    }
}
