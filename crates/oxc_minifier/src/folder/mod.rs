//! Constant Folding
//!
//! <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeFoldConstants.java>

mod tri;
mod ty;
mod util;

use std::{cmp::Ordering, mem};

use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    number::NumberBase,
    operator::{BinaryOperator, LogicalOperator, UnaryOperator},
};

use crate::compressor::ast_util::{
    get_boolean_value, get_number_value, get_side_free_bigint_value, get_side_free_number_value,
    get_side_free_string_value, get_string_value, is_exact_int64, IsLiteralValue,
    MayHaveSideEffects, NumberValue,
};

use tri::Tri;
use ty::Ty;
use util::bigint_less_than_number;

pub struct Folder<'a> {
    ast: AstBuilder<'a>,
    evaluate: bool,
}

impl<'a> Folder<'a> {
    pub fn new(ast: AstBuilder<'a>) -> Self {
        Self { ast, evaluate: false }
    }

    pub fn with_evaluate(mut self, yes: bool) -> Self {
        self.evaluate = yes;
        self
    }

    pub fn fold_expression<'b>(&mut self, expr: &'b mut Expression<'a>) {
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
                BinaryOperator::ShiftLeft
                | BinaryOperator::ShiftRight
                | BinaryOperator::ShiftRightZeroFill => self.try_fold_shift(
                    binary_expr.span,
                    binary_expr.operator,
                    &binary_expr.left,
                    &binary_expr.right,
                ),
                // NOTE: string concat folding breaks our current evaluation of Test262 tests. The
                // minifier is tested by comparing output of running the minifier once and twice,
                // respectively. Since Test262Error messages include string concats, the outputs
                // don't match (even though the produced code is valid). Additionally, We'll likely
                // want to add `evaluate` checks for all constant folding, not just additions, but
                // we're adding this here until a decision is made.
                BinaryOperator::Addition if self.evaluate => {
                    self.try_fold_addition(binary_expr.span, &binary_expr.left, &binary_expr.right)
                }
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
            Expression::LogicalExpression(logic_expr) => match logic_expr.operator {
                LogicalOperator::And | LogicalOperator::Or => {
                    self.try_fold_and_or(logic_expr.operator, logic_expr)
                }
                LogicalOperator::Coalesce => None,
            },
            _ => None,
        };
        if let Some(folded_expr) = folded_expr {
            *expr = folded_expr;
        }
    }

    fn try_fold_addition<'b>(
        &mut self,
        span: Span,
        left: &'b Expression<'a>,
        right: &'b Expression<'a>,
    ) -> Option<Expression<'a>> {
        // skip any potentially dangerous compressions
        if left.may_have_side_effects() || right.may_have_side_effects() {
            return None;
        }

        let left_type = Ty::from(left);
        let right_type = Ty::from(right);
        match (left_type, right_type) {
            (Ty::Undetermined, _) | (_, Ty::Undetermined) => None,

            // string concatenation
            (Ty::Str, _) | (_, Ty::Str) => {
                // no need to use get_side_effect_free_string_value b/c we checked for side effects
                // at the beginning
                let left_string = get_string_value(left)?;
                let right_string = get_string_value(right)?;
                // let value = left_string.to_owned().
                let value = left_string + right_string;
                Some(self.ast.expression_string_literal(span, value))
            },

            // number addition
            (Ty::Number, _) | (_, Ty::Number)
                // when added, booleans get treated as numbers where `true` is 1 and `false` is 0
                | (Ty::Boolean, Ty::Boolean) => {
                let left_number = get_number_value(left)?;
                let right_number = get_number_value(right)?;
                let Ok(value) = TryInto::<f64>::try_into(left_number + right_number) else { return None };
                // Float if value has a fractional part, otherwise Decimal
                let number_base = if is_exact_int64(value) { NumberBase::Decimal } else { NumberBase::Float };
                // todo: add raw &str
                Some(self.ast.expression_numeric_literal(span, value, "", number_base))
            },
            _ => None
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
        Some(self.ast.expression_boolean_literal(span, value))
    }

    fn evaluate_comparison<'b>(
        &mut self,
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
        &mut self,
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

            if matches!((left, right), (Ty::Number, Ty::Str)) || matches!(right, Ty::Boolean) {
                let right_number = get_side_free_number_value(right_expr);

                if let Some(NumberValue::Number(num)) = right_number {
                    let number_literal_expr = self.ast.expression_numeric_literal(
                        right_expr.span(),
                        num,
                        num.to_string(),
                        if num.fract() == 0.0 { NumberBase::Decimal } else { NumberBase::Float },
                    );

                    return self.try_abstract_equality_comparison(left_expr, &number_literal_expr);
                }

                return Tri::Unknown;
            }

            if matches!((left, right), (Ty::Str, Ty::Number)) || matches!(left, Ty::Boolean) {
                let left_number = get_side_free_number_value(left_expr);

                if let Some(NumberValue::Number(num)) = left_number {
                    let number_literal_expr = self.ast.expression_numeric_literal(
                        left_expr.span(),
                        num,
                        num.to_string(),
                        if num.fract() == 0.0 { NumberBase::Decimal } else { NumberBase::Float },
                    );

                    return self.try_abstract_equality_comparison(&number_literal_expr, right_expr);
                }

                return Tri::Unknown;
            }

            if matches!(left, Ty::BigInt) || matches!(right, Ty::BigInt) {
                let left_bigint = get_side_free_bigint_value(left_expr);
                let right_bigint = get_side_free_bigint_value(right_expr);

                if let (Some(l_big), Some(r_big)) = (left_bigint, right_bigint) {
                    return Tri::for_boolean(l_big.eq(&r_big));
                }
            }

            if matches!(left, Ty::Str | Ty::Number) && matches!(right, Ty::Object) {
                return Tri::Unknown;
            }

            if matches!(left, Ty::Object) && matches!(right, Ty::Str | Ty::Number) {
                return Tri::Unknown;
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
            if let (Some(left_string), Some(right_string)) = (left_string, right_string) {
                // In JS, browsers parse \v differently. So do not compare strings if one contains \v.
                if left_string.contains('\u{000B}') || right_string.contains('\u{000B}') {
                    return Tri::Unknown;
                }

                return Tri::for_boolean(left_string.cmp(&right_string) == Ordering::Less);
            }

            if let (Expression::UnaryExpression(left), Expression::UnaryExpression(right)) =
                (left_expr, right_expr)
            {
                if (left.operator, right.operator) == (UnaryOperator::Typeof, UnaryOperator::Typeof)
                {
                    if let (Expression::Identifier(left), Expression::Identifier(right)) =
                        (&left.argument, &right.argument)
                    {
                        if left.name == right.name {
                            // Special case: `typeof a < typeof a` is always false.
                            return Tri::False;
                        }
                    }
                }
            }
        }

        let left_bigint = get_side_free_bigint_value(left_expr);
        let right_bigint = get_side_free_bigint_value(right_expr);

        let left_num = get_side_free_number_value(left_expr);
        let right_num = get_side_free_number_value(right_expr);

        match (left_bigint, right_bigint, left_num, right_num) {
            // Next, try to evaluate based on the value of the node. Try comparing as BigInts first.
            (Some(l_big), Some(r_big), _, _) => {
                return Tri::for_boolean(l_big < r_big);
            }
            // try comparing as Numbers.
            (_, _, Some(l_num), Some(r_num)) => match (l_num, r_num) {
                (NumberValue::NaN, _) | (_, NumberValue::NaN) => {
                    return Tri::for_boolean(will_negative);
                }
                (NumberValue::Number(l), NumberValue::Number(r)) => return Tri::for_boolean(l < r),
                _ => {}
            },
            // Finally, try comparisons between BigInt and Number.
            (Some(l_big), _, _, Some(r_num)) => {
                return bigint_less_than_number(&l_big, &r_num, Tri::False, will_negative);
            }
            (_, Some(r_big), Some(l_num), _) => {
                return bigint_less_than_number(&r_big, &l_num, Tri::True, will_negative);
            }
            _ => {}
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
                Ty::Number => {
                    let left_number = get_side_free_number_value(left_expr);
                    let right_number = get_side_free_number_value(right_expr);

                    if let (Some(l_num), Some(r_num)) = (left_number, right_number) {
                        if l_num.is_nan() || r_num.is_nan() {
                            return Tri::False;
                        }

                        return Tri::for_boolean(l_num == r_num);
                    }

                    Tri::Unknown
                }
                Ty::Str => {
                    let left_string = get_side_free_string_value(left_expr);
                    let right_string = get_side_free_string_value(right_expr);
                    if let (Some(left_string), Some(right_string)) = (left_string, right_string) {
                        // In JS, browsers parse \v differently. So do not compare strings if one contains \v.
                        if left_string.contains('\u{000B}') || right_string.contains('\u{000B}') {
                            return Tri::Unknown;
                        }

                        return Tri::for_boolean(left_string == right_string);
                    }

                    if let (Expression::UnaryExpression(left), Expression::UnaryExpression(right)) =
                        (left_expr, right_expr)
                    {
                        if (left.operator, right.operator)
                            == (UnaryOperator::Typeof, UnaryOperator::Typeof)
                        {
                            if let (Expression::Identifier(left), Expression::Identifier(right)) =
                                (&left.argument, &right.argument)
                            {
                                if left.name == right.name {
                                    // Special case, typeof a == typeof a is always true.
                                    return Tri::True;
                                }
                            }
                        }
                    }

                    Tri::Unknown
                }
                Ty::Void | Ty::Null => Tri::True,
                _ => Tri::Unknown,
            };
        }

        // Then, try to evaluate based on the value of the expression.
        // There's only one special case:
        // Any strict equality comparison against NaN returns false.
        if left_expr.is_nan() || right_expr.is_nan() {
            return Tri::False;
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
                Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => {
                    Some("function")
                }
                Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => Some("string"),
                Expression::NumericLiteral(_) => Some("number"),
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
                return Some(self.ast.expression_string_literal(span, type_name));
            }
        }

        None
    }

    fn try_fold_unary_operator(
        &mut self,
        unary_expr: &UnaryExpression<'a>,
    ) -> Option<Expression<'a>> {
        if let Some(boolean) = get_boolean_value(&unary_expr.argument) {
            match unary_expr.operator {
                // !100 -> false
                // !100n -> false
                // after this, it will be compressed to !1 or !0 in `compress_boolean`
                UnaryOperator::LogicalNot => match &unary_expr.argument {
                    Expression::NumericLiteral(number_literal) => {
                        let value = number_literal.value;
                        // Don't fold !0 and !1 back to false.
                        if value == 0_f64 || (value - 1_f64).abs() < f64::EPSILON {
                            return None;
                        }
                        return Some(
                            self.ast.expression_boolean_literal(unary_expr.span, !boolean),
                        );
                    }
                    Expression::BigIntLiteral(_) => {
                        return Some(
                            self.ast.expression_boolean_literal(unary_expr.span, !boolean),
                        );
                    }
                    _ => {}
                },
                // +1 -> 1
                // NaN -> NaN
                // +Infinity -> Infinity
                UnaryOperator::UnaryPlus => match &unary_expr.argument {
                    Expression::NumericLiteral(number_literal) => {
                        return Some(self.ast.expression_numeric_literal(
                            unary_expr.span,
                            number_literal.value,
                            number_literal.raw,
                            number_literal.base,
                        ));
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
                        if let Some(NumberValue::Number(value)) =
                            get_number_value(&unary_expr.argument)
                        {
                            return Some(self.ast.expression_numeric_literal(
                                unary_expr.span,
                                value,
                                value.to_string(),
                                if value.fract() == 0.0 {
                                    NumberBase::Decimal
                                } else {
                                    NumberBase::Float
                                },
                            ));
                        }
                    }
                },
                // -4 -> -4, fold UnaryExpression -4 to NumericLiteral -4
                // -NaN -> NaN
                UnaryOperator::UnaryNegation => match &unary_expr.argument {
                    Expression::NumericLiteral(number_literal) => {
                        let value = -number_literal.value;
                        return Some(self.ast.expression_numeric_literal(
                            unary_expr.span,
                            value,
                            value.to_string(),
                            number_literal.base,
                        ));
                    }
                    Expression::BigIntLiteral(_big_int_literal) => {
                        // let value = big_int_literal.value.clone().neg();
                        // let literal =
                        // self.ast.bigint_literal(unary_expr.span, value, big_int_literal.base);
                        // return Some(self.ast.literal_bigint_expression(literal));
                        return None;
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
                    Expression::NumericLiteral(number_literal) => {
                        if number_literal.value.fract() == 0.0 {
                            let int_value =
                                NumericLiteral::ecmascript_to_int32(number_literal.value);
                            return Some(self.ast.expression_numeric_literal(
                                unary_expr.span,
                                f64::from(!int_value),
                                number_literal.raw,
                                NumberBase::Decimal, // since it be converted to i32, it should always be decimal.
                            ));
                        }
                    }
                    Expression::BigIntLiteral(_big_int_literal) => {
                        // let value = big_int_literal.value.clone().not();
                        // let leteral =
                        // self.ast.bigint_literal(unary_expr.span, value, big_int_literal.base);
                        // return Some(self.ast.literal_bigint_expression(leteral));
                        return None;
                    }
                    Expression::Identifier(ident) => {
                        if ident.name == "NaN" {
                            let value = -1_f64;
                            return Some(self.ast.expression_numeric_literal(
                                unary_expr.span,
                                value,
                                "-1",
                                NumberBase::Decimal,
                            ));
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
    fn try_detach_unary_op(&mut self, unary_expr: &UnaryExpression<'a>) -> Option<Expression<'a>> {
        if let Expression::Identifier(ident) = &unary_expr.argument {
            if matches!(ident.name.as_str(), "NaN" | "Infinity") {
                let ident = IdentifierReference {
                    span: unary_expr.span,
                    name: ident.name.clone(),
                    reference_id: ident.reference_id.clone(),
                    reference_flag: ident.reference_flag,
                };
                return Some(self.ast.expression_from_identifier_reference(ident));
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
            Expression::NumericLiteral(number_literal) => number_literal.value != 0_f64,
            _ => !unary_expr.may_have_side_effects(),
        };

        if can_replace {
            let argument = self.ast.expression_numeric_literal(
                unary_expr.argument.span(),
                0_f64,
                "0",
                NumberBase::Decimal,
            );
            return Some(self.ast.expression_unary(unary_expr.span, UnaryOperator::Void, argument));
        }
        None
    }

    /// ported from [closure-compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/PeepholeFoldConstants.java#L1114-L1162)
    #[allow(clippy::cast_possible_truncation)]
    fn try_fold_shift<'b>(
        &mut self,
        span: Span,
        op: BinaryOperator,
        left: &'b Expression<'a>,
        right: &'b Expression<'a>,
    ) -> Option<Expression<'a>> {
        let left_num = get_side_free_number_value(left);
        let right_num = get_side_free_number_value(right);

        if let (Some(NumberValue::Number(left_val)), Some(NumberValue::Number(right_val))) =
            (left_num, right_num)
        {
            if left_val.fract() != 0.0 || right_val.fract() != 0.0 {
                return None;
            }

            // only the lower 5 bits are used when shifting, so don't do anything
            // if the shift amount is outside [0,32)
            if !(0.0..32.0).contains(&right_val) {
                return None;
            }

            let right_val_int = right_val as i32;
            let bits = NumericLiteral::ecmascript_to_int32(left_val);

            let result_val: f64 = match op {
                BinaryOperator::ShiftLeft => f64::from(bits << right_val_int),
                BinaryOperator::ShiftRight => f64::from(bits >> right_val_int),
                BinaryOperator::ShiftRightZeroFill => {
                    // JavaScript always treats the result of >>> as unsigned.
                    // We must force Rust to do the same here.
                    #[allow(clippy::cast_sign_loss)]
                    let res = bits as u32 >> right_val_int as u32;
                    f64::from(res)
                }
                _ => unreachable!("Unknown binary operator {:?}", op),
            };

            return Some(self.ast.expression_numeric_literal(
                span,
                result_val,
                result_val.to_string(),
                NumberBase::Decimal,
            ));
        }

        None
    }

    /// port from [closure-compiler](https://github.com/google/closure-compiler/blob/09094b551915a6487a980a783831cba58b5739d1/src/com/google/javascript/jscomp/PeepholeFoldConstants.java#L587)
    /// Try to fold a AND/OR node.
    fn try_fold_and_or(
        &mut self,
        op: LogicalOperator,
        logic_expr: &mut LogicalExpression<'a>,
    ) -> Option<Expression<'a>> {
        let boolean_value = get_boolean_value(&logic_expr.left);

        if let Some(boolean_value) = boolean_value {
            // (TRUE || x) => TRUE (also, (3 || x) => 3)
            // (FALSE && x) => FALSE
            if (boolean_value && op == LogicalOperator::Or)
                || (!boolean_value && op == LogicalOperator::And)
            {
                return Some(self.move_out_expression(&mut logic_expr.left));
            } else if !logic_expr.left.may_have_side_effects() {
                // (FALSE || x) => x
                // (TRUE && x) => x
                return Some(self.move_out_expression(&mut logic_expr.right));
            }
            // Left side may have side effects, but we know its boolean value.
            // e.g. true_with_sideeffects || foo() => true_with_sideeffects, foo()
            // or: false_with_sideeffects && foo() => false_with_sideeffects, foo()
            let left = self.move_out_expression(&mut logic_expr.left);
            let right = self.move_out_expression(&mut logic_expr.right);
            let mut vec = self.ast.new_vec_with_capacity(2);
            vec.push(left);
            vec.push(right);
            let sequence_expr = self.ast.expression_sequence(logic_expr.span, vec);
            return Some(sequence_expr);
        } else if let Expression::LogicalExpression(left_child) = &mut logic_expr.left {
            if left_child.operator == logic_expr.operator {
                let left_child_right_boolean = get_boolean_value(&left_child.right);
                let left_child_op = left_child.operator;
                if let Some(right_boolean) = left_child_right_boolean {
                    if !left_child.right.may_have_side_effects() {
                        // a || false || b => a || b
                        // a && true && b => a && b
                        if !right_boolean && left_child_op == LogicalOperator::Or
                            || right_boolean && left_child_op == LogicalOperator::And
                        {
                            let left = self.move_out_expression(&mut left_child.left);
                            let right = self.move_out_expression(&mut logic_expr.right);
                            let logic_expr = self.ast.expression_logical(
                                logic_expr.span,
                                left,
                                left_child_op,
                                right,
                            );
                            return Some(logic_expr);
                        }
                    }
                }
            }
        }
        None
    }

    pub(crate) fn fold_condition<'b>(&mut self, stmt: &'b mut Statement<'a>) {
        match stmt {
            Statement::WhileStatement(while_stmt) => {
                let minimized_expr = self.fold_expression_in_condition(&mut while_stmt.test);

                if let Some(min_expr) = minimized_expr {
                    while_stmt.test = min_expr;
                }
            }
            Statement::ForStatement(for_stmt) => {
                let test_expr = for_stmt.test.as_mut();

                if let Some(test_expr) = test_expr {
                    let minimized_expr = self.fold_expression_in_condition(test_expr);

                    if let Some(min_expr) = minimized_expr {
                        for_stmt.test = Some(min_expr);
                    }
                }
            }
            _ => {}
        };
    }

    fn fold_expression_in_condition(
        &mut self,
        expr: &mut Expression<'a>,
    ) -> Option<Expression<'a>> {
        let folded_expr = match expr {
            Expression::UnaryExpression(unary_expr) => match unary_expr.operator {
                UnaryOperator::LogicalNot => {
                    let should_fold = self.try_minimize_not(&mut unary_expr.argument);

                    if should_fold {
                        Some(self.move_out_expression(&mut unary_expr.argument))
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        };

        folded_expr
    }

    fn move_out_expression(&mut self, expr: &mut Expression<'a>) -> Expression<'a> {
        let null_expr = self.ast.expression_null_literal(expr.span());
        mem::replace(expr, null_expr)
    }

    /// ported from [closure compiler](https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeMinimizeConditions.java#L401-L435)
    fn try_minimize_not(&mut self, expr: &mut Expression<'a>) -> bool {
        let span = &mut expr.span();

        match expr {
            Expression::BinaryExpression(binary_expr) => {
                let new_op = binary_expr.operator.equality_inverse_operator();

                match new_op {
                    Some(new_op) => {
                        binary_expr.operator = new_op;
                        binary_expr.span = *span;

                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}
