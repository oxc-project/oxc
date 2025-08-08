use std::iter::repeat_with;

use oxc_allocator::{CloneIn, TakeIn, Vec};
use oxc_ast::{NONE, ast::*};
use oxc_ecmascript::constant_evaluation::DetermineValueType;
use oxc_ecmascript::{ToJsString, ToNumber, side_effects::MayHaveSideEffects};
use oxc_span::GetSpan;
use oxc_span::SPAN;
use oxc_syntax::{
    es_target::ESTarget,
    number::NumberBase,
    operator::{BinaryOperator, UnaryOperator},
};
use oxc_traverse::Ancestor;

use crate::ctx::Ctx;

use super::{LatePeepholeOptimizations, PeepholeOptimizations};

/// A peephole optimization that minimizes code by simplifying conditional
/// expressions, replacing IFs with HOOKs, replacing object constructors
/// with literals, and simplifying returns.
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeSubstituteAlternateSyntax.java>
impl<'a> PeepholeOptimizations {
    pub fn substitute_object_property(&self, prop: &mut ObjectProperty<'a>, ctx: &mut Ctx<'a, '_>) {
        // <https://tc39.es/ecma262/2024/multipage/ecmascript-language-expressions.html#sec-runtime-semantics-propertydefinitionevaluation>
        if !prop.method {
            if let PropertyKey::StringLiteral(str) = &prop.key {
                // "{ __proto__ }" sets prototype, while "{ ['__proto__'] }" does not
                if str.value == "__proto__" {
                    return;
                }
            }
        }

        self.try_compress_property_key(&mut prop.key, &mut prop.computed, ctx);
    }

    pub fn substitute_assignment_target_property_property(
        &self,
        prop: &mut AssignmentTargetPropertyProperty<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        self.try_compress_property_key(&mut prop.name, &mut prop.computed, ctx);
    }

    pub fn substitute_assignment_target_property(
        &self,
        prop: &mut AssignmentTargetProperty<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        self.try_compress_assignment_target_property(prop, ctx);
    }

    pub fn try_compress_assignment_target_property(
        &self,
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

    pub fn substitute_binding_property(
        &self,
        prop: &mut BindingProperty<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        self.try_compress_property_key(&mut prop.key, &mut prop.computed, ctx);
    }

    pub fn substitute_method_definition(
        &self,
        prop: &mut MethodDefinition<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        let property_key_parent: ClassPropertyKeyParent = prop.into();
        if let PropertyKey::StringLiteral(str) = &prop.key {
            if property_key_parent.should_keep_as_computed_property(&str.value) {
                return;
            }
        }
        self.try_compress_property_key(&mut prop.key, &mut prop.computed, ctx);
    }

    pub fn substitute_property_definition(
        &self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        let property_key_parent: ClassPropertyKeyParent = prop.into();
        if let PropertyKey::StringLiteral(str) = &prop.key {
            if property_key_parent.should_keep_as_computed_property(&str.value) {
                return;
            }
        }
        self.try_compress_property_key(&mut prop.key, &mut prop.computed, ctx);
    }

    pub fn substitute_accessor_property(
        &self,
        prop: &mut AccessorProperty<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        let property_key_parent: ClassPropertyKeyParent = prop.into();
        if let PropertyKey::StringLiteral(str) = &prop.key {
            if property_key_parent.should_keep_as_computed_property(&str.value) {
                return;
            }
        }
        self.try_compress_property_key(&mut prop.key, &mut prop.computed, ctx);
    }

    pub fn substitute_return_statement(
        &self,
        stmt: &mut ReturnStatement<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        self.compress_return_statement(stmt, ctx);
    }

    pub fn substitute_variable_declaration(
        &self,
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        for declarator in &mut decl.declarations {
            self.compress_variable_declarator(declarator, ctx);
        }
    }

    pub fn substitute_call_expression(&self, expr: &mut CallExpression<'a>, ctx: &mut Ctx<'a, '_>) {
        self.try_flatten_arguments(&mut expr.arguments, ctx);
    }

    pub fn substitute_new_expression(&self, expr: &mut NewExpression<'a>, ctx: &mut Ctx<'a, '_>) {
        self.try_flatten_arguments(&mut expr.arguments, ctx);
    }

    pub fn substitute_exit_expression(&self, expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        // Change syntax
        match expr {
            Expression::ArrowFunctionExpression(e) => {
                self.try_compress_arrow_expression(e, ctx);
            }
            Expression::ChainExpression(e) => {
                self.try_compress_chain_call_expression(e, ctx);
            }
            Expression::BinaryExpression(e) => Self::swap_binary_expressions(e),
            Expression::FunctionExpression(e) => self.try_remove_name_from_functions(e, ctx),
            Expression::ClassExpression(e) => self.try_remove_name_from_classes(e, ctx),
            _ => {}
        }

        // Fold
        if let Some(folded_expr) = match expr {
            Expression::LogicalExpression(e) => Self::try_compress_is_object_and_not_null(e, ctx)
                .or_else(|| Self::try_rotate_logical_expression(e, ctx)),
            Expression::TemplateLiteral(t) => Self::try_fold_template_literal(t, ctx),
            Expression::BinaryExpression(e) => Self::try_fold_loose_equals_undefined(e, ctx)
                .or_else(|| Self::try_compress_typeof_undefined(e, ctx)),
            Expression::UnaryExpression(e) => Self::try_remove_unary_plus(e, ctx),
            Expression::NewExpression(e) => Self::get_fold_constructor_name(&e.callee, ctx)
                .and_then(|name| {
                    Self::try_fold_object_or_array_constructor(e.span, name, &mut e.arguments, ctx)
                })
                .or_else(|| Self::try_fold_new_expression(e, ctx)),
            Expression::CallExpression(e) => Self::get_fold_constructor_name(&e.callee, ctx)
                .and_then(|name| {
                    Self::try_fold_object_or_array_constructor(e.span, name, &mut e.arguments, ctx)
                })
                .or_else(|| self.try_fold_simple_function_call(e, ctx)),
            _ => None,
        } {
            *expr = folded_expr;
            ctx.state.changed = true;
        }
    }

    fn swap_binary_expressions(e: &mut BinaryExpression<'a>) {
        if e.operator.is_equality()
            && (e.left.is_literal() || e.left.is_no_substitution_template() || e.left.is_void_0())
            && !e.right.is_literal()
        {
            std::mem::swap(&mut e.left, &mut e.right);
        }
    }

    /// `() => { return foo })` -> `() => foo`
    fn try_compress_arrow_expression(
        &self,
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
    fn try_compress_typeof_undefined(
        expr: &mut BinaryExpression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        let Expression::UnaryExpression(unary_expr) = &expr.left else { return None };
        if !unary_expr.operator.is_typeof() {
            return None;
        }
        if !expr.right.is_specific_string_literal("undefined") {
            return None;
        }
        let (new_eq_op, new_comp_op) = match expr.operator {
            BinaryOperator::Equality | BinaryOperator::StrictEquality => {
                (BinaryOperator::StrictEquality, BinaryOperator::GreaterThan)
            }
            BinaryOperator::Inequality | BinaryOperator::StrictInequality => {
                (BinaryOperator::StrictInequality, BinaryOperator::LessThan)
            }
            _ => return None,
        };
        if let Expression::Identifier(ident) = &unary_expr.argument {
            if ctx.is_global_reference(ident) {
                let left = expr.left.take_in(ctx.ast);
                let right = ctx.ast.expression_string_literal(expr.right.span(), "u", None);
                return Some(ctx.ast.expression_binary(expr.span, left, new_comp_op, right));
            }
        }

        let Expression::UnaryExpression(unary_expr) = expr.left.take_in(ctx.ast) else {
            unreachable!()
        };
        let right = ctx.ast.void_0(expr.right.span());
        Some(ctx.ast.expression_binary(expr.span, unary_expr.unbox().argument, new_eq_op, right))
    }

    /// Remove unary `+` if `ToNumber` conversion is done by the parent expression
    ///
    /// - `1 - +b` => `1 - b` (for other operators as well)
    /// - `+a - 1` => `a - 1` (for other operators as well)
    fn try_remove_unary_plus(
        expr: &mut UnaryExpression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if expr.operator != UnaryOperator::UnaryPlus {
            return None;
        }

        let parent_expression = ctx.ancestors().next()?;
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
            return None;
        }

        Some(expr.argument.take_in(ctx.ast))
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
    fn try_rotate_logical_expression(
        expr: &mut LogicalExpression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        let Expression::LogicalExpression(right) = &mut expr.right else { return None };
        if right.operator != expr.operator {
            return None;
        }

        let mut new_left = ctx.ast.expression_logical(
            expr.span,
            expr.left.take_in(ctx.ast),
            expr.operator,
            right.left.take_in(ctx.ast),
        );

        {
            let Expression::LogicalExpression(new_left2) = &mut new_left else { unreachable!() };
            if let Some(expr) = Self::try_rotate_logical_expression(new_left2, ctx) {
                new_left = expr;
            }
        }

        Some(ctx.ast.expression_logical(
            expr.span,
            new_left,
            expr.operator,
            right.right.take_in(ctx.ast),
        ))
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
    fn try_compress_is_object_and_not_null(
        expr: &mut LogicalExpression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        let inversed = match expr.operator {
            LogicalOperator::And => false,
            LogicalOperator::Or => true,
            LogicalOperator::Coalesce => return None,
        };

        if let Some(new_expr) = Self::try_compress_is_object_and_not_null_for_left_and_right(
            &expr.left,
            &expr.right,
            expr.span,
            ctx,
            inversed,
        ) {
            return Some(new_expr);
        }

        let Expression::LogicalExpression(left) = &mut expr.left else {
            return None;
        };
        let inversed = match expr.operator {
            LogicalOperator::And => false,
            LogicalOperator::Or => true,
            LogicalOperator::Coalesce => return None,
        };

        Self::try_compress_is_object_and_not_null_for_left_and_right(
            &left.right,
            &expr.right,
            Span::new(left.right.span().start, expr.span.end),
            ctx,
            inversed,
        )
        .map(|new_expr| {
            ctx.ast.expression_logical(
                expr.span,
                left.left.take_in(ctx.ast),
                expr.operator,
                new_expr,
            )
        })
    }

    fn try_compress_is_object_and_not_null_for_left_and_right(
        left: &Expression<'a>,
        right: &Expression<'a>,
        span: Span,
        ctx: &mut Ctx<'a, '_>,
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

        let mut new_left_expr = typeof_binary_expr.clone_in(ctx.ast.allocator);
        if let Expression::BinaryExpression(new_left_expr_binary) = &mut new_left_expr {
            new_left_expr_binary.operator =
                if inversed { BinaryOperator::Inequality } else { BinaryOperator::Equality };
        } else {
            unreachable!();
        }

        let new_right_expr = if inversed {
            ctx.ast.expression_unary(
                SPAN,
                UnaryOperator::LogicalNot,
                ctx.ast.expression_identifier(is_null_id_ref.span, is_null_id_ref.name),
            )
        } else {
            ctx.ast.expression_unary(
                SPAN,
                UnaryOperator::LogicalNot,
                ctx.ast.expression_unary(
                    SPAN,
                    UnaryOperator::LogicalNot,
                    ctx.ast.expression_identifier(is_null_id_ref.span, is_null_id_ref.name),
                ),
            )
        };
        Some(ctx.ast.expression_logical(
            span,
            new_left_expr,
            if inversed { LogicalOperator::Or } else { LogicalOperator::And },
            new_right_expr,
        ))
    }

    fn try_fold_loose_equals_undefined(
        e: &mut BinaryExpression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        // `foo == void 0` -> `foo == null`, `foo == undefined` -> `foo == null`
        // `foo != void 0` -> `foo == null`, `foo == undefined` -> `foo == null`
        if e.operator == BinaryOperator::Inequality || e.operator == BinaryOperator::Equality {
            let (left, right) = if ctx.is_expression_undefined(&e.right) {
                (e.left.take_in(ctx.ast), ctx.ast.expression_null_literal(e.right.span()))
            } else if ctx.is_expression_undefined(&e.left) {
                (e.right.take_in(ctx.ast), ctx.ast.expression_null_literal(e.left.span()))
            } else {
                return None;
            };

            return Some(ctx.ast.expression_binary(e.span, left, e.operator, right));
        }

        None
    }

    /// Removes redundant argument of `ReturnStatement`
    ///
    /// `return undefined` -> `return`
    /// `return void 0` -> `return`
    fn compress_return_statement(&self, stmt: &mut ReturnStatement<'a>, ctx: &mut Ctx<'a, '_>) {
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
        for ancestor in ctx.ancestors() {
            if let Ancestor::FunctionBody(func) = ancestor {
                if *func.r#async() && *func.generator() {
                    return;
                }
            }
        }
        stmt.argument = None;
        ctx.state.changed = true;
    }

    fn compress_variable_declarator(
        &self,
        decl: &mut VariableDeclarator<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) {
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
    fn try_fold_simple_function_call(
        &self,
        call_expr: &mut CallExpression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        if call_expr.optional || call_expr.arguments.len() >= 2 {
            return None;
        }
        let Expression::Identifier(ident) = &call_expr.callee else { return None };
        let name = ident.name.as_str();
        if !matches!(name, "Boolean" | "Number" | "String" | "BigInt") {
            return None;
        }
        let args = &mut call_expr.arguments;
        let arg = match args.get_mut(0) {
            None => None,
            Some(arg) => Some(arg.as_expression_mut()?),
        };
        if !ctx.is_global_reference(ident) {
            return None;
        }
        let span = call_expr.span;
        match name {
            // `Boolean(a)` -> `!!(a)`
            // http://www.ecma-international.org/ecma-262/6.0/index.html#sec-boolean-constructor-boolean-value
            // and
            // http://www.ecma-international.org/ecma-262/6.0/index.html#sec-logical-not-operator-runtime-semantics-evaluation
            "Boolean" => match arg {
                None => Some(ctx.ast.expression_boolean_literal(span, false)),
                Some(arg) => {
                    let mut arg = arg.take_in(ctx.ast);
                    self.try_fold_expr_in_boolean_context(&mut arg, ctx);
                    let arg = ctx.ast.expression_unary(span, UnaryOperator::LogicalNot, arg);
                    Some(self.minimize_not(span, arg, ctx))
                }
            },
            "String" => {
                match arg {
                    // `String()` -> `''`
                    None => Some(ctx.ast.expression_string_literal(span, "", None)),
                    // `String(a)` -> `'' + (a)`
                    Some(arg) => {
                        if !arg.is_literal() {
                            return None;
                        }
                        Some(ctx.ast.expression_binary(
                            span,
                            ctx.ast.expression_string_literal(call_expr.span, "", None),
                            BinaryOperator::Addition,
                            arg.take_in(ctx.ast),
                        ))
                    }
                }
            }
            "Number" => Some(ctx.ast.expression_numeric_literal(
                span,
                match arg {
                    None => 0.0,
                    Some(arg) => arg.to_number(ctx)?,
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
        }
    }

    /// Fold `Object` or `Array` constructor
    fn get_fold_constructor_name(
        callee: &Expression<'a>,
        ctx: &mut Ctx<'a, '_>,
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
    fn try_fold_object_or_array_constructor(
        span: Span,
        name: &'a str,
        args: &mut Vec<'a, Argument<'a>>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        match name {
            "Object" if args.is_empty() => Some(ctx.ast.expression_object(span, ctx.ast.vec())),
            "Array" => {
                // `new Array` -> `[]`
                if args.is_empty() {
                    Some(ctx.ast.expression_array(span, ctx.ast.vec()))
                } else if args.len() == 1 {
                    let arg = args[0].as_expression_mut()?;
                    // `new Array(0)` -> `[]`
                    if arg.is_number_0() {
                        Some(ctx.ast.expression_array(span, ctx.ast.vec()))
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
                                return Some(
                                    ctx.ast.expression_array(span, ctx.ast.vec_from_iter(elisions)),
                                );
                            }
                        }
                        let callee = ctx.ast.expression_identifier(n.span, "Array");
                        let args = args.take_in(ctx.ast);
                        Some(ctx.ast.expression_call(span, callee, NONE, args, false))
                    }
                    // `new Array(literal)` -> `[literal]`
                    else if arg.is_literal() || matches!(arg, Expression::ArrayExpression(_)) {
                        let elements =
                            ctx.ast.vec1(ArrayExpressionElement::from(arg.take_in(ctx.ast)));
                        Some(ctx.ast.expression_array(span, elements))
                    }
                    // `new Array(x)` -> `Array(x)`
                    else {
                        let callee = ctx.ast.expression_identifier(span, "Array");
                        let args = args.take_in(ctx.ast);
                        Some(ctx.ast.expression_call(span, callee, NONE, args, false))
                    }
                } else {
                    // // `new Array(1, 2, 3)` -> `[1, 2, 3]`
                    let elements = ctx.ast.vec_from_iter(
                        args.iter_mut()
                            .filter_map(|arg| arg.as_expression_mut())
                            .map(|arg| ArrayExpressionElement::from(arg.take_in(ctx.ast))),
                    );
                    Some(ctx.ast.expression_array(span, elements))
                }
            }
            _ => None,
        }
    }

    /// `new Error()` -> `Error()` (also for NativeErrors)
    /// `new AggregateError()` -> `AggregateError()`
    /// `new Function()` -> `Function()`
    /// `new RegExp()` -> `RegExp()`
    fn try_fold_new_expression(
        e: &mut NewExpression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        let Expression::Identifier(ident) = &e.callee else { return None };
        let name = ident.name.as_str();
        if !matches!(name, "Error" | "AggregateError" | "Function" | "RegExp")
            && !Self::is_native_error_name(name)
        {
            return None;
        }
        if !ctx.is_global_reference(ident) {
            return None;
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
            return Some(ctx.ast.expression_call_with_pure(
                e.span,
                e.callee.take_in(ctx.ast),
                NONE,
                e.arguments.take_in(ctx.ast),
                false,
                e.pure,
            ));
        }
        None
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

    fn try_compress_chain_call_expression(
        &self,
        chain_expr: &mut ChainExpression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        if let ChainElement::CallExpression(call_expr) = &mut chain_expr.expression {
            // `window.Object?.()` -> `Object?.()`
            if call_expr.arguments.is_empty()
                && call_expr
                    .callee
                    .as_member_expression()
                    .is_some_and(|mem_expr| mem_expr.is_specific_member_access("window", "Object"))
            {
                call_expr.callee = ctx.ast.expression_identifier(call_expr.callee.span(), "Object");
                ctx.state.changed = true;
            }
        }
    }

    fn try_fold_template_literal(
        t: &TemplateLiteral<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        t.to_js_string(ctx).map(|val| {
            ctx.ast.expression_string_literal(t.span(), ctx.ast.atom_from_cow(&val), None)
        })
    }

    // <https://github.com/swc-project/swc/blob/4e2dae558f60a9f5c6d2eac860743e6c0b2ec562/crates/swc_ecma_minifier/src/compress/pure/properties.rs>
    fn try_compress_property_key(
        &self,
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
    fn try_flatten_arguments(&self, args: &mut Vec<'a, Argument<'a>>, ctx: &mut Ctx<'a, '_>) {
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

    /// Remove name from function expressions if it is not used.
    ///
    /// e.g. `var a = function f() {}` -> `var a = function () {}`
    ///
    /// This compression is not safe if the code relies on `Function::name`.
    fn try_remove_name_from_functions(&self, func: &mut Function<'a>, ctx: &mut Ctx<'a, '_>) {
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
    fn try_remove_name_from_classes(&self, class: &mut Class<'a>, ctx: &mut Ctx<'a, '_>) {
        if ctx.options().keep_names.class {
            return;
        }

        if class.id.as_ref().is_some_and(|id| ctx.scoping().symbol_is_unused(id.symbol_id())) {
            class.id = None;
            ctx.state.changed = true;
        }
    }
}

impl<'a> LatePeepholeOptimizations {
    pub fn substitute_exit_expression(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        if let Expression::NewExpression(e) = expr {
            Self::try_compress_typed_array_constructor(e, ctx);
        }

        if let Some(folded_expr) = match expr {
            Expression::BooleanLiteral(_) => Self::try_compress_boolean(expr, ctx),
            Expression::ArrayExpression(_) => Self::try_compress_array_expression(expr, ctx),
            _ => None,
        } {
            *expr = folded_expr;
        }
    }

    /// `new Int8Array(0)` -> `new Int8Array()` (also for other TypedArrays)
    fn try_compress_typed_array_constructor(e: &mut NewExpression<'a>, ctx: &mut Ctx<'a, '_>) {
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
    fn try_compress_boolean(
        expr: &mut Expression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        let Expression::BooleanLiteral(lit) = expr else { return None };
        let num = ctx.ast.expression_numeric_literal(
            lit.span,
            if lit.value { 0.0 } else { 1.0 },
            None,
            NumberBase::Decimal,
        );
        Some(ctx.ast.expression_unary(lit.span, UnaryOperator::LogicalNot, num))
    }

    /// Transforms long array expression with string literals to `"str1,str2".split(',')`
    fn try_compress_array_expression(
        expr: &mut Expression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Expression<'a>> {
        // this threshold is chosen by hand by checking the minsize output
        const THRESHOLD: usize = 40;

        let Expression::ArrayExpression(array) = expr else { unreachable!() };

        let is_all_string = array.elements.iter().all(|element| {
            element.as_expression().is_some_and(|expr| matches!(expr, Expression::StringLiteral(_)))
        });
        if !is_all_string {
            return None;
        }

        let element_count = array.elements.len();
        // replace with `.split` only when the saved size is great enough
        // because using `.split` in some places and not in others may cause gzipped size to be bigger
        let can_save = element_count * 2 > ".split('.')".len() + THRESHOLD;
        if !can_save {
            return None;
        }

        let strings = array.elements.iter().map(|element| {
            let Expression::StringLiteral(str) = element.to_expression() else { unreachable!() };
            str.value.as_str()
        });
        let delimiter = Self::pick_delimiter(&strings)?;

        let concatenated_string = strings.collect::<std::vec::Vec<_>>().join(delimiter);

        // "str1,str2".split(',')
        Some(ctx.ast.expression_call(
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
        ))
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

    pub fn substitute_catch_clause(&self, catch: &mut CatchClause<'a>, ctx: &mut Ctx<'a, '_>) {
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
        CompressOptions,
        options::CompressOptionsKeepNames,
        tester::{default_options, test, test_same, test_same_options},
    };

    #[test]
    fn test_fold_return_result() {
        test("function f(){return !1;}", "function f(){return !1}");
        test("function f(){return null;}", "function f(){return null}");
        test("function f(){return void 0;}", "function f(){}");
        test("function f(){return void foo();}", "function f(){return void foo()}");
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
        test("new RegExp('a', 'g')", "RegExp('a', 'g')");
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
                    "var x='{expected_partial}{}'.split('{delimiter}')",
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
    #[ignore]
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
        test("class C { ['prototype'] = 0 }", "class C { prototype = 0 }");
        test("class C { accessor ['prototype'] = 0 }", "class C { accessor prototype = 0 }");
        test_same("class C { ['constructor'] = 0 }"); // class C { constructor = 0 } is an early error
        test_same("class C { accessor ['constructor'] = 0 }"); // class C { accessor constructor = 0 } is an early error
        test_same("class C { static ['constructor'] = 0 }"); // class C { static constructor = 0 } is an early error
        test_same("class C { static accessor ['constructor'] = 0 }"); // class C { static accessor constructor = 0 } is an early error
        test_same("class C { ['constructor']() {} }"); // computed `constructor` is not treated as a constructor
        test_same("class C { *['constructor']() {} }"); // class C { *constructor() {} } is an early error
        test_same("class C { async ['constructor']() {} }"); // class C { async constructor() {} } is an early error
        test_same("class C { async *['constructor']() {} }"); // class C { async *constructor() {} } is an early error
        test_same("class C { get ['constructor']() {} }"); // class C { get constructor() {} } is an early error
        test_same("class C { set ['constructor'](v) {} }"); // class C { set constructor(v) {} } is an early error
        test("class C { static ['constructor']() {} }", "class C { static constructor() {} }");
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
        test_same("try { foo } catch(e) { var e = 2; bar(e) }");
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
}
