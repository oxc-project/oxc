//! Constant Folding
//!
//! <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeFoldConstants.java>

use std::{cmp::Ordering, mem};

use num_bigint::BigInt;

use oxc_ast::{ast::*, AstBuilder, Visit};
use oxc_span::{GetSpan, Span, SPAN};
use oxc_syntax::{
    number::NumberBase,
    operator::{BinaryOperator, LogicalOperator, UnaryOperator},
};
use oxc_traverse::{Traverse, TraverseCtx};

use crate::{
    keep_var::KeepVar,
    node_util::{is_exact_int64, MayHaveSideEffects, NodeUtil, NumberValue},
    tri::Tri,
    ty::Ty,
    CompressorPass,
};

pub struct FoldConstants<'a> {
    ast: AstBuilder<'a>,
    evaluate: bool,
}

impl<'a> CompressorPass<'a> for FoldConstants<'a> {}

impl<'a> Traverse<'a> for FoldConstants<'a> {
    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.fold_condition(stmt, ctx);
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.fold_expression(expr, ctx);
        self.fold_conditional_expression(expr, ctx);
    }
}

impl<'a> FoldConstants<'a> {
    pub fn new(ast: AstBuilder<'a>) -> Self {
        Self { ast, evaluate: false }
    }

    pub fn with_evaluate(mut self, yes: bool) -> Self {
        self.evaluate = yes;
        self
    }

    fn fold_expression_and_get_boolean_value(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<bool> {
        self.fold_expression(expr, ctx);
        ctx.get_boolean_value(expr)
    }

    fn fold_if_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::IfStatement(if_stmt) = stmt else { return };

        // Descend and remove `else` blocks first.
        if let Some(alternate) = &mut if_stmt.alternate {
            self.fold_if_statement(alternate, ctx);
            if matches!(alternate, Statement::EmptyStatement(_)) {
                if_stmt.alternate = None;
            }
        }

        match self.fold_expression_and_get_boolean_value(&mut if_stmt.test, ctx) {
            Some(true) => {
                *stmt = self.ast.move_statement(&mut if_stmt.consequent);
            }
            Some(false) => {
                *stmt = if let Some(alternate) = &mut if_stmt.alternate {
                    self.ast.move_statement(alternate)
                } else {
                    // Keep hoisted `vars` from the consequent block.
                    let mut keep_var = KeepVar::new(self.ast);
                    keep_var.visit_statement(&if_stmt.consequent);
                    keep_var
                        .get_variable_declaration_statement()
                        .unwrap_or_else(|| self.ast.statement_empty(SPAN))
                };
            }
            None => {}
        }
    }

    fn fold_conditional_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::ConditionalExpression(conditional_expr) = expr else {
            return;
        };
        match self.fold_expression_and_get_boolean_value(&mut conditional_expr.test, ctx) {
            Some(true) => {
                *expr = self.ast.move_expression(&mut conditional_expr.consequent);
            }
            Some(false) => {
                *expr = self.ast.move_expression(&mut conditional_expr.alternate);
            }
            _ => {}
        }
    }

    pub fn fold_expression<'b>(&mut self, expr: &'b mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
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
                BinaryOperator::Addition if self.evaluate => self.try_fold_addition(
                    binary_expr.span,
                    &binary_expr.left,
                    &binary_expr.right,
                    ctx,
                ),
                _ => None,
            },
            Expression::LogicalExpression(logic_expr) => {
                self.try_fold_logical_expression(logic_expr, ctx)
            }
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
                Some(self.ast.expression_string_literal(span, value))
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
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let value = match self.evaluate_comparison(op, left, right, ctx) {
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
        ctx: &mut TraverseCtx<'a>,
    ) -> Tri {
        if left.may_have_side_effects() || right.may_have_side_effects() {
            return Tri::Unknown;
        }

        match op {
            BinaryOperator::Equality => self.try_abstract_equality_comparison(left, right, ctx),
            BinaryOperator::Inequality => {
                self.try_abstract_equality_comparison(left, right, ctx).not()
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
        &mut self,
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
                    let number_literal_expr = self.ast.expression_numeric_literal(
                        right_expr.span(),
                        num,
                        num.to_string(),
                        if num.fract() == 0.0 { NumberBase::Decimal } else { NumberBase::Float },
                    );

                    return self.try_abstract_equality_comparison(
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
                    let number_literal_expr = self.ast.expression_numeric_literal(
                        left_expr.span(),
                        num,
                        num.to_string(),
                        if num.fract() == 0.0 { NumberBase::Decimal } else { NumberBase::Float },
                    );

                    return self.try_abstract_equality_comparison(
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

        let left_bigint = ctx.get_side_free_bigint_value(left_expr);
        let right_bigint = ctx.get_side_free_bigint_value(right_expr);

        let left_num = ctx.get_side_free_number_value(left_expr);
        let right_num = ctx.get_side_free_number_value(right_expr);

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
            NumberValue::NaN => Tri::for_boolean(will_negative),
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
                                Tri::for_boolean(num.is_sign_positive()).xor(invert)
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

                        return Tri::for_boolean(l_num == r_num);
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

    /// ported from [closure-compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/PeepholeFoldConstants.java#L1114-L1162)
    #[allow(clippy::cast_possible_truncation)]
    fn try_fold_shift<'b>(
        &mut self,
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

            return Some(self.ast.expression_numeric_literal(
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
    pub fn try_fold_logical_expression(
        &mut self,
        logical_expr: &mut LogicalExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let op = logical_expr.operator;
        if !matches!(op, LogicalOperator::And | LogicalOperator::Or) {
            return None;
        }
        if let Some(boolean_value) = ctx.get_boolean_value(&logical_expr.left) {
            // (TRUE || x) => TRUE (also, (3 || x) => 3)
            // (FALSE && x) => FALSE
            if (boolean_value && op == LogicalOperator::Or)
                || (!boolean_value && op == LogicalOperator::And)
            {
                return Some(self.move_out_expression(&mut logical_expr.left));
            } else if !logical_expr.left.may_have_side_effects() {
                // (FALSE || x) => x
                // (TRUE && x) => x
                return Some(self.move_out_expression(&mut logical_expr.right));
            }
            // Left side may have side effects, but we know its boolean value.
            // e.g. true_with_sideeffects || foo() => true_with_sideeffects, foo()
            // or: false_with_sideeffects && foo() => false_with_sideeffects, foo()
            let left = self.move_out_expression(&mut logical_expr.left);
            let right = self.move_out_expression(&mut logical_expr.right);
            let mut vec = self.ast.vec_with_capacity(2);
            vec.push(left);
            vec.push(right);
            let sequence_expr = self.ast.expression_sequence(logical_expr.span, vec);
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
                            let left = self.move_out_expression(&mut left_child.left);
                            let right = self.move_out_expression(&mut logical_expr.right);
                            let logic_expr = self.ast.expression_logical(
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

    pub(crate) fn fold_condition<'b>(
        &mut self,
        stmt: &'b mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
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
            Statement::IfStatement(_) => {
                self.fold_if_statement(stmt, ctx);
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
                    let should_fold = Self::try_minimize_not(&mut unary_expr.argument);

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
    fn try_minimize_not(expr: &mut Expression<'a>) -> bool {
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
