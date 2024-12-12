//! ES2022: Class Properties
//! Transform of super member expressions.

use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_traverse::TraverseCtx;

use crate::Helper;

use super::ClassProperties;

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    /// Transform member expression where object is `super`.
    ///
    /// - `super.prop` -> `_superPropGet(_classBinding, "prop", _classBinding)`
    /// - `super[expr]` -> `_superPropGet(_classBinding, expr, _classBinding)`
    //
    // `#[inline]` so that compiler sees that `expr` is an `Expression::StaticMemberExpression`
    // or `Expression::ComputedMemberExpression`.
    #[inline]
    pub fn transform_member_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let member = expr.to_member_expression();
        if matches!(member.object(), Expression::Super(_)) {
            self.transform_member_expression_impl(expr, ctx);
        }
    }

    fn transform_member_expression_impl(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let property = match expr {
            // `super.prop` -> `"prop"`
            Expression::StaticMemberExpression(member) => {
                let property = &member.property;
                ctx.ast.expression_string_literal(
                    property.span,
                    property.name.clone(),
                    Some(property.name.clone()),
                )
            }
            // `super[expr]` -> `expr`
            Expression::ComputedMemberExpression(member) => {
                ctx.ast.move_expression(&mut member.expression)
            }
            Expression::PrivateFieldExpression(_) => {
                unreachable!("`super` cannot access private fields")
            }
            _ => return,
        };

        *expr = self.create_super_prop_get(expr.span(), property, ctx);
    }

    // `_superPropGet(_classBinding, "prop", _classBinding)`
    fn create_super_prop_get(
        &mut self,
        span: Span,
        property: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let class_binding = self.get_temp_binding(ctx);
        // (_classBinding, "prop", _classBinding)
        let arguments = ctx.ast.vec_from_array([
            Argument::from(class_binding.create_read_expression(ctx)),
            Argument::from(property),
            Argument::from(class_binding.create_read_expression(ctx)),
        ]);
        self.ctx.helper_call_expr(Helper::SuperPropGet, span, arguments, ctx)
    }
}
