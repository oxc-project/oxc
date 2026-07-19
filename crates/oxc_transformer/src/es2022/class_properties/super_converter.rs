//! ES2022: Class Properties
//! Transform of `super` expressions.

use oxc_allocator::{ArenaBox, ArenaVec, ReplaceWith, TakeIn};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_traverse::ast_operations::get_var_name_from_node;

use crate::{
    Helper,
    common::{helper_loader::helper_call_expr, var_declarations::VarDeclarationsStore},
    context::TraverseCtx,
    utils::ast_builder::{create_assignment, create_prototype_member},
};

use super::ClassProperties;

#[derive(Debug)]
pub(super) enum ClassPropertiesSuperConverterMode {
    // `static prop` or `static {}`
    Static,
    // `#method() {}`
    PrivateMethod,
    // `static #method() {}`
    StaticPrivateMethod,
}

/// Convert `super` expressions.
pub(super) struct ClassPropertiesSuperConverter<'a, 'v> {
    mode: ClassPropertiesSuperConverterMode,
    pub(super) class_properties: &'v mut ClassProperties<'a>,
}

impl<'a, 'v> ClassPropertiesSuperConverter<'a, 'v> {
    pub(super) fn new(
        mode: ClassPropertiesSuperConverterMode,
        class_properties: &'v mut ClassProperties<'a>,
    ) -> Self {
        Self { mode, class_properties }
    }
}

