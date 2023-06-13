//! Constant Folding
//!
//! <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeFoldConstants.java>

#[allow(clippy::wildcard_imports)]
use oxc_hir::hir::*;
use oxc_hir::hir_util::{get_boolean_value, get_number_value, IsLiteralValue, MayHaveSideEffects};
use oxc_span::{Atom, Span};
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
            Expression::ObjectExpression(_) => Self::Object,
            Expression::StringLiteral(_) => Self::Str,
            Expression::Identifier(ident) => match ident.name.as_str() {
                "undefined" => Self::Void,
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
                BinaryOperator::Equality => self.try_fold_comparison(
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
                if !unary_expr.may_have_side_effects() => {
                    self.try_fold_unary_operator(unary_expr)
                }
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
        match op {
            BinaryOperator::Equality => self.try_abstract_equality_comparison(left, right),
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
            if left != right {
                return Tri::False;
            }
            return match left {
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
                Expression::UnaryExpression(unary_expr)
                    if unary_expr.operator == UnaryOperator::Void =>
                {
                    Some("undefined")
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

    fn try_fold_unary_operator<'b>(
        &mut self,
        unary_expr: &'b mut UnaryExpression<'a>,
    ) -> Option<Expression<'a>> {
        // fold its children first, so that we can fold - -4.
        self.fold_expression(&mut unary_expr.argument);

        if let Some(boolean) = get_boolean_value(&unary_expr.argument) {
            match unary_expr.operator {
                // !100 -> false
                // after this, it will be compressed to !1 or !0 in `compress_boolean`
                UnaryOperator::LogicalNot => {
                    if let Expression::NumberLiteral(number_literal) = &unary_expr.argument {
                        let value = number_literal.value;
                        // Don't fold !0 and !1 back to false.
                        if value == 0_f64 || (value - 1_f64).abs() < f64::EPSILON {
                            return None
                        }
                        let bool_literal =
                            self.hir.boolean_literal(unary_expr.span, !boolean);
                        return Some(self.hir.literal_boolean_expression(bool_literal))
                    }
                }
                // +1 -> 1
                // NaN -> NaN
                // +Infinity -> Infinity
                UnaryOperator::UnaryPlus => match &unary_expr.argument {
                    Expression::NumberLiteral(number_literal) => {
                        let number_literal = self.hir.number_literal(
                            unary_expr.span,
                            number_literal.value,
                            number_literal.raw,
                            number_literal.base,
                        );
                        return Some(self.hir.literal_number_expression(number_literal))
                    }
                    Expression::Identifier(ident) => {
                        if matches!(ident.name.as_str(), "NaN" | "Infinity") {
                            return self.try_detach_unary_op(unary_expr)
                        }
                    }
                    _ => {
                        // +true -> 1
                        // +false -> 0
                        // +null -> 0 
                        if let Some(value) = get_number_value(&unary_expr.argument) {
                            let raw = self.hir.new_str(value.to_string().as_str());
                            let number_literal = self.hir.number_literal(
                                unary_expr.span,
                                value,
                                raw,
                                if value.fract() == 0.0 { NumberBase::Decimal} else {NumberBase::Float} ,
                            );
                            return Some(self.hir.literal_number_expression(number_literal))
                        }
                    }
                },
                // -4 -> -4, fold UnaryExpression -4 to NumberLiteral -4
                // -NaN -> NaN
                UnaryOperator::UnaryNegation => match &unary_expr.argument {
                    Expression::NumberLiteral(number_literal) => {
                        let value = -number_literal.value;
                        let raw = self.hir.new_str(value.to_string().as_str());
                        let number_literal =
                            self.hir.number_literal(unary_expr.span, value, raw, number_literal.base);
                        return Some(self.hir.literal_number_expression(number_literal))
                    }
                    Expression::BigintLiteral(_big_int_literal) => {
                        return None
                    }
                    Expression::Identifier(ident) => {
                        if ident.name == "NaN" {
                            return self.try_detach_unary_op(unary_expr)
                        }
                    }
                    _ => {},
                },
                // ~10 -> -11
                UnaryOperator::BitwiseNot => {
                    if let Expression::NumberLiteral(number_literal) = &unary_expr.argument && number_literal.value.fract() == 0.0 {
                            let int_value = NumberLiteral::ecmascript_to_int32(number_literal.value);
                            let number_literal = self.hir.number_literal(
                                unary_expr.span,
                                f64::from(!int_value),
                                number_literal.raw,
                                NumberBase::Decimal, // since it be converted to i32, it should always be decimal.
                            );
                            return Some(self.hir.literal_number_expression(number_literal))
                    }
                }
                _ => {},
            }
        }

        None
    }

    fn try_detach_unary_op(
        &mut self,
        unary_expr: &mut UnaryExpression<'a>,
    ) -> Option<Expression<'a>> {
        if let Expression::Identifier(ident) = &unary_expr.argument {
            if matches!(ident.name.as_str(), "NaN" | "Infinity") {
                let ident = self.hir.identifier_reference(
                    unary_expr.span,
                    ident.name.clone(),
                    ident.reference_id,
                );
                return Some(self.hir.identifier_reference_expression(ident));
            }
        }

        None
    }
}
