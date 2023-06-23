//! Constant Folding
//!
//! <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeFoldConstants.java>

use std::ops::Not;

use std::cmp::Ordering;

#[allow(clippy::wildcard_imports)]
use oxc_hir::hir::*;
use oxc_hir::hir_util::{
    get_boolean_value, get_number_value, get_side_free_number_value, get_side_free_string_value,
    IsLiteralValue, MayHaveSideEffects, NumberValue,
};
use oxc_span::{Atom, GetSpan, Span};
use oxc_syntax::{
    operator::{BinaryOperator, UnaryOperator},
    NumberBase,
};

use super::Compressor;

/// Tri state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tri {
    True,
    False,
    Unknown,
}

impl Tri {
    pub fn not(self) -> Self {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Unknown => Self::Unknown,
        }
    }

    pub fn for_boolean(boolean: bool) -> Self {
        if boolean { Self::True } else { Self::False }
    }
}

/// JavaScript Language Type
///
/// <https://tc39.es/ecma262/#sec-ecmascript-language-types>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ty {
    BigInt,
    Boolean,
    Null,
    Number,
    Object,
    Str,
    Void,
    Undetermined,
}

impl<'a> From<&Expression<'a>> for Ty {
    fn from(expr: &Expression<'a>) -> Self {
        // TODO: complete this
        match expr {
            Expression::BigintLiteral(_) => Self::BigInt,
            Expression::BooleanLiteral(_) => Self::Boolean,
            Expression::NullLiteral(_) => Self::Null,
            Expression::NumberLiteral(_) => Self::Number,
            Expression::StringLiteral(_) => Self::Str,
            Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::RegExpLiteral(_)
            | Expression::FunctionExpression(_) => Self::Object,
            Expression::Identifier(ident) => match ident.name.as_str() {
                "undefined" => Self::Void,
                "NaN" | "Infinity" => Self::Number,
                _ => Self::Undetermined,
            },
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::Void => Self::Void,
                UnaryOperator::UnaryNegation => {
                    let argument_ty = Self::from(&unary_expr.argument);
                    if argument_ty == Self::BigInt {
                        return Self::BigInt;
                    }
                    Self::Number
                }
                UnaryOperator::LogicalNot => Self::Boolean,
                UnaryOperator::Typeof => Self::Str,
                _ => Self::Undetermined,
            },
            _ => Self::Undetermined,
        }
    }
}

