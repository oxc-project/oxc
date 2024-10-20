use std::cmp::Ordering;

use num_bigint::BigInt;

use oxc_ast::ast::*;
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, ValueType},
    side_effects::MayHaveSideEffects,
};
use oxc_span::{GetSpan, Span, SPAN};
use oxc_syntax::{
    number::{NumberBase, ToJsString},
    operator::{BinaryOperator, LogicalOperator, UnaryOperator},
};
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

use crate::{
    node_util::{is_exact_int64, Ctx},
    tri::Tri,
    CompressorPass,
};

/// Constant Folding
///
/// <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeFoldConstants.java>
pub struct PeepholeFoldConstants {
    changed: bool,
}

impl<'a> CompressorPass<'a> for PeepholeFoldConstants {
    fn changed(&self) -> bool {
        self.changed
    }

    fn build(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.changed = false;
        oxc_traverse::walk_program(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for PeepholeFoldConstants {
    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = Ctx(ctx);
        if let Some(folded_expr) = match expr {
            Expression::CallExpression(e) => {
                Self::try_fold_useless_object_dot_define_properties_call(e, ctx)
            }
            Expression::NewExpression(e) => Self::try_fold_ctor_cal(e, ctx),
            // TODO
            // return tryFoldSpread(subtree);
            Expression::ArrayExpression(e) => Self::try_flatten_array_expression(e, ctx),
            Expression::ObjectExpression(e) => Self::try_flatten_object_expression(e, ctx),
            Expression::BinaryExpression(e) => Self::try_fold_binary_expression(e, ctx),
            Expression::UnaryExpression(e) => {
                ctx.eval_unary_expression(e).map(|v| ctx.value_to_expr(e.span, v))
            }
            // TODO: return tryFoldGetProp(subtree);
            Expression::LogicalExpression(e) => Self::try_fold_logical_expression(e, ctx),
            // TODO: tryFoldGetElem
            // TODO: tryFoldAssign
            _ => None,
        } {
            *expr = folded_expr;
            self.changed = true;
        };
    }
}

impl<'a, 'b> PeepholeFoldConstants {
    pub fn new() -> Self {
        Self { changed: false }
    }

    fn try_fold_useless_object_dot_define_properties_call(
        _call_expr: &mut CallExpression<'a>,
        _ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        None
    }

    fn try_fold_ctor_cal(
        _new_expr: &mut NewExpression<'a>,
        _ctx: Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        None
    }

    fn try_flatten_array_expression(
        _new_expr: &mut ArrayExpression<'a>,
        _ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        None
    }

    fn try_flatten_object_expression(
        _new_expr: &mut ObjectExpression<'a>,
        _ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        None
    }

    fn try_fold_logical_expression(
        logical_expr: &mut LogicalExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        match logical_expr.operator {
            LogicalOperator::And | LogicalOperator::Or => Self::try_fold_and_or(logical_expr, ctx),
            LogicalOperator::Coalesce => Self::try_fold_coalesce(logical_expr, ctx),
        }
    }

    /// Try to fold a AND / OR node.
    ///
    /// port from [closure-compiler](https://github.com/google/closure-compiler/blob/09094b551915a6487a980a783831cba58b5739d1/src/com/google/javascript/jscomp/PeepholeFoldConstants.java#L587)
    pub fn try_fold_and_or(
        logical_expr: &mut LogicalExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        let op = logical_expr.operator;
        debug_assert!(matches!(op, LogicalOperator::And | LogicalOperator::Or));

        let left = &logical_expr.left;
        let left_val = ctx.get_boolean_value(left);

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
                let left_child_right_boolean = ctx.get_boolean_value(&left_child.right);
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
        logical_expr: &mut LogicalExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        debug_assert_eq!(logical_expr.operator, LogicalOperator::Coalesce);
        let left = &logical_expr.left;
        let left_val = ValueType::from(left);
        match left_val {
            ValueType::Null | ValueType::Undefined => {
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
            | ValueType::BigInt
            | ValueType::String
            | ValueType::Boolean
            | ValueType::Object => {
                // non-nullish condition => this expression evaluates to the left side.
                Some(ctx.ast.move_expression(&mut logical_expr.left))
            }
            ValueType::Undetermined => None,
        }
    }

    fn try_fold_binary_expression(
        e: &mut BinaryExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        // TODO: tryReduceOperandsForOp
        match e.operator {
            BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::ShiftRightZeroFill
            | BinaryOperator::Addition
            | BinaryOperator::Subtraction
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::Multiplication
            | BinaryOperator::Exponential => {
                ctx.eval_binary_expression(e).map(|v| ctx.value_to_expr(e.span, v))
            }
            BinaryOperator::BitwiseAnd | BinaryOperator::BitwiseOR | BinaryOperator::BitwiseXOR => {
                // TODO:
                // self.try_fold_arithmetic_op(e.span, &e.left, &e.right, ctx)
                // if (result != subtree) {
                // return result;
                // }
                // return tryFoldLeftChildOp(subtree, left, right);
                None
            }
            op if op.is_equality() || op.is_compare() => {
                Self::try_fold_comparison(e.span, e.operator, &e.left, &e.right, ctx)
            }
            _ => None,
        }
    }

    fn try_fold_comparison(
        span: Span,
        op: BinaryOperator,
        left: &'b Expression<'a>,
        right: &'b Expression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        let value = match Self::evaluate_comparison(op, left, right, ctx) {
            Tri::True => true,
            Tri::False => false,
            Tri::Unknown => return None,
        };
        Some(ctx.ast.expression_boolean_literal(span, value))
    }

    fn evaluate_comparison(
        op: BinaryOperator,
        left: &'b Expression<'a>,
        right: &'b Expression<'a>,
        ctx: Ctx<'a, 'b>,
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
    fn try_abstract_equality_comparison(
        left_expr: &'b Expression<'a>,
        right_expr: &'b Expression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Tri {
        let left = ValueType::from(left_expr);
        let right = ValueType::from(right_expr);
        if left != ValueType::Undetermined && right != ValueType::Undetermined {
            if left == right {
                return Self::try_strict_equality_comparison(left_expr, right_expr, ctx);
            }
            if matches!(
                (left, right),
                (ValueType::Null, ValueType::Undefined) | (ValueType::Undefined, ValueType::Null)
            ) {
                return Tri::True;
            }

            if matches!((left, right), (ValueType::Number, ValueType::String))
                || matches!(right, ValueType::Boolean)
            {
                let right_number = ctx.get_side_free_number_value(right_expr);

                if let Some(num) = right_number {
                    let number_literal_expr = ctx.ast.expression_numeric_literal(
                        right_expr.span(),
                        num,
                        num.to_js_string(),
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

            if matches!((left, right), (ValueType::String, ValueType::Number))
                || matches!(left, ValueType::Boolean)
            {
                let left_number = ctx.get_side_free_number_value(left_expr);

                if let Some(num) = left_number {
                    let number_literal_expr = ctx.ast.expression_numeric_literal(
                        left_expr.span(),
                        num,
                        num.to_js_string(),
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

            if matches!(left, ValueType::BigInt) || matches!(right, ValueType::BigInt) {
                let left_bigint = ctx.get_side_free_bigint_value(left_expr);
                let right_bigint = ctx.get_side_free_bigint_value(right_expr);

                if let (Some(l_big), Some(r_big)) = (left_bigint, right_bigint) {
                    return Tri::from(l_big.eq(&r_big));
                }
            }

            if matches!(left, ValueType::String | ValueType::Number)
                && matches!(right, ValueType::Object)
            {
                return Tri::Unknown;
            }

            if matches!(left, ValueType::Object)
                && matches!(right, ValueType::String | ValueType::Number)
            {
                return Tri::Unknown;
            }

            return Tri::False;
        }
        Tri::Unknown
    }

    /// <https://tc39.es/ecma262/#sec-abstract-relational-comparison>
    fn try_abstract_relational_comparison(
        left_expr: &'b Expression<'a>,
        right_expr: &'b Expression<'a>,
        will_negative: bool,
        ctx: Ctx<'a, 'b>,
    ) -> Tri {
        let left = ValueType::from(left_expr);
        let right = ValueType::from(right_expr);

        // First, check for a string comparison.
        if left == ValueType::String && right == ValueType::String {
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
            (_, _, Some(l_num), Some(r_num)) => {
                return if l_num.is_nan() || r_num.is_nan() {
                    Tri::from(will_negative)
                } else {
                    Tri::from(l_num < r_num)
                }
            }
            // Finally, try comparisons between BigInt and Number.
            (Some(l_big), _, _, Some(r_num)) => {
                return Self::bigint_less_than_number(&l_big, r_num, Tri::False, will_negative);
            }
            (_, Some(r_big), Some(l_num), _) => {
                return Self::bigint_less_than_number(&r_big, l_num, Tri::True, will_negative);
            }
            _ => {}
        }

        Tri::Unknown
    }

    /// ported from [closure compiler](https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeFoldConstants.java#L1250)
    #[allow(clippy::cast_possible_truncation)]
    pub fn bigint_less_than_number(
        bigint_value: &BigInt,
        number_value: f64,
        invert: Tri,
        will_negative: bool,
    ) -> Tri {
        // if invert is false, then the number is on the right in tryAbstractRelationalComparison
        // if it's true, then the number is on the left
        match number_value {
            v if v.is_nan() => Tri::from(will_negative),
            v if v.is_infinite() && v.is_sign_positive() => Tri::True.xor(invert),
            v if v.is_infinite() && v.is_sign_negative() => Tri::False.xor(invert),
            num => {
                if let Some(Ordering::Equal | Ordering::Greater) =
                    num.abs().partial_cmp(&2_f64.powi(53))
                {
                    Tri::Unknown
                } else {
                    let number_as_bigint = BigInt::from(num as i64);

                    match bigint_value.cmp(&number_as_bigint) {
                        Ordering::Less => Tri::True.xor(invert),
                        Ordering::Greater => Tri::False.xor(invert),
                        Ordering::Equal => {
                            if is_exact_int64(num) {
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
    #[expect(clippy::float_cmp)]
    fn try_strict_equality_comparison(
        left_expr: &'b Expression<'a>,
        right_expr: &'b Expression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Tri {
        let left = ValueType::from(left_expr);
        let right = ValueType::from(right_expr);
        if left != ValueType::Undetermined && right != ValueType::Undetermined {
            // Strict equality can only be true for values of the same type.
            if left != right {
                return Tri::False;
            }
            return match left {
                ValueType::Number => {
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
                ValueType::String => {
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
                ValueType::Undefined | ValueType::Null => Tri::True,
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
}

/// <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeFoldConstantsTest.java>
#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    static MAX_SAFE_FLOAT: f64 = 9_007_199_254_740_991_f64;
    static NEG_MAX_SAFE_FLOAT: f64 = -9_007_199_254_740_991_f64;

    static MAX_SAFE_INT: i64 = 9_007_199_254_740_991_i64;
    static NEG_MAX_SAFE_INT: i64 = -9_007_199_254_740_991_i64;

    use crate::tester;

    fn test(source_text: &str, expected: &str) {
        let allocator = Allocator::default();
        let mut pass = super::PeepholeFoldConstants::new();
        tester::test(&allocator, source_text, expected, &mut pass);
    }

    fn test_same(source_text: &str) {
        test(source_text, source_text);
    }

    #[test]
    fn undefined_comparison1() {
        test("undefined == undefined", "true");
        test("undefined == null", "true");
        test("undefined == void 0", "true");

        test("undefined == 0", "false");
        test("undefined == 1", "false");
        test("undefined == 'hi'", "false");
        test("undefined == true", "false");
        test("undefined == false", "false");

        test("undefined === undefined", "true");
        test("undefined === null", "false");
        test("undefined === void 0", "true");

        test_same("undefined == this");
        test_same("undefined == x");

        test("undefined != undefined", "false");
        test("undefined != null", "false");
        test("undefined != void 0", "false");

        test("undefined != 0", "true");
        test("undefined != 1", "true");
        test("undefined != 'hi'", "true");
        test("undefined != true", "true");
        test("undefined != false", "true");

        test("undefined !== undefined", "false");
        test("undefined !== void 0", "false");
        test("undefined !== null", "true");

        test_same("undefined != this");
        test_same("undefined != x");

        test("undefined < undefined", "false");
        test("undefined > undefined", "false");
        test("undefined >= undefined", "false");
        test("undefined <= undefined", "false");

        test("0 < undefined", "false");
        test("true > undefined", "false");
        test("'hi' >= undefined", "false");
        test("null <= undefined", "false");

        test("undefined < 0", "false");
        test("undefined > true", "false");
        test("undefined >= 'hi'", "false");
        test("undefined <= null", "false");

        test("null == undefined", "true");
        test("0 == undefined", "false");
        test("1 == undefined", "false");
        test("'hi' == undefined", "false");
        test("true == undefined", "false");
        test("false == undefined", "false");
        test("null === undefined", "false");
        test("void 0 === undefined", "true");

        test("undefined == NaN", "false");
        test("NaN == undefined", "false");
        test("undefined == Infinity", "false");
        test("Infinity == undefined", "false");
        test("undefined == -Infinity", "false");
        test("-Infinity == undefined", "false");
        test("({}) == undefined", "false");
        test("undefined == ({})", "false");
        test("([]) == undefined", "false");
        test("undefined == ([])", "false");
        test("(/a/g) == undefined", "false");
        test("undefined == (/a/g)", "false");
        test("(function(){}) == undefined", "false");
        test("undefined == (function(){})", "false");

        test("undefined != NaN", "true");
        test("NaN != undefined", "true");
        test("undefined != Infinity", "true");
        test("Infinity != undefined", "true");
        test("undefined != -Infinity", "true");
        test("-Infinity != undefined", "true");
        test("({}) != undefined", "true");
        test("undefined != ({})", "true");
        test("([]) != undefined", "true");
        test("undefined != ([])", "true");
        test("(/a/g) != undefined", "true");
        test("undefined != (/a/g)", "true");
        test("(function(){}) != undefined", "true");
        test("undefined != (function(){})", "true");

        test_same("this == undefined");
        test_same("x == undefined");
    }

    #[test]
    fn test_undefined_comparison2() {
        test("\"123\" !== void 0", "true");
        test("\"123\" === void 0", "false");

        test("void 0 !== \"123\"", "true");
        test("void 0 === \"123\"", "false");
    }

    #[test]
    fn test_undefined_comparison3() {
        test("\"123\" !== undefined", "true");
        test("\"123\" === undefined", "false");

        test("undefined !== \"123\"", "true");
        test("undefined === \"123\"", "false");
    }

    #[test]
    fn test_null_comparison1() {
        test("null == undefined", "true");
        test("null == null", "true");
        test("null == void 0", "true");

        test("null == 0", "false");
        test("null == 1", "false");
        test("null == 0n", "false");
        test("null == 1n", "false");
        test("null == 'hi'", "false");
        test("null == true", "false");
        test("null == false", "false");

        test("null === undefined", "false");
        test("null === null", "true");
        test("null === void 0", "false");
        test_same("null===x");

        test_same("null==this");
        test_same("null==x");

        test("null != undefined", "false");
        test("null != null", "false");
        test("null != void 0", "false");

        test("null != 0", "true");
        test("null != 1", "true");
        // test("null != 0n", "true");
        // test("null != 1n", "true");
        test("null != 'hi'", "true");
        test("null != true", "true");
        test("null != false", "true");

        test("null !== undefined", "true");
        test("null !== void 0", "true");
        test("null !== null", "false");

        test_same("null!=this");
        test_same("null!=x");

        test("null < null", "false");
        test("null > null", "false");
        test("null >= null", "true");
        test("null <= null", "true");

        test("0 < null", "false");
        test("0 > null", "false");
        test("0 >= null", "true");
        test("0n < null", "false");
        test("0n > null", "false");
        test("0n >= null", "true");
        test("true > null", "true");
        test("'hi' < null", "false");
        test("'hi' >= null", "false");
        test("null <= null", "true");

        test("null < 0", "false");
        test("null < 0n", "false");
        test("null > true", "false");
        test("null < 'hi'", "false");
        test("null >= 'hi'", "false");
        test("null <= null", "true");

        test("null == null", "true");
        test("0 == null", "false");
        test("1 == null", "false");
        test("'hi' == null", "false");
        test("true == null", "false");
        test("false == null", "false");
        test("null === null", "true");
        test("void 0 === null", "false");

        test("null == NaN", "false");
        test("NaN == null", "false");
        test("null == Infinity", "false");
        test("Infinity == null", "false");
        test("null == -Infinity", "false");
        test("-Infinity == null", "false");
        test("({}) == null", "false");
        test("null == ({})", "false");
        test("([]) == null", "false");
        test("null == ([])", "false");
        test("(/a/g) == null", "false");
        test("null == (/a/g)", "false");
        test("(function(){}) == null", "false");
        test("null == (function(){})", "false");

        test("null != NaN", "true");
        test("NaN != null", "true");
        test("null != Infinity", "true");
        test("Infinity != null", "true");
        test("null != -Infinity", "true");
        test("-Infinity != null", "true");
        test("({}) != null", "true");
        test("null != ({})", "true");
        test("([]) != null", "true");
        test("null != ([])", "true");
        test("(/a/g) != null", "true");
        test("null != (/a/g)", "true");
        test("(function(){}) != null", "true");
        test("null != (function(){})", "true");

        test_same("({a:f()})==null");
        test_same("null=={a:f()}");
        test_same("[f()]==null");
        test_same("null==[f()]");

        test_same("this==null");
        test_same("x==null");
    }

    #[test]
    fn test_boolean_boolean_comparison() {
        test_same("!x==!y");
        test_same("!x<!y");
        test_same("!x!==!y");

        test_same("!x==!x"); // foldable
        test_same("!x<!x"); // foldable
        test_same("!x!==!x"); // foldable
    }

    #[test]
    fn test_boolean_number_comparison() {
        test_same("!x==+y");
        test_same("!x<=+y");
        test("!x !== +y", "true");
    }

    #[test]
    fn test_number_boolean_comparison() {
        test_same("+x==!y");
        test_same("+x<=!y");
        test("+x === !y", "false");
    }

    #[test]
    fn test_boolean_string_comparison() {
        test_same("!x==''+y");
        test_same("!x<=''+y");
        test("!x !== '' + y", "true");
    }

    #[test]
    fn test_string_boolean_comparison() {
        test_same("''+x==!y");
        test_same("''+x<=!y");
        test("'' + x === !y", "false");
    }

    #[test]
    fn test_string_string_comparison() {
        test("'a' < 'b'", "true");
        test("'a' <= 'b'", "true");
        test("'a' > 'b'", "false");
        test("'a' >= 'b'", "false");
        test("+'a' < +'b'", "false");
        test_same("typeof a < 'a'");
        test_same("'a' >= typeof a");
        test("typeof a < typeof a", "false");
        test("typeof a >= typeof a", "true");
        test("typeof 3 > typeof 4", "false");
        test("typeof function() {} < typeof function() {}", "false");
        test("'a' == 'a'", "true");
        test("'b' != 'a'", "true");
        test_same("'undefined' == typeof a");
        test_same("typeof a != 'number'");
        test_same("'undefined' == typeof a");
        test_same("'undefined' == typeof a");
        test("typeof a == typeof a", "true");
        test("'a' === 'a'", "true");
        test("'b' !== 'a'", "true");
        test("typeof a === typeof a", "true");
        test("typeof a !== typeof a", "false");
        test_same("'' + x <= '' + y");
        test_same("'' + x != '' + y");
        test_same("'' + x === '' + y");

        test_same("'' + x <= '' + x"); // potentially foldable
        test_same("'' + x != '' + x"); // potentially foldable
        test_same("'' + x === '' + x"); // potentially foldable
    }

    #[test]
    fn test_number_string_comparison() {
        test("1 < '2'", "true");
        test("2 > '1'", "true");
        test("123 > '34'", "true");
        test("NaN >= 'NaN'", "false");
        test("1 == '2'", "false");
        test("1 != '1'", "false");
        test("NaN == 'NaN'", "false");
        test("1 === '1'", "false");
        test("1 !== '1'", "true");
        test_same("+x>''+y");
        test_same("+x==''+y");
        test("+x !== '' + y", "true");
    }

    #[test]
    fn test_string_number_comparison() {
        test("'1' < 2", "true");
        test("'2' > 1", "true");
        test("'123' > 34", "true");
        test("'NaN' < NaN", "false");
        test("'1' == 2", "false");
        test("'1' != 1", "false");
        test("'NaN' == NaN", "false");
        test("'1' === 1", "false");
        test("'1' !== 1", "true");
        test_same("''+x<+y");
        test_same("''+x==+y");
        test("'' + x === +y", "false");
    }

    #[test]
    fn test_bigint_number_comparison() {
        test("1n < 2", "true");
        test("1n > 2", "false");
        test("1n == 1", "true");
        test("1n == 2", "false");

        // comparing with decimals is allowed
        test("1n < 1.1", "true");
        test("1n < 1.9", "true");
        test("1n < 0.9", "false");
        test("-1n < -1.1", "false");
        test("-1n < -1.9", "false");
        test("-1n < -0.9", "true");
        test("1n > 1.1", "false");
        test("1n > 0.9", "true");
        test("-1n > -1.1", "true");
        test("-1n > -0.9", "false");

        // Don't fold unsafely large numbers because there might be floating-point error
        test(&format!("0n > {MAX_SAFE_INT}"), "false");
        test(&format!("0n < {MAX_SAFE_INT}"), "true");
        test(&format!("0n > {NEG_MAX_SAFE_INT}"), "true");
        test(&format!("0n < {NEG_MAX_SAFE_INT}"), "false");
        test(&format!("0n > {MAX_SAFE_FLOAT}"), "false");
        test(&format!("0n < {MAX_SAFE_FLOAT}"), "true");
        test(&format!("0n > {NEG_MAX_SAFE_FLOAT}"), "true");
        test(&format!("0n < {NEG_MAX_SAFE_FLOAT}"), "false");

        // comparing with Infinity is allowed
        test("1n < Infinity", "true");
        test("1n > Infinity", "false");
        test("1n < -Infinity", "false");
        test("1n > -Infinity", "true");

        // null is interpreted as 0 when comparing with bigint
        test("1n < null", "false");
        test("1n > null", "true");
    }

    #[test]
    fn test_bigint_string_comparison() {
        test("1n < '2'", "true");
        test("2n > '1'", "true");
        test("123n > '34'", "true");
        test("1n == '1'", "true");
        test("1n == '2'", "false");
        test("1n != '1'", "false");
        test("1n === '1'", "false");
        test("1n !== '1'", "true");
    }

    #[test]
    fn test_string_bigint_comparison() {
        test("'1' < 2n", "true");
        test("'2' > 1n", "true");
        test("'123' > 34n", "true");
        test("'1' == 1n", "true");
        test("'1' == 2n", "false");
        test("'1' != 1n", "false");
        test("'1' === 1n", "false");
        test("'1' !== 1n", "true");
    }

    #[test]
    fn test_nan_comparison() {
        test("NaN < 1", "false");
        test("NaN <= 1", "false");
        test("NaN > 1", "false");
        test("NaN >= 1", "false");
        test("NaN < 1n", "false");
        test("NaN <= 1n", "false");
        test("NaN > 1n", "false");
        test("NaN >= 1n", "false");

        test("NaN < NaN", "false");
        test("NaN >= NaN", "false");
        test("NaN == NaN", "false");
        test("NaN === NaN", "false");

        test("NaN < null", "false");
        test("null >= NaN", "false");
        test("NaN == null", "false");
        test("null != NaN", "true");
        test("null === NaN", "false");

        test("NaN < undefined", "false");
        test("undefined >= NaN", "false");
        test("NaN == undefined", "false");
        test("undefined != NaN", "true");
        test("undefined === NaN", "false");

        test_same("NaN<x");
        test_same("x>=NaN");
        test_same("NaN==x");
        test_same("x!=NaN");
        test("NaN === x", "false");
        test("x !== NaN", "true");
        test_same("NaN==foo()");
    }

    #[test]
    fn js_typeof() {
        test("x = typeof 1", "x = \"number\"");
        test("x = typeof 'foo'", "x = \"string\"");
        test("x = typeof true", "x = \"boolean\"");
        test("x = typeof false", "x = \"boolean\"");
        test("x = typeof null", "x = \"object\"");
        test("x = typeof undefined", "x = \"undefined\"");
        test("x = typeof void 0", "x = \"undefined\"");
        test("x = typeof []", "x = \"object\"");
        test("x = typeof [1]", "x = \"object\"");
        test("x = typeof [1,[]]", "x = \"object\"");
        test("x = typeof {}", "x = \"object\"");
        test("x = typeof function() {}", "x = 'function'");

        test_same("x = typeof[1,[foo()]]");
        test_same("x = typeof{bathwater:baby()}");
    }

    #[test]
    fn unary_ops() {
        // TODO: need to port
        // These cases are handled by PeepholeRemoveDeadCode in closure-compiler.
        // test_same("!foo()");
        // test_same("~foo()");
        // test_same("-foo()");

        // These cases are handled here.
        test("a=!true", "a=false");
        test("a=!10", "a=false");
        test("a=!false", "a=true");
        test_same("a=!foo()");
        // test("a=-0", "a=-0.0");
        // test("a=-(0)", "a=-0.0");
        test_same("a=-Infinity");
        test("a=-NaN", "a=NaN");
        test_same("a=-foo()");
        test("a=~~0", "a=0");
        test("a=~~10", "a=10");
        test("a=~-7", "a=6");
        test_same("a=~~foo()");

        // test("a=+true", "a=1");
        test("a=+10", "a=10");
        // test("a=+false", "a=0");
        test_same("a=+foo()");
        test_same("a=+f");
        // test("a=+(f?true:false)", "a=+(f?1:0)");
        test("a=+0", "a=0");
        test("a=+Infinity", "a=Infinity");
        test("a=+NaN", "a=NaN");
        test("a=+-7", "a=-7");
        // test("a=+.5", "a=.5");

        test("a=~0xffffffff", "a=0");
        test("a=~~0xffffffff", "a=-1");
        // test_same("a=~.5", PeepholeFoldConstants.FRACTIONAL_BITWISE_OPERAND);
    }

    #[test]
    fn unary_with_big_int() {
        test("-(1n)", "-1n");
        test("- -1n", "1n");
        test("!1n", "false");
        test("~0n", "-1n");

        test("~-1n", "0n");
        test("~~1n", "1n");

        test("~0x3n", "-4n");
        test("~0b11n", "-4n");
    }

    #[test]
    fn test_unary_ops_string_compare() {
        test_same("a = -1");
        test("a = ~0", "a = -1");
        test("a = ~1", "a = -2");
        test("a = ~101", "a = -102");

        test("a = ~1.1", "a = -2");
        test("a = ~0x3", "a = -4"); // Hexadecimal number
        test("a = ~9", "a = -10"); // Despite `-10` is longer than `~9`, the compiler still folds it.
        test_same("a = ~b");
        test("a = ~NaN", "a = -1");
        test("a = ~-Infinity", "a = -1");
        test("x = ~2147483658.0", "x = 2147483637");
        test("x = ~-2147483658", "x = -2147483639");
    }

    #[test]
    fn test_fold_logical_op() {
        test("x = true && x", "x = x");
        test("x = [foo()] && x", "x = ([foo()],x)");

        test("x = false && x", "x = false");
        test("x = true || x", "x = true");
        test("x = false || x", "x = x");
        test("x = 0 && x", "x = 0");
        test("x = 3 || x", "x = 3");
        test("x = 0n && x", "x = 0n");
        test("x = 3n || x", "x = 3n");
        test("x = false || 0", "x = 0");

        // unfoldable, because the right-side may be the result
        test("a = x && true", "a=x && true");
        test("a = x && false", "a=x && false");
        test("a = x || 3", "a=x || 3");
        test("a = x || false", "a=x || false");
        test("a = b ? c : x || false", "a=b ? c:x || false");
        test("a = b ? x || false : c", "a=b ? x || false:c");
        test("a = b ? c : x && true", "a=b ? c:x && true");
        test("a = b ? x && true : c", "a=b ? x && true:c");

        // folded, but not here.
        test_same("a = x || false ? b : c");
        test_same("a = x && true ? b : c");

        test("x = foo() || true || bar()", "x = foo() || true");
        test("x = foo() || true && bar()", "x = foo() || bar()");
        test("x = foo() || false && bar()", "x = foo() || false");
        test("x = foo() && false && bar()", "x = foo() && false");
        test("x = foo() && false || bar()", "x = (foo() && false,bar())");
        test("x = foo() || false || bar()", "x = foo() || bar()");
        test("x = foo() && true && bar()", "x = foo() && bar()");
        test("x = foo() || true || bar()", "x = foo() || true");
        test("x = foo() && false && bar()", "x = foo() && false");
        test("x = foo() && 0 && bar()", "x = foo() && 0");
        test("x = foo() && 1 && bar()", "x = foo() && bar()");
        test("x = foo() || 0 || bar()", "x = foo() || bar()");
        test("x = foo() || 1 || bar()", "x = foo() || 1");
        test("x = foo() && 0n && bar()", "x = foo() && 0n");
        test("x = foo() && 1n && bar()", "x = foo() && bar()");
        test("x = foo() || 0n || bar()", "x = foo() || bar()");
        test("x = foo() || 1n || bar()", "x = foo() || 1n");
        test_same("x = foo() || bar() || baz()");
        test_same("x = foo() && bar() && baz()");

        test("0 || b()", "b()");
        test("1 && b()", "b()");
        test("a() && (1 && b())", "a() && b()");
        test("(a() && 1) && b()", "a() && b()");

        test("(x || '') || y;", "x || y");
        test("false || (x || '');", "x || ''");
        test("(x && 1) && y;", "x && y");
        test("true && (x && 1);", "x && 1");

        // Really not foldable, because it would change the type of the
        // expression if foo() returns something truthy but not true.
        // Cf. FoldConstants.tryFoldAndOr().
        // An example would be if foo() is 1 (truthy) and bar() is 0 (falsey):
        // (1 && true) || 0 == true
        // 1 || 0 == 1, but true =/= 1
        test_same("x = foo() && true || bar()");
        test_same("foo() && true || bar()");
    }

    #[test]
    fn test_fold_logical_op2() {
        test("x = function(){} && x", "x=x");
        test("x = true && function(){}", "x=function(){}");
        test("x = [(function(){alert(x)})()] && x", "x=([function(){alert(x)}()],x)");
    }

    #[test]
    fn test_fold_nullish_coalesce() {
        // fold if left is null/undefined
        test("null ?? 1", "1");
        test("undefined ?? false", "false");
        test("(a(), null) ?? 1", "(a(), null, 1)");

        test("x = [foo()] ?? x", "x = [foo()]");

        // short circuit on all non nullish LHS
        test("x = false ?? x", "x = false");
        test("x = true ?? x", "x = true");
        test("x = 0 ?? x", "x = 0");
        test("x = 3 ?? x", "x = 3");

        // unfoldable, because the right-side may be the result
        test_same("a = x ?? true");
        test_same("a = x ?? false");
        test_same("a = x ?? 3");
        test_same("a = b ? c : x ?? false");
        test_same("a = b ? x ?? false : c");

        // folded, but not here.
        test_same("a = x ?? false ? b : c");
        test_same("a = x ?? true ? b : c");

        test_same("x = foo() ?? true ?? bar()");
        test("x = foo() ?? (true && bar())", "x = foo() ?? bar()");
        test_same("x = (foo() || false) ?? bar()");

        test("a() ?? (1 ?? b())", "a() ?? 1");
        test("(a() ?? 1) ?? b()", "a() ?? 1 ?? b()");
    }

    #[test]
    fn test_fold_void() {
        test_same("void 0");
        test("void 1", "void 0");
        test("void x", "void 0");
        test_same("void x()");
    }

    #[test]
    fn test_fold_bit_shift() {
        test("x = 1 << 0", "x=1");
        test("x = -1 << 0", "x=-1");
        test("x = 1 << 1", "x=2");
        test("x = 3 << 1", "x=6");
        test("x = 1 << 8", "x=256");

        test("x = 1 >> 0", "x=1");
        test("x = -1 >> 0", "x=-1");
        test("x = 1 >> 1", "x=0");
        test("x = 2 >> 1", "x=1");
        test("x = 5 >> 1", "x=2");
        test("x = 127 >> 3", "x=15");
        test("x = 3 >> 1", "x=1");
        test("x = 3 >> 2", "x=0");
        test("x = 10 >> 1", "x=5");
        test("x = 10 >> 2", "x=2");
        test("x = 10 >> 5", "x=0");

        test("x = 10 >>> 1", "x=5");
        test("x = 10 >>> 2", "x=2");
        test("x = 10 >>> 5", "x=0");
        test("x = -1 >>> 1", "x=2147483647"); // 0x7fffffff
        test("x = -1 >>> 0", "x=4294967295"); // 0xffffffff
        test("x = -2 >>> 0", "x=4294967294"); // 0xfffffffe
        test("x = 0x90000000 >>> 28", "x=9");

        test("x = 0xffffffff << 0", "x=-1");
        test("x = 0xffffffff << 4", "x=-16");
        test("1 << 32", "1<<32");
        test("1 << -1", "1<<-1");
        test("1 >> 32", "1>>32");

        // Regression on #6161, ported from <https://github.com/tc39/test262/blob/main/test/language/expressions/unsigned-right-shift/S9.6_A2.2.js>.
        test("-2147483647 >>> 0", "2147483649");
        test("-2147483648 >>> 0", "2147483648");
        test("-2147483649 >>> 0", "2147483647");
        test("-4294967295 >>> 0", "1");
        test("-4294967296 >>> 0", "0");
        test("-4294967297 >>> 0", "4294967295");
        test("4294967295 >>> 0", "4294967295");
        test("4294967296 >>> 0", "0");
        test("4294967297 >>> 0", "1");
        test("8589934591 >>> 0", "4294967295");
        test("8589934592 >>> 0", "0");
        test("8589934593 >>> 0", "1");
    }

    #[test]
    fn test_fold_arithmetic() {
        test("x = 10 + 20", "x = 30");
        test("x = 2 / 4", "x = 0.5");
        test("x = 2.25 * 3", "x = 6.75");
        test_same("z = x * y");
        test_same("x = y * 5");
        test("x = 1 / 0", "x = Infinity");
        test("x = 3 % 2", "x = 1");
        test("x = 3 % -2", "x = 1");
        test("x = -1 % 3", "x = -1");
        test("x = 1 % 0", "x = NaN");

        test("x = 2 ** 3", "x = 8");
        test("x = 2 ** -3", "x = 0.125");
        // FIXME
        // test_same("x = 2 ** 55"); // backs off folding because 2 ** 55 is too large
        // test_same("x = 3 ** -1"); // backs off because 3**-1 is shorter than 0.3333333333333333

        test("x = 0 / 0", "x = NaN");
        test("x = 0 % 0", "x = NaN");
        test("x = (-1) ** 0.5", "x = NaN");
    }

    #[test]
    #[ignore]
    fn test_fold_arithmetic2() {
        test_same("x = y + 10 + 20");
        test_same("x = y / 2 / 4");
        test("x = y * 2.25 * 3", "x = y * 6.75");
        test_same("x = y * 2.25 * z * 3");
        test_same("z = x * y");
        test_same("x = y * 5");
        test("x = y + (z * 24 * 60 * 60 * 1000)", "x = y + z * 864E5");
    }

    #[test]
    #[ignore]
    fn test_fold_arithmetic3() {
        test("x = null * undefined", "x = NaN");
        test("x = null * 1", "x = 0");
        test("x = (null - 1) * 2", "x = -2");
        test("x = (null + 1) * 2", "x = 2");
        test("x = null ** 0", "x = 1");
        test("x = (-0) ** 3", "x = -0");
    }

    #[test]
    fn test_fold_arithmetic_infinity() {
        test("x=-Infinity-2", "x=-Infinity");
        test("x=Infinity-2", "x=Infinity");
        test("x=Infinity*5", "x=Infinity");
        test("x = Infinity ** 2", "x = Infinity");
        test("x = Infinity ** -2", "x = 0");

        test("x = Infinity / Infinity", "x = NaN");
        test("x = Infinity % Infinity", "x = NaN");
        test("x = Infinity / 0", "x = NaN");
        test("x = Infinity % 0", "x = NaN");
    }
}