impl<'a> ClassPropertiesSuperConverter<'a, '_> {
    /// Transform static member expression where object is `super`.
    ///
    /// `super.prop` -> `_superPropGet(_Class, "prop", _Class)`
    //
    // `#[inline]` so that compiler sees that `expr` is an `Expression::StaticMemberExpression`.
    #[inline]
    pub(super) fn transform_static_member_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Some(member) = expr.as_static_member_expression() else { unreachable!() };
        if member.object.is_super() {
            *expr = self.transform_static_member_expression_impl(member, false, ctx);
        }
    }

    fn transform_static_member_expression_impl(
        &mut self,
        member: &StaticMemberExpression<'a>,
        is_callee: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let property = &member.property;
        let property = Expression::new_string_literal(property.span, property.name, None, ctx);
        self.create_super_prop_get(member.span, property, is_callee, ctx)
    }

    /// Transform computed member expression where object is `super`.
    ///
    /// `super[prop]` -> `_superPropGet(_Class, prop, _Class)`
    //
    // `#[inline]` so that compiler sees that `expr` is an `Expression::ComputedMemberExpression`.
    #[inline]
    pub(super) fn transform_computed_member_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Some(member) = expr.as_computed_member_expression_mut() else { unreachable!() };
        if member.object.is_super() {
            *expr = self.transform_computed_member_expression_impl(member, false, ctx);
        }
    }

    fn transform_computed_member_expression_impl(
        &mut self,
        member: &mut ComputedMemberExpression<'a>,
        is_callee: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let property = member.expression.take_in(ctx);
        self.create_super_prop_get(member.span, property, is_callee, ctx)
    }

    /// Transform call expression where callee contains `super`.
    ///
    /// `super.method()` -> `_superPropGet(_Class, "method", _Class, 2)([])`
    /// `super.method(1)` -> `_superPropGet(_Class, "method", _Class, 2)([1])`
    //
    // `#[inline]` so can bail out fast without a function call if `callee` is not a member expression
    // with `super` as member expression object (fairly rare).
    // Actual transform is broken out into separate functions.
    #[inline]
    pub(super) fn transform_call_expression_for_super_member_expr(
        &mut self,
        call_expr: &mut CallExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match call_expr.callee.tag() {
            ExpressionTag::StaticMemberExpression
                if call_expr.callee.to_static_member_expression().object.is_super() =>
            {
                self.transform_call_expression_for_super_static_member_expr(call_expr, ctx);
            }
            ExpressionTag::ComputedMemberExpression
                if call_expr.callee.to_computed_member_expression().object.is_super() =>
            {
                self.transform_call_expression_for_super_computed_member_expr(call_expr, ctx);
            }
            _ => {}
        }
    }

    fn transform_call_expression_for_super_static_member_expr(
        &mut self,
        call_expr: &mut CallExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let callee = &mut call_expr.callee;
        let Some(member) = callee.as_static_member_expression() else { unreachable!() };
        *callee = self.transform_static_member_expression_impl(member, true, ctx);
        Self::transform_super_call_expression_arguments(&mut call_expr.arguments, ctx);
    }

    fn transform_call_expression_for_super_computed_member_expr(
        &mut self,
        call_expr: &mut CallExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let callee = &mut call_expr.callee;
        let Some(member) = callee.as_computed_member_expression_mut() else { unreachable!() };
        *callee = self.transform_computed_member_expression_impl(member, true, ctx);
        Self::transform_super_call_expression_arguments(&mut call_expr.arguments, ctx);
    }

    /// [A, B, C] -> [[A, B, C]]
    fn transform_super_call_expression_arguments(
        arguments: &mut ArenaVec<'a, Argument<'a>>,
        ctx: &TraverseCtx<'a>,
    ) {
        let elements = arguments.drain(..).map(ArrayExpressionElement::from);
        let elements = ArenaVec::from_iter_in(elements, ctx);
        let array = Expression::new_array_expression(SPAN, elements, ctx);
        arguments.push(Argument::from(array));
    }

    /// Transform assignment expression where the left-hand side is a member expression with `super`.
    ///
    /// * `super.prop = value`
    ///   -> `_superPropSet(_Class, "prop", value, _Class, 1)`
    /// * `super.prop += value`
    ///   -> `_superPropSet(_Class, "prop", _superPropGet(_Class, "prop", _Class) + value, _Class, 1)`
    /// * `super.prop &&= value`
    ///   -> `_superPropGet(_Class, "prop", _Class) && _superPropSet(_Class, "prop", value, _Class, 1)`
    ///
    /// * `super[prop] = value`
    ///   -> `_superPropSet(_Class, prop, value, _Class, 1)`
    /// * `super[prop] += value`
    ///   -> `_superPropSet(_Class, prop, _superPropGet(_Class, prop, _Class) + value, _Class, 1)`
    /// * `super[prop] &&= value`
    ///   -> `_superPropGet(_Class, prop, _Class) && _superPropSet(_Class, prop, value, _Class, 1)`
    //
    // `#[inline]` so can bail out fast without a function call if `left` is not a member expression
    // with `super` as member expression object (fairly rare).
    // Actual transform is broken out into separate functions.
    pub(super) fn transform_assignment_expression_for_super_assignment_target(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Some(assign_expr) = expr.as_assignment_expression() else { unreachable!() };
        match &assign_expr.left {
            AssignmentTarget::MemberExpression(member)
                if member.as_static_member_expression().is_some_and(|m| m.object.is_super()) =>
            {
                self.transform_assignment_expression_for_super_static_member_expr(expr, ctx);
            }
            AssignmentTarget::MemberExpression(member)
                if member.as_computed_member_expression().is_some_and(|m| m.object.is_super()) =>
            {
                self.transform_assignment_expression_for_super_computed_member_expr(expr, ctx);
            }
            _ => {}
        }
    }

    /// Transform assignment expression where the left-hand side is a static member expression
    /// with `super`.
    ///
    /// * `super.prop = value`
    ///   -> `_superPropSet(_Class, "prop", value, _Class, 1)`
    /// * `super.prop += value`
    ///   -> `_superPropSet(_Class, "prop", _superPropGet(_Class, "prop", _Class) + value, _Class, 1)`
    /// * `super.prop &&= value`
    ///   -> `_superPropGet(_Class, "prop", _Class) && _superPropSet(_Class, "prop", value, _Class, 1)`
    //
    // `#[inline]` so that compiler sees that `expr` is an `Expression::AssignmentExpression`.
    #[inline]
    fn transform_assignment_expression_for_super_static_member_expr(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        expr.replace_with(|expr| {
            let ExpressionKindOwned::AssignmentExpression(assign_expr) = expr.into_kind() else {
                unreachable!()
            };
            let AssignmentExpression { span, operator, right: value, left, .. } =
                assign_expr.unbox();
            let AssignmentTarget::MemberExpression(member) = left else { unreachable!() };
            let member = member.into_static_member_expression().unwrap();
            let property = Expression::new_string_literal(
                member.property.span,
                member.property.name,
                None,
                ctx,
            );
            self.transform_super_assignment_expression_impl(span, operator, property, value, ctx)
        });
    }

    /// Transform assignment expression where the left-hand side is a computed member expression
    /// with `super` as member expr object.
    ///
    /// * `super[prop] = value`
    ///   -> `_superPropSet(_Class, prop, value, _Class, 1)`
    /// * `super[prop] += value`
    ///   -> `_superPropSet(_Class, prop, _superPropGet(_Class, prop, _Class) + value, _Class, 1)`
    /// * `super[prop] &&= value`
    ///   -> `_superPropGet(_Class, prop, _Class) && _superPropSet(_Class, prop, value, _Class, 1)`
    ///
    // `#[inline]` so that compiler sees that `expr` is an `Expression::AssignmentExpression`.
    #[inline]
    fn transform_assignment_expression_for_super_computed_member_expr(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        expr.replace_with(|expr| {
            let ExpressionKindOwned::AssignmentExpression(assign_expr) = expr.into_kind() else {
                unreachable!()
            };
            let AssignmentExpression { span, operator, right: value, left, .. } =
                assign_expr.unbox();
            let AssignmentTarget::MemberExpression(member) = left else { unreachable!() };
            let member = member.into_computed_member_expression().unwrap();
            let property = member.unbox().expression.into_inner_expression();
            self.transform_super_assignment_expression_impl(span, operator, property, value, ctx)
        });
    }

    /// Transform assignment expression where the left-hand side is a member expression with `super`
    /// as member expr object.
    ///
    /// * `=` -> `_superPropSet(_Class, <prop>, <value>, _Class, 1)`
    /// * `+=` -> `_superPropSet(_Class, <prop>, _superPropGet(_Class, <prop>, _Class) + <value>, 1)`
    /// * `&&=` -> `_superPropGet(_Class, <prop>, _Class) && _superPropSet(_Class, <prop>, <value>, _Class, 1)`
    fn transform_super_assignment_expression_impl(
        &mut self,
        span: Span,
        operator: AssignmentOperator,
        property: Expression<'a>,
        value: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        if operator == AssignmentOperator::Assign {
            // `super[prop] = value` -> `_superPropSet(_Class, prop, value, _Class, 1)`
            self.create_super_prop_set(span, property, value, ctx)
        } else {
            // Make 2 copies of `object`
            let (property1, property2) = self.class_properties.duplicate_object(property, ctx);

            if let Some(operator) = operator.to_binary_operator() {
                // `super[prop] += value`
                // -> `_superPropSet(_Class, prop, _superPropGet(_Class, prop, _Class) + value, _Class, 1)`

                // `_superPropGet(_Class, prop, _Class)`
                let get_call = self.create_super_prop_get(SPAN, property2, false, ctx);

                // `_superPropGet(_Class, prop, _Class) + value`
                let value = Expression::new_binary_expression(SPAN, get_call, operator, value, ctx);

                // `_superPropSet(_Class, prop, _superPropGet(_Class, prop, _Class) + value, 1)`
                self.create_super_prop_set(span, property1, value, ctx)
            } else if let Some(operator) = operator.to_logical_operator() {
                // `super[prop] &&= value`
                // -> `_superPropGet(_Class, prop, _Class) && _superPropSet(_Class, prop, value, _Class, 1)`

                // `_superPropGet(_Class, prop, _Class)`
                let get_call = self.create_super_prop_get(SPAN, property1, false, ctx);

                // `_superPropSet(_Class, prop, value, _Class, 1)`
                let set_call = self.create_super_prop_set(span, property2, value, ctx);

                // `_superPropGet(_Class, prop, _Class) && _superPropSet(_Class, prop, value, _Class, 1)`
                Expression::new_logical_expression(span, get_call, operator, set_call, ctx)
            } else {
                // The above covers all types of `AssignmentOperator`
                unreachable!();
            }
        }
    }

    /// Transform update expression where the argument is a member expression with `super`.
    ///
    /// * `++super.prop` or `super.prop--`
    ///   See [`Self::transform_update_expression_for_super_static_member_expr`]
    ///
    /// * `++super[prop]` or `super[prop]--`
    ///   See [`Self::transform_update_expression_for_super_computed_member_expr`]
    //
    // `#[inline]` so can bail out fast without a function call if `argument` is not a member expression
    // with `super` as member expression object (fairly rare).
    // Actual transform is broken out into separate functions.
    pub(super) fn transform_update_expression_for_super_assignment_target(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Some(update_expr) = expr.as_update_expression() else { unreachable!() };

        match &update_expr.argument {
            SimpleAssignmentTarget::MemberExpression(member)
                if member.as_static_member_expression().is_some_and(|m| m.object.is_super()) =>
            {
                self.transform_update_expression_for_super_static_member_expr(expr, ctx);
            }
            SimpleAssignmentTarget::MemberExpression(member)
                if member.as_computed_member_expression().is_some_and(|m| m.object.is_super()) =>
            {
                self.transform_update_expression_for_super_computed_member_expr(expr, ctx);
            }
            _ => {}
        }
    }

    /// Transform update expression (`++` or `--`) where argument is a static member expression
    /// with `super`.
    ///
    /// * `++super.prop` ->
    /// ```js
    /// _superPropSet(
    ///   _Outer,
    ///   "prop",
    ///   (
    ///     _super$prop = _superPropGet(_Outer, "prop", _Outer),
    ///     ++_super$prop
    ///   ),
    ///   _Outer,
    ///   1
    /// )
    /// ```
    ///
    /// * `super.prop--` ->
    /// ```js
    /// (
    ///   _superPropSet(
    ///     _Outer,
    ///     "prop",
    ///     (
    ///       _super$prop = _superPropGet(_Outer, "prop", _Outer),
    ///       _super$prop2 = _super$prop--,
    ///       _super$prop
    ///     ),
    ///     _Outer,
    ///     1
    ///   ),
    ///   _super$prop2
    /// )
    /// ```
    ///
    // `#[inline]` so that compiler sees that `expr` is an `Expression::UpdateExpression`.
    #[inline]
    fn transform_update_expression_for_super_static_member_expr(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        expr.replace_with(|expr| {
            let ExpressionKindOwned::UpdateExpression(mut update_expr) = expr.into_kind() else {
                unreachable!()
            };
            let SimpleAssignmentTarget::MemberExpression(member) = &mut update_expr.argument else {
                unreachable!()
            };
            let member = member.to_static_member_expression();

            let temp_var_name_base = get_var_name_from_node(member);

            let property = Expression::new_string_literal(
                member.property.span,
                member.property.name,
                None,
                ctx,
            );

            self.transform_super_update_expression_impl(
                &temp_var_name_base,
                update_expr,
                property,
                ctx,
            )
        });
    }

    /// Transform update expression (`++` or `--`) where argument is a computed member expression
    /// with `super`.
    ///
    /// * `++super[prop]` ->
    /// ```js
    /// _superPropSet(
    ///   _Outer,
    ///   prop,
    ///   (
    ///     _super$prop = _superPropGet(_Outer, prop, _Outer),
    ///     ++_super$prop
    ///   ),
    ///   _Outer,
    ///   1
    /// )
    /// ```
    ///
    /// * `super[prop]--` ->
    /// ```js
    /// (
    ///   _superPropSet(
    ///     _Outer,
    ///     prop,
    ///     (
    ///       _super$prop = _superPropGet(_Outer, prop, _Outer),
    ///       _super$prop2 = _super$prop--,
    ///       _super$prop
    ///     ),
    ///     _Outer,
    ///     1
    ///   ),
    ///   _super$prop2
    /// )
    /// ```
    //
    // `#[inline]` so that compiler sees that `expr` is an `Expression::UpdateExpression`.
    #[inline]
    fn transform_update_expression_for_super_computed_member_expr(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        expr.replace_with(|expr| {
            let ExpressionKindOwned::UpdateExpression(mut update_expr) = expr.into_kind() else {
                unreachable!()
            };
            let SimpleAssignmentTarget::MemberExpression(member) = &mut update_expr.argument else {
                unreachable!()
            };
            let member = member.to_computed_member_expression_mut();

            let temp_var_name_base = get_var_name_from_node(&*member);

            let property = member.expression.get_inner_expression_mut().take_in(ctx);

            self.transform_super_update_expression_impl(
                &temp_var_name_base,
                update_expr,
                property,
                ctx,
            )
        });
    }

    /// Transform update expression (`++` or `--`) where argument is a member expression with `super`.
    ///
    /// * `++super[prop]` ->
    /// ```js
    /// _superPropSet(
    ///   _Outer,
    ///   prop,
    ///   (
    ///     _super$prop = _superPropGet(_Outer, prop, _Outer),
    ///     ++_super$prop
    ///   ),
    ///   _Outer,
    ///   1
    /// )
    /// ```
    ///
    /// * `super[prop]--` ->
    /// ```js
    /// (
    ///   _superPropSet(
    ///     _Outer,
    ///     prop,
    ///     (
    ///       _super$prop = _superPropGet(_Outer, prop, _Outer),
    ///       _super$prop2 = _super$prop--,
    ///       _super$prop
    ///     ),
    ///     _Outer,
    ///     1
    ///   ),
    ///   _super$prop2
    /// )
    /// ```
    fn transform_super_update_expression_impl(
        &mut self,
        temp_var_name_base: &str,
        mut update_expr: ArenaBox<'a, UpdateExpression<'a>>,
        property: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        // Make 2 copies of `property`
        let (property1, property2) = self.class_properties.duplicate_object(property, ctx);

        // `_superPropGet(_Class, prop, _Class)`
        let get_call = self.create_super_prop_get(SPAN, property2, false, ctx);

        // `_super$prop = _superPropGet(_Class, prop, _Class)`
        let temp_binding = VarDeclarationsStore::create_uid_var(temp_var_name_base, ctx);
        let assignment = create_assignment(&temp_binding, get_call, SPAN, ctx);

        // `++_super$prop` / `_super$prop++` (reusing existing `UpdateExpression`)
        let span = update_expr.span;
        let prefix = update_expr.prefix;
        update_expr.span = SPAN;
        update_expr.argument = temp_binding.create_read_write_simple_target(ctx);
        let update_expr = Expression::UpdateExpression(update_expr);

        if prefix {
            // Source = `++super$prop` (prefix `++`)
            // `(_super$prop = _superPropGet(_Class, prop, _Class), ++_super$prop)`
            let value = Expression::new_sequence_expression(SPAN, [assignment, update_expr], ctx);
            // `_superPropSet(_Class, prop, value, _Class, 1)`
            self.create_super_prop_set(span, property1, value, ctx)
        } else {
            // Source = `super.prop++` (postfix `++`)
            // `_super$prop2 = _super$prop++`
            let temp_binding2 = VarDeclarationsStore::create_uid_var(temp_var_name_base, ctx);
            let assignment2 = create_assignment(&temp_binding2, update_expr, SPAN, ctx);

            // `(_super$prop = _superPropGet(_Class, prop, _Class), _super$prop2 = _super$prop++, _super$prop)`
            let value = Expression::new_sequence_expression(
                SPAN,
                [assignment, assignment2, temp_binding.create_read_expression(ctx)],
                ctx,
            );

            // `_superPropSet(_Class, prop, value, _Class, 1)`
            let set_call = self.create_super_prop_set(span, property1, value, ctx);
            // `(_superPropSet(_Class, prop, value, _Class, 1), _super$prop2)`
            Expression::new_sequence_expression(
                span,
                [set_call, temp_binding2.create_read_expression(ctx)],
                ctx,
            )
        }
    }

    /// Member:
    ///  `_superPropGet(_Class, prop, _Class)`
    ///
    /// Callee:
    ///  `_superPropGet(_Class, prop, _Class, 2)`
    fn create_super_prop_get(
        &mut self,
        _span: Span,
        property: Expression<'a>,
        is_callee: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let (class, receiver) = self.get_class_binding_arguments(ctx);
        let property = Argument::from(property);

        let arguments = if is_callee {
            // `(_Class, prop, _Class, 2)`
            let two = Expression::new_numeric_literal(SPAN, 2.0, None, NumberBase::Decimal, ctx);
            ArenaVec::from_array_in([class, property, receiver, Argument::from(two)], ctx)
        } else {
            // `(_Class, prop, _Class)`
            ArenaVec::from_array_in([class, property, receiver], ctx)
        };

        // `_superPropGet(_Class, prop, _Class)` or `_superPropGet(_Class, prop, _Class, 2)`
        helper_call_expr(Helper::SuperPropGet, arguments, ctx)
    }

    /// `_superPropSet(_Class, prop, value, _Class, 1)`
    fn create_super_prop_set(
        &mut self,
        _span: Span,
        property: Expression<'a>,
        value: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let (class, receiver) = self.get_class_binding_arguments(ctx);
        let arguments = ArenaVec::from_array_in(
            [
                class,
                Argument::from(property),
                Argument::from(value),
                receiver,
                Argument::new_numeric_literal(SPAN, 1.0, None, NumberBase::Decimal, ctx),
            ],
            ctx,
        );
        helper_call_expr(Helper::SuperPropSet, arguments, ctx)
    }

    /// * [`ClassPropertiesSuperConverterMode::Static`]
    ///   (_Class, _Class)
    ///
    /// * [`ClassPropertiesSuperConverterMode::PrivateMethod`]
    ///   (_Class.prototype, this)
    ///
    /// * [`ClassPropertiesSuperConverterMode::StaticPrivateMethod`]
    ///   (_Class, this)
    fn get_class_binding_arguments(
        &mut self,
        ctx: &mut TraverseCtx<'a>,
    ) -> (Argument<'a>, Argument<'a>) {
        let temp_binding =
            self.class_properties.current_class_mut().bindings.get_or_init_static_binding(ctx);
        let mut class = temp_binding.create_read_expression(ctx);
        let receiver = match self.mode {
            ClassPropertiesSuperConverterMode::Static => temp_binding.create_read_expression(ctx),
            ClassPropertiesSuperConverterMode::PrivateMethod => {
                // TODO(improve-on-babel): `superPropGet` and `superPropSet` helper function has a flag
                // to use `class.prototype` rather than `class`. We should consider using that flag here.
                // <https://github.com/babel/babel/blob/1fbdb64a7fcc3488797e312506dbacff746d4e41/packages/babel-helpers/src/helpers/superPropGet.ts>
                class = create_prototype_member(class, SPAN, ctx);
                Expression::new_this_expression(SPAN, ctx)
            }
            ClassPropertiesSuperConverterMode::StaticPrivateMethod => {
                Expression::new_this_expression(SPAN, ctx)
            }
        };

        (Argument::from(class), Argument::from(receiver))
    }
}
