mod options;

use std::rc::Rc;

pub use options::ES2016Options;
use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::context::Ctx;

#[allow(dead_code)]
pub struct ES2016<'a> {
    ctx: Ctx<'a>,
    options: ES2016Options,
}

impl<'a> ES2016<'a> {
    pub fn new(options: ES2016Options, ctx: Ctx<'a>) -> Self {
        Self { ctx, options }
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        if self.options.exponentiation_operator {
            match expr {
                Expression::BinaryExpression(e)
                    if matches!(e.operator, BinaryOperator::Exponential) =>
                {
                    *expr = self.ctx.ast.expression_call(
                        SPAN,
                        self.ctx.ast.vec_from_iter([
                            self.ctx
                                .ast
                                .argument_expression(self.ctx.ast.move_expression(&mut e.left)),
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
}
