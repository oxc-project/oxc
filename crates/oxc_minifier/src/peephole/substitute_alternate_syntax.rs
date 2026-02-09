use std::iter::repeat_with;

use crate::generated::ancestor::Ancestor;
use oxc_allocator::{CloneIn, TakeIn, Vec};
use oxc_ast::{NONE, ast::*};
use oxc_compat::ESFeature;
use oxc_ecmascript::constant_evaluation::{ConstantEvaluation, ConstantValue, DetermineValueType};
use oxc_ecmascript::side_effects::MayHaveSideEffectsContext;
use oxc_ecmascript::{ToJsString, ToNumber, side_effects::MayHaveSideEffects};
use oxc_semantic::ReferenceFlags;
use oxc_span::GetSpan;
use oxc_span::SPAN;
use oxc_syntax::precedence::GetPrecedence;
use oxc_syntax::{
    number::NumberBase,
    operator::{BinaryOperator, UnaryOperator},
};

use crate::TraverseCtx;

use super::PeepholeOptimizations;

/// A peephole optimization that minimizes code by simplifying conditional
/// expressions, replacing IFs with HOOKs, replacing object constructors
/// with literals, and simplifying returns.
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeSubstituteAlternateSyntax.java>
impl<'a> PeepholeOptimizations {
    pub fn substitute_object_property(prop: &mut ObjectProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        // <https://tc39.es/ecma262/2024/multipage/ecmascript-language-expressions.html#sec-runtime-semantics-propertydefinitionevaluation>
        if !prop.method
            && let PropertyKey::StringLiteral(str) = &prop.key
        {
            // "{ __proto__ }" sets prototype, while "{ ['__proto__'] }" does not
            if str.value == "__proto__" {
                return;
            }
        }

        Self::try_compress_property_key(&mut prop.key, &mut prop.computed, ctx);
    }

    pub fn substitute_assignment_target_property_property(
        prop: &mut AssignmentTargetPropertyProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        Self::try_compress_property_key(&mut prop.name, &mut prop.computed, ctx);
    }

    pub fn substitute_assignment_target_property(
        prop: &mut AssignmentTargetProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        Self::try_compress_assignment_target_property(prop, ctx);
    }

    pub fn try_compress_assignment_target_property(
        prop: &mut AssignmentTargetProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // `a: a` -> `a`
        if let AssignmentTargetProperty::AssignmentTargetPropertyProperty(assign_target_prop_prop) =
            prop
        {
            let Some(prop_name) = assign_target_prop_prop.name.static_name() else { return };
            let Some(ident) = assign_target_prop_prop.binding.identifier_mut() else {
                return;
            };
            if prop_name == ident.name {
                *prop = ctx.ast.assignment_target_property_assignment_target_property_identifier(
                    ident.span,
                    ident.take_in(ctx.ast),
                    None,
                );
                ctx.state.changed = true;
            }
        }
    }

    pub fn substitute_binding_property(prop: &mut BindingProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::try_compress_property_key(&mut prop.key, &mut prop.computed, ctx);
    }

    pub fn substitute_method_definition(
        prop: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let property_key_parent: ClassPropertyKeyParent = prop.into();
        // Only check for computed property restrictions if this is actually a computed property
        if prop.computed
            && let PropertyKey::StringLiteral(str) = &prop.key
            && property_key_parent.should_keep_as_computed_property(&str.value)
        {
            return;
        }
        Self::try_compress_property_key(&mut prop.key, &mut prop.computed, ctx);
    }

    pub fn substitute_property_definition(
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let property_key_parent: ClassPropertyKeyParent = prop.into();
        // Only check for computed property restrictions if this is actually a computed property
        if prop.computed
            && let PropertyKey::StringLiteral(str) = &prop.key
            && property_key_parent.should_keep_as_computed_property(&str.value)
        {
            return;
        }
        Self::try_compress_property_key(&mut prop.key, &mut prop.computed, ctx);
    }

    pub fn substitute_accessor_property(
        prop: &mut AccessorProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let property_key_parent: ClassPropertyKeyParent = prop.into();
        // Only check for computed property restrictions if this is actually a computed property
        if prop.computed
            && let PropertyKey::StringLiteral(str) = &prop.key
            && property_key_parent.should_keep_as_computed_property(&str.value)
        {
            return;
        }
        Self::try_compress_property_key(&mut prop.key, &mut prop.computed, ctx);
    }

    pub fn substitute_for_statement(stmt: &mut ForStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::try_rewrite_arguments_copy_loop(stmt, ctx);
    }

    pub fn substitute_variable_declaration(
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        for declarator in &mut decl.declarations {
            Self::compress_variable_declarator(declarator, ctx);
        }
    }

    pub fn substitute_call_expression(expr: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::try_flatten_arguments(&mut expr.arguments, ctx);
        Self::try_rewrite_object_callee_indirect_call(expr, ctx);
    }

    pub fn substitute_new_expression(expr: &mut NewExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::try_flatten_arguments(&mut expr.arguments, ctx);
    }

    pub fn substitute_chain_expression(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::ChainExpression(e) = expr else { return };
        Self::try_flatten_nested_chain_expression(e, ctx);
        Self::substitute_chain_call_expression(e, ctx);
    }

    pub fn substitute_swap_binary_expressions(e: &mut BinaryExpression<'a>) {
        if e.operator.is_equality()
            && (e.left.is_literal() || e.left.is_no_substitution_template() || e.left.is_void_0())
            && !e.right.is_literal()
        {
            std::mem::swap(&mut e.left, &mut e.right);
        }
    }

    /// `() => { return foo })` -> `() => foo`
    pub fn substitute_arrow_expression(
        arrow_expr: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !arrow_expr.expression
            && arrow_expr.body.directives.is_empty()
            && arrow_expr.body.statements.len() == 1
            && let Some(body) = arrow_expr.body.statements.first_mut()
            && let Statement::ReturnStatement(ret_stmt) = body
        {
            let return_stmt_arg = ret_stmt.argument.as_mut().map(|arg| arg.take_in(ctx.ast));
            if let Some(arg) = return_stmt_arg {
                *body = ctx.ast.statement_expression(arg.span(), arg);
                arrow_expr.expression = true;
                ctx.state.changed = true;
            }
        }
    }

    /// Compress `typeof foo == "undefined"`
    ///
    /// - `typeof foo == "undefined"` (if foo is not resolved) -> `typeof foo > "u"`
    /// - `typeof foo != "undefined"` (if foo is not resolved) -> `typeof foo < "u"`
    /// - `typeof foo == "undefined"` -> `foo === undefined`
    /// - `typeof foo != "undefined"` -> `foo !== undefined`
    /// - `typeof foo.bar == "undefined"` -> `foo.bar === undefined` (for any expression e.g.`typeof (foo + "")`)
    /// - `typeof foo.bar != "undefined"` -> `foo.bar !== undefined` (for any expression e.g.`typeof (foo + "")`)
    ///
    /// Enabled by `compress.typeofs`
    pub fn substitute_typeof_undefined(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::BinaryExpression(e) = expr else { return };
        let Expression::UnaryExpression(unary_expr) = &e.left else { return };
        if !unary_expr.operator.is_typeof() {
            return;
        }
        let (new_eq_op, new_comp_op) = match e.operator {
            BinaryOperator::Equality | BinaryOperator::StrictEquality => {
                (BinaryOperator::StrictEquality, BinaryOperator::GreaterThan)
            }
            BinaryOperator::Inequality | BinaryOperator::StrictInequality => {
                (BinaryOperator::StrictInequality, BinaryOperator::LessThan)
            }
            _ => return,
        };
        if !e.right.is_specific_string_literal("undefined") {
            return;
        }
        *expr = if let Expression::Identifier(ident) = &unary_expr.argument
            && ctx.is_global_reference(ident)
        {
            let left = e.left.take_in(ctx.ast);
            let right = ctx.ast.expression_string_literal(e.right.span(), "u", None);
            ctx.ast.expression_binary(e.span, left, new_comp_op, right)
        } else {
            let span = e.span;
            let Expression::UnaryExpression(unary_expr) = &mut e.left else { return };
            ctx.ast.expression_binary(
                span,
                unary_expr.take_in(ctx.ast).argument,
                new_eq_op,
                ctx.ast.void_0(e.right.span()),
            )
        };
        ctx.state.changed = true;
    }

    /// Remove unary `+` if `ToNumber` conversion is done by the parent expression
    ///
    /// - `1 - +b` => `1 - b` (for other operators as well)
    /// - `+a - 1` => `a - 1` (for other operators as well)
    pub fn substitute_unary_plus(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::UnaryExpression(e) = expr else { return };
        if e.operator != UnaryOperator::UnaryPlus {
            return;
        }
        let Some(parent_expression) = ctx.ancestors().next() else { return };
        let parent_expression_does_to_number_conversion = match parent_expression {
            Ancestor::BinaryExpressionLeft(e) => {
                Self::is_binary_operator_that_does_number_conversion(*e.operator())
                    && e.right().value_type(ctx).is_number()
            }
            Ancestor::BinaryExpressionRight(e) => {
                Self::is_binary_operator_that_does_number_conversion(*e.operator())
                    && e.left().value_type(ctx).is_number()
            }
            _ => false,
        };
        if !parent_expression_does_to_number_conversion {
            return;
        }
        *expr = e.argument.take_in(ctx.ast);
        ctx.state.changed = true;
    }

    /// For `+a - n` => `a - n` (assuming n is a number)
    ///
    /// Before compression the evaluation is:
    /// 1. `a_2 = ToNumber(a)`
    /// 2. `a_3 = ToNumeric(a_2)`
    /// 3. `n_2 = ToNumeric(n)` (no-op since n is a number)
    /// 4. If the type of `a_3` is not number, throw an error
    /// 5. Calculate the result of the binary operation
    ///
    /// After compression, step 1 is removed. The difference we need to care is
    /// the difference with `ToNumber(a)` and `ToNumeric(a)` because `ToNumeric(a_2)` is a no-op.
    ///
    /// - When `a` is an object and `ToPrimitive(a, NUMBER)` returns a BigInt,
    ///   - `ToNumeric(a)` will return that value. But the binary operation will throw an error in step 4.
    ///   - `ToNumber(a)` will throw an error.
    /// - When `a` is an object and `ToPrimitive(a, NUMBER)` returns a value other than BigInt,
    ///   `ToNumeric(a)` and `ToNumber(a)` works the same. Because the step 2 in `ToNumeric` is always `false`.
    /// - When `a` is BigInt,
    ///   - `ToNumeric(a)` will return that value. But the binary operation will throw an error in step 4.
    ///   - `ToNumber(a)` will throw an error.
    /// - When `a` is not a object nor a BigInt, `ToNumeric(a)` and `ToNumber(a)` works the same.
    ///   Because the step 2 in `ToNumeric` is always `false`.
    ///
    /// Thus, removing `+` is fine.
    fn is_binary_operator_that_does_number_conversion(operator: BinaryOperator) -> bool {
        matches!(
            operator,
            BinaryOperator::Exponential
                | BinaryOperator::Multiplication
                | BinaryOperator::Division
                | BinaryOperator::Remainder
                | BinaryOperator::Subtraction
                | BinaryOperator::ShiftLeft
                | BinaryOperator::ShiftRight
                | BinaryOperator::ShiftRightZeroFill
                | BinaryOperator::BitwiseAnd
                | BinaryOperator::BitwiseXOR
                | BinaryOperator::BitwiseOR
        )
    }

    /// `a || (b || c);` -> `(a || b) || c;`
    pub fn substitute_rotate_logical_expression(
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::LogicalExpression(e) = expr else { return };
        let Expression::LogicalExpression(right) = &e.right else { return };
        if right.operator != e.operator {
            return;
        }
        let Expression::LogicalExpression(mut right) = e.right.take_in(ctx.ast) else { return };
        let mut new_left = ctx.ast.expression_logical(
            e.span,
            e.left.take_in(ctx.ast),
            e.operator,
            right.left.take_in(ctx.ast),
        );
        Self::substitute_rotate_logical_expression(&mut new_left, ctx);
        *expr =
            ctx.ast.expression_logical(e.span, new_left, e.operator, right.right.take_in(ctx.ast));
        ctx.state.changed = true;
    }

    /// Rotate associative binary operators:
    /// - `a | (b | c)` -> `(a | b) | c`
    ///
    /// Rotate commutative operators to reduce parentheses:
    /// - `a * (b % c)` -> `b % c * a`
    /// - `a * (b / c)` -> `b / c * a`
    pub fn substitute_rotate_binary_expression(
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::BinaryExpression(e) = expr else { return };

        // Handle associative rotation
        let is_associative = matches!(
            e.operator,
            BinaryOperator::BitwiseOR | BinaryOperator::BitwiseAnd | BinaryOperator::BitwiseXOR
        );
        if is_associative
            && let Expression::BinaryExpression(right) = &e.right
            && right.operator == e.operator
            && !right.right.may_have_side_effects(ctx)
        {
            let Expression::BinaryExpression(mut right) = e.right.take_in(ctx.ast) else {
                return;
            };
            let mut new_left = ctx.ast.expression_binary(
                e.span,
                e.left.take_in(ctx.ast),
                e.operator,
                right.left.take_in(ctx.ast),
            );
            Self::substitute_rotate_binary_expression(&mut new_left, ctx);
            *expr = ctx.ast.expression_binary(
                e.span,
                new_left,
                e.operator,
                right.right.take_in(ctx.ast),
            );
            ctx.state.changed = true;
            return;
        }

        // Handle commutative rotation
        if let Expression::BinaryExpression(right) = &e.right
            && matches!(e.operator, BinaryOperator::Multiplication)
            && e.operator.precedence() == right.operator.precedence()
        {
            // Don't swap if left does not need a parentheses
            if let Expression::BinaryExpression(left) = &e.left
                && e.operator.precedence() <= left.operator.precedence()
            {
                return;
            }

            // Don't swap if any of the value may have side effects as they may update the other values
            if !e.left.may_have_side_effects(ctx)
                && !right.left.may_have_side_effects(ctx)
                && !right.right.may_have_side_effects(ctx)
            {
                let left = e.left.take_in(ctx.ast);
                let right = e.right.take_in(ctx.ast);
                e.right = left;
                e.left = right;
                ctx.state.changed = true;
            }
        }
    }

    /// Compress `typeof foo === 'object' && foo !== null` into `typeof foo == 'object' && !!foo`.
    ///
    /// - `typeof foo === 'object' && foo !== null` => `typeof foo == 'object' && !!foo`
    /// - `typeof foo == 'object' && foo != null` => `typeof foo == 'object' && !!foo`
    /// - `typeof foo !== 'object' || foo === null` => `typeof foo != 'object' || !foo`
    /// - `typeof foo != 'object' || foo == null` => `typeof foo != 'object' || !foo`
    ///
    /// If `typeof foo == 'object'`, then `foo` is guaranteed to be an object or null.
    /// - If `foo` is an object, then `foo !== null` is `true`. If `foo` is null, then `foo !== null` is `false`.
    /// - If `foo` is an object, then `foo != null` is `true`. If `foo` is null, then `foo != null` is `false`.
    /// - If `foo` is an object, then `!!foo` is `true`. If `foo` is null, then `!!foo` is `false`.
    ///
    /// This compression is safe for `document.all` because `typeof document.all` is not `'object'`.
    pub fn substitute_is_object_and_not_null(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::LogicalExpression(e) = expr else { return };
        let inversed = match e.operator {
            LogicalOperator::And => false,
            LogicalOperator::Or => true,
            LogicalOperator::Coalesce => return,
        };
        if let Some(new_expr) = Self::try_compress_is_object_and_not_null_for_left_and_right(
            &e.left, &e.right, e.span, ctx, inversed,
        ) {
            *expr = new_expr;
            ctx.state.changed = true;
            return;
        }
        let Expression::LogicalExpression(left) = &e.left else {
            return;
        };
        if left.operator != e.operator {
            return;
        }
        let Some(new_expr) = Self::try_compress_is_object_and_not_null_for_left_and_right(
            &left.right,
            &e.right,
            left.right.span().merge_within(e.right.span(), e.span).unwrap_or(SPAN),
            ctx,
            inversed,
        ) else {
            return;
        };
        let span = e.span;
        let Expression::LogicalExpression(left) = &mut e.left else {
            return;
        };
        *expr = ctx.ast.expression_logical(span, left.left.take_in(ctx.ast), e.operator, new_expr);
        ctx.state.changed = true;
    }

    fn try_compress_is_object_and_not_null_for_left_and_right(
        left: &Expression<'a>,
        right: &Expression<'a>,
        span: Span,
        ctx: &TraverseCtx<'a>,
        inversed: bool,
    ) -> Option<Expression<'a>> {
        let pair = Self::commutative_pair(
            (&left, &right),
            |a_expr| {
                let Expression::BinaryExpression(a) = a_expr else { return None };
                let is_target_ops = if inversed {
                    matches!(
                        a.operator,
                        BinaryOperator::StrictInequality | BinaryOperator::Inequality
                    )
                } else {
                    matches!(a.operator, BinaryOperator::StrictEquality | BinaryOperator::Equality)
                };
                if !is_target_ops {
                    return None;
                }
                let (id, ()) = Self::commutative_pair(
                    (&a.left, &a.right),
                    |a_a| {
                        let Expression::UnaryExpression(a_a) = a_a else { return None };
                        if a_a.operator != UnaryOperator::Typeof {
                            return None;
                        }
                        let Expression::Identifier(id) = &a_a.argument else { return None };
                        Some(id)
                    },
                    |b| b.is_specific_string_literal("object").then_some(()),
                )?;
                Some((id, a_expr))
            },
            |b| {
                let Expression::BinaryExpression(b) = b else {
                    return None;
                };
                let is_target_ops = if inversed {
                    matches!(b.operator, BinaryOperator::StrictEquality | BinaryOperator::Equality)
                } else {
                    matches!(
                        b.operator,
                        BinaryOperator::StrictInequality | BinaryOperator::Inequality
                    )
                };
                if !is_target_ops {
                    return None;
                }
                let (id, ()) = Self::commutative_pair(
                    (&b.left, &b.right),
                    |a_a| {
                        let Expression::Identifier(id) = a_a else { return None };
                        Some(id)
                    },
                    |b| b.is_null().then_some(()),
                )?;
                Some(id)
            },
        );
        let ((typeof_id_ref, typeof_binary_expr), is_null_id_ref) = pair?;
        if typeof_id_ref.name != is_null_id_ref.name {
            return None;
        }
        // It should also return None when the reference might refer to a reference value created by a with statement
        // when the minifier supports with statements
        if ctx.is_global_reference(typeof_id_ref) {
            return None;
        }

        let mut new_left_expr = typeof_binary_expr.clone_in_with_semantic_ids(ctx.ast.allocator);
        if let Expression::BinaryExpression(new_left_expr_binary) = &mut new_left_expr {
            new_left_expr_binary.operator =
                if inversed { BinaryOperator::Inequality } else { BinaryOperator::Equality };
        } else {
            unreachable!();
        }

        let is_null_id_ref = ctx.ast.expression_identifier_with_reference_id(
            is_null_id_ref.span,
            is_null_id_ref.name,
            is_null_id_ref.reference_id(),
        );

        let new_right_expr = if inversed {
            ctx.ast.expression_unary(SPAN, UnaryOperator::LogicalNot, is_null_id_ref)
        } else {
            ctx.ast.expression_unary(
                SPAN,
                UnaryOperator::LogicalNot,
                ctx.ast.expression_unary(SPAN, UnaryOperator::LogicalNot, is_null_id_ref),
            )
        };
        Some(ctx.ast.expression_logical(
            span,
            new_left_expr,
            if inversed { LogicalOperator::Or } else { LogicalOperator::And },
            new_right_expr,
        ))
    }

    pub fn substitute_loose_equals_undefined(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::BinaryExpression(e) = expr else { return };
        // `foo == void 0` -> `foo == null`, `foo == undefined` -> `foo == null`
        // `foo != void 0` -> `foo == null`, `foo == undefined` -> `foo == null`
        if e.operator == BinaryOperator::Inequality || e.operator == BinaryOperator::Equality {
            let (left, right) = if ctx.is_expression_undefined(&e.right) {
                (e.left.take_in(ctx.ast), ctx.ast.expression_null_literal(e.right.span()))
            } else if ctx.is_expression_undefined(&e.left) {
                (e.right.take_in(ctx.ast), ctx.ast.expression_null_literal(e.left.span()))
            } else {
                return;
            };
            *expr = ctx.ast.expression_binary(e.span, left, e.operator, right);
            ctx.state.changed = true;
        }
    }

    #[expect(clippy::float_cmp)]
    /// Rewrite classic `arguments` copy loop to spread form
    ///
    /// Transforms the common Babel/TS output:
    /// ```js
    ///   for (var e = arguments.length, r = Array(e), a = 0; a < e; a++)
    ///     r[a] = arguments[a];
    /// ```
    /// into:
    /// ```js
    ///   for (var r = [...arguments]; 0; ) ;
    /// ```
    /// which gets folded later into:
    /// ```js
    ///   var r = [...arguments]
    /// ```
    ///
    /// Other supported inputs:
    /// ```js
    ///   for (var e = arguments.length, r = Array(e > 1 ? e - 1 : 0), a = 1; a < e; a++)
    ///     r[a - 1] = arguments[a];
    ///   for (var r = [], a = 0; a < arguments.length; a++)
    ///     r[a] = arguments[a];
    ///   for (var r = [], a = 1; a < arguments.length; a++)
    ///     r[a - 1] = arguments[a];
    /// ```
    fn try_rewrite_arguments_copy_loop(for_stmt: &mut ForStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        #[derive(PartialEq, Eq)]
        enum VerifyArrayArgResult {
            WithOffset,
            WithoutOffset,
            Invalid,
        }

        /// Verify whether `arg_expr` is `e > offset ? e - offset : 0` or `e`
        fn verify_array_arg(
            arg_expr: &Expression,
            name_e: &str,
            offset: f64,
        ) -> VerifyArrayArgResult {
            match arg_expr {
                Expression::Identifier(id) => {
                    if offset == 0.0 && id.name == name_e {
                        VerifyArrayArgResult::WithoutOffset
                    } else {
                        VerifyArrayArgResult::Invalid
                    }
                }
                Expression::ConditionalExpression(cond_expr) => {
                    let Expression::BinaryExpression(test_expr) = &cond_expr.test else {
                        return VerifyArrayArgResult::Invalid;
                    };
                    let Expression::BinaryExpression(cons_expr) = &cond_expr.consequent else {
                        return VerifyArrayArgResult::Invalid;
                    };
                    if test_expr.operator == BinaryOperator::GreaterThan
                        && test_expr.left.is_specific_id(name_e)
                        && matches!(&test_expr.right, Expression::NumericLiteral(n) if n.value == offset)
                        && cons_expr.operator == BinaryOperator::Subtraction
                        && matches!(&cons_expr.left, Expression::Identifier(id) if id.name == name_e)
                        && matches!(&cons_expr.right, Expression::NumericLiteral(n) if n.value == offset)
                        && matches!(&cond_expr.alternate, Expression::NumericLiteral(n) if n.value == 0.0)
                    {
                        VerifyArrayArgResult::WithOffset
                    } else {
                        VerifyArrayArgResult::Invalid
                    }
                }
                _ => VerifyArrayArgResult::Invalid,
            }
        }

        // In non-strict mode, a different value may be reassigned to the `arguments` variable
        if !ctx.current_scope_flags().is_strict_mode() {
            return;
        }

        // Parse statement: `r[a - offset] = arguments[a];`
        let body_assign_expr = {
            let assign = match &mut for_stmt.body {
                Statement::ExpressionStatement(expr_stmt) => expr_stmt,
                Statement::BlockStatement(block) if block.body.len() == 1 => {
                    match &mut block.body[0] {
                        Statement::ExpressionStatement(expr_stmt) => expr_stmt,
                        _ => return,
                    }
                }
                _ => return,
            };
            let Expression::AssignmentExpression(assign_expr) = &mut assign.expression else {
                return;
            };
            if !assign_expr.operator.is_assign() {
                return;
            }
            assign_expr
        };

        // reference counts in the for-loop
        let mut a_ref_count = 0;
        let mut e_ref_count = 0;

        let (r_id_name, a_id_name, offset) = {
            let AssignmentTarget::ComputedMemberExpression(lhs_member_expr) =
                &body_assign_expr.left
            else {
                return;
            };
            let Expression::Identifier(lhs_member_expr_obj) = &lhs_member_expr.object else {
                return;
            };
            let (base_name, offset) = match &lhs_member_expr.expression {
                Expression::Identifier(id) => (id.name, 0.0),
                Expression::BinaryExpression(b) => {
                    if b.operator != BinaryOperator::Subtraction {
                        return;
                    }
                    let Expression::Identifier(id) = &b.left else { return };
                    let Expression::NumericLiteral(n) = &b.right else { return };
                    if n.value.fract() != 0.0 || n.value < 0.0 {
                        return;
                    }
                    (id.name, n.value)
                }
                _ => return,
            };
            (lhs_member_expr_obj.name, base_name, offset)
        };

        let arguments_id = {
            let Expression::ComputedMemberExpression(rhs_member_expr) = &mut body_assign_expr.right
            else {
                return;
            };
            let ComputedMemberExpression { object, expression, .. } = rhs_member_expr.as_mut();
            let Expression::Identifier(rhs_member_expr_obj) = object else {
                return;
            };
            if rhs_member_expr_obj.name != "arguments"
                || !ctx.is_global_reference(rhs_member_expr_obj)
            {
                return;
            }
            let Expression::Identifier(rhs_member_expr_expr_id) = expression else {
                return;
            };
            if rhs_member_expr_expr_id.name != a_id_name {
                return;
            }
            rhs_member_expr_obj
        };
        a_ref_count += 2;

        // Parse update: `a++`
        {
            let Some(Expression::UpdateExpression(u)) = &for_stmt.update else {
                return;
            };
            let SimpleAssignmentTarget::AssignmentTargetIdentifier(id) = &u.argument else {
                return;
            };
            if a_id_name != id.name {
                return;
            }
            a_ref_count += 1;
        };

        // Parse test: `a < e` or `a < arguments.length`
        let e_id_info = {
            let Some(Expression::BinaryExpression(b)) = &for_stmt.test else {
                return;
            };
            if b.operator != BinaryOperator::LessThan {
                return;
            }
            let Expression::Identifier(left) = &b.left else { return };
            if left.name != a_id_name {
                return;
            }
            match &b.right {
                Expression::Identifier(right) => Some((
                    &right.name,
                    ctx.scoping().get_reference(right.reference_id()).symbol_id(),
                )),
                Expression::StaticMemberExpression(sm) => {
                    let Expression::Identifier(id) = &sm.object else {
                        return;
                    };
                    if id.name != "arguments"
                        || !ctx.is_global_reference(id)
                        || sm.property.name != "length"
                    {
                        return;
                    }
                    None
                }
                _ => return,
            }
        };
        if e_id_info.is_some() {
            e_ref_count += 1;
        }
        a_ref_count += 1;

        let init_decl_len = if e_id_info.is_some() { 3 } else { 2 };

        let Some(init) = &mut for_stmt.init else { return };
        let ForStatementInit::VariableDeclaration(var_init) = init else { return };
        // Need at least two declarators: r, a (optional `e` may precede them)
        if var_init.declarations.len() < init_decl_len {
            return;
        }

        // make sure `arguments` points to the arguments object
        // this is checked after the structure checks above because this check is slower than the structure checks
        if ctx.ancestor_scopes().all(|s| {
            let scope_flags = ctx.scoping().scope_flags(s);
            !scope_flags.is_function() || scope_flags.is_arrow()
        }) {
            return;
        }

        let mut idx = 0usize;

        // Check `e = arguments.length`
        if let Some((e_id_name, _)) = e_id_info {
            let de = var_init
                .declarations
                .get(idx)
                .expect("var_init.declarations.len() check above ensures this");
            let BindingPattern::BindingIdentifier(de_id) = &de.id else { return };
            if de_id.name != e_id_name {
                return;
            }
            let Some(Expression::StaticMemberExpression(sm)) = &de.init else { return };
            let Expression::Identifier(id) = &sm.object else { return };
            if id.name != "arguments"
                || !ctx.is_global_reference(id)
                || sm.property.name != "length"
            {
                return;
            }

            idx += 1;
        }

        // Check `a = 0` or `a = k`
        let a_id_symbol_id = {
            let de_a = var_init
                .declarations
                .get(idx + 1)
                .expect("var_init.declarations.len() check above ensures this");
            let BindingPattern::BindingIdentifier(de_id) = &de_a.id else { return };
            if de_id.name != a_id_name {
                return;
            }
            if !matches!(&de_a.init, Some(Expression::NumericLiteral(n)) if n.value == offset) {
                return;
            }
            de_id.symbol_id()
        };

        // Check `r = Array(e > 1 ? e - 1 : 0)`, or `r = []`
        let r_id_pat_with_info = {
            let de_r = var_init
                .declarations
                .get_mut(idx)
                .expect("var_init.declarations.len() check above ensures this");
            match &de_r.init {
                // Array(e > 1 ? e - 1 : 0) or Array(e)
                Some(Expression::CallExpression(call)) => {
                    let Expression::Identifier(id) = &call.callee else { return };
                    if id.name != "Array" || !ctx.is_global_reference(id) {
                        return;
                    }
                    if call.arguments.len() != 1 {
                        return;
                    }
                    let Some((e_id_name, _)) = e_id_info else { return };
                    let Some(arg_expr) = call.arguments[0].as_expression() else { return };
                    let result = verify_array_arg(arg_expr, e_id_name, offset);
                    if result == VerifyArrayArgResult::Invalid {
                        return;
                    }
                    e_ref_count += if result == VerifyArrayArgResult::WithOffset { 2 } else { 1 };
                }
                Some(Expression::ArrayExpression(arr)) => {
                    if !arr.elements.is_empty() {
                        return;
                    }
                }
                _ => return,
            }
            let BindingPattern::BindingIdentifier(de_id) = &de_r.id else { return };
            if de_id.name != r_id_name {
                return;
            }
            let de_id_symbol_id = de_id.symbol_id();
            (&mut de_r.id, de_id_symbol_id)
        };

        // bail out if `e` or `a` is used outside the for-loop
        {
            if let Some((_, e_id_symbol_id)) = e_id_info
                && e_id_symbol_id.is_none_or(|id| {
                    ctx.scoping().get_resolved_references(id).count() != e_ref_count
                })
            {
                return;
            }
            if ctx.scoping().get_resolved_references(a_id_symbol_id).count() != a_ref_count {
                return;
            }
        }

        // Build `var r = [...arguments]` (with optional `.slice(offset)`) as the only declarator and drop test/update/body.

        let r_id_pat = {
            let (r_id, de_id_symbol_id) = r_id_pat_with_info;
            // `var r = [...arguments]` / `var r = [...arguments].slice(n)` is not needed
            // if r is not used by other places because `[...arguments]` does not have a sideeffect
            // `r` is used once in the for-loop (assignment for each index)
            (ctx.scoping().get_resolved_references(de_id_symbol_id).count() > 1)
                .then(|| r_id.take_in(ctx.ast))
        };

        let base_arr = ctx.ast.expression_array(
            SPAN,
            ctx.ast.vec1(ctx.ast.array_expression_element_spread_element(
                SPAN,
                Expression::Identifier(arguments_id.take_in_box(ctx.ast)),
            )),
        );
        // wrap with `.slice(offset)`
        let arr = if offset > 0.0 {
            let obj = base_arr;
            let callee =
                Expression::StaticMemberExpression(ctx.ast.alloc_static_member_expression(
                    SPAN,
                    obj,
                    ctx.ast.identifier_name(SPAN, "slice"),
                    false,
                ));
            ctx.ast.expression_call(
                SPAN,
                callee,
                NONE,
                ctx.ast.vec1(Argument::from(ctx.ast.expression_numeric_literal(
                    SPAN,
                    offset,
                    None,
                    NumberBase::Decimal,
                ))),
                false,
            )
        } else {
            base_arr
        };

        var_init.declarations = if let Some(r_id_pat) = r_id_pat {
            let new_decl =
                ctx.ast.variable_declarator(SPAN, var_init.kind, r_id_pat, NONE, Some(arr), false);
            ctx.ast.vec1(new_decl)
        } else {
            ctx.ast.vec()
        };
        for_stmt.test =
            Some(ctx.ast.expression_numeric_literal(for_stmt.span, 0.0, None, NumberBase::Decimal));
        for_stmt.update = None;
        for_stmt.body = ctx.ast.statement_empty(SPAN);
        ctx.state.changed = true;
    }

    /// Removes redundant argument of `ReturnStatement`
    ///
    /// `return undefined` -> `return`
    /// `return void 0` -> `return`
    pub fn substitute_return_statement(stmt: &mut ReturnStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Some(argument) = &stmt.argument else { return };
        if !match argument {
            Expression::Identifier(ident) => ctx.is_identifier_undefined(ident),
            Expression::UnaryExpression(e) => {
                e.operator.is_void() && !argument.may_have_side_effects(ctx)
            }
            _ => false,
        } {
            return;
        }
        // `return undefined` has a different semantic in async generator function.
        if ctx.is_closest_function_scope_an_async_generator() {
            return;
        }
        stmt.argument = None;
        ctx.state.changed = true;
    }

    fn compress_variable_declarator(decl: &mut VariableDeclarator<'a>, ctx: &mut TraverseCtx<'a>) {
        // Destructuring Pattern has error throwing side effect.
        if matches!(
            decl.kind,
            VariableDeclarationKind::Const
                | VariableDeclarationKind::Using
                | VariableDeclarationKind::AwaitUsing
        ) || decl.id.is_destructuring_pattern()
        {
            return;
        }
        if !decl.kind.is_var()
            && decl.init.as_ref().is_some_and(|init| ctx.is_expression_undefined(init))
        {
            decl.init = None;
            ctx.state.changed = true;
        }
    }

    /// Fold `Boolean`, ///
    /// `Boolean(a)` -> `!!a`
    /// `Number(0)` -> `0`
    /// `String()` -> `''`
    /// `BigInt(1)` -> `1`
    pub fn substitute_simple_function_call(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::CallExpression(e) = expr else { return };
        if e.optional || e.arguments.len() >= 2 {
            return;
        }
        let Expression::Identifier(ident) = &e.callee else { return };
        let name = ident.name.as_str();
        if !matches!(name, "Boolean" | "Number" | "String" | "BigInt") {
            return;
        }
        if !ctx.is_global_reference(ident) {
            return;
        }
        let span = e.span;
        let args = &mut e.arguments;
        let arg = match args.get_mut(0) {
            None => None,
            Some(arg) => match arg.as_expression_mut() {
                Some(arg) => Some(arg),
                None => return,
            },
        };
        let changed = match name {
            // `Boolean(a)` -> `!!(a)`
            // http://www.ecma-international.org/ecma-262/6.0/index.html#sec-boolean-constructor-boolean-value
            // and
            // http://www.ecma-international.org/ecma-262/6.0/index.html#sec-logical-not-operator-runtime-semantics-evaluation
            "Boolean" => match arg {
                None => Some(ctx.ast.expression_boolean_literal(span, false)),
                Some(arg) => {
                    let mut arg = arg.take_in(ctx.ast);
                    Self::minimize_expression_in_boolean_context(&mut arg, ctx);
                    let arg = ctx.ast.expression_unary(span, UnaryOperator::LogicalNot, arg);
                    Some(Self::minimize_not(span, arg, ctx))
                }
            },
            "String" => {
                match arg {
                    // `String()` -> `''`
                    None => Some(ctx.ast.expression_string_literal(span, "", None)),
                    Some(arg) => arg
                        .evaluate_value_to_string(ctx)
                        .filter(|_| !arg.may_have_side_effects(ctx))
                        .map(|s| ctx.value_to_expr(e.span, ConstantValue::String(s))),
                }
            }
            "Number" => Some(ctx.ast.expression_numeric_literal(
                span,
                match arg {
                    None => 0.0,
                    Some(arg) => match arg.to_number(ctx) {
                        Some(n) => n,
                        None => return,
                    },
                },
                None,
                NumberBase::Decimal,
            )),
            // `BigInt(1n)` -> `1n`
            "BigInt" => match arg {
                None => None,
                Some(arg) => {
                    matches!(arg, Expression::BigIntLiteral(_)).then(|| arg.take_in(ctx.ast))
                }
            },
            _ => None,
        };
        if let Some(changed) = changed {
            *expr = changed;
            ctx.state.changed = true;
        }
    }

    /// Fold `Object` or `Array` constructor
    fn get_fold_constructor_name(
        callee: &Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Option<&'a str> {
        match callee {
            Expression::StaticMemberExpression(e) => {
                if !matches!(&e.object, Expression::Identifier(ident) if ident.name == "window") {
                    return None;
                }
                Some(e.property.name.as_str())
            }
            Expression::Identifier(ident) => {
                let name = ident.name.as_str();
                if !matches!(name, "Object" | "Array") {
                    return None;
                }
                if !ctx.is_global_reference(ident) {
                    return None;
                }
                Some(name)
            }
            _ => None,
        }
    }

    /// `window.Object()`, `new Object()`, `Object()`  -> `{}`
    /// `window.Array()`, `new Array()`, `Array()`  -> `[]`
    pub fn substitute_object_or_array_constructor(
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let callee = match expr {
            Expression::NewExpression(e) => &e.callee,
            Expression::CallExpression(e) => &e.callee,
            _ => return,
        };
        let Some(name) = Self::get_fold_constructor_name(callee, ctx) else { return };
        let (span, callee, args, is_new_expr) = match expr {
            Expression::NewExpression(e) => {
                let NewExpression { span, callee, arguments, .. } = e.as_mut();
                (span, callee, arguments, true)
            }
            Expression::CallExpression(e) => {
                let CallExpression { span, callee, arguments, .. } = e.as_mut();
                (span, callee, arguments, false)
            }
            _ => return,
        };
        match name {
            "Object" if args.is_empty() => {
                *expr = ctx.ast.expression_object(*span, ctx.ast.vec());
                ctx.state.changed = true;
            }
            "Array" => {
                // `new Array` -> `[]`
                if args.is_empty() {
                    *expr = ctx.ast.expression_array(*span, ctx.ast.vec());
                    ctx.state.changed = true;
                } else if args.len() == 1 {
                    let Some(arg) = args[0].as_expression_mut() else { return };
                    // `new Array(0)` -> `[]`
                    if arg.is_number_0() {
                        *expr = ctx.ast.expression_array(*span, ctx.ast.vec());
                        ctx.state.changed = true;
                    }
                    // `new Array(8)` -> `Array(8)`
                    else if let Expression::NumericLiteral(n) = arg {
                        // new Array(2) -> `[,,]`
                        // this does not work with IE8 and below
                        // learned from https://github.com/babel/minify/pull/45
                        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        if n.value.fract() == 0.0 {
                            let n_int = n.value as usize;
                            if (1..=6).contains(&n_int) {
                                let elisions = repeat_with(|| {
                                    ArrayExpressionElement::Elision(ctx.ast.elision(n.span))
                                })
                                .take(n_int);
                                *expr = ctx
                                    .ast
                                    .expression_array(*span, ctx.ast.vec_from_iter(elisions));
                                ctx.state.changed = true;
                                return;
                            }
                        }
                        if is_new_expr {
                            let callee = callee.take_in(ctx.ast);
                            let args = args.take_in(ctx.ast);
                            *expr = ctx.ast.expression_call(*span, callee, NONE, args, false);
                            ctx.state.changed = true;
                        }
                    }
                    // `new Array(literal)` -> `[literal]`
                    else if arg.is_literal() || matches!(arg, Expression::ArrayExpression(_)) {
                        let elements =
                            ctx.ast.vec1(ArrayExpressionElement::from(arg.take_in(ctx.ast)));
                        *expr = ctx.ast.expression_array(*span, elements);
                        ctx.state.changed = true;
                    }
                    // `new Array(x)` -> `Array(x)`
                    else if is_new_expr {
                        let callee = callee.take_in(ctx.ast);
                        let args = args.take_in(ctx.ast);
                        *expr = ctx.ast.expression_call(*span, callee, NONE, args, false);
                        ctx.state.changed = true;
                    }
                } else {
                    // `new Array(1, 2, 3)` -> `[1, 2, 3]`
                    let elements = ctx.ast.vec_from_iter(
                        args.iter_mut()
                            .filter_map(|arg| arg.as_expression_mut())
                            .map(|arg| ArrayExpressionElement::from(arg.take_in(ctx.ast))),
                    );
                    *expr = ctx.ast.expression_array(*span, elements);
                    ctx.state.changed = true;
                }
            }
            _ => {}
        }
    }

    /// `new Error()` -> `Error()` (also for NativeErrors)
    /// `new AggregateError()` -> `AggregateError()`
    /// `new Function()` -> `Function()`
    /// `new RegExp()` -> `RegExp()`
    pub fn substitute_global_new_expression(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::NewExpression(e) = expr else { return };
        let Expression::Identifier(ident) = &e.callee else { return };
        let name = ident.name.as_str();
        if !matches!(name, "Error" | "AggregateError" | "Function" | "RegExp")
            && !Self::is_native_error_name(name)
        {
            return;
        }
        if !ctx.is_global_reference(ident) {
            return;
        }
        if match name {
            "RegExp" => {
                let arguments_len = e.arguments.len();
                arguments_len == 0
                    || (arguments_len >= 1
                        && e.arguments[0].as_expression().is_some_and(|first_argument| {
                            let ty = first_argument.value_type(ctx);
                            !ty.is_undetermined() && !ty.is_object()
                        }))
            }
            "Error" | "AggregateError" | "Function" => true,
            _ if Self::is_native_error_name(name) => true,
            _ => unreachable!(),
        } {
            *expr = ctx.ast.expression_call_with_pure(
                e.span,
                e.callee.take_in(ctx.ast),
                NONE,
                e.arguments.take_in(ctx.ast),
                false,
                e.pure,
            );
            ctx.state.changed = true;
        }
    }

    /// Whether the name matches any native error name.
    ///
    /// See <https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-native-error-types-used-in-this-standard> for the list of native errors.
    fn is_native_error_name(name: &str) -> bool {
        matches!(
            name,
            "EvalError"
                | "RangeError"
                | "ReferenceError"
                | "SyntaxError"
                | "TypeError"
                | "URIError"
        )
    }

    pub fn substitute_chain_call_expression(
        expr: &mut ChainExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let ChainElement::CallExpression(call_expr) = &mut expr.expression {
            // `window.Object?.()` -> `Object?.()`
            if call_expr.arguments.is_empty()
                && call_expr
                    .callee
                    .as_member_expression()
                    .is_some_and(|mem_expr| mem_expr.is_specific_member_access("window", "Object"))
            {
                let object = ctx.ast.ident("Object");
                let reference_id = ctx.create_unbound_reference(object, ReferenceFlags::Read);
                call_expr.callee = ctx.ast.expression_identifier_with_reference_id(
                    call_expr.callee.span(),
                    "Object",
                    reference_id,
                );
                ctx.state.changed = true;
            }
        }
    }

    pub fn substitute_template_literal(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::TemplateLiteral(t) = expr else { return };
        let Some(val) = t.to_js_string(ctx) else { return };
        *expr = ctx.ast.expression_string_literal(t.span(), ctx.ast.atom_from_cow(&val), None);
        ctx.state.changed = true;
    }

    // <https://github.com/swc-project/swc/blob/4e2dae558f60a9f5c6d2eac860743e6c0b2ec562/crates/swc_ecma_minifier/src/compress/pure/properties.rs>
    fn try_compress_property_key(
        key: &mut PropertyKey<'a>,
        computed: &mut bool,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match key {
            PropertyKey::NumericLiteral(_) => {
                if *computed {
                    *computed = false;
                }
            }
            PropertyKey::StringLiteral(s) => {
                let value = s.value.as_str();
                if TraverseCtx::is_identifier_name_patched(value) {
                    *computed = false;
                    *key = PropertyKey::StaticIdentifier(
                        ctx.ast.alloc_identifier_name(s.span, s.value),
                    );
                    ctx.state.changed = true;
                    return;
                }
                if let Some(value) = TraverseCtx::string_to_equivalent_number_value(value)
                    && value >= 0.0
                {
                    *computed = false;
                    *key = PropertyKey::NumericLiteral(ctx.ast.alloc_numeric_literal(
                        s.span,
                        value,
                        None,
                        NumberBase::Decimal,
                    ));
                    ctx.state.changed = true;
                    return;
                }
                if *computed {
                    *computed = false;
                }
            }
            _ => {}
        }
    }

    // `foo(...[1,2,3])` -> `foo(1,2,3)`
    // `new Foo(...[1,2,3])` -> `new Foo(1,2,3)`
    fn try_flatten_arguments(args: &mut Vec<'a, Argument<'a>>, ctx: &mut TraverseCtx<'a>) {
        let (new_size, should_fold) =
            args.iter().fold((0, false), |(mut new_size, mut should_fold), arg| {
                new_size += if let Argument::SpreadElement(spread_el) = arg {
                    if let Expression::ArrayExpression(array_expr) = &spread_el.argument {
                        should_fold = true;
                        array_expr.elements.len()
                    } else {
                        1
                    }
                } else {
                    1
                };

                (new_size, should_fold)
            });
        if !should_fold {
            return;
        }

        let old_args = std::mem::replace(args, ctx.ast.vec_with_capacity(new_size));
        let new_args = args;

        for arg in old_args {
            if let Argument::SpreadElement(mut spread_el) = arg {
                if let Expression::ArrayExpression(array_expr) = &mut spread_el.argument {
                    for el in &mut array_expr.elements {
                        match el {
                            ArrayExpressionElement::SpreadElement(spread_el) => {
                                new_args.push(ctx.ast.argument_spread_element(
                                    spread_el.span,
                                    spread_el.argument.take_in(ctx.ast),
                                ));
                            }
                            ArrayExpressionElement::Elision(elision) => {
                                new_args.push(ctx.ast.void_0(elision.span).into());
                            }
                            match_expression!(ArrayExpressionElement) => {
                                new_args.push(el.to_expression_mut().take_in(ctx.ast).into());
                            }
                        }
                    }
                } else {
                    new_args.push(ctx.ast.argument_spread_element(
                        spread_el.span,
                        spread_el.argument.take_in(ctx.ast),
                    ));
                }
            } else {
                new_args.push(arg);
            }
        }
        ctx.state.changed = true;
    }

    /// Flatten nested chain expressions
    /// `(foo?.bar)?.baz` -> `foo?.bar?.baz`
    fn try_flatten_nested_chain_expression(
        expr: &mut ChainExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match &mut expr.expression {
            ChainElement::StaticMemberExpression(member) => {
                if let Expression::ChainExpression(chain) = member.object.without_parentheses_mut()
                {
                    member.object = Expression::from(chain.expression.take_in(ctx.ast));
                    ctx.state.changed = true;
                }
            }
            ChainElement::ComputedMemberExpression(member) => {
                if let Expression::ChainExpression(chain) = member.object.without_parentheses_mut()
                {
                    member.object = Expression::from(chain.expression.take_in(ctx.ast));
                    ctx.state.changed = true;
                }
            }
            ChainElement::PrivateFieldExpression(member) => {
                if let Expression::ChainExpression(chain) = member.object.without_parentheses_mut()
                {
                    member.object = Expression::from(chain.expression.take_in(ctx.ast));
                    ctx.state.changed = true;
                }
            }
            ChainElement::CallExpression(call) => {
                if let Expression::ChainExpression(chain) = call.callee.without_parentheses_mut() {
                    call.callee = Expression::from(chain.expression.take_in(ctx.ast));
                    ctx.state.changed = true;
                }
            }
            ChainElement::TSNonNullExpression(_) => {
                // noop
            }
        }
    }

    /// `Object(expr)(args)` -> `(0, expr)(args)`
    ///
    /// If `expr` is `null` or `undefined`, both before and after throws an TypeError ("something is not a function").
    /// It is because `Object(expr)` returns `{}`.
    ///
    /// If `expr` is other primitive values, both before and after throws an TypeError ("something is not a function").
    /// It is because `Object(expr)` returns the Object wrapped values (e.g. `new Boolean()`).
    ///
    /// If `expr` is an object / function, `Object(expr)` returns `expr` as-is.
    /// Note that we need to wrap `expr` as `(0, expr)` so that the `this` value is preserved.
    ///
    /// <https://tc39.es/ecma262/2025/multipage/fundamental-objects.html#sec-object-value>
    fn try_rewrite_object_callee_indirect_call(
        expr: &mut CallExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::CallExpression(inner_call) = &mut expr.callee else { return };
        if inner_call.optional || inner_call.arguments.len() != 1 {
            return;
        }
        let Expression::Identifier(callee) = &inner_call.callee else {
            return;
        };
        if callee.name != "Object" || !ctx.is_global_reference(callee) {
            return;
        }

        let span = inner_call.span;
        let Some(arg_expr) = inner_call.arguments[0].as_expression_mut() else {
            return;
        };

        let new_callee = ctx.ast.expression_sequence(
            span,
            ctx.ast.vec_from_array([
                ctx.ast.expression_numeric_literal(span, 0.0, None, NumberBase::Decimal),
                arg_expr.take_in(ctx.ast),
            ]),
        );
        expr.callee = new_callee;
        ctx.state.changed = true;
    }

    /// Remove name from function expressions if it is not used.
    ///
    /// e.g. `var a = function f() {}` -> `var a = function () {}`
    ///
    /// This compression is not safe if the code relies on `Function::name`.
    pub fn try_remove_name_from_functions(func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.options().keep_names.function {
            return;
        }
        if func.id.as_ref().is_some_and(|id| ctx.scoping().symbol_is_unused(id.symbol_id())) {
            func.id = None;
            ctx.state.changed = true;
        }
    }

    /// Remove name from class expressions if it is not used.
    ///
    /// e.g. `var a = class C {}` -> `var a = class {}`
    ///
    /// This compression is not safe if the code relies on `Class::name`.
    pub fn try_remove_name_from_classes(class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.options().keep_names.class {
            return;
        }

        if class.id.as_ref().is_some_and(|id| ctx.scoping().symbol_is_unused(id.symbol_id())) {
            class.id = None;
            ctx.state.changed = true;
        }
    }

    /// `new Int8Array(0)` -> `new Int8Array()` (also for other TypedArrays)
    pub fn substitute_typed_array_constructor(e: &mut NewExpression<'a>, ctx: &TraverseCtx<'a>) {
        let Expression::Identifier(ident) = &e.callee else { return };
        let name = ident.name.as_str();
        if !Self::is_typed_array_name(name) || !ctx.is_global_reference(ident) {
            return;
        }
        if e.arguments.len() == 1
            && e.arguments[0].as_expression().is_some_and(Expression::is_number_0)
        {
            e.arguments.clear();
        }
    }

    /// Transforms boolean expression `true` => `!0` `false` => `!1`.
    pub fn substitute_boolean(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::BooleanLiteral(lit) = expr else { return };
        let num = ctx.ast.expression_numeric_literal(
            lit.span,
            if lit.value { 0.0 } else { 1.0 },
            None,
            NumberBase::Decimal,
        );
        *expr = ctx.ast.expression_unary(lit.span, UnaryOperator::LogicalNot, num);
        ctx.state.changed = true;
    }

    /// Transforms long array expression with string literals to `"str1,str2".split(',')`
    pub fn substitute_array_expression(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        // this threshold is chosen by hand by checking the minsize output
        const THRESHOLD: usize = 40;

        let Expression::ArrayExpression(array) = expr else {
            return;
        };

        let is_all_string = array.elements.iter().all(|element| {
            element.as_expression().is_some_and(|expr| matches!(expr, Expression::StringLiteral(_)))
        });
        if !is_all_string {
            return;
        }

        let element_count = array.elements.len();
        // replace with `.split` only when the saved size is great enough
        // because using `.split` in some places and not in others may cause gzipped size to be bigger
        let can_save = element_count * 2 > ".split('.')".len() + THRESHOLD;
        if !can_save {
            return;
        }

        let strings = array.elements.iter().map(|element| {
            let Expression::StringLiteral(str) = element.to_expression() else { unreachable!() };
            str.value.as_str()
        });
        let Some(delimiter) = Self::pick_delimiter(&strings) else { return };

        let concatenated_string = strings.collect::<std::vec::Vec<_>>().join(delimiter);

        // "str1,str2".split(',')
        *expr = ctx.ast.expression_call_with_pure(
            expr.span(),
            Expression::StaticMemberExpression(ctx.ast.alloc_static_member_expression(
                expr.span(),
                ctx.ast.expression_string_literal(
                    expr.span(),
                    ctx.ast.atom(&concatenated_string),
                    None,
                ),
                ctx.ast.identifier_name(expr.span(), "split"),
                false,
            )),
            NONE,
            ctx.ast.vec1(Argument::from(ctx.ast.expression_string_literal(
                expr.span(),
                ctx.ast.atom(delimiter),
                None,
            ))),
            false,
            true,
        );
        ctx.state.changed = true;
    }

    fn pick_delimiter<'s>(
        strings: &(impl Iterator<Item = &'s str> + Clone),
    ) -> Option<&'static str> {
        // These delimiters are chars that appears a lot in the program
        // therefore probably have a small Huffman encoding.
        const DELIMITERS: [&str; 5] = [".", ",", "(", ")", " "];

        let is_all_length_1 = strings.clone().all(|s| s.len() == 1);
        if is_all_length_1 {
            return Some("");
        }

        DELIMITERS.into_iter().find(|&delimiter| strings.clone().all(|s| !s.contains(delimiter)))
    }

    pub fn substitute_catch_clause(catch: &mut CatchClause<'a>, ctx: &TraverseCtx<'a>) {
        if ctx.supports_feature(ESFeature::ES2019OptionalCatchBinding)
            && let Some(param) = &catch.param
            && let BindingPattern::BindingIdentifier(ident) = &param.pattern
            && (catch.body.body.is_empty() || ctx.scoping().symbol_is_unused(ident.symbol_id()))
        {
            catch.param = None;
        }
    }

    /// Whether the name matches any TypedArray name.
    ///
    /// See <https://tc39.es/ecma262/multipage/indexed-collections.html#sec-typedarray-objects> for the list of TypedArrays.
    fn is_typed_array_name(name: &str) -> bool {
        matches!(
            name,
            "Int8Array"
                | "Uint8Array"
                | "Uint8ClampedArray"
                | "Int16Array"
                | "Uint16Array"
                | "Int32Array"
                | "Uint32Array"
                | "Float32Array"
                | "Float64Array"
                | "BigInt64Array"
                | "BigUint64Array"
        )
    }

    /// Checks if the expression result is unused (i.e., in an expression statement context).
    fn is_expression_result_unused(ctx: &TraverseCtx<'a>) -> bool {
        matches!(ctx.parent(), Ancestor::ExpressionStatementExpression(_))
    }

    /// Optimizes the usage of Immediately Invoked Function Expressions (IIFEs)
    /// within the given expression and context by performing various substitutions
    /// to clean up and simplify the code.
    ///
    /// - Replaces empty IIFEs (e.g., `(() => {})()` or `(function() {})()`) with the value `undefined`.
    /// - Simplifies single-expression non-async arrow function IIFEs (e.g., `(() => foo())()` to `foo()`).
    /// - Converts arrow function IIFEs that return void or execute one expression
    ///   (e.g., `(() => { foo() })()` or `(() => { return foo() })()`) into simpler expressions.
    pub fn substitute_iife_call(e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::CallExpression(call_expr) = e else { return };

        if !call_expr.arguments.is_empty() || !call_expr.callee.is_function() {
            return;
        }

        let is_empty_iife = match &call_expr.callee {
            Expression::FunctionExpression(f) => {
                f.params.is_empty()
                    && f.body.as_ref().is_some_and(|body| body.is_empty())
                    // ignore async/generator if a return value is not used
                    && ((!f.r#async && !f.generator) || Self::is_expression_result_unused(ctx))
            }
            Expression::ArrowFunctionExpression(f) => {
                f.params.is_empty()
                    && f.body.is_empty()
                    // ignore async if a return value is not used
                    && (!f.r#async || Self::is_expression_result_unused(ctx))
            }
            _ => false,
        };

        if is_empty_iife {
            *e = ctx.ast.void_0(call_expr.span);
            ctx.state.changed = true;
            // Replace "(() => {})()" with "undefined"
            // Replace "(function () => { return })()" with "undefined"
            return;
        }

        let is_pure =
            (call_expr.pure && ctx.annotations()) || ctx.manual_pure_functions(&call_expr.callee);

        if let Expression::ArrowFunctionExpression(f) = &mut call_expr.callee
            && !f.r#async
            && !f.params.has_parameter()
            && f.body.statements.len() == 1
        {
            if f.expression {
                // Replace "(() => foo())()" with "foo()"
                let expr = f.get_expression_mut().unwrap();
                if is_pure && Self::is_expression_result_unused(ctx) {
                    *e = ctx.ast.void_0(call_expr.span);
                } else {
                    *e = expr.take_in(ctx.ast);
                }
                ctx.state.changed = true;
                return;
            }
            match &mut f.body.statements[0] {
                Statement::ExpressionStatement(expr_stmt) => {
                    // Replace "(() => { foo() })()" with "(foo(), undefined)"
                    if is_pure && Self::is_expression_result_unused(ctx) {
                        *e = ctx.ast.void_0(call_expr.span);
                    } else {
                        *e = ctx.ast.expression_sequence(expr_stmt.span, {
                            let mut sequence = ctx.ast.vec();
                            sequence.push(expr_stmt.expression.take_in(ctx.ast));
                            sequence.push(ctx.ast.void_0(call_expr.span));
                            sequence
                        });
                    }

                    ctx.state.changed = true;
                }
                Statement::ReturnStatement(ret_stmt) => {
                    if let Some(argument) = &mut ret_stmt.argument {
                        // Replace "(() => { return foo() })()" with "foo()"
                        if is_pure && Self::is_expression_result_unused(ctx) {
                            *e = ctx.ast.void_0(call_expr.span);
                        } else {
                            *e = argument.take_in(ctx.ast);
                        }
                        ctx.state.changed = true;
                    }
                }
                _ => {}
            }
        }
    }
}

struct ClassPropertyKeyParent {
    pub ty: ClassPropertyKeyParentType,
    /// Whether the property is static.
    pub r#static: bool,
}

impl ClassPropertyKeyParent {
    /// Whether the key should be kept as a computed property to avoid early errors.
    ///
    /// <https://tc39.es/ecma262/2024/multipage/ecmascript-language-functions-and-classes.html#sec-static-semantics-classelementkind>
    /// <https://tc39.es/ecma262/2024/multipage/ecmascript-language-functions-and-classes.html#sec-class-definitions-static-semantics-early-errors>
    /// <https://arai-a.github.io/ecma262-compare/?pr=2417&id=sec-class-definitions-static-semantics-early-errors>
    fn should_keep_as_computed_property(&self, key: &str) -> bool {
        match key {
            "prototype" => self.r#static,
            "constructor" => match self.ty {
                // Uncaught SyntaxError: Class constructor may not be an accessor
                ClassPropertyKeyParentType::MethodDefinition => !self.r#static,
                // Uncaught SyntaxError: Classes may not have a field named 'constructor'
                // Uncaught SyntaxError: Class constructor may not be a private method
                ClassPropertyKeyParentType::AccessorProperty
                | ClassPropertyKeyParentType::PropertyDefinition => true,
            },
            "#constructor" => true,
            _ => false,
        }
    }
}

enum ClassPropertyKeyParentType {
    PropertyDefinition,
    AccessorProperty,
    MethodDefinition,
}

impl From<&PropertyDefinition<'_>> for ClassPropertyKeyParent {
    fn from(prop: &PropertyDefinition<'_>) -> Self {
        Self { ty: ClassPropertyKeyParentType::PropertyDefinition, r#static: prop.r#static }
    }
}

impl From<&AccessorProperty<'_>> for ClassPropertyKeyParent {
    fn from(accessor: &AccessorProperty<'_>) -> Self {
        Self { ty: ClassPropertyKeyParentType::AccessorProperty, r#static: accessor.r#static }
    }
}

impl From<&MethodDefinition<'_>> for ClassPropertyKeyParent {
    fn from(method: &MethodDefinition<'_>) -> Self {
        Self { ty: ClassPropertyKeyParentType::MethodDefinition, r#static: method.r#static }
    }
}

impl<T> From<&mut T> for ClassPropertyKeyParent
where
    ClassPropertyKeyParent: for<'a> std::convert::From<&'a T>,
{
    fn from(prop: &mut T) -> Self {
        (&*prop).into()
    }
}