impl<'a> Compressor<'a> {
    pub(crate) fn fold_expression<'b>(&mut self, expr: &'b mut Expression<'a>) {
        let folded_expr = match expr {
            Expression::BinaryExpression(binary_expr) => match binary_expr.operator {
                BinaryOperator::Equality
                | BinaryOperator::Inequality
                | BinaryOperator::StrictEquality
                | BinaryOperator::StrictInequality
                | BinaryOperator::LessThan
                | BinaryOperator::LessEqualThan
                | BinaryOperator::GreaterThan
                | BinaryOperator::GreaterEqualThan => self.try_fold_comparison(
                    binary_expr.span,
                    binary_expr.operator,
                    &binary_expr.left,
                    &binary_expr.right,
                ),
                _ => None,
            },
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::Typeof => {
                    self.try_fold_typeof(unary_expr.span, &unary_expr.argument)
                }
                UnaryOperator::UnaryPlus
                | UnaryOperator::UnaryNegation
                | UnaryOperator::LogicalNot
                | UnaryOperator::BitwiseNot
                    if !unary_expr.may_have_side_effects() =>
                {
                    self.try_fold_unary_operator(unary_expr)
                }
                UnaryOperator::Void => self.try_reduce_void(unary_expr),
                _ => None,
            },
            _ => None,
        };
        if let Some(folded_expr) = folded_expr {
            *expr = folded_expr;
        }
    }

    fn try_fold_comparison<'b>(
        &mut self,
        span: Span,
        op: BinaryOperator,
        left: &'b Expression<'a>,
        right: &'b Expression<'a>,
    ) -> Option<Expression<'a>> {
        let value = match self.evaluate_comparison(op, left, right) {
            Tri::True => true,
            Tri::False => false,
            Tri::Unknown => return None,
        };
        let boolean_literal = self.hir.boolean_literal(span, value);
        Some(self.hir.literal_boolean_expression(boolean_literal))
    }

    fn evaluate_comparison<'b>(
        &self,
        op: BinaryOperator,
        left: &'b Expression<'a>,
        right: &'b Expression<'a>,
    ) -> Tri {
        if left.may_have_side_effects() || right.may_have_side_effects() {
            return Tri::Unknown;
        }

        match op {
            BinaryOperator::Equality => self.try_abstract_equality_comparison(left, right),
            BinaryOperator::Inequality => self.try_abstract_equality_comparison(left, right).not(),
            BinaryOperator::StrictEquality => self.try_strict_equality_comparison(left, right),
            BinaryOperator::StrictInequality => {
                self.try_strict_equality_comparison(left, right).not()
            }
            BinaryOperator::LessThan => self.try_abstract_relational_comparison(left, right, false),
            BinaryOperator::GreaterThan => {
                self.try_abstract_relational_comparison(right, left, false)
            }
            BinaryOperator::LessEqualThan => {
                self.try_abstract_relational_comparison(right, left, true).not()
            }
            BinaryOperator::GreaterEqualThan => {
                self.try_abstract_relational_comparison(left, right, true).not()
            }
            _ => Tri::Unknown,
        }
    }

    /// <https://tc39.es/ecma262/#sec-abstract-equality-comparison>
    fn try_abstract_equality_comparison<'b>(
        &self,
        left_expr: &'b Expression<'a>,
        right_expr: &'b Expression<'a>,
    ) -> Tri {
        let left = Ty::from(left_expr);
        let right = Ty::from(right_expr);
        if left != Ty::Undetermined && right != Ty::Undetermined {
            if left == right {
                return self.try_strict_equality_comparison(left_expr, right_expr);
            }
            if matches!((left, right), (Ty::Null, Ty::Void) | (Ty::Void, Ty::Null)) {
                return Tri::True;
            }

            return Tri::False;
        }
        Tri::Unknown
    }

    /// <https://tc39.es/ecma262/#sec-abstract-relational-comparison>
    fn try_abstract_relational_comparison<'b>(
        &self,
        left_expr: &'b Expression<'a>,
        right_expr: &'b Expression<'a>,
        will_negative: bool,
    ) -> Tri {
        let left = Ty::from(left_expr);
        let right = Ty::from(right_expr);

        // First, check for a string comparison.
        if left == Ty::Str && right == Ty::Str {
            let left_string = get_side_free_string_value(left_expr);
            let right_string = get_side_free_string_value(right_expr);
            if let Some(left_string) = left_string && let Some(right_string) = right_string {
                // In JS, browsers parse \v differently. So do not compare strings if one contains \v.
                if left_string.contains('\u{000B}') || right_string.contains('\u{000B}') {
                    return Tri::Unknown;
                }

                return Tri::for_boolean(left_string.cmp(&right_string) == Ordering::Less)
            }

            if let (Expression::UnaryExpression(left), Expression::UnaryExpression(right)) = (left_expr, right_expr)
                && (left.operator, right.operator) == (UnaryOperator::Typeof, UnaryOperator::Typeof)
                && let Expression::Identifier(left) = &left.argument
                && let Expression::Identifier(right) = &right.argument
                && left.name == right.name
            {
                // Special case: `typeof a < typeof a` is always false.
                return Tri::False;
            }
        }

        // try comparing as Numbers.
        let left_num = get_side_free_number_value(left_expr);
        let right_num = get_side_free_number_value(right_expr);
        if let Some(left_num) = left_num && let Some(right_num) = right_num {
            match (left_num, right_num) {
                (NumberValue::NaN, _) | (_, NumberValue::NaN) => return Tri::for_boolean(will_negative),
                (NumberValue::Number(left_num), NumberValue::Number(right_num)) => return Tri::for_boolean(left_num < right_num),
                _ => {}
            }
        }

        Tri::Unknown
    }

    /// <https://tc39.es/ecma262/#sec-strict-equality-comparison>
    fn try_strict_equality_comparison<'b>(
        &self,
        left_expr: &'b Expression<'a>,
        right_expr: &'b Expression<'a>,
    ) -> Tri {
        let left = Ty::from(left_expr);
        let right = Ty::from(right_expr);
        if left != Ty::Undetermined && right != Ty::Undetermined {
            // Strict equality can only be true for values of the same type.
            if left != right {
                return Tri::False;
            }
            return match left {
                Ty::Str => {
                    let left_string = get_side_free_string_value(left_expr);
                    let right_string = get_side_free_string_value(right_expr);
                    if let Some(left_string) = left_string && let Some(right_string) = right_string {
                        // In JS, browsers parse \v differently. So do not compare strings if one contains \v.
                        if left_string.contains('\u{000B}') || right_string.contains('\u{000B}') {
                            return Tri::Unknown;
                        }

                        return Tri::for_boolean(left_string == right_string)
                    }

                    if let (Expression::UnaryExpression(left), Expression::UnaryExpression(right)) = (left_expr, right_expr)
                        && (left.operator, right.operator) == (UnaryOperator::Typeof, UnaryOperator::Typeof)
                        && let Expression::Identifier(left) = &left.argument
                        && let Expression::Identifier(right) = &right.argument
                        && left.name == right.name
                    {
                        // Special case, typeof a == typeof a is always true.
                        return Tri::True;
                    }

                    Tri::Unknown
                }
                Ty::Void | Ty::Null => Tri::True,
                _ => Tri::Unknown,
            };
        }
        Tri::Unknown
    }

    /// Folds 'typeof(foo)' if foo is a literal, e.g.
    /// typeof("bar") --> "string"
    /// typeof(6) --> "number"
    fn try_fold_typeof<'b>(
        &mut self,
        span: Span,
        argument: &'b Expression<'a>,
    ) -> Option<Expression<'a>> {
        if argument.is_literal_value(true) {
            let type_name = match argument {
                Expression::FunctionExpression(_) | Expression::ArrowExpression(_) => {
                    Some("function")
                }
                Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => Some("string"),
                Expression::NumberLiteral(_) => Some("number"),
                Expression::BooleanLiteral(_) => Some("boolean"),
                Expression::NullLiteral(_)
                | Expression::ObjectExpression(_)
                | Expression::ArrayExpression(_) => Some("object"),
                Expression::Identifier(_) if argument.is_undefined() => Some("undefined"),
                Expression::UnaryExpression(unary_expr) => {
                    match unary_expr.operator {
                        UnaryOperator::Void => Some("undefined"),
                        // `unary_expr.argument` is literal value, so it's safe to fold
                        UnaryOperator::LogicalNot => Some("boolean"),
                        _ => None,
                    }
                }
                _ => None,
            };

            if let Some(type_name) = type_name {
                let string_literal = self.hir.string_literal(span, Atom::from(type_name));
                return Some(self.hir.literal_string_expression(string_literal));
            }
        }

        None
    }

    #[allow(clippy::too_many_lines)]
    fn try_fold_unary_operator<'b>(
        &mut self,
        unary_expr: &'b mut UnaryExpression<'a>,
    ) -> Option<Expression<'a>> {
        if let Some(boolean) = get_boolean_value(&unary_expr.argument) {
            match unary_expr.operator {
                // !100 -> false
                // !100n -> false
                // after this, it will be compressed to !1 or !0 in `compress_boolean`
                UnaryOperator::LogicalNot => match &unary_expr.argument {
                    Expression::NumberLiteral(number_literal) => {
                        let value = number_literal.value;
                        // Don't fold !0 and !1 back to false.
                        if value == 0_f64 || (value - 1_f64).abs() < f64::EPSILON {
                            return None;
                        }
                        let bool_literal = self.hir.boolean_literal(unary_expr.span, !boolean);
                        return Some(self.hir.literal_boolean_expression(bool_literal));
                    }
                    Expression::BigintLiteral(_) => {
                        let bool_literal = self.hir.boolean_literal(unary_expr.span, !boolean);
                        return Some(self.hir.literal_boolean_expression(bool_literal));
                    }
                    _ => {}
                },
                // +1 -> 1
                // NaN -> NaN
                // +Infinity -> Infinity
                UnaryOperator::UnaryPlus => match &unary_expr.argument {
                    Expression::NumberLiteral(number_literal) => {
                        let literal = self.hir.number_literal(
                            unary_expr.span,
                            number_literal.value,
                            number_literal.raw,
                            number_literal.base,
                        );
                        return Some(self.hir.literal_number_expression(literal));
                    }
                    Expression::Identifier(ident) => {
                        if matches!(ident.name.as_str(), "NaN" | "Infinity") {
                            return self.try_detach_unary_op(unary_expr);
                        }
                    }
                    _ => {
                        // +true -> 1
                        // +false -> 0
                        // +null -> 0
                        if let Some(value) = get_number_value(&unary_expr.argument)
                            && let NumberValue::Number(value) = value {
                            let raw = self.hir.new_str(value.to_string().as_str());
                            let literal = self.hir.number_literal(
                                unary_expr.span,
                                value,
                                raw,
                                if value.fract() == 0.0 {
                                    NumberBase::Decimal
                                } else {
                                    NumberBase::Float
                                },
                            );
                            return Some(self.hir.literal_number_expression(literal));
                        }
                    }
                },
                // -4 -> -4, fold UnaryExpression -4 to NumberLiteral -4
                // -NaN -> NaN
                UnaryOperator::UnaryNegation => match &unary_expr.argument {
                    Expression::NumberLiteral(number_literal) => {
                        let value = -number_literal.value;
                        let raw = self.hir.new_str(value.to_string().as_str());
                        let literal = self.hir.number_literal(
                            unary_expr.span,
                            value,
                            raw,
                            number_literal.base,
                        );
                        return Some(self.hir.literal_number_expression(literal));
                    }
                    Expression::BigintLiteral(big_int_literal) => {
                        use std::ops::Neg;

                        let value = big_int_literal.value.clone().neg();
                        let literal = self.hir.bigint_literal(unary_expr.span, value);
                        return Some(self.hir.literal_bigint_expression(literal));
                    }
                    Expression::Identifier(ident) => {
                        if ident.name == "NaN" {
                            return self.try_detach_unary_op(unary_expr);
                        }
                    }
                    _ => {}
                },
                // ~10 -> -11
                // ~NaN -> -1
                UnaryOperator::BitwiseNot => match &unary_expr.argument {
                    Expression::NumberLiteral(number_literal) => {
                        if number_literal.value.fract() == 0.0 {
                            let int_value =
                                NumberLiteral::ecmascript_to_int32(number_literal.value);
                            let literal = self.hir.number_literal(
                                unary_expr.span,
                                f64::from(!int_value),
                                number_literal.raw,
                                NumberBase::Decimal, // since it be converted to i32, it should always be decimal.
                            );
                            return Some(self.hir.literal_number_expression(literal));
                        }
                    }
                    Expression::BigintLiteral(big_int_literal) => {
                        let value = big_int_literal.value.clone().not();
                        let leteral = self.hir.bigint_literal(unary_expr.span, value);
                        return Some(self.hir.literal_bigint_expression(leteral));
                    }
                    Expression::Identifier(ident) => {
                        if ident.name == "NaN" {
                            let value = -1_f64;
                            let raw = self.hir.new_str("-1");
                            let literal = self.hir.number_literal(
                                unary_expr.span,
                                value,
                                raw,
                                NumberBase::Decimal,
                            );
                            return Some(self.hir.literal_number_expression(literal));
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        None
    }

    // +NaN -> NaN
    // !Infinity -> Infinity
    fn try_detach_unary_op(
        &mut self,
        unary_expr: &mut UnaryExpression<'a>,
    ) -> Option<Expression<'a>> {
        if let Expression::Identifier(ident) = &unary_expr.argument {
            if matches!(ident.name.as_str(), "NaN" | "Infinity") {
                let ident = self.hir.identifier_reference(
                    unary_expr.span,
                    ident.name.clone(),
                    ident.reference_id.clone().into_inner(),
                    ident.reference_flag,
                );
                return Some(self.hir.identifier_reference_expression(ident));
            }
        }

        None
    }

    /// port from [closure-compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/PeepholeFoldConstants.java#L195)
    /// void 0 -> void 0
    /// void 1 -> void 0
    /// void x -> void 0
    fn try_reduce_void(&mut self, unary_expr: &UnaryExpression<'a>) -> Option<Expression<'a>> {
        let can_replace = match &unary_expr.argument {
            Expression::NumberLiteral(number_literal) => number_literal.value != 0_f64,
            _ => !unary_expr.may_have_side_effects(),
        };

        if can_replace {
            let number_literal = self.hir.number_literal(
                unary_expr.argument.span(),
                0_f64,
                self.hir.new_str("0"),
                NumberBase::Decimal,
            );

            let argument = self.hir.literal_number_expression(number_literal);
            return Some(self.hir.unary_expression(unary_expr.span, UnaryOperator::Void, argument));
        }
        None
    }
}
