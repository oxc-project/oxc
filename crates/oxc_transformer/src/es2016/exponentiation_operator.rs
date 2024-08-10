use oxc_ast::ast::{BinaryExpression, BinaryOperator, Expression, TSTypeParameterInstantiation};
use oxc_span::SPAN;

use crate::context::Ctx;

pub struct ExponentiationOperator<'a> {
    pub ctx: Ctx<'a>,
}

impl<'a> ExponentiationOperator<'a> {
    pub fn transform_expression(&self, expr: &mut Expression<'a>) {
        match expr {
            Expression::BinaryExpression(e)
                if matches!(e.operator, BinaryOperator::Exponential) =>
            {
                *expr = self.ctx.ast.expression_call(
                    SPAN,
                    self.ctx.ast.vec_from_iter([
                        self.ctx.ast.argument_expression(self.ctx.ast.move_expression(&mut e.left)),
                        self.ctx
                            .ast
                            .argument_expression(self.ctx.ast.move_expression(&mut e.right)),
                    ]),
                    self.ctx.ast.expression_member(self.ctx.ast.member_expression_static(
                        SPAN,
                        self.ctx.ast.expression_identifier_reference(SPAN, "Math"),
                        self.ctx.ast.identifier_name(SPAN, "pow"),
                        false,
                    )),
                    Option::<TSTypeParameterInstantiation>::None,
                    false,
                );
            }
            _ => return,
        };
    }
}
