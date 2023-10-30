use serde::Deserialize;
use std::rc::Rc;

use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder};
use oxc_span::SPAN;
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, LogicalOperator};

use crate::{
    context::TransformerCtx,
    options::{TransformOptions, TransformTarget},
    utils::CreateVars,
};

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

    ast: Rc<AstBuilder<'a>>,
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
    pub fn new(
        ast: Rc<AstBuilder<'a>>,
        ctx: TransformerCtx<'a>,
        options: &TransformOptions,
    ) -> Option<Self> {
        (options.target < TransformTarget::ES2020 || options.nullish_coalescing_operator.is_some())
            .then(|| {
                let no_document_all = options.assumptions.no_document_all
                    || options.nullish_coalescing_operator.is_some_and(|o| o.loose);
                let vars = ast.new_vec();
                Self { no_document_all, ast, ctx, vars }
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
            reference = self.ast.copy(&logical_expr.left);
            assignment = self.ast.copy(&logical_expr.left);
        } else {
            let ident = self.create_new_var(&logical_expr.left);
            reference = self.ast.identifier_reference_expression(ident.clone());
            let left = AssignmentTarget::SimpleAssignmentTarget(
                self.ast.simple_assignment_target_identifier(ident),
            );
            let right = self.ast.copy(&logical_expr.left);
            assignment =
                self.ast.assignment_expression(SPAN, AssignmentOperator::Assign, left, right);
        };

        let test = if self.no_document_all {
            let null = self.ast.literal_null_expression(NullLiteral::new(SPAN));
            self.ast.binary_expression(SPAN, assignment, BinaryOperator::Inequality, null)
        } else {
            let op = BinaryOperator::StrictInequality;
            let null = self.ast.literal_null_expression(NullLiteral::new(SPAN));
            let left = self.ast.binary_expression(SPAN, self.ast.copy(&assignment), op, null);

            let right =
                self.ast.binary_expression(SPAN, self.ast.copy(&reference), op, self.ast.void_0());

            self.ast.logical_expression(SPAN, left, LogicalOperator::And, right)
        };

        let right = self.ast.move_expression(&mut logical_expr.right);

        *expr = self.ast.conditional_expression(SPAN, test, reference, right);
    }
}
