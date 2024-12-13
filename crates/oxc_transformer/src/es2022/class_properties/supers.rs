//! ES2022: Class Properties
//! Transform of `super` expressions.

use oxc_ast::ast::*;
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
        if matches!(member.object, Expression::Super(_)) {
            *expr = self.transform_static_member_expression_impl(member, ctx);
        }
    }

    fn transform_static_member_expression_impl(
        &mut self,
        member: &mut StaticMemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let property = &member.property;
        let property = ctx.ast.expression_string_literal(
            property.span,
            property.name.clone(),
            Some(property.name.clone()),
        );
        self.create_super_prop_get(member.span, property, ctx)
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
        if matches!(member.object, Expression::Super(_)) {
            *expr = self.transform_computed_member_expression_impl(member, ctx);
        }
    }

    fn transform_computed_member_expression_impl(
        &mut self,
        member: &mut ComputedMemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let property = ctx.ast.move_expression(&mut member.expression);
        self.create_super_prop_get(member.span, property, ctx)
    }

    /// `_superPropGet(_Class, prop, _Class)`
    fn create_super_prop_get(
        &mut self,
        span: Span,
        property: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let class_binding = self.get_temp_binding(ctx);
        // `(_Class, prop, _Class)`
        let arguments = ctx.ast.vec_from_array([
            Argument::from(class_binding.create_read_expression(ctx)),
            Argument::from(property),
            Argument::from(class_binding.create_read_expression(ctx)),
        ]);
        // `_superPropGet(_Class, prop, _Class)`
        self.ctx.helper_call_expr(Helper::SuperPropGet, span, arguments, ctx)
    }
}
