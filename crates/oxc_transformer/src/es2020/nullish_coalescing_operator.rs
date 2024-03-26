use serde::Deserialize;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, LogicalOperator};

use crate::{context::TransformerCtx, options::TransformTarget, utils::CreateVars};

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub struct NullishCoalescingOperatorOptions {
    /// When true, this transform will pretend `document.all` does not exist,
    /// and perform loose equality checks with null instead of strict equality checks against both null and undefined.
    #[serde(default)]
    loose: bool,
}

/// ES2020: Nullish Coalescing Operator
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-nullish-coalescing-operator>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-nullish-coalescing-operator>
pub struct NullishCoalescingOperator<'a> {
    no_document_all: bool,
    ctx: TransformerCtx<'a>,
    vars: Vec<'a, VariableDeclarator<'a>>,
}

impl<'a> CreateVars<'a> for NullishCoalescingOperator<'a> {
    fn ctx(&self) -> &TransformerCtx<'a> {
        &self.ctx
    }

    fn vars_mut(&mut self) -> &mut Vec<'a, VariableDeclarator<'a>> {
        &mut self.vars
    }
}

impl<'a> NullishCoalescingOperator<'a> {
    pub fn new(ctx: TransformerCtx<'a>) -> Option<Self> {
        (ctx.options.target < TransformTarget::ES2020
            || ctx.options.nullish_coalescing_operator.is_some())
        .then(|| {
            let no_document_all = ctx.options.assumptions.no_document_all
                || ctx.options.nullish_coalescing_operator.is_some_and(|o| o.loose);
            let vars = ctx.ast.new_vec();
            Self { no_document_all, ctx, vars }
        })
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        // left ?? right
        let Expression::LogicalExpression(logical_expr) = expr else { return };
        if logical_expr.operator != LogicalOperator::Coalesce {
            return;
        }

        let reference;
        let assignment;

        // skip creating extra reference when `left` is static
        if self.ctx.symbols().is_static(&logical_expr.left) {
            reference = self.ctx.ast.copy(&logical_expr.left);
            assignment = self.ctx.ast.copy(&logical_expr.left);
        } else {
            let ident = self.create_new_var_with_expression(&logical_expr.left);
            reference = self.ctx.ast.identifier_reference_expression(ident.clone());
            let left = self.ctx.ast.simple_assignment_target_identifier(ident);
            let right = self.ctx.ast.copy(&logical_expr.left);
            assignment =
                self.ctx.ast.assignment_expression(SPAN, AssignmentOperator::Assign, left, right);
        };

        let test = if self.no_document_all {
            let null = self.ctx.ast.literal_null_expression(NullLiteral::new(SPAN));
            self.ctx.ast.binary_expression(SPAN, assignment, BinaryOperator::Inequality, null)
        } else {
            let op = BinaryOperator::StrictInequality;
            let null = self.ctx.ast.literal_null_expression(NullLiteral::new(SPAN));
            let left =
                self.ctx.ast.binary_expression(SPAN, self.ctx.ast.copy(&assignment), op, null);

            let right = self.ctx.ast.binary_expression(
                SPAN,
                self.ctx.ast.copy(&reference),
                op,
                self.ctx.ast.void_0(),
            );

            self.ctx.ast.logical_expression(SPAN, left, LogicalOperator::And, right)
        };

        let right = self.ctx.ast.move_expression(&mut logical_expr.right);

        *expr = self.ctx.ast.conditional_expression(SPAN, test, reference, right);
    }
}
