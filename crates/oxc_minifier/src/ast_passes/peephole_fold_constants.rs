use std::{cmp::Ordering, mem};

use num_bigint::BigInt;
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span, SPAN};
use oxc_syntax::{
    number::NumberBase,
    operator::{BinaryOperator, LogicalOperator, UnaryOperator},
};
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

use crate::{
    node_util::{
        is_exact_int64, IsLiteralValue, MayHaveSideEffects, NodeUtil, NumberValue, ValueType,
    },
    tri::Tri,
    ty::Ty,
    CompressorPass,
};

/// Constant Folding
///
/// <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeFoldConstants.java>
pub struct PeepholeFoldConstants {
    evaluate: bool,
}

impl<'a> CompressorPass<'a> for PeepholeFoldConstants {}

impl<'a> Traverse<'a> for PeepholeFoldConstants {
    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.fold_expression(expr, ctx);
    }
}

impl<'a> PeepholeFoldConstants {
    pub fn new() -> Self {
        Self { evaluate: false }
    }

    pub fn with_evaluate(mut self, yes: bool) -> Self {
        self.evaluate = yes;
        self
    }

    // [optimizeSubtree](https://github.com/google/closure-compiler/blob/75335a5138dde05030747abfd3c852cd34ea7429/src/com/google/javascript/jscomp/PeepholeFoldConstants.java#L72)
    // TODO: tryReduceOperandsForOp
    pub fn fold_expression(&self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(folded_expr) = match expr {
            Expression::BinaryExpression(e) => self.try_fold_binary_operator(e, ctx),
            Expression::LogicalExpression(e)
                if matches!(e.operator, LogicalOperator::And | LogicalOperator::Or) =>
            {
                self.try_fold_and_or(e, ctx)
            }
            Expression::LogicalExpression(e) if e.operator == LogicalOperator::Coalesce => {
                self.try_fold_coalesce(e, ctx)
            }
            Expression::UnaryExpression(e) => self.try_fold_unary_expression(e, ctx),
            _ => None,
        } {
            *expr = folded_expr;
        };
    }

    fn try_fold_binary_operator(
        &self,
        binary_expr: &mut BinaryExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        match binary_expr.operator {
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
                ctx,
            ),
            BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftRightZeroFill => self.try_fold_shift(
                binary_expr.span,
                binary_expr.operator,
                &binary_expr.left,
                &binary_expr.right,
                ctx,
            ),
            // NOTE: string concat folding breaks our current evaluation of Test262 tests. The
            // minifier is tested by comparing output of running the minifier once and twice,
            // respectively. Since Test262Error messages include string concats, the outputs
            // don't match (even though the produced code is valid). Additionally, We'll likely
            // want to add `evaluate` checks for all constant folding, not just additions, but
            // we're adding this here until a decision is made.
            BinaryOperator::Addition if self.evaluate => {
                self.try_fold_addition(binary_expr.span, &binary_expr.left, &binary_expr.right, ctx)
            }
            _ => None,
        }
    }

    fn try_fold_unary_expression(
        &self,
        expr: &mut UnaryExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        match expr.operator {
            UnaryOperator::Void => Self::try_reduce_void(expr, ctx),
            UnaryOperator::Typeof => self.try_fold_type_of(expr, ctx),
            UnaryOperator::LogicalNot => {
                expr.argument.to_boolean().map(|b| ctx.ast.expression_boolean_literal(SPAN, !b))
            }
            // `-NaN` -> `NaN`
            UnaryOperator::UnaryNegation if expr.argument.is_nan() => {
                Some(ctx.ast.move_expression(&mut expr.argument))
            }
            // `+1` -> `1`
            UnaryOperator::UnaryPlus if expr.argument.is_number() => {
                Some(ctx.ast.move_expression(&mut expr.argument))
            }
            _ => None,
        }
    }

    /// `void 1` -> `void 0`
    fn try_reduce_void(
        expr: &mut UnaryExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if (!expr.argument.is_number() || !expr.argument.is_number_0())
            && !expr.may_have_side_effects()
        {
            let _ = mem::replace(&mut expr.argument, ctx.ast.number_0());
        }
        None
    }

    /// Folds 'typeof(foo)' if foo is a literal, e.g.
    /// `typeof("bar") --> "string"`
    /// `typeof(6) --> "number"`
    fn try_fold_type_of(
        &self,
        expr: &mut UnaryExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if !expr.argument.is_literal_value(/* include_function */ true) {
            return None;
        }
        let s = match &mut expr.argument {
            Expression::FunctionExpression(_) => "function",
            Expression::StringLiteral(_) => "string",
            Expression::NumericLiteral(_) => "number",
            Expression::BooleanLiteral(_) => "boolean",
            Expression::NullLiteral(_)
            | Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_) => "object",
            Expression::UnaryExpression(e) if e.operator == UnaryOperator::Void => "undefined",
            Expression::BigIntLiteral(_) => "bigint",
            Expression::Identifier(ident) if ctx.is_identifier_undefined(ident) => "undefined",
            _ => return None,
        };
        Some(ctx.ast.expression_string_literal(SPAN, s))
    }

    fn try_fold_addition<'b>(
        &self,
        span: Span,
        left: &'b Expression<'a>,
        right: &'b Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
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
                let left_string = ctx.get_string_value(left)?;
                let right_string = ctx.get_string_value(right)?;
                // let value = left_string.to_owned().
                let value = left_string + right_string;
                Some(ctx.ast.expression_string_literal(span, value))
            },

            // number addition
            (Ty::Number, _) | (_, Ty::Number)
                // when added, booleans get treated as numbers where `true` is 1 and `false` is 0
                | (Ty::Boolean, Ty::Boolean) => {
                let left_number = ctx.get_number_value(left)?;
                let right_number = ctx.get_number_value(right)?;
                let Ok(value) = TryInto::<f64>::try_into(left_number + right_number) else { return None };
                // Float if value has a fractional part, otherwise Decimal
                let number_base = if is_exact_int64(value) { NumberBase::Decimal } else { NumberBase::Float };
                // todo: add raw &str
                Some(ctx.ast.expression_numeric_literal(span, value, "", number_base))
            },
            _ => None
        }
    }

    fn try_fold_comparison<'b>(
        &self,
        span: Span,
        op: BinaryOperator,
        left: &'b Expression<'a>,
        right: &'b Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let value = match self.evaluate_comparison(op, left, right, ctx) {
            Tri::True => true,
            Tri::False => false,
            Tri::Unknown => return None,
        };
        Some(ctx.ast.expression_boolean_literal(span, value))
    }

    fn evaluate_comparison<'b>(
        &self,
        op: BinaryOperator,
        left: &'b Expression<'a>,
        right: &'b Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Tri {
        if left.may_have_side_effects() || right.may_have_side_effects() {
            return Tri::Unknown;
        }
        match op {
            BinaryOperator::Equality => Self::try_abstract_equality_comparison(left, right, ctx),
            BinaryOperator::Inequality => {
                Self::try_abstract_equality_comparison(left, right, ctx).not()
            }
            BinaryOperator::StrictEquality => {
                Self::try_strict_equality_comparison(left, right, ctx)
            }
            BinaryOperator::StrictInequality => {
                Self::try_strict_equality_comparison(left, right, ctx).not()
            }
            BinaryOperator::LessThan => {
                Self::try_abstract_relational_comparison(left, right, false, ctx)
            }
            BinaryOperator::GreaterThan => {
                Self::try_abstract_relational_comparison(right, left, false, ctx)
            }
            BinaryOperator::LessEqualThan => {
                Self::try_abstract_relational_comparison(right, left, true, ctx).not()
            }
            BinaryOperator::GreaterEqualThan => {
                Self::try_abstract_relational_comparison(left, right, true, ctx).not()
            }
            _ => Tri::Unknown,
        }
    }

    /// <https://tc39.es/ecma262/#sec-abstract-equality-comparison>
    fn try_abstract_equality_comparison<'b>(
        left_expr: &'b Expression<'a>,
        right_expr: &'b Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Tri {
        let left = Ty::from(left_expr);
        let right = Ty::from(right_expr);
        if left != Ty::Undetermined && right != Ty::Undetermined {
            if left == right {
                return Self::try_strict_equality_comparison(left_expr, right_expr, ctx);
            }
            if matches!((left, right), (Ty::Null, Ty::Void) | (Ty::Void, Ty::Null)) {
                return Tri::True;
            }

            if matches!((left, right), (Ty::Number, Ty::Str)) || matches!(right, Ty::Boolean) {
                let right_number = ctx.get_side_free_number_value(right_expr);

                if let Some(NumberValue::Number(num)) = right_number {
                    let number_literal_expr = ctx.ast.expression_numeric_literal(
                        right_expr.span(),
                        num,
                        num.to_string(),
                        if num.fract() == 0.0 { NumberBase::Decimal } else { NumberBase::Float },
                    );

                    return Self::try_abstract_equality_comparison(
                        left_expr,
                        &number_literal_expr,
                        ctx,
                    );
                }

                return Tri::Unknown;
            }

            if matches!((left, right), (Ty::Str, Ty::Number)) || matches!(left, Ty::Boolean) {
                let left_number = ctx.get_side_free_number_value(left_expr);

                if let Some(NumberValue::Number(num)) = left_number {
                    let number_literal_expr = ctx.ast.expression_numeric_literal(
                        left_expr.span(),
                        num,
                        num.to_string(),
                        if num.fract() == 0.0 { NumberBase::Decimal } else { NumberBase::Float },
                    );

                    return Self::try_abstract_equality_comparison(
                        &number_literal_expr,
                        right_expr,
                        ctx,
                    );
                }

                return Tri::Unknown;
            }

            if matches!(left, Ty::BigInt) || matches!(right, Ty::BigInt) {
                let left_bigint = ctx.get_side_free_bigint_value(left_expr);
                let right_bigint = ctx.get_side_free_bigint_value(right_expr);

                if let (Some(l_big), Some(r_big)) = (left_bigint, right_bigint) {
                    return Tri::from(l_big.eq(&r_big));
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
        left_expr: &'b Expression<'a>,
        right_expr: &'b Expression<'a>,
        will_negative: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Tri {
        let left = Ty::from(left_expr);
        let right = Ty::from(right_expr);

        // First, check for a string comparison.
        if left == Ty::Str && right == Ty::Str {
            let left_string = ctx.get_side_free_string_value(left_expr);
            let right_string = ctx.get_side_free_string_value(right_expr);
            if let (Some(left_string), Some(right_string)) = (left_string, right_string) {
                // In JS, browsers parse \v differently. So do not compare strings if one contains \v.
                if left_string.contains('\u{000B}') || right_string.contains('\u{000B}') {
                    return Tri::Unknown;
                }

                return Tri::from(left_string.cmp(&right_string) == Ordering::Less);
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

        let left_bigint = ctx.get_side_free_bigint_value(left_expr);
        let right_bigint = ctx.get_side_free_bigint_value(right_expr);

        let left_num = ctx.get_side_free_number_value(left_expr);
        let right_num = ctx.get_side_free_number_value(right_expr);

        match (left_bigint, right_bigint, left_num, right_num) {
            // Next, try to evaluate based on the value of the node. Try comparing as BigInts first.
            (Some(l_big), Some(r_big), _, _) => {
                return Tri::from(l_big < r_big);
            }
            // try comparing as Numbers.
            (_, _, Some(l_num), Some(r_num)) => match (l_num, r_num) {
                (NumberValue::NaN, _) | (_, NumberValue::NaN) => {
                    return Tri::from(will_negative);
                }
                (NumberValue::Number(l), NumberValue::Number(r)) => return Tri::from(l < r),
                _ => {}
            },
            // Finally, try comparisons between BigInt and Number.
            (Some(l_big), _, _, Some(r_num)) => {
                return Self::bigint_less_than_number(&l_big, &r_num, Tri::False, will_negative);
            }
            (_, Some(r_big), Some(l_num), _) => {
                return Self::bigint_less_than_number(&r_big, &l_num, Tri::True, will_negative);
            }
            _ => {}
        }

        Tri::Unknown
    }

    /// ported from [closure compiler](https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeFoldConstants.java#L1250)
    #[allow(clippy::cast_possible_truncation)]
    pub fn bigint_less_than_number(
        bigint_value: &BigInt,
        number_value: &NumberValue,
        invert: Tri,
        will_negative: bool,
    ) -> Tri {
        // if invert is false, then the number is on the right in tryAbstractRelationalComparison
        // if it's true, then the number is on the left
        match number_value {
            NumberValue::NaN => Tri::from(will_negative),
            NumberValue::PositiveInfinity => Tri::True.xor(invert),
            NumberValue::NegativeInfinity => Tri::False.xor(invert),
            NumberValue::Number(num) => {
                if let Some(Ordering::Equal | Ordering::Greater) =
                    num.abs().partial_cmp(&2_f64.powi(53))
                {
                    Tri::Unknown
                } else {
                    let number_as_bigint = BigInt::from(*num as i64);

                    match bigint_value.cmp(&number_as_bigint) {
                        Ordering::Less => Tri::True.xor(invert),
                        Ordering::Greater => Tri::False.xor(invert),
                        Ordering::Equal => {
                            if is_exact_int64(*num) {
                                Tri::False
                            } else {
                                Tri::from(num.is_sign_positive()).xor(invert)
                            }
                        }
                    }
                }
            }
        }
    }

    /// <https://tc39.es/ecma262/#sec-strict-equality-comparison>
    fn try_strict_equality_comparison<'b>(
        left_expr: &'b Expression<'a>,
        right_expr: &'b Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
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
                    let left_number = ctx.get_side_free_number_value(left_expr);
                    let right_number = ctx.get_side_free_number_value(right_expr);

                    if let (Some(l_num), Some(r_num)) = (left_number, right_number) {
                        if l_num.is_nan() || r_num.is_nan() {
                            return Tri::False;
                        }

                        return Tri::from(l_num == r_num);
                    }

                    Tri::Unknown
                }
                Ty::Str => {
                    let left_string = ctx.get_side_free_string_value(left_expr);
                    let right_string = ctx.get_side_free_string_value(right_expr);
                    if let (Some(left_string), Some(right_string)) = (left_string, right_string) {
                        // In JS, browsers parse \v differently. So do not compare strings if one contains \v.
                        if left_string.contains('\u{000B}') || right_string.contains('\u{000B}') {
                            return Tri::Unknown;
                        }

                        return Tri::from(left_string == right_string);
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

    /// ported from [closure-compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/PeepholeFoldConstants.java#L1114-L1162)
    #[allow(clippy::cast_possible_truncation)]
    fn try_fold_shift<'b>(
        &self,
        span: Span,
        op: BinaryOperator,
        left: &'b Expression<'a>,
        right: &'b Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let left_num = ctx.get_side_free_number_value(left);
        let right_num = ctx.get_side_free_number_value(right);

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

            return Some(ctx.ast.expression_numeric_literal(
                span,
                result_val,
                result_val.to_string(),
                NumberBase::Decimal,
            ));
        }

        None
    }

    /// Try to fold a AND / OR node.
    ///
    /// port from [closure-compiler](https://github.com/google/closure-compiler/blob/09094b551915a6487a980a783831cba58b5739d1/src/com/google/javascript/jscomp/PeepholeFoldConstants.java#L587)
    pub fn try_fold_and_or(
        &self,
        logical_expr: &mut LogicalExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let op = logical_expr.operator;
        debug_assert!(matches!(op, LogicalOperator::And | LogicalOperator::Or));

        let left = &logical_expr.left;
        let left_val = ctx.get_boolean_value(left).to_option();

        if let Some(lval) = left_val {
            // Bail `0 && (module.exports = {})` for `cjs-module-lexer`.
            if !lval {
                if let Expression::AssignmentExpression(assign_expr) = &logical_expr.right {
                    if let Some(member_expr) = assign_expr.left.as_member_expression() {
                        if member_expr.is_specific_member_access("module", "exports") {
                            return None;
                        }
                    }
                }
            }

            // (TRUE || x) => TRUE (also, (3 || x) => 3)
            // (FALSE && x) => FALSE
            if if lval { op == LogicalOperator::Or } else { op == LogicalOperator::And } {
                return Some(ctx.ast.move_expression(&mut logical_expr.left));
            } else if !left.may_have_side_effects() {
                let parent = ctx.ancestry.parent();
                // Bail `let o = { f() { assert.ok(this !== o); } }; (true && o.f)(); (true && o.f)``;`
                if parent.is_tagged_template_expression()
                    || matches!(parent, Ancestor::CallExpressionCallee(_))
                {
                    return None;
                }
                // (FALSE || x) => x
                // (TRUE && x) => x
                return Some(ctx.ast.move_expression(&mut logical_expr.right));
            }
            // Left side may have side effects, but we know its boolean value.
            // e.g. true_with_sideeffects || foo() => true_with_sideeffects, foo()
            // or: false_with_sideeffects && foo() => false_with_sideeffects, foo()
            let left = ctx.ast.move_expression(&mut logical_expr.left);
            let right = ctx.ast.move_expression(&mut logical_expr.right);
            let mut vec = ctx.ast.vec_with_capacity(2);
            vec.push(left);
            vec.push(right);
            let sequence_expr = ctx.ast.expression_sequence(logical_expr.span, vec);
            return Some(sequence_expr);
        } else if let Expression::LogicalExpression(left_child) = &mut logical_expr.left {
            if left_child.operator == logical_expr.operator {
                let left_child_right_boolean = ctx.get_boolean_value(&left_child.right).to_option();
                let left_child_op = left_child.operator;
                if let Some(right_boolean) = left_child_right_boolean {
                    if !left_child.right.may_have_side_effects() {
                        // a || false || b => a || b
                        // a && true && b => a && b
                        if !right_boolean && left_child_op == LogicalOperator::Or
                            || right_boolean && left_child_op == LogicalOperator::And
                        {
                            let left = ctx.ast.move_expression(&mut left_child.left);
                            let right = ctx.ast.move_expression(&mut logical_expr.right);
                            let logic_expr = ctx.ast.expression_logical(
                                logical_expr.span,
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

    /// Try to fold a nullish coalesce `foo ?? bar`.
    pub fn try_fold_coalesce(
        &self,
        logical_expr: &mut LogicalExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        debug_assert_eq!(logical_expr.operator, LogicalOperator::Coalesce);
        let left = &logical_expr.left;
        let left_val = ctx.get_known_value_type(left);
        match left_val {
            ValueType::Null | ValueType::Void => {
                Some(if left.may_have_side_effects() {
                    // e.g. `(a(), null) ?? 1` => `(a(), null, 1)`
                    let expressions = ctx.ast.vec_from_iter([
                        ctx.ast.move_expression(&mut logical_expr.left),
                        ctx.ast.move_expression(&mut logical_expr.right),
                    ]);
                    ctx.ast.expression_sequence(SPAN, expressions)
                } else {
                    // nullish condition => this expression evaluates to the right side.
                    ctx.ast.move_expression(&mut logical_expr.right)
                })
            }
            ValueType::Number
            | ValueType::Bigint
            | ValueType::String
            | ValueType::Boolean
            | ValueType::Object => {
                // non-nullish condition => this expression evaluates to the left side.
                Some(ctx.ast.move_expression(&mut logical_expr.left))
            }
            ValueType::Undetermined => None,
        }
    }
}
