//! ES2022: Class Properties
//! Transform of `super` expressions.

use oxc_allocator::Vec as ArenaVec;
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::Helper;

use super::ClassProperties;

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
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
        let Expression::StaticMemberExpression(member) = expr else { unreachable!() };
        if member.object.is_super() {
            *expr = self.transform_static_member_expression_impl(member, false, ctx);
        }
    }

    fn transform_static_member_expression_impl(
        &mut self,
        member: &mut StaticMemberExpression<'a>,
        is_callee: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let property = &member.property;
        let property =
            ctx.ast.expression_string_literal(property.span, property.name.clone(), None);
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
        let Expression::ComputedMemberExpression(member) = expr else { unreachable!() };
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
        let property = ctx.ast.move_expression(&mut member.expression);
        self.create_super_prop_get(member.span, property, is_callee, ctx)
    }

    /// Transform call expression where callee contains `super`.
    ///
    /// `super.method()` -> `_superPropGet(_Class, "method", _Class, 2)([])`
    /// `super.method(1)` -> `_superPropGet(_Class, "method", _Class, 2)([1])`
    //
    // `#[inline]` so that compiler sees that `expr` is an `Expression::CallExpression`.
    #[inline]
    pub(super) fn transform_super_call_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::CallExpression(call) = expr else { unreachable!() };
        let callee = &mut call.callee;
        match callee {
            Expression::StaticMemberExpression(member) if member.object.is_super() => {
                *callee = self.transform_static_member_expression_impl(member, true, ctx);
            }
            Expression::ComputedMemberExpression(member) if member.object.is_super() => {
                *callee = self.transform_computed_member_expression_impl(member, true, ctx);
            }
            _ => return,
        };
        Self::transform_super_call_expression_arguments(&mut call.arguments, ctx);
    }

    /// [A, B, C] -> [[A, B, C]]
    pub(super) fn transform_super_call_expression_arguments(
        arguments: &mut ArenaVec<'a, Argument<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let owned_arguments = ctx.ast.move_vec(arguments);
        let elements =
            ctx.ast.vec_from_iter(owned_arguments.into_iter().map(ArrayExpressionElement::from));
        let array = ctx.ast.expression_array(SPAN, elements, None);
        arguments.push(Argument::from(array));
    }

    /// Member:
    ///  `_superPropGet(_Class, prop, _Class)`
    ///
    /// Callee:
    ///  `_superPropGet(_Class, prop, _Class, 2)`
    fn create_super_prop_get(
        &mut self,
        span: Span,
        property: Expression<'a>,
        is_callee: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let class_binding = self.get_temp_binding(ctx);

        let ident1 = Argument::from(class_binding.create_read_expression(ctx));
        let ident2 = Argument::from(class_binding.create_read_expression(ctx));
        let property = Argument::from(property);

        let arguments = if is_callee {
            // `(_Class, prop, _Class, 2)`
            let two = ctx.ast.expression_numeric_literal(SPAN, 2.0, None, NumberBase::Decimal);
            ctx.ast.vec_from_array([ident1, property, ident2, Argument::from(two)])
        } else {
            // `(_Class, prop, _Class)`
            ctx.ast.vec_from_array([ident1, property, ident2])
        };

        // `_superPropGet(_Class, prop, _Class)` or `_superPropGet(_Class, prop, _Class, 2)`
        self.ctx.helper_call_expr(Helper::SuperPropGet, span, arguments, ctx)
    }
}
