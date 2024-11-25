//! ES2022: Class Properties
//! Transform of private property uses e.g. `this.#prop`.

use std::mem;

use oxc_allocator::Box as ArenaBox;
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_syntax::{
    reference::{ReferenceFlags, ReferenceId},
    symbol::SymbolFlags,
};
use oxc_traverse::{ast_operations::get_var_name_from_node, BoundIdentifier, TraverseCtx};

use crate::common::helper_loader::Helper;

use super::{
    utils::{create_assignment, create_underscore_ident_name},
    ClassProperties, PrivateProp,
};

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    /// Transform private field expression.
    ///
    /// Instance prop: `object.#prop` -> `_classPrivateFieldGet2(_prop, object)`
    /// Static prop: `object.#prop` -> `_assertClassBrand(Class, object, _prop)._`
    //
    // `#[inline]` so that compiler sees that `expr` is an `Expression::PrivateFieldExpression`.
    #[inline]
    pub(super) fn transform_private_field_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let owned_expr = ctx.ast.move_expression(expr);
        let Expression::PrivateFieldExpression(field_expr) = owned_expr else { unreachable!() };
        *expr = self.transform_private_field_expression_impl(field_expr, ctx);
    }

    fn transform_private_field_expression_impl(
        &mut self,
        field_expr: ArenaBox<'a, PrivateFieldExpression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let prop_details = self.lookup_private_property(&field_expr.field);
        // TODO: Should never be `None` - only because implementation is incomplete.
        let Some((prop, class_name_binding, is_declaration)) = prop_details else {
            return Expression::PrivateFieldExpression(field_expr);
        };
        let prop_ident = prop.binding.create_read_expression(ctx);

        // TODO: Move this to top of function once `lookup_private_property` does not return `Option`
        let PrivateFieldExpression { span, object, .. } = field_expr.unbox();

        if prop.is_static {
            // TODO: Ensure there are tests for nested classes with references to private static props
            // of outer class inside inner class, to make sure we're getting the right `class_name_binding`.
            let class_name_binding = class_name_binding.as_ref().unwrap();

            // If `object` is reference to class name, there's no need for the class brand assertion
            if let Some(reference_id) =
                Self::shortcut_static_class(is_declaration, class_name_binding, &object, ctx)
            {
                // `_prop._`
                ctx.symbols_mut()
                    .delete_resolved_reference(class_name_binding.symbol_id, reference_id);
                Self::create_underscore_member_expression(prop_ident, span, ctx)
            } else {
                // `_assertClassBrand(Class, object, _prop)._`
                self.create_assert_class_brand_underscore(
                    class_name_binding.create_read_expression(ctx),
                    object,
                    prop_ident,
                    span,
                    ctx,
                )
            }
        } else {
            // `_classPrivateFieldGet2(_prop, object)`
            self.create_private_field_get(prop_ident, object, span, ctx)
        }
    }

    /// Check if can use shorter version of static private prop transform.
    ///
    /// Can if both:
    /// 1. Class is a declaration, not an expression.
    /// 2. `object` is an `IdentifierReference` referring to class name binding.
    ///
    /// If can use shorter version, returns `ReferenceId` of the `IdentifierReference`.
    //
    // TODO(improve-on-babel): No reason not to use the short version for class expressions too.
    // TODO: Take `SymbolId` instead of `class_name_binding: &BoundIdentifier<'a>`?
    fn shortcut_static_class(
        is_declaration: bool,
        class_name_binding: &BoundIdentifier<'a>,
        object: &Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<ReferenceId> {
        if is_declaration {
            if let Expression::Identifier(ident) = object {
                let reference_id = ident.reference_id();
                if let Some(symbol_id) = ctx.symbols().get_reference(reference_id).symbol_id() {
                    if symbol_id == class_name_binding.symbol_id {
                        return Some(reference_id);
                    }
                }
            }
        }

        None
    }

    /// Transform call expression where callee is private field.
    ///
    /// Instance prop: `object.#prop(arg)` -> `_classPrivateFieldGet2(_prop, object).call(object, arg)`
    /// Static prop: `object.#prop(arg)` -> `_assertClassBrand(Class, object, _prop)._.call(object, arg)`
    ///
    /// Output in both cases contains a `CallExpression`, so mutate existing `CallExpression`
    /// rather than creating a new one.
    //
    // `#[inline]` so that compiler sees that `expr` is an `Expression::CallExpression`
    #[inline]
    pub(super) fn transform_call_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::CallExpression(call_expr) = expr else { unreachable!() };
        if matches!(&call_expr.callee, Expression::PrivateFieldExpression(_)) {
            self.transform_call_expression_impl(expr, ctx);
        };
    }

    fn transform_call_expression_impl(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Unfortunately no way to make compiler see that these branches are provably unreachable.
        // This function is much too large inline, because `transform_static_assignment_expression`
        // and `transform_instance_assignment_expression` are inlined into it.
        let Expression::CallExpression(call_expr) = expr else { unreachable!() };
        let Expression::PrivateFieldExpression(field_expr) = &mut call_expr.callee else {
            unreachable!()
        };
        let prop_details = self.lookup_private_property(&field_expr.field);
        // TODO: Should never be `None` - only because implementation is incomplete.
        let Some((prop, class_name_binding, is_declaration)) = prop_details else { return };
        let prop_ident = prop.binding.create_read_expression(ctx);

        let object = ctx.ast.move_expression(&mut field_expr.object);

        // Get replacement for callee
        let (callee, object) = if prop.is_static {
            // TODO: If `object` is reference to class name, and class is declaration, use shortcut `_prop._.call(Class)`.
            // TODO(improve-on-babel): No reason not to apply these shortcuts for class expressions too.

            // `object.#prop(arg)` -> `_assertClassBrand(Class, object, _prop)._.call(object, arg)`
            // or shortcut `_prop._.call(object, arg)`
            let class_name_binding = class_name_binding.as_ref().unwrap();
            let class_ident = class_name_binding.create_read_expression(ctx);

            // If `object` is reference to class name, there's no need for the class brand assertion
            // TODO: Combine this check with `duplicate_object`. Both check if `object` is an identifier,
            // and look up the `SymbolId`
            if Self::shortcut_static_class(is_declaration, class_name_binding, &object, ctx)
                .is_some()
            {
                // `_prop._`
                let callee =
                    Self::create_underscore_member_expression(prop_ident, field_expr.span, ctx);
                (callee, object)
            } else {
                // Make 2 copies of `object`
                let (object1, object2) = self.duplicate_object(object, ctx);

                // `_assertClassBrand(Class, object, _prop)._`
                // TODO: Ensure there are tests for nested classes with references to private static props
                // of outer class inside inner class, to make sure we're getting the right `class_name_binding`.
                let assert_obj = self.create_assert_class_brand_underscore(
                    class_ident,
                    object1,
                    prop_ident,
                    field_expr.span,
                    ctx,
                );
                (assert_obj, object2)
            }
        } else {
            // `object.#prop(arg)` -> `_classPrivateFieldGet2(_prop, object).call(object, arg)`
            // Make 2 copies of `object`
            let (object1, object2) = self.duplicate_object(object, ctx);

            // `_classPrivateFieldGet2(_prop, object)`
            let get_call = self.create_private_field_get(prop_ident, object1, field_expr.span, ctx);
            (get_call, object2)
        };

        // Substitute `<callee>.call` as callee of call expression
        call_expr.callee = Expression::from(ctx.ast.member_expression_static(
            SPAN,
            callee,
            ctx.ast.identifier_name(SPAN, Atom::from("call")),
            false,
        ));
        // Add `object` to call arguments
        call_expr.arguments.insert(0, Argument::from(object));
    }

    /// Transform assignment to private field.
    //
    // `#[inline]` so that compiler sees that `expr` is an `Expression::AssignmentExpression`
    #[inline]
    pub(super) fn transform_assignment_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::AssignmentExpression(assign_expr) = expr else { unreachable!() };
        if matches!(&assign_expr.left, AssignmentTarget::PrivateFieldExpression(_)) {
            self.transform_assignment_expression_impl(expr, ctx);
        };
    }

    fn transform_assignment_expression_impl(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Unfortunately no way to make compiler see that these branches are provably unreachable.
        // This function is much too large inline, because `transform_static_assignment_expression`
        // and `transform_instance_assignment_expression` are inlined into it.
        let Expression::AssignmentExpression(assign_expr) = expr else { unreachable!() };
        let AssignmentTarget::PrivateFieldExpression(field_expr) = &mut assign_expr.left else {
            unreachable!()
        };

        let prop_details = self.lookup_private_property(&field_expr.field);
        // TODO: Should never be `None` - only because implementation is incomplete.
        let Some((prop, class_name_binding, is_declaration)) = prop_details else { return };

        // Note: `transform_static_assignment_expression` and `transform_instance_assignment_expression`
        // are marked `#[inline]`, so hopefully compiler will see these clones of `BoundIdentifier`s
        // can be elided.
        // Can't break this up into separate functions otherwise, as `&BoundIdentifier`s keep `&self` ref
        // taken by `lookup_private_property` alive.
        let prop_binding = prop.binding.clone();

        if prop.is_static {
            let class_name_binding = class_name_binding.as_ref().unwrap().clone();
            self.transform_static_assignment_expression(
                expr,
                prop_binding,
                class_name_binding,
                is_declaration,
                ctx,
            );
        } else {
            self.transform_instance_assignment_expression(expr, prop_binding, ctx);
        }
    }

    /// Transform assignment expression with static private prop as assignee.
    ///
    /// * `object.#prop = value`
    ///   -> `_prop._ = _assertClassBrand(Class, object, value)`
    /// * `object.#prop += value`
    ///   -> `_prop._ = _assertClassBrand(Class, object, _assertClassBrand(Class, object, _prop)._ + value)`
    /// * `object.#prop &&= value`
    ///   -> `_assertClassBrand(Class, object, _prop)._ && (_prop._ = _assertClassBrand(Class, object, value))`
    ///
    /// Output in all cases contains an `AssignmentExpression`, so mutate existing `AssignmentExpression`
    /// rather than creating a new one.
    //
    // `#[inline]` so that compiler sees `expr` is an `Expression::AssignmentExpression` with
    // `AssignmentTarget::PrivateFieldExpression` on left, and that clones in
    // `transform_assignment_expression` can be elided.
    #[inline]
    #[expect(clippy::needless_pass_by_value)]
    fn transform_static_assignment_expression(
        &mut self,
        expr: &mut Expression<'a>,
        prop_binding: BoundIdentifier<'a>,
        class_name_binding: BoundIdentifier<'a>,
        is_declaration: bool,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::AssignmentExpression(assign_expr) = expr else { unreachable!() };
        let operator = assign_expr.operator;
        let AssignmentTarget::PrivateFieldExpression(field_expr) = &mut assign_expr.left else {
            unreachable!()
        };

        // Check if object (`object` in `object.#prop`) is a reference to class name
        // TODO: Combine this check with `duplicate_object`. Both check if `object` is an identifier,
        // and look up the `SymbolId`.
        let object_reference_id = Self::shortcut_static_class(
            is_declaration,
            &class_name_binding,
            &field_expr.object,
            ctx,
        );

        // If `object` is reference to class name, there's no need for the class brand assertion.
        // `Class.#prop = value` -> `_prop._ = value`
        // `Class.#prop += value` -> `_prop._ = _prop._ + value`
        // `Class.#prop &&= value` -> `_prop._ && (_prop._ = 1)`
        // TODO(improve-on-babel): These shortcuts could be shorter - just swap `Class.#prop` for `_prop._`.
        // Or does that behave slightly differently if `Class.#prop` is an object with `valueOf` method?
        if let Some(reference_id) = object_reference_id {
            // Replace left side of assignment with `_prop._`
            let field_expr_span = field_expr.span;
            assign_expr.left = Self::create_underscore_member_expr_target(
                prop_binding.create_read_expression(ctx),
                field_expr_span,
                ctx,
            );

            // Delete reference for `object` as `object.#prop` has been removed
            ctx.symbols_mut().delete_resolved_reference(class_name_binding.symbol_id, reference_id);

            if operator == AssignmentOperator::Assign {
                // `Class.#prop = value` -> `_prop._ = value`
                // Left side already replaced with `_prop._`. Nothing further to do.
            } else {
                let prop_obj = Self::create_underscore_member_expression(
                    prop_binding.create_read_expression(ctx),
                    field_expr_span,
                    ctx,
                );

                if let Some(operator) = operator.to_binary_operator() {
                    // `Class.#prop += value` -> `_prop._ = _prop._ + value`
                    let value = ctx.ast.move_expression(&mut assign_expr.right);
                    assign_expr.operator = AssignmentOperator::Assign;
                    assign_expr.right = ctx.ast.expression_binary(SPAN, prop_obj, operator, value);
                } else if let Some(operator) = operator.to_logical_operator() {
                    // `Class.#prop &&= value` -> `_prop._ && (_prop._ = 1)`
                    let span = assign_expr.span;
                    assign_expr.span = SPAN;
                    assign_expr.operator = AssignmentOperator::Assign;
                    let right = ctx.ast.move_expression(expr);
                    *expr = ctx.ast.expression_logical(span, prop_obj, operator, right);
                } else {
                    // The above covers all types of `AssignmentOperator`
                    unreachable!();
                }
            }
        } else {
            // Substitute left side of assignment with `_prop._`, and get owned `object` from old left side
            let assignee = Self::create_underscore_member_expr_target(
                prop_binding.create_read_expression(ctx),
                SPAN,
                ctx,
            );
            let old_assignee = mem::replace(&mut assign_expr.left, assignee);
            let field_expr = match old_assignee {
                AssignmentTarget::PrivateFieldExpression(field_expr) => field_expr.unbox(),
                _ => unreachable!(),
            };
            let object = field_expr.object;

            let class_ident = class_name_binding.create_read_expression(ctx);
            let value = ctx.ast.move_expression(&mut assign_expr.right);

            if operator == AssignmentOperator::Assign {
                // Replace right side of assignment with `_assertClassBrand(Class, object, _prop)`
                // TODO: Ensure there are tests for nested classes with references to private static props
                // of outer class inside inner class, to make sure we're getting the right `class_name_binding`.
                assign_expr.right = self.create_assert_class_brand(class_ident, object, value, ctx);
            } else {
                let class_ident = class_name_binding.create_read_expression(ctx);
                let value = ctx.ast.move_expression(&mut assign_expr.right);

                // Make 2 copies of `object`
                let (object1, object2) = self.duplicate_object(object, ctx);

                let prop_ident = prop_binding.create_read_expression(ctx);
                let class_ident2 = class_name_binding.create_read_expression(ctx);

                if let Some(operator) = operator.to_binary_operator() {
                    // `object.#prop += value`
                    // -> `_prop._ = _assertClassBrand(Class, object, _assertClassBrand(Class, object, _prop)._ + value)`

                    // `_assertClassBrand(Class, object, _prop)._`
                    let get_expr = self.create_assert_class_brand_underscore(
                        class_ident,
                        object2,
                        prop_ident,
                        SPAN,
                        ctx,
                    );
                    // `_assertClassBrand(Class, object, _prop)._ + value`
                    let value = ctx.ast.expression_binary(SPAN, get_expr, operator, value);
                    // `_assertClassBrand(Class, object, _assertClassBrand(Class, object, _prop)._ + value)`
                    assign_expr.right =
                        self.create_assert_class_brand(class_ident2, object1, value, ctx);
                } else if let Some(operator) = operator.to_logical_operator() {
                    // `object.#prop &&= value`
                    // -> `_assertClassBrand(Class, object, _prop)._ && (_prop._ = _assertClassBrand(Class, object, value))`

                    // `_assertClassBrand(Class, object, _prop)._`
                    let left = self.create_assert_class_brand_underscore(
                        class_ident,
                        object1,
                        prop_ident,
                        SPAN,
                        ctx,
                    );
                    // Mutate existing assignment expression to `_prop._ = _assertClassBrand(Class, object, value)`
                    // and take ownership of it
                    let span = assign_expr.span;
                    assign_expr.span = SPAN;
                    assign_expr.operator = AssignmentOperator::Assign;
                    assign_expr.right =
                        self.create_assert_class_brand(class_ident2, object2, value, ctx);
                    let right = ctx.ast.move_expression(expr);
                    // `_assertClassBrand(Class, object, _prop)._ && (_prop._ = _assertClassBrand(Class, object, value))`
                    *expr = ctx.ast.expression_logical(span, left, operator, right);
                } else {
                    // The above covers all types of `AssignmentOperator`
                    unreachable!();
                }
            }
        }
    }

    /// Transform assignment expression with instance private prop as assignee.
    ///
    /// * `object.#prop = value`
    ///   -> `_classPrivateFieldSet2(_prop, object, value)`
    /// * `object.#prop += value`
    ///   -> `_classPrivateFieldSet2(_prop, object, _classPrivateFieldGet2(_prop, object) + value)`
    /// * `object.#prop &&= value`
    ///   -> `_classPrivateFieldGet2(_prop, object) && _classPrivateFieldSet2(_prop, object, value)`
    ///
    /// Output in all cases contains an `AssignmentExpression`, so mutate existing `AssignmentExpression`
    /// rather than creating a new one.
    //
    // `#[inline]` so that compiler sees `expr` is an `Expression::AssignmentExpression` with
    // `AssignmentTarget::PrivateFieldExpression` on left, and that clones in
    // `transform_assignment_expression` can be elided.
    #[inline]
    #[expect(clippy::needless_pass_by_value)]
    fn transform_instance_assignment_expression(
        &mut self,
        expr: &mut Expression<'a>,
        prop_binding: BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let assign_expr = match ctx.ast.move_expression(expr) {
            Expression::AssignmentExpression(assign_expr) => assign_expr.unbox(),
            _ => unreachable!(),
        };
        let AssignmentExpression { span, operator, right: value, left } = assign_expr;
        let object = match left {
            AssignmentTarget::PrivateFieldExpression(field_expr) => field_expr.unbox().object,
            _ => unreachable!(),
        };

        let prop_ident = prop_binding.create_read_expression(ctx);

        // TODO: Different output in for statements e.g. `private/1-helpermemberexpressionfunction/input.js`

        if operator == AssignmentOperator::Assign {
            // `object.#prop = value` -> `_classPrivateFieldSet2(_prop, object, value)`
            *expr = self.create_private_field_set(prop_ident, object, value, span, ctx);
        } else {
            // Make 2 copies of `object`
            let (object1, object2) = self.duplicate_object(object, ctx);

            let prop_ident2 = prop_binding.create_read_expression(ctx);

            if let Some(operator) = operator.to_binary_operator() {
                // `object.#prop += value`
                // -> `_classPrivateFieldSet2(_prop, object, _classPrivateFieldGet2(_prop, object) + value)`

                // `_classPrivateFieldGet2(_prop, object)`
                let get_call = self.create_private_field_get(prop_ident, object2, SPAN, ctx);

                // `_classPrivateFieldGet2(_prop, object) + value`
                let value = ctx.ast.expression_binary(SPAN, get_call, operator, value);

                // `_classPrivateFieldSet2(_prop, object, _classPrivateFieldGet2(_prop, object) + value)`
                *expr = self.create_private_field_set(prop_ident2, object1, value, span, ctx);
            } else if let Some(operator) = operator.to_logical_operator() {
                // `object.#prop &&= value`
                // -> `_classPrivateFieldGet2(_prop, object) && _classPrivateFieldSet2(_prop, object, value)`

                // `_classPrivateFieldGet2(_prop, object)`
                let get_call = self.create_private_field_get(prop_ident, object1, SPAN, ctx);

                // `_classPrivateFieldSet2(_prop, object, value)`
                let set_call =
                    self.create_private_field_set(prop_ident2, object2, value, SPAN, ctx);
                // `_classPrivateFieldGet2(_prop, object) && _classPrivateFieldSet2(_prop, object, value)`
                *expr = ctx.ast.expression_logical(span, get_call, operator, set_call);
            } else {
                // The above covers all types of `AssignmentOperator`
                unreachable!();
            }
        }
    }

    /// Transform update expression (`++` or `--`) where argument is private field.
    ///
    /// Instance prop:
    ///
    /// * `++object.#prop` ->
    /// ```js
    /// _classPrivateFieldSet(
    ///   _prop, object,
    ///   (_object$prop = _classPrivateFieldGet(_prop, object), ++_object$prop)
    /// ),
    /// ```
    ///
    /// * `object.#prop++` ->
    /// ```js
    /// (
    ///   _classPrivateFieldSet(
    ///     _prop, object,
    ///     (
    ///       _object$prop = _classPrivateFieldGet(_prop, object),
    ///       _object$prop2 = _object$prop++,
    ///       _object$prop
    ///     )
    ///   ),
    ///   _object$prop2
    /// )
    /// ```
    ///
    /// Static prop:
    ///
    /// * `++object.#prop++` ->
    /// ```js
    /// _prop._ = _assertClassBrand(
    ///   Class, object,
    ///   (_object$prop = _assertClassBrand(Class, object, _prop)._, ++_object$prop)
    /// )
    /// ```
    ///
    /// * `object.#prop++` ->
    /// ```js
    /// (
    ///   _prop._ = _assertClassBrand(
    ///     Class, object,
    ///     (
    ///       _object$prop = _assertClassBrand(Class, object, _prop)._,
    ///       _object$prop2 = _object$prop++,
    ///       _object$prop
    ///     )
    ///   ),
    ///   _object$prop2
    /// )
    /// ```
    ///
    /// Output in all cases contains an `UpdateExpression`, so mutate existing `UpdateExpression`
    /// rather than creating a new one.
    //
    // `#[inline]` so that compiler sees that `expr` is an `Expression::UpdateExpression`
    #[inline]
    pub(super) fn transform_update_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::UpdateExpression(update_expr) = expr else { unreachable!() };
        if matches!(&update_expr.argument, SimpleAssignmentTarget::PrivateFieldExpression(_)) {
            self.transform_update_expression_impl(expr, ctx);
        };
    }

    // TODO: Split up this function into 2 halves for static and instance props
    fn transform_update_expression_impl(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Unfortunately no way to make compiler see that these branches are provably unreachable.
        // This function is much too large inline, because `transform_static_assignment_expression`
        // and `transform_instance_assignment_expression` are inlined into it.
        let Expression::UpdateExpression(update_expr) = expr else { unreachable!() };
        let field_expr = match &mut update_expr.argument {
            SimpleAssignmentTarget::PrivateFieldExpression(field_expr) => field_expr.as_mut(),
            _ => unreachable!(),
        };

        let prop_details = self.lookup_private_property(&field_expr.field);
        // TODO: Should never be `None` - only because implementation is incomplete.
        let Some((prop, class_name_binding, is_declaration)) = prop_details else { return };
        let prop_ident = prop.binding.create_read_expression(ctx);
        let prop_ident2 = prop.binding.create_read_expression(ctx);

        let temp_var_name_base = get_var_name_from_node(field_expr);
        let temp_binding = ctx.generate_uid_in_current_scope(
            &temp_var_name_base,
            SymbolFlags::FunctionScopedVariable,
        );

        // TODO(improve-on-babel): Could avoid `move_expression` here and replace `update_expr.argument` instead.
        // Only doing this first to match the order Babel creates temp vars.
        let object = ctx.ast.move_expression(&mut field_expr.object);

        if prop.is_static {
            // TODO: If `object` is reference to class name, and class is declaration, use shortcuts:
            // `++Class.#prop` -> `_prop._ = ((_Class$prop = _prop._), ++_Class$prop)`
            // `Class.#prop++` -> `_prop._ = (_Class$prop = _prop._, _Class$prop2 = _Class$prop++, _Class$prop), _Class$prop2`
            // TODO(improve-on-babel): These shortcuts could be shorter - just `_prop._++` / `++_prop._`.
            // Or does that behave slightly differently if `Class.#prop` is an object with `valueOf` method?
            // TODO(improve-on-babel): No reason not to apply these shortcuts for class expressions too.

            // ```
            // _prop._ = _assertClassBrand(
            //   Class, object,
            //   (_object$prop = _assertClassBrand(Class, object, _prop)._, ++_object$prop)
            // )
            // ```
            let class_name_binding = class_name_binding.as_ref().unwrap().clone();

            // Check if object (`object` in `object.#prop`) is a reference to class name
            // TODO: Combine this check with `duplicate_object`. Both check if `object` is an identifier,
            // and look up the `SymbolId`.
            let object_reference_id =
                Self::shortcut_static_class(is_declaration, &class_name_binding, &object, ctx);

            // `_assertClassBrand(Class, object, _prop)._` or `_prop._`
            let (get_expr, object) = if let Some(reference_id) = object_reference_id {
                // Delete reference for `object` as `object.#prop` is being removed
                ctx.symbols_mut()
                    .delete_resolved_reference(class_name_binding.symbol_id, reference_id);

                // `_prop._`
                let get_expr = Self::create_underscore_member_expression(prop_ident, SPAN, ctx);
                (get_expr, object)
            } else {
                // Make 2 copies of `object`
                let (object1, object2) = self.duplicate_object(object, ctx);

                // `_assertClassBrand(Class, object, _prop)._`
                let get_call = self.create_assert_class_brand_underscore(
                    class_name_binding.create_read_expression(ctx),
                    object2,
                    prop_ident,
                    SPAN,
                    ctx,
                );
                (get_call, object1)
            };
            // `_object$prop = _assertClassBrand(Class, object, _prop)._`
            self.ctx.var_declarations.insert_var(&temp_binding, None, ctx);
            let assignment = create_assignment(&temp_binding, get_expr, ctx);

            // `++_object$prop` / `_object$prop++` (reusing existing `UpdateExpression`)
            let UpdateExpression { span, prefix, .. } = **update_expr;
            update_expr.span = SPAN;
            update_expr.argument = temp_binding.create_read_write_simple_target(ctx);
            let update_expr = ctx.ast.move_expression(expr);

            if prefix {
                // Source = `++object.#prop` (prefix `++`)

                // `(_object$prop = _assertClassBrand(Class, object, _prop)._, ++_object$prop)`
                let mut value = ctx
                    .ast
                    .expression_sequence(SPAN, ctx.ast.vec_from_array([assignment, update_expr]));

                // `_assertClassBrand(Class, object, <value>)`
                if object_reference_id.is_none() {
                    let class_ident = class_name_binding.create_read_expression(ctx);
                    value = self.create_assert_class_brand(class_ident, object, value, ctx);
                }

                // `_prop._ = <value>`
                *expr = ctx.ast.expression_assignment(
                    span,
                    AssignmentOperator::Assign,
                    Self::create_underscore_member_expr_target(prop_ident2, SPAN, ctx),
                    value,
                );
            } else {
                // Source = `object.#prop++` (postfix `++`)

                // `_object$prop2 = _object$prop++`
                let temp_binding2 = ctx.generate_uid_in_current_scope(
                    &temp_var_name_base,
                    SymbolFlags::FunctionScopedVariable,
                );
                self.ctx.var_declarations.insert_var(&temp_binding2, None, ctx);
                let assignment2 = create_assignment(&temp_binding2, update_expr, ctx);

                // `(_object$prop = _assertClassBrand(Class, object, _prop)._, _object$prop2 = _object$prop++, _object$prop)`
                let mut value = ctx.ast.expression_sequence(
                    SPAN,
                    ctx.ast.vec_from_array([
                        assignment,
                        assignment2,
                        temp_binding.create_read_expression(ctx),
                    ]),
                );

                // `_assertClassBrand(Class, object, <value>)`
                if object_reference_id.is_none() {
                    let class_ident = class_name_binding.create_read_expression(ctx);
                    value = self.create_assert_class_brand(class_ident, object, value, ctx);
                }

                // `_prop._ = <value>`
                let assignment3 = ctx.ast.expression_assignment(
                    SPAN,
                    AssignmentOperator::Assign,
                    Self::create_underscore_member_expr_target(prop_ident2, SPAN, ctx),
                    value,
                );

                // `(_prop._ = <value>, _object$prop2)`
                // TODO(improve-on-babel): Final `_object$prop2` is only needed if this expression
                // is consumed (i.e. not in an `ExpressionStatement`)
                *expr = ctx.ast.expression_sequence(
                    span,
                    ctx.ast
                        .vec_from_array([assignment3, temp_binding2.create_read_expression(ctx)]),
                );
            }
        } else {
            // Make 2 copies of `object`
            let (object1, object2) = self.duplicate_object(object, ctx);

            // `_classPrivateFieldGet(_prop, object)`
            let get_call = self.create_private_field_get(prop_ident, object2, SPAN, ctx);

            // `_object$prop = _classPrivateFieldGet(_prop, object)`
            self.ctx.var_declarations.insert_var(&temp_binding, None, ctx);
            let assignment = create_assignment(&temp_binding, get_call, ctx);

            // `++_object$prop` / `_object$prop++` (reusing existing `UpdateExpression`)
            let UpdateExpression { span, prefix, .. } = **update_expr;
            update_expr.span = SPAN;
            update_expr.argument = temp_binding.create_read_write_simple_target(ctx);
            let update_expr = ctx.ast.move_expression(expr);

            if prefix {
                // Source = `++object.#prop` (prefix `++`)
                // `(_object$prop = _classPrivateFieldGet(_prop, object), ++_object$prop)`
                let value = ctx
                    .ast
                    .expression_sequence(SPAN, ctx.ast.vec_from_array([assignment, update_expr]));
                // `_classPrivateFieldSet(_prop, object, <value>)`
                *expr = self.create_private_field_set(prop_ident2, object1, value, span, ctx);
            } else {
                // Source = `object.#prop++` (postfix `++`)
                // `_object$prop2 = _object$prop++`
                let temp_binding2 = ctx.generate_uid_in_current_scope(
                    &temp_var_name_base,
                    SymbolFlags::FunctionScopedVariable,
                );
                self.ctx.var_declarations.insert_var(&temp_binding2, None, ctx);
                let assignment2 = create_assignment(&temp_binding2, update_expr, ctx);

                // `(_object$prop = _classPrivateFieldGet(_prop, object), _object$prop2 = _object$prop++, _object$prop)`
                let value = ctx.ast.expression_sequence(
                    SPAN,
                    ctx.ast.vec_from_array([
                        assignment,
                        assignment2,
                        temp_binding.create_read_expression(ctx),
                    ]),
                );

                // `_classPrivateFieldSet(_prop, object, <value>)`
                let set_call =
                    self.create_private_field_set(prop_ident2, object1, value, span, ctx);
                // `(_classPrivateFieldSet(_prop, object, <value>), _object$prop2)`
                // TODO(improve-on-babel): Final `_object$prop2` is only needed if this expression
                // is consumed (i.e. not in an `ExpressionStatement`)
                *expr = ctx.ast.expression_sequence(
                    span,
                    ctx.ast.vec_from_array([set_call, temp_binding2.create_read_expression(ctx)]),
                );
            }
        }
    }

    /// Transform chain expression where includes a private field.
    //
    // `#[inline]` so that compiler sees that `expr` is an `Expression::ChainExpression`
    #[inline]
    #[expect(clippy::unused_self)]
    pub(super) fn transform_chain_expression(
        &mut self,
        expr: &mut Expression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::ChainExpression(_chain_expr) = expr else { unreachable!() };

        // TODO: `object?.#prop`
        // TODO: `object?.#prop()`
    }

    /// Transform tagged template expression where tag is a private field.
    ///
    /// "object.#prop`xyz`" -> "_classPrivateFieldGet(_prop, object).bind(object)`xyz`"
    //
    // `#[inline]` so that compiler sees that `expr` is an `Expression::TaggedTemplateExpression`
    #[inline]
    #[expect(clippy::unused_self)]
    pub(super) fn transform_tagged_template_expression(
        &mut self,
        expr: &mut Expression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::TaggedTemplateExpression(_tagged_temp_expr) = expr else { unreachable!() };

        // TODO: "object.#prop`xyz`"
        // See `private/tagged-template` fixture.
    }

    /// Transform private field in assignment pattern.
    ///
    /// Instance prop:
    /// * `[object.#prop] = arr` -> `[_toSetter(_classPrivateFieldSet, [_prop, object])._] = arr`
    /// * `({x: object.#prop} = obj)` -> `({ x: _toSetter(_classPrivateFieldSet, [_prop, object])._ } = obj)`
    ///
    /// Static prop:
    /// (same as `Expression::PrivateFieldExpression` is transformed to)
    /// * `[object.#prop] = arr` -> `[_assertClassBrand(Class, object, _prop)._] = arr`
    /// * `({x: object.#prop} = obj)` -> `({ x: _assertClassBrand(Class, object, _prop)._ } = obj)`
    //
    // `#[inline]` because most `AssignmentTarget`s are not `PrivateFieldExpression`s.
    // So we want to bail out in that common case without the cost of a function call.
    // Transform of `PrivateFieldExpression`s in broken out into `transform_assignment_target_impl` to
    // keep this function as small as possible.
    #[inline]
    pub(super) fn transform_assignment_target(
        &mut self,
        target: &mut AssignmentTarget<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // `object.#prop` in assignment pattern.
        // Must be in assignment pattern, as `enter_expression` already transformed `AssignmentExpression`s.
        if matches!(target, AssignmentTarget::PrivateFieldExpression(_)) {
            self.transform_assignment_target_impl(target, ctx);
        }
    }

    #[expect(clippy::unused_self)]
    fn transform_assignment_target_impl(
        &mut self,
        target: &mut AssignmentTarget<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        let AssignmentTarget::PrivateFieldExpression(_private_field) = target else {
            unreachable!()
        };

        // TODO: `[object.#prop] = value`
        // TODO: `({x: object.#prop} = value)`
    }

    /// Duplicate object to be used in get/set pair.
    ///
    /// If `object` may have side effects, create a temp var `_object` and assign to it.
    ///
    /// * `this` -> (`this`, `this`)
    /// * Bound identifier `object` -> `object`, `object`
    /// * Unbound identifier `object` -> `_object = object`, `_object`
    /// * Anything else `foo()` -> `_foo = foo()`, `_foo`
    ///
    /// Returns 2 `Expression`s. The first must be inserted into output first.
    fn duplicate_object(
        &mut self,
        object: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> (Expression<'a>, Expression<'a>) {
        // TODO: Handle if in a function's params
        let temp_var_binding = match &object {
            Expression::Identifier(ident) => {
                let reference = ctx.symbols_mut().get_reference_mut(ident.reference_id());
                if let Some(symbol_id) = reference.symbol_id() {
                    // Reading bound identifier cannot have side effects, so no need for temp var
                    let binding = BoundIdentifier::new(ident.name.clone(), symbol_id);
                    let object1 = binding.create_spanned_read_expression(ident.span, ctx);
                    return (object1, object);
                }

                // Previously `x += 1` (`x` read + write), but moving to `_x = x` (`x` read only)
                *reference.flags_mut() = ReferenceFlags::Read;

                ctx.generate_uid_in_current_scope(&ident.name, SymbolFlags::FunctionScopedVariable)
            }
            Expression::ThisExpression(this) => {
                // Reading `this` cannot have side effects, so no need for temp var
                let object1 = ctx.ast.expression_this(this.span);
                return (object1, object);
            }
            _ => ctx.generate_uid_in_current_scope_based_on_node(
                &object,
                SymbolFlags::FunctionScopedVariable,
            ),
        };

        self.ctx.var_declarations.insert_var(&temp_var_binding, None, ctx);

        let object1 = create_assignment(&temp_var_binding, object, ctx);
        let object2 = temp_var_binding.create_read_expression(ctx);

        (object1, object2)
    }

    /// `_classPrivateFieldGet2(_prop, object)`
    fn create_private_field_get(
        &self,
        prop_ident: Expression<'a>,
        object: Expression<'a>,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        self.ctx.helper_call_expr(
            Helper::ClassPrivateFieldGet2,
            span,
            ctx.ast.vec_from_array([Argument::from(prop_ident), Argument::from(object)]),
            ctx,
        )
    }

    /// `_classPrivateFieldSet2(_prop, object, value)`
    fn create_private_field_set(
        &self,
        prop_ident: Expression<'a>,
        object: Expression<'a>,
        value: Expression<'a>,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        self.ctx.helper_call_expr(
            Helper::ClassPrivateFieldSet2,
            span,
            ctx.ast.vec_from_array([
                Argument::from(prop_ident),
                Argument::from(object),
                Argument::from(value),
            ]),
            ctx,
        )
    }

    /// `_assertClassBrand(Class, object, value)` or `_assertClassBrand(Class, object, _prop)`
    fn create_assert_class_brand(
        &self,
        class_ident: Expression<'a>,
        object: Expression<'a>,
        value_or_prop_ident: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        self.ctx.helper_call_expr(
            Helper::AssertClassBrand,
            SPAN,
            ctx.ast.vec_from_array([
                Argument::from(class_ident),
                Argument::from(object),
                Argument::from(value_or_prop_ident),
            ]),
            ctx,
        )
    }

    /// `_assertClassBrand(Class, object, _prop)._`
    fn create_assert_class_brand_underscore(
        &self,
        class_ident: Expression<'a>,
        object: Expression<'a>,
        prop_ident: Expression<'a>,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let func_call = self.create_assert_class_brand(class_ident, object, prop_ident, ctx);
        Self::create_underscore_member_expression(func_call, span, ctx)
    }

    /// Lookup details of private property referred to by `ident`.
    fn lookup_private_property(
        &self,
        ident: &PrivateIdentifier<'a>,
    ) -> Option<(&PrivateProp<'a>, &Option<BoundIdentifier<'a>>, /* is_declaration */ bool)> {
        // Check for binding in closest class first, then enclosing classes
        // TODO: Check there are tests for bindings in enclosing classes.
        for private_props in self.private_props_stack.as_slice().iter().rev() {
            if let Some(prop) = private_props.props.get(&ident.name) {
                return Some((
                    prop,
                    &private_props.class_name_binding,
                    private_props.is_declaration,
                ));
            }
        }
        // TODO: This should be unreachable. Only returning `None` because implementation is incomplete.
        None
    }

    /// Create `<object>._` assignment target.
    fn create_underscore_member_expr_target(
        object: Expression<'a>,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> AssignmentTarget<'a> {
        AssignmentTarget::from(Self::create_underscore_member_expr(object, span, ctx))
    }

    /// Create `<object>._` expression.
    fn create_underscore_member_expression(
        object: Expression<'a>,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        Expression::from(Self::create_underscore_member_expr(object, span, ctx))
    }

    /// Create `<object>._` member expression.
    fn create_underscore_member_expr(
        object: Expression<'a>,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> MemberExpression<'a> {
        ctx.ast.member_expression_static(span, object, create_underscore_ident_name(ctx), false)
    }
}
