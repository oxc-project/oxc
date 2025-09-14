use std::iter::repeat_with;

use oxc_allocator::{CloneIn, TakeIn, Vec};
use oxc_ast::{NONE, ast::*};
use oxc_ecmascript::constant_evaluation::{ConstantEvaluation, ConstantValue, DetermineValueType};
use oxc_ecmascript::{ToJsString, ToNumber, side_effects::MayHaveSideEffects};
use oxc_semantic::ReferenceFlags;
use oxc_span::GetSpan;
use oxc_span::SPAN;
use oxc_syntax::{
    es_target::ESTarget,
    number::NumberBase,
    operator::{BinaryOperator, UnaryOperator},
};
use oxc_traverse::Ancestor;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

/// A peephole optimization that minimizes code by simplifying conditional
/// expressions, replacing IFs with HOOKs, replacing object constructors
/// with literals, and simplifying returns.
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeSubstituteAlternateSyntax.java>
impl<'a> PeepholeOptimizations {
    pub fn substitute_object_property(prop: &mut ObjectProperty<'a>, ctx: &mut Ctx<'a, '_>) {
        // <https://tc39.es/ecma262/2024/multipage/ecmascript-language-expressions.html#sec-runtime-semantics-propertydefinitionevaluation>
        if !prop.method {
            if let PropertyKey::StringLiteral(str) = &prop.key {
                // "{ __proto__ }" sets prototype, while "{ ['__proto__'] }" does not
                if str.value == "__proto__" {
                    return;
                }
            }
        }

        Self::try_compress_property_key(&mut prop.key, &mut prop.computed, ctx);
    }

    pub fn substitute_assignment_target_property_property(
        prop: &mut AssignmentTargetPropertyProperty<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        Self::try_compress_property_key(&mut prop.name, &mut prop.computed, ctx);
    }

    pub fn substitute_assignment_target_property(
        prop: &mut AssignmentTargetProperty<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        Self::try_compress_assignment_target_property(prop, ctx);
    }

    pub fn try_compress_assignment_target_property(
        prop: &mut AssignmentTargetProperty<'a>,
        ctx: &mut Ctx<'a, '_>,
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

    pub fn substitute_binding_property(prop: &mut BindingProperty<'a>, ctx: &mut Ctx<'a, '_>) {
        Self::try_compress_property_key(&mut prop.key, &mut prop.computed, ctx);
    }

    pub fn substitute_method_definition(prop: &mut MethodDefinition<'a>, ctx: &mut Ctx<'a, '_>) {
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
        ctx: &mut Ctx<'a, '_>,
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

    pub fn substitute_accessor_property(prop: &mut AccessorProperty<'a>, ctx: &mut Ctx<'a, '_>) {
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

    pub fn substitute_for_statement(stmt: &mut ForStatement<'a>, ctx: &mut Ctx<'a, '_>) {
        Self::try_rewrite_arguments_copy_loop(stmt, ctx);
    }

    pub fn substitute_variable_declaration(
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        for declarator in &mut decl.declarations {
            Self::compress_variable_declarator(declarator, ctx);
        }
    }

    pub fn substitute_call_expression(expr: &mut CallExpression<'a>, ctx: &mut Ctx<'a, '_>) {
        Self::try_flatten_arguments(&mut expr.arguments, ctx);
        Self::try_rewrite_object_callee_indirect_call(expr, ctx);
    }

    pub fn substitute_new_expression(expr: &mut NewExpression<'a>, ctx: &mut Ctx<'a, '_>) {
        Self::try_flatten_arguments(&mut expr.arguments, ctx);
    }

    pub fn substitute_chain_expression(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
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
        ctx: &mut Ctx<'a, '_>,
    ) {
        if !arrow_expr.expression
            && arrow_expr.body.directives.is_empty()
            && arrow_expr.body.statements.len() == 1
        {
            if let Some(body) = arrow_expr.body.statements.first_mut() {
                if let Statement::ReturnStatement(ret_stmt) = body {
                    let return_stmt_arg =
                        ret_stmt.argument.as_mut().map(|arg| arg.take_in(ctx.ast));
                    if let Some(arg) = return_stmt_arg {
                        *body = ctx.ast.statement_expression(arg.span(), arg);
                        arrow_expr.expression = true;
                        ctx.state.changed = true;
                    }
                }
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
    pub fn substitute_typeof_undefined(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
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
    pub fn substitute_unary_plus(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
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
    pub fn substitute_rotate_logical_expression(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
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
    pub fn substitute_is_object_and_not_null(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
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
        let Some(new_expr) = Self::try_compress_is_object_and_not_null_for_left_and_right(
            &left.right,
            &e.right,
            Span::new(left.right.span().start, e.span.end),
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
        ctx: &Ctx<'a, '_>,
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

    pub fn substitute_loose_equals_undefined(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
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
    fn try_rewrite_arguments_copy_loop(for_stmt: &mut ForStatement<'a>, ctx: &mut Ctx<'a, '_>) {
        /// Verify whether `arg_expr` is `e > offset ? e - offset : 0` or `e`
        fn verify_array_arg(arg_expr: &Expression, name_e: &str, offset: f64) -> bool {
            match arg_expr {
                Expression::Identifier(id) => offset == 0.0 && id.name == name_e,
                Expression::ConditionalExpression(cond_expr) => {
                    let Expression::BinaryExpression(test_expr) = &cond_expr.test else {
                        return false;
                    };
                    let Expression::BinaryExpression(cons_expr) = &cond_expr.consequent else {
                        return false;
                    };
                    test_expr.operator == BinaryOperator::GreaterThan
                        && test_expr.left.is_specific_id(name_e)
                        && matches!(&test_expr.right, Expression::NumericLiteral(n) if n.value == offset)
                        && cons_expr.operator == BinaryOperator::Subtraction
                        && matches!(&cons_expr.left, Expression::Identifier(id) if id.name == name_e)
                        && matches!(&cons_expr.right, Expression::NumericLiteral(n) if n.value == offset)
                        && matches!(&cond_expr.alternate, Expression::NumericLiteral(n) if n.value == 0.0)
                }
                _ => false,
            }
        }

        // FIXME: this function treats `arguments` not inside a function scope as if they are inside it
        //        we should check in a different way than `ctx.is_global_reference`

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
        };

        // Parse test: `a < e` or `a < arguments.length`
        let e_id_name = {
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
                Expression::Identifier(right) => Some(&right.name),
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

        let init_decl_len = if e_id_name.is_some() { 3 } else { 2 };

        let Some(init) = &mut for_stmt.init else { return };
        let ForStatementInit::VariableDeclaration(var_init) = init else { return };
        // Need at least two declarators: r, a (optional `e` may precede them)
        if var_init.declarations.len() < init_decl_len {
            return;
        }

        let mut idx = 0usize;

        // Check `e = arguments.length`
        if let Some(e_id_name) = e_id_name {
            let de = var_init
                .declarations
                .get(idx)
                .expect("var_init.declarations.len() check above ensures this");
            let BindingPatternKind::BindingIdentifier(de_id) = &de.id.kind else { return };
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
        {
            let de_a = var_init
                .declarations
                .get(idx + 1)
                .expect("var_init.declarations.len() check above ensures this");
            let BindingPatternKind::BindingIdentifier(de_id) = &de_a.id.kind else { return };
            if de_id.name != a_id_name {
                return;
            }
            if !matches!(&de_a.init, Some(Expression::NumericLiteral(n)) if n.value == offset) {
                return;
            }
        }

        // Check `r = Array(e > 1 ? e - 1 : 0)`, or `r = []`
        let r_id_pat = {
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
                    let Some(e_id_name) = e_id_name else { return };
                    let Some(arg_expr) = call.arguments[0].as_expression() else { return };
                    if !verify_array_arg(arg_expr, e_id_name, offset) {
                        return;
                    }
                }
                Some(Expression::ArrayExpression(arr)) => {
                    if !arr.elements.is_empty() {
                        return;
                    }
                }
                _ => return,
            }
            let BindingPatternKind::BindingIdentifier(de_id) = &de_r.id.kind else { return };
            if de_id.name != r_id_name {
                return;
            }
            // `var r = [...arguments]` / `var r = [...arguments].slice(n)` is not needed
            // if r is not used by other places because `[...arguments]` does not have a sideeffect
            // `r` is used once in the for-loop (assignment for each index)
            (ctx.scoping().get_resolved_references(de_id.symbol_id()).count() > 1)
                .then(|| de_r.id.take_in(ctx.ast))
        };

        // Build `var r = [...arguments]` (with optional `.slice(offset)`) as the only declarator and drop test/update/body.

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
                Option::<TSTypeParameterInstantiation>::None,
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
                ctx.ast.variable_declarator(SPAN, var_init.kind, r_id_pat, Some(arr), false);
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
    pub fn substitute_return_statement(stmt: &mut ReturnStatement<'a>, ctx: &mut Ctx<'a, '_>) {
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

    fn compress_variable_declarator(decl: &mut VariableDeclarator<'a>, ctx: &mut Ctx<'a, '_>) {
        // Destructuring Pattern has error throwing side effect.
        if matches!(
            decl.kind,
            VariableDeclarationKind::Const
                | VariableDeclarationKind::Using
                | VariableDeclarationKind::AwaitUsing
        ) || decl.id.kind.is_destructuring_pattern()
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
    pub fn substitute_simple_function_call(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
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
    fn get_fold_constructor_name(callee: &Expression<'a>, ctx: &Ctx<'a, '_>) -> Option<&'a str> {
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
        ctx: &mut Ctx<'a, '_>,
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
    pub fn substitute_global_new_expression(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
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

    pub fn substitute_chain_call_expression(expr: &mut ChainExpression<'a>, ctx: &mut Ctx<'a, '_>) {
        if let ChainElement::CallExpression(call_expr) = &mut expr.expression {
            // `window.Object?.()` -> `Object?.()`
            if call_expr.arguments.is_empty()
                && call_expr
                    .callee
                    .as_member_expression()
                    .is_some_and(|mem_expr| mem_expr.is_specific_member_access("window", "Object"))
            {
                let reference_id = ctx.create_unbound_reference("Object", ReferenceFlags::Read);
                call_expr.callee = ctx.ast.expression_identifier_with_reference_id(
                    call_expr.callee.span(),
                    "Object",
                    reference_id,
                );
                ctx.state.changed = true;
            }
        }
    }

    pub fn substitute_template_literal(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::TemplateLiteral(t) = expr else { return };
        let Some(val) = t.to_js_string(ctx) else { return };
        *expr = ctx.ast.expression_string_literal(t.span(), ctx.ast.atom_from_cow(&val), None);
        ctx.state.changed = true;
    }

    // <https://github.com/swc-project/swc/blob/4e2dae558f60a9f5c6d2eac860743e6c0b2ec562/crates/swc_ecma_minifier/src/compress/pure/properties.rs>
    fn try_compress_property_key(
        key: &mut PropertyKey<'a>,
        computed: &mut bool,
        ctx: &mut Ctx<'a, '_>,
    ) {
        match key {
            PropertyKey::NumericLiteral(_) => {
                if *computed {
                    *computed = false;
                }
            }
            PropertyKey::StringLiteral(s) => {
                let value = s.value.as_str();
                if Ctx::is_identifier_name_patched(value) {
                    *computed = false;
                    *key = PropertyKey::StaticIdentifier(
                        ctx.ast.alloc_identifier_name(s.span, s.value),
                    );
                    ctx.state.changed = true;
                    return;
                }
                if let Some(value) = Ctx::string_to_equivalent_number_value(value) {
                    if value >= 0.0 {
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
    fn try_flatten_arguments(args: &mut Vec<'a, Argument<'a>>, ctx: &mut Ctx<'a, '_>) {
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
    fn try_flatten_nested_chain_expression(expr: &mut ChainExpression<'a>, ctx: &mut Ctx<'a, '_>) {
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
        ctx: &mut Ctx<'a, '_>,
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
    pub fn try_remove_name_from_functions(func: &mut Function<'a>, ctx: &mut Ctx<'a, '_>) {
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
    pub fn try_remove_name_from_classes(class: &mut Class<'a>, ctx: &mut Ctx<'a, '_>) {
        if ctx.options().keep_names.class {
            return;
        }

        if class.id.as_ref().is_some_and(|id| ctx.scoping().symbol_is_unused(id.symbol_id())) {
            class.id = None;
            ctx.state.changed = true;
        }
    }

    /// `new Int8Array(0)` -> `new Int8Array()` (also for other TypedArrays)
    pub fn substitute_typed_array_constructor(e: &mut NewExpression<'a>, ctx: &Ctx<'a, '_>) {
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
    pub fn substitute_boolean(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
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
    pub fn substitute_array_expression(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
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
            Option::<TSTypeParameterInstantiation>::None,
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

    pub fn substitute_catch_clause(catch: &mut CatchClause<'a>, ctx: &Ctx<'a, '_>) {
        if ctx.options().target >= ESTarget::ES2019 {
            if let Some(param) = &catch.param {
                if let BindingPatternKind::BindingIdentifier(ident) = &param.pattern.kind {
                    if catch.body.body.is_empty()
                        || ctx.scoping().symbol_is_unused(ident.symbol_id())
                    {
                        catch.param = None;
                    }
                }
            }
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

/// Port from <https://github.com/google/closure-compiler/blob/v20240609/test/com/google/javascript/jscomp/PeepholeSubstituteAlternateSyntaxTest.java>
#[cfg(test)]
mod test {
    use oxc_syntax::es_target::ESTarget;

    use crate::{
        CompressOptions, CompressOptionsUnused,
        options::CompressOptionsKeepNames,
        tester::{default_options, test, test_options, test_same, test_same_options},
    };

    #[test]
    fn test_fold_return_result() {
        test("function f(){return !1;}", "function f(){return !1}");
        test("function f(){return null;}", "function f(){return null}");
        test("function f(){return void 0;}", "function f(){}");
        test("function f(){return void foo();}", "function f(){foo()}");
        test("function f(){return undefined;}", "function f(){}");
        test("function f(){if(a()){return undefined;}}", "function f(){a()}");
        test_same("function a(undefined) { return undefined; }");
        test_same("function f(){return foo()}");

        // `return undefined` has a different semantic in async generator function.
        test("function foo() { return undefined }", "function foo() { }");
        test("function* foo() { return undefined }", "function* foo() { }");
        test("async function foo() { return undefined }", "async function foo() { }");
        test_same("async function* foo() { return void 0 }");
        test_same("class Foo { async * foo() { return void 0 } }");
        test(
            "async function* foo() { function bar () { return void 0 } return bar }",
            "async function* foo() { function bar () {} return bar }",
        );
        test(
            "async function* foo() { let bar = () => { return void 0 }; return bar }",
            "async function* foo() { return () => {} }",
        );
    }

    #[test]
    fn test_undefined() {
        test("let x = undefined", "let x");
        test("const x = undefined", "const x = void 0");
        test("var x = undefined", "var x = void 0");
        test_same("var undefined = 1;function f() {var undefined=2,x;}");
        test_same("function f(undefined) {}");
        test_same("try { foo } catch(undefined) {foo(undefined)}");
        test("for (undefined in {}) {}", "for(undefined in {});");
        test("undefined++;", "undefined++");
        test("undefined += undefined;", "undefined+=void 0");
        // shadowed
        test_same("(function(undefined) { let x = typeof undefined; })()");
        // destructuring throw error side effect
        test_same("var {} = void 0");
        test_same("var [] = void 0");
        // `delete undefined` returns `false`
        // `delete void 0` returns `true`
        test_same("delete undefined");
    }

    #[test]
    fn test_fold_true_false_comparison() {
        test("v = x == true", "v = x == 1");
        test("v = x == false", "v = x == 0");
        test("v = x != true", "v = x != 1");
        test("v = x < true", "v = x < !0");
        test("v = x <= true", "v = x <= !0");
        test("v = x > true", "v = x > !0");
        test("v = x >= true", "v = x >= !0");

        test("v = x instanceof true", "v = x instanceof !0");
        test("v = x + false", "v = x + !1");

        // Order: should perform the nearest.
        test("v = x == x instanceof false", "v = x == x instanceof !1");
        test("v = x in x >> true", "v = x in x >> !0");
        test("v = x == fake(false)", "v = x == fake(!1)");

        // The following should not be folded.
        test("v = x === true", "v = x === !0");
        test("v = x !== false", "v = x !== !1");
    }

    /// Based on https://github.com/terser/terser/blob/58ba5c163fa1684f2a63c7bc19b7ebcf85b74f73/test/compress/assignment.js
    #[test]
    fn test_fold_normal_assignment_to_combined_assignment() {
        test("x = x + 3", "x += 3");
        test("x = x - 3", "x -= 3");
        test("x = x / 3", "x /= 3");
        test("x = x * 3", "x *= 3");
        test("x = x >> 3", "x >>= 3");
        test("x = x << 3", "x <<= 3");
        test("x = x >>> 3", "x >>>= 3");
        test("x = x | 3", "x |= 3");
        test("x = x ^ 3", "x ^= 3");
        test("x = x % 3", "x %= 3");
        test("x = x & 3", "x &= 3");
        test("x = x + g()", "x += g()");
        test("x = x - g()", "x -= g()");
        test("x = x / g()", "x /= g()");
        test("x = x * g()", "x *= g()");
        test("x = x >> g()", "x >>= g()");
        test("x = x << g()", "x <<= g()");
        test("x = x >>> g()", "x >>>= g()");
        test("x = x | g()", "x |= g()");
        test("x = x ^ g()", "x ^= g()");
        test("x = x % g()", "x %= g()");
        test("x = x & g()", "x &= g()");

        test_same("x = 3 + x");
        test_same("x = 3 - x");
        test_same("x = 3 / x");
        test_same("x = 3 * x");
        test_same("x = 3 >> x");
        test_same("x = 3 << x");
        test_same("x = 3 >>> x");
        test_same("x = 3 | x");
        test_same("x = 3 ^ x");
        test_same("x = 3 % x");
        test_same("x = 3 & x");
        test_same("x = g() + x");
        test_same("x = g() - x");
        test_same("x = g() / x");
        test_same("x = g() * x");
        test_same("x = g() >> x");
        test_same("x = g() << x");
        test_same("x = g() >>> x");
        test_same("x = g() | x");
        test_same("x = g() ^ x");
        test_same("x = g() % x");
        test_same("x = g() & x");

        test_same("x = (x -= 2) ^ x");

        // GetValue(x) has no sideeffect when x is a resolved identifier
        test("var x; x.y = x.y + 3", "var x; x.y += 3");
        test("var x; x.#y = x.#y + 3", "var x; x.#y += 3");
        test_same("x.y = x.y + 3");
        // this can be compressed if `y` does not have side effect
        test_same("var x; x[y] = x[y] + 3");
        // GetValue(x) has a side effect in this case
        // Example case: `var a = { get b() { console.log('b'); return { get c() { console.log('c') } } } }; a.b.c = a.b.c + 1`
        test_same("var x; x.y.z = x.y.z + 3");
        // This case is not supported, since the minifier does not support with statements
        // test_same("var x; with (z) { x.y || (x.y = 3) }");
    }

    #[test]
    fn test_fold_subtraction_assignment() {
        test("x -= 1", "--x");
        test("x -= -1", "++x");
        test_same("x -= 2");
        test_same("x += 1"); // The string concatenation may be triggered, so we don't fold this.
        test_same("x += -1");
    }

    #[test]
    fn test_fold_literal_object_constructors() {
        test("x = new Object", "x = ({})");
        test("x = new Object()", "x = ({})");
        test("x = Object()", "x = ({})");

        test_same("x = (function (){function Object(){this.x=4}return new Object();})();");

        test("x = new window.Object", "x = ({})");
        test("x = new window.Object()", "x = ({})");

        // Mustn't fold optional chains
        test("x = window.Object()", "x = ({})");
        test("x = window.Object?.()", "x = Object?.()");

        test(
            "x = (function (){function Object(){this.x=4};return new window.Object;})();",
            "x = (function (){function Object(){this.x=4}return {};})();",
        );
    }

    #[test]
    fn test_fold_literal_array_constructors() {
        test("x = new Array", "x = []");
        test("x = new Array()", "x = []");
        test("x = Array()", "x = []");
        // do not fold optional chains
        test_same("x = Array?.()");

        // One argument
        test("x = new Array(0)", "x = []");
        test("x = new Array(\"a\")", "x = [\"a\"]");
        test("x = new Array(1)", "x = [,]");
        test("x = new Array(6)", "x = [,,,,,,]");
        test("x = new Array(7)", "x = Array(7)");
        test("x = new Array(7n)", "x = [7n]");
        test("x = new Array(y)", "x = Array(y)");
        test("x = new Array(foo())", "x = Array(foo())");
        test("x = Array(0)", "x = []");
        test("x = Array(\"a\")", "x = [\"a\"]");
        test_same("x = Array(7)");
        test_same("x = Array(y)");
        test_same("x = Array(foo())");

        // 1+ arguments
        test("x = new Array(1, 2, 3, 4)", "x = [1, 2, 3, 4]");
        test("x = Array(1, 2, 3, 4)", "x = [1, 2, 3, 4]");
        test("x = new Array('a', 1, 2, 'bc', 3, {}, 'abc')", "x = ['a', 1, 2, 'bc', 3, {}, 'abc']");
        test("x = Array('a', 1, 2, 'bc', 3, {}, 'abc')", "x = ['a', 1, 2, 'bc', 3, {}, 'abc']");
        test("x = new Array(Array(1, '2', 3, '4'))", "x = [[1, '2', 3, '4']]");
        test("x = Array(Array(1, '2', 3, '4'))", "x = [[1, '2', 3, '4']]");
        test(
            "x = new Array(Object(), Array(\"abc\", Object(), Array(Array())))",
            "x = [{}, [\"abc\", {}, [[]]]]",
        );
        test(
            "x = new Array(Object(), Array(\"abc\", Object(), Array(Array())))",
            "x = [{}, [\"abc\", {}, [[]]]]",
        );
    }

    #[test]
    fn test_fold_new_expressions() {
        test("let _ = new Error()", "let _ = /* @__PURE__ */ Error()");
        test("let _ = new Error('a')", "let _ = /* @__PURE__ */ Error('a')");
        test("let _ = new Error('a', { cause: b })", "let _ = Error('a', { cause: b })");
        test_same("var Error; new Error()");
        test("let _ = new EvalError()", "let _ = /* @__PURE__ */ EvalError()");
        test("let _ = new RangeError()", "let _ = /* @__PURE__ */ RangeError()");
        test("let _ = new ReferenceError()", "let _ = /* @__PURE__ */ ReferenceError()");
        test("let _ = new SyntaxError()", "let _ = /* @__PURE__ */ SyntaxError()");
        test("let _ = new TypeError()", "let _ = /* @__PURE__ */ TypeError()");
        test("let _ = new URIError()", "let _ = /* @__PURE__ */ URIError()");
        test("let _ = new AggregateError('a')", "let _ = /* @__PURE__ */ AggregateError('a')");

        test("new Function()", "Function()");
        test(
            "new Function('a', 'b', 'console.log(a, b)')",
            "Function('a', 'b', 'console.log(a, b)')",
        );
        test_same("var Function; new Function()");

        test("new RegExp()", "");
        test("new RegExp('a')", "");
        test("new RegExp(0)", "");
        test("new RegExp(null)", "");
        test("x = new RegExp('a', 'g')", "x = RegExp('a', 'g')");
        test_same("new RegExp(foo)");
        test("new RegExp(/foo/)", "");
    }

    #[test]
    fn test_compress_typed_array_constructor() {
        test("new Int8Array(0)", "new Int8Array()");
        test("new Uint8Array(0)", "new Uint8Array()");
        test("new Uint8ClampedArray(0)", "new Uint8ClampedArray()");
        test("new Int16Array(0)", "new Int16Array()");
        test("new Uint16Array(0)", "new Uint16Array()");
        test("new Int32Array(0)", "new Int32Array()");
        test("new Uint32Array(0)", "new Uint32Array()");
        test("new Float32Array(0)", "new Float32Array()");
        test("new Float64Array(0)", "new Float64Array()");
        test("new BigInt64Array(0)", "new BigInt64Array()");
        test("new BigUint64Array(0)", "new BigUint64Array()");

        test_same("var Int8Array; new Int8Array(0)");
        test_same("new Int8Array(1)");
        test_same("new Int8Array(a)");
        test_same("new Int8Array(0, a)");
    }

    #[test]
    fn test_string_array_splitting() {
        const REPEAT: usize = 20;
        let additional_args = ",'1'".repeat(REPEAT);
        let test_with_longer_args =
            |source_text_partial: &str, expected_partial: &str, delimiter: &str| {
                let expected = &format!(
                    "var x=/* @__PURE__ */'{expected_partial}{}'.split('{delimiter}')",
                    format!("{delimiter}1").repeat(REPEAT)
                );
                test(&format!("var x=[{source_text_partial}{additional_args}]"), expected);
            };
        let test_same_with_longer_args = |source_text_partial: &str| {
            test_same(&format!("var x=[{source_text_partial}{additional_args}]"));
        };

        test_same_with_longer_args("'1','2','3','4'");
        test_same_with_longer_args("'1','2','3','4','5'");
        test_same_with_longer_args("`1${a}`,'2','3','4','5','6'");
        test_with_longer_args("'1','2','3','4','5','6'", "123456", "");
        test_with_longer_args("'1','2','3','4','5','00'", "1.2.3.4.5.00", ".");
        test_with_longer_args("'1','2','3','4','5','6','7'", "1234567", "");
        test_with_longer_args("'1','2','3','4','5','6','00'", "1.2.3.4.5.6.00", ".");
        test_with_longer_args("'.,',',',',',',',',',','", ".,(,(,(,(,(,", "(");
        test_with_longer_args("',,','.',',',',',',',','", ",,(.(,(,(,(,", "(");
        test_with_longer_args("'a,','.',',',',',',',','", "a,(.(,(,(,(,", "(");
        test_with_longer_args("`1`,'2','3','4','5','6'", "123456", "");

        // all possible delimiters used, leave it alone
        test_same_with_longer_args("'.', ',', '(', ')', ' '");

        test_options(
            &format!("var x=['1','2','3','4','5','6'{additional_args}]"),
            "",
            &CompressOptions { unused: CompressOptionsUnused::Remove, ..default_options() },
        );
    }

    #[test]
    fn test_template_string_to_string() {
        test("x = `abcde`", "x = 'abcde'");
        test("x = `ab cd ef`", "x = 'ab cd ef'");
        test_same("x = `hello ${name}`");
        test_same("tag `hello ${name}`");
        test_same("tag `hello`");
        test("x = `hello ${'foo'}`", "x = 'hello foo'");
        test("x = `${2} bananas`", "x = '2 bananas'");
        test("x = `This is ${true}`", "x = 'This is true'");
    }

    #[test]
    #[ignore = "TODO: Function.bind to Function.call optimization not yet implemented"]
    fn test_bind_to_call() {
        test("((function(){}).bind())()", "((function(){}))()");
        test("((function(){}).bind(a))()", "((function(){})).call(a)");
        test("((function(){}).bind(a,b))()", "((function(){})).call(a,b)");

        test("((function(){}).bind())(a)", "((function(){}))(a)");
        test("((function(){}).bind(a))(b)", "((function(){})).call(a,b)");
        test("((function(){}).bind(a,b))(c)", "((function(){})).call(a,b,c)");

        // Without using type information we don't know "f" is a function.
        test_same("(f.bind())()");
        test_same("(f.bind(a))()");
        test_same("(f.bind())(a)");
        test_same("(f.bind(a))(b)");
    }

    // FIXME: the cases commented out can be implemented
    #[test]
    fn test_rotate_associative_operators() {
        test("a || (b || c)", "(a || b) || c");
        // float multiplication is not always associative
        // <https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-numeric-types-number-multiply>
        test_same("a * (b * c)");
        // test("a | (b | c)", "(a | b) | c");
        test_same("a % (b % c)");
        test_same("a / (b / c)");
        test_same("a - (b - c);");
        // test("a * (b % c);", "b % c * a");
        // test("a * (b / c);", "b / c * a");
        // cannot transform to `c / d * a * b`
        test_same("a * b * (c / d)");
        // test("(a + b) * (c % d)", "c % d * (a + b)");
        test_same("(a / b) * (c % d)");
        test_same("(c = 5) * (c % d)");
        // test("!a * c * (d % e)", "d % e * c * !a");
    }

    #[test]
    fn nullish_coalesce() {
        test("a ?? (b ?? c);", "(a ?? b) ?? c");
    }

    #[test]
    fn test_fold_arrow_function_return() {
        test("const foo = () => { return 'baz' }", "const foo = () => 'baz'");
        test("const foo = () => { foo.foo; return 'baz' }", "const foo = () => (foo.foo, 'baz')");
    }

    #[test]
    fn test_fold_is_typeof_equals_undefined_resolved() {
        test("var x; v = typeof x !== 'undefined'", "var x; v = x !== void 0");
        test("var x; v = typeof x != 'undefined'", "var x; v = x !== void 0");
        test("var x; v = 'undefined' !== typeof x", "var x; v = x !== void 0");
        test("var x; v = 'undefined' != typeof x", "var x; v = x !== void 0");

        test("var x; v = typeof x === 'undefined'", "var x; v = x === void 0");
        test("var x; v = typeof x == 'undefined'", "var x; v = x === void 0");
        test("var x; v = 'undefined' === typeof x", "var x; v = x === void 0");
        test("var x; v = 'undefined' == typeof x", "var x; v = x === void 0");

        test(
            "var x; function foo() { v = typeof x !== 'undefined' }",
            "var x; function foo() { v = x !== void 0 }",
        );
        test(
            "v = typeof x !== 'undefined'; function foo() { var x }",
            "v = typeof x < 'u'; function foo() { var x }",
        );
        test("v = typeof x !== 'undefined'; { var x }", "v = x !== void 0; var x;");
        test("v = typeof x !== 'undefined'; { let x }", "v = typeof x < 'u'; { let x }");
        test("v = typeof x !== 'undefined'; var x", "v = x !== void 0; var x");
        // input and output both errors with same TDZ error
        test("v = typeof x !== 'undefined'; let x", "v = x !== void 0; let x");

        test("v = typeof x.y === 'undefined'", "v = x.y === void 0");
        test("v = typeof x.y !== 'undefined'", "v = x.y !== void 0");
        test("v = typeof (x + '') === 'undefined'", "v = x + '' === void 0");
    }

    /// Port from <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_parser/js_parser_test.go#L4658>
    #[test]
    fn test_fold_is_typeof_equals_undefined() {
        test("v = typeof x !== 'undefined'", "v = typeof x < 'u'");
        test("v = typeof x != 'undefined'", "v = typeof x < 'u'");
        test("v = 'undefined' !== typeof x", "v = typeof x < 'u'");
        test("v = 'undefined' != typeof x", "v = typeof x < 'u'");

        test("v = typeof x === 'undefined'", "v = typeof x > 'u'");
        test("v = typeof x == 'undefined'", "v = typeof x > 'u'");
        test("v = 'undefined' === typeof x", "v = typeof x > 'u'");
        test("v = 'undefined' == typeof x", "v = typeof x > 'u'");
    }

    #[test]
    fn test_fold_is_object_and_not_null() {
        test(
            "var foo; v = typeof foo === 'object' && foo !== null",
            "var foo; v = typeof foo == 'object' && !!foo",
        );
        test(
            "var foo; v = typeof foo == 'object' && foo !== null",
            "var foo; v = typeof foo == 'object' && !!foo",
        );
        test(
            "var foo; v = typeof foo === 'object' && foo != null",
            "var foo; v = typeof foo == 'object' && !!foo",
        );
        test(
            "var foo; v = typeof foo == 'object' && foo != null",
            "var foo; v = typeof foo == 'object' && !!foo",
        );
        test(
            "var foo; v = typeof foo !== 'object' || foo === null",
            "var foo; v = typeof foo != 'object' || !foo",
        );
        test(
            "var foo; v = typeof foo != 'object' || foo === null",
            "var foo; v = typeof foo != 'object' || !foo",
        );
        test(
            "var foo; v = typeof foo !== 'object' || foo == null",
            "var foo; v = typeof foo != 'object' || !foo",
        );
        test(
            "var foo; v = typeof foo != 'object' || foo == null",
            "var foo; v = typeof foo != 'object' || !foo",
        );
        test(
            "var foo, bar; v = typeof foo === 'object' && foo !== null && bar !== 1",
            "var foo, bar; v = typeof foo == 'object' && !!foo && bar !== 1",
        );
        test(
            "var foo, bar; v = bar !== 1 && typeof foo === 'object' && foo !== null",
            "var foo, bar; v = bar !== 1 && typeof foo == 'object' && !!foo",
        );
        test_same("var foo; v = typeof foo.a == 'object' && foo.a !== null"); // cannot be folded because accessing foo.a might have a side effect
        test_same("v = foo !== null && typeof foo == 'object'"); // cannot be folded because accessing foo might have a side effect
        test_same("v = typeof foo == 'object' && foo !== null"); // cannot be folded because accessing foo might have a side effect
        test_same("var foo, bar; v = typeof foo == 'object' && bar !== null");
        test_same("var foo; v = typeof foo == 'string' && foo !== null");
    }

    #[test]
    fn test_swap_binary_expressions() {
        test_same("v = a === 0");
        test("v = 0 === a", "v = a === 0");
        test_same("v = a === '0'");
        test("v = '0' === a", "v = a === '0'");
        test("v = a === `0`", "v = a === '0'");
        test("v = `0` === a", "v = a === '0'");
        test_same("v = a === void 0");
        test("v = void 0 === a", "v = a === void 0");

        test_same("v = a !== 0");
        test("v = 0 !== a", "v = a !== 0");
        test_same("v = a == 0");
        test("v = 0 == a", "v = a == 0");
        test_same("v = a != 0");
        test("v = 0 != a", "v = a != 0");
    }

    #[test]
    fn test_remove_unary_plus() {
        test("v = 1 - +foo", "v = 1 - foo");
        test("v = +foo - 1", "v = foo - 1");
        test_same("v = 1n - +foo");
        test_same("v = +foo - 1n");
        test_same("v = +foo - bar");
        test_same("v = foo - +bar");
        test_same("v = 1 + +foo"); // cannot compress into `1 + foo` because `foo` can be a string

        test("v = +d / 1000", "v = d / 1000");
        test("v = 1000 * +d", "v = 1000 * d");
        test("v = +d * 1000", "v = d * 1000");
        test("v = 2 - +this._x.call(null, node.data)", "v = 2 - this._x.call(null, node.data)");

        test("v = 5 | +b", "v = 5 | b");
        test("v = +b | 5", "v = b | 5");
        test("v = 7 & +c", "v = 7 & c");
        test("v = 3 ^ +d", "v = 3 ^ d");
        // Don't remove - unsafe for BigInt operations
        test_same("v = a - +b");
        test_same("v = +a - b");
        test_same("v = a | +b");
        test_same("v = +a | b");
    }

    #[test]
    fn test_fold_loose_equals_undefined() {
        test_same("v = foo != null");
        test("v = foo != undefined", "v = foo != null");
        test("v = foo != void 0", "v = foo != null");
        test("v = undefined != foo", "v = foo != null");
        test("v = void 0 != foo", "v = foo != null");
    }

    #[test]
    fn test_property_key() {
        // Object Property
        test(
            "v = { '0': _, 'a': _, [1]: _, ['1']: _, ['b']: _, ['c.c']: _, '1.1': _, '😊': _, 'd.d': _ }",
            "v = {  0: _,   a: _,    1: _,     1: _,     b: _,   'c.c': _, '1.1': _, '😊': _, 'd.d': _ }",
        );
        // AssignmentTargetPropertyProperty
        test(
            "({ '0': _, 'a': _, [1]: _, ['1']: _, ['b']: _, ['c.c']: _, '1.1': _, '😊': _, 'd.d': _ } = {})",
            "({  0: _,   a: _,    1: _,   1: _,     b: _,   'c.c': _, '1.1': _, '😊': _, 'd.d': _ } = {})",
        );
        // Binding Property
        test(
            "var { '0': _, 'a': _, [1]: _, ['1']: _, ['b']: _, ['c.c']: _, '1.1': _, '😊': _, 'd.d': _ } = {}",
            "var {  0: _,   a: _,    1: _,   1: _,     b: _,   'c.c': _, '1.1': _, '😊': _, 'd.d': _ } = {}",
        );
        // Method Definition
        test(
            "class F { '0'(){}; 'a'(){}; [1](){}; ['1'](){}; ['b'](){}; ['c.c'](){}; '1.1'(){}; '😊'(){}; 'd.d'(){} }",
            "class F {  0(){};   a(){};    1(){};    1(){};     b(){};   'c.c'(){}; '1.1'(){}; '😊'(){}; 'd.d'(){} }",
        );
        // Property Definition
        test(
            "class F { '0' = _; 'a' = _; [1] = _; ['1'] = _; ['b'] = _; ['c.c'] = _; '1.1' = _; '😊' = _; 'd.d' = _ }",
            "class F {  0 = _;   a = _;    1 = _;    1 = _;     b = _;   'c.c' = _; '1.1' = _; '😊' = _; 'd.d' = _ }",
        );
        // Accessor Property
        test(
            "class F { accessor '0' = _; accessor 'a' = _; accessor [1] = _; accessor ['1'] = _; accessor ['b'] = _; accessor ['c.c'] = _; accessor '1.1' = _; accessor '😊' = _; accessor 'd.d' = _ }",
            "class F { accessor  0 = _;  accessor  a = _;    accessor 1 = _;accessor     1 = _; accessor     b = _; accessor   'c.c' = _; accessor '1.1' = _; accessor '😊' = _; accessor 'd.d' = _ }",
        );

        test("class C { ['-1']() {} }", "class C { '-1'() {} }");

        // <https://tc39.es/ecma262/2024/multipage/ecmascript-language-expressions.html#sec-runtime-semantics-propertydefinitionevaluation>
        test_same("v = ({ ['__proto__']: 0 })"); // { __proto__: 0 } will have `isProtoSetter = true`
        test("v = ({ ['__proto__']() {} })", "v = ({ __proto__() {} })");
        test("({ ['__proto__']: _ } = {})", "({ __proto__: _ } = {})");
        test("class C { ['__proto__'] = 0 }", "class C { __proto__ = 0 }");
        test("class C { ['__proto__']() {} }", "class C { __proto__() {} }");
        test("class C { accessor ['__proto__'] = 0 }", "class C { accessor __proto__ = 0 }");
        test("class C { static ['__proto__'] = 0 }", "class C { static __proto__ = 0 }");
        test(
            "class C { static accessor ['__proto__'] = 0 }",
            "class C { static accessor __proto__ = 0 }",
        );

        // Patch KATAKANA MIDDLE DOT and HALFWIDTH KATAKANA MIDDLE DOT
        // <https://github.com/oxc-project/unicode-id-start/pull/3>
        test_same("x = { 'x・': 0 };");
        test_same("x = { 'x･': 0 };");
        test_same("x = y['x・'];");
        test_same("x = y['x･'];");

        // <https://tc39.es/ecma262/2024/multipage/ecmascript-language-functions-and-classes.html#sec-static-semantics-classelementkind>
        // <https://tc39.es/ecma262/2024/multipage/ecmascript-language-functions-and-classes.html#sec-class-definitions-static-semantics-early-errors>
        // <https://arai-a.github.io/ecma262-compare/?pr=2417&id=sec-class-definitions-static-semantics-early-errors>
        test_same("class C { static ['prototype']() {} }"); // class C { static prototype() {} } is an early error
        test_same("class C { static ['prototype'] = 0 }"); // class C { prototype = 0 } is an early error
        test_same("class C { static accessor ['prototype'] = 0 }"); // class C { accessor prototype = 0 } is an early error
        test("class C { ['prototype']() {} }", "class C { prototype() {} }");
        test("class C { 'prototype'() {} }", "class C { prototype() {} }");
        test("class C { ['prototype'] = 0 }", "class C { prototype = 0 }");
        test("class C { 'prototype' = 0 }", "class C { prototype = 0 }");
        test("class C { accessor ['prototype'] = 0 }", "class C { accessor prototype = 0 }");
        test_same("class C { ['constructor'] = 0 }"); // class C { constructor = 0 } is an early error
        test_same("class C { accessor ['constructor'] = 0 }"); // class C { accessor constructor = 0 } is an early error
        test_same("class C { static ['constructor'] = 0 }"); // class C { static constructor = 0 } is an early error
        test_same("class C { static accessor ['constructor'] = 0 }"); // class C { static accessor constructor = 0 } is an early error
        test_same("class C { ['constructor']() {} }"); // computed `constructor` is not treated as a constructor
        test("class C { 'constructor'() {} }", "class C { constructor() {} }");
        test_same("class C { *['constructor']() {} }"); // class C { *constructor() {} } is an early error
        test_same("class C { async ['constructor']() {} }"); // class C { async constructor() {} } is an early error
        test_same("class C { async *['constructor']() {} }"); // class C { async *constructor() {} } is an early error
        test_same("class C { get ['constructor']() {} }"); // class C { get constructor() {} } is an early error
        test_same("class C { set ['constructor'](v) {} }"); // class C { set constructor(v) {} } is an early error
        test("class C { static ['constructor']() {} }", "class C { static constructor() {} }");
        test("class C { static 'constructor'() {} }", "class C { static constructor() {} }");
        test_same("class C { ['#constructor'] = 0 }"); // class C { #constructor = 0 } is an early error
        test_same("class C { accessor ['#constructor'] = 0 }"); // class C { accessor #constructor = 0 } is an early error
        test_same("class C { ['#constructor']() {} }"); // class C { #constructor() {} } is an early error
        test_same("class C { static ['#constructor'] = 0 }"); // class C { static #constructor = 0 } is an early error
        test_same("class C { static accessor ['#constructor'] = 0 }"); // class C { static accessor #constructor = 0 } is an early error
        test_same("class C { static ['#constructor']() {} }"); // class C { static #constructor() {} } is an early error
    }

    #[test]
    fn fold_function_spread_args() {
        test_same("f(...a)");
        test_same("f(...a, ...b)");
        test_same("f(...a, b, ...c)");
        test_same("new F(...a)");

        test("f(...[])", "f()");
        test("f(...[1])", "f(1)");
        test("f(...[1, 2])", "f(1, 2)");
        test("f(...[1,,,3])", "f(1, void 0, void 0, 3)");
        test("f(a, ...[])", "f(a)");
        test("new F(...[])", "new F()");
        test("new F(...[1])", "new F(1)");
    }

    #[test]
    fn test_fold_boolean_constructor() {
        test("var a = Boolean(true)", "var a = !0");
        // Don't fold the existence check to preserve behavior
        test("var a = Boolean?.(true)", "var a = Boolean?.(!0)");

        test("var a = Boolean(false)", "var a = !1");
        // Don't fold the existence check to preserve behavior
        test("var a = Boolean?.(false)", "var a = Boolean?.(!1)");

        test("var a = Boolean(1)", "var a = !0");
        // Don't fold the existence check to preserve behavior
        test_same("var a = Boolean?.(1)");

        test("var a = Boolean(x)", "var a = !!x");
        // Don't fold the existence check to preserve behavior
        test_same("var a = Boolean?.(x)");

        test("var a = Boolean({})", "var a = !0");
        // Don't fold the existence check to preserve behavior
        test_same("var a = Boolean?.({})");

        test("var a = Boolean()", "var a = !1;");
        test_same("var a = Boolean(!0, !1);");
    }

    #[test]
    fn test_fold_string_constructor() {
        test("x = String()", "x = ''");
        test("var a = String(23)", "var a = '23'");
        // Don't fold the existence check to preserve behavior
        test_same("var a = String?.(23)");

        test("var a = String('hello')", "var a = 'hello'");
        test("var a = String(true)", "var a = 'true'");
        test("var a = String(!0)", "var a = 'true'");
        // Don't fold the existence check to preserve behavior
        test_same("var a = String?.('hello')");

        test_same("var s = Symbol(), a = String(s);");

        test_same("var a = String('hello', bar());");
        test_same("var a = String({valueOf: function() { return 1; }});");
    }

    #[test]
    fn test_fold_number_constructor() {
        test("x = Number()", "x = 0");
        test("x = Number(true)", "x = 1");
        test("x = Number(false)", "x = 0");
        test("x = Number('foo')", "x = NaN");
    }

    #[test]
    fn test_fold_big_int_constructor() {
        test("var x = BigInt(1n)", "var x = 1n");
        test_same("BigInt()");
        test_same("BigInt(1)");
    }

    #[test]
    fn optional_catch_binding() {
        test("try { foo } catch(e) {}", "try { foo } catch {}");
        test("try { foo } catch(e) {foo}", "try { foo } catch {foo}");
        test_same("try { foo } catch(e) { bar(e) }");
        test_same("try { foo } catch([e]) {}");
        test_same("try { foo } catch({e}) {}");
        test_same("try { foo } catch(e) { var e = baz; bar(e) }");
        test("try { foo } catch(e) { var e = 2 }", "try { foo } catch { var e = 2 }");
        test_same("try { foo } catch(e) { var e = 2 } bar(e)");

        // FIXME catch(a) has no references but it cannot be removed.
        // test_same(
        // r#"var a = "PASS";
        // try {
        // throw "FAIL1";
        // } catch (a) {
        // var a = "FAIL2";
        // }
        // console.log(a);"#,
        // );

        let target = ESTarget::ES2018;
        let options = CompressOptions { target, ..CompressOptions::default() };
        test_same_options("try { foo } catch(e) {}", &options);
    }

    #[test]
    fn test_remove_name_from_expressions() {
        test("var a = function f() {}", "var a = function () {}");
        test_same("var a = function f() { return f; }");

        test("var a = class C {}", "var a = class {}");
        test_same("var a = class C { foo() { return C } }");

        let options = CompressOptions {
            keep_names: CompressOptionsKeepNames::function_only(),
            ..default_options()
        };
        test_same_options("var a = function f() {}", &options);

        let options = CompressOptions {
            keep_names: CompressOptionsKeepNames::class_only(),
            ..default_options()
        };
        test_same_options("var a = class C {}", &options);
    }

    #[test]
    fn test_compress_destructuring_assignment_target() {
        test_same("var {y} = x");
        test_same("var {y, z} = x");
        test_same("var {y: z, z: y} = x");
        test("var {y: y} = x", "var {y} = x");
        test("var {y: z, 'z': y} = x", "var {y: z, z: y} = x");
        test("var {y: y, 'z': z} = x", "var {y, z} = x");
    }

    #[test]
    fn test_object_callee_indirect_call() {
        test("Object(f)(1,2)", "f(1, 2)");
        test("(Object(g))(a)", "g(a)");
        test("Object(a.b)(x)", "(0, a.b)(x)");
        test_same("Object?.(f)(1)");
        test_same("function Object(x){return x} Object(f)(1)");
        test_same("Object(...a)(1)");
    }

    #[test]
    fn test_rewrite_arguments_copy_loop() {
        test(
            "for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = arguments[a]; console.log(r)",
            "var r = [...arguments]; console.log(r)",
        );
        test(
            "for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) { r[a] = arguments[a]; } console.log(r)",
            "var r = [...arguments]; console.log(r)",
        );
        test(
            "for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) { r[a] = arguments[a] } console.log(r)",
            "var r = [...arguments]; console.log(r)",
        );
        test(
            "for (var e = arguments.length, r = new Array(e), a = 0; a < e; a++) r[a] = arguments[a]; console.log(r)",
            "var r = [...arguments]; console.log(r)",
        );
        test(
            "for (var e = arguments.length, r = Array(e > 1 ? e - 1 : 0), a = 1; a < e; a++) r[a - 1] = arguments[a]; console.log(r)",
            "var r = [...arguments].slice(1); console.log(r)",
        );
        test(
            "for (var e = arguments.length, r = Array(e > 2 ? e - 2 : 0), a = 2; a < e; a++) r[a - 2] = arguments[a]; console.log(r)",
            "var r = [...arguments].slice(2); console.log(r)",
        );
        test(
            "for (var e = arguments.length, r = [], a = 0; a < e; a++) r[a] = arguments[a]; console.log(r)",
            "var r = [...arguments]; console.log(r)",
        );
        test(
            "for (var r = [], a = 0; a < arguments.length; a++) r[a] = arguments[a]; console.log(r)",
            "var r = [...arguments]; console.log(r)",
        );
        test(
            "for (var r = [], a = 1; a < arguments.length; a++) r[a - 1] = arguments[a]; console.log(r)",
            "var r = [...arguments].slice(1); console.log(r)",
        );
        test(
            "for (var r = [], a = 2; a < arguments.length; a++) r[a - 2] = arguments[a]; console.log(r)",
            "var r = [...arguments].slice(2); console.log(r)",
        );
        test(
            "for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = arguments[a];",
            "",
        );
        test(
            "for (var e = arguments.length, r = Array(e > 1 ? e - 1 : 0), a = 1; a < e; a++) r[a - 1] = arguments[a]",
            "",
        );
        test_same(
            "for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) console.log(r[a]);",
        );
        test(
            "for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) { r[a] = arguments[a]; console.log(r); }",
            "for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) (r[a] = arguments[a], console.log(r))",
        );
        test_same(
            "for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] += arguments[a];",
        );
        test_same(
            "for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a + 1] = arguments[a];",
        );
        test_same(
            "for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a - 0.5] = arguments[a];",
        );
        test(
            "var arguments; for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = arguments[a];",
            "for (var arguments, e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = arguments[a];",
        );
        test_same("for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = foo[a];");
        test_same(
            "for (var e = arguments.length, r = Array(e), a = 0; a < e; e--) r[a] = arguments[a];",
        );
        test_same(
            "for (var e = arguments.length, r = Array(e), a = 0; a < e; r++) r[a] = arguments[a];",
        );
        test_same(
            "for (var e = arguments.length, r = Array(e), a = 0; a < r; r++) r[a] = arguments[a];",
        );
        test(
            "var arguments; for (var r = [], a = 0; a < arguments.length; a++) r[a] = arguments[a];",
            "for (var arguments, r = [], a = 0; a < arguments.length; a++) r[a] = arguments[a];",
        );
        test_same(
            "for (var e = arguments.length, r = Array(e > 1 ? e - 2 : 0), a = 2; a < e; a++) r[a - 2] = arguments[a];",
        );
    }

    #[test]
    fn test_flatten_nested_chain_expression() {
        test("(a.b)?.c", "a.b?.c");

        test("(a?.b)?.c", "a?.b?.c");
        test("(a?.b?.c)?.d", "a?.b?.c?.d");
        test("(((a?.b)?.c)?.d)?.e", "a?.b?.c?.d?.e");
        test("(a?.b)?.()", "a?.b?.()");
        test("(a?.b)?.(arg)", "a?.b?.(arg)");
        test("(a?.b)?.[0]", "a?.b?.[0]");
        test("(a?.b)?.[key]", "a?.b?.[key]");
        test("(a?.#b)?.c", "a?.#b?.c");
        test_same("a.b?.c");
        test_same("a?.b?.c");
        test_same("(a?.b).c");
    }
}
