use std::{cell::RefCell, rc::Rc};

use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder};
use oxc_semantic::SymbolTable;
use oxc_span::Span;
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, LogicalOperator};

use crate::{options::Assumptions, utils::CreateVars};

/// ES2020: Nullish Coalescing Operator
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-nullish-coalescing-operator>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-nullish-coalescing-operator>
pub struct NullishCoalescingOperator<'a> {
    ast: Rc<AstBuilder<'a>>,
    symbols: Rc<RefCell<SymbolTable>>,
    assumptions: Assumptions,
    vars: Vec<'a, VariableDeclarator<'a>>,
}

impl<'a> CreateVars<'a> for NullishCoalescingOperator<'a> {
    fn ast(&self) -> &AstBuilder<'a> {
        &self.ast
    }

    fn vars_mut(&mut self) -> &mut Vec<'a, VariableDeclarator<'a>> {
        &mut self.vars
    }
}

impl<'a> NullishCoalescingOperator<'a> {
    pub fn new(
        ast: Rc<AstBuilder<'a>>,
        symbols: Rc<RefCell<SymbolTable>>,
        assumptions: Assumptions,
    ) -> Self {
        let vars = ast.new_vec();
        Self { ast, symbols, assumptions, vars }
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        // left ?? right
        let Expression::LogicalExpression(logical_expr) = expr else { return };
        if logical_expr.operator != LogicalOperator::Coalesce {
            return;
        }

        let span = Span::default();
        let reference;
        let assignment;

        // skip creating extra reference when `left` is static
        if self.symbols.borrow().is_static(&logical_expr.left) {
            reference = self.ast.copy(&logical_expr.left);
            assignment = self.ast.copy(&logical_expr.left);
        } else {
            let name = self.create_new_var(&logical_expr.left);
            let ident = IdentifierReference::new(span, name);
            reference = self.ast.identifier_reference_expression(ident.clone());
            let left = AssignmentTarget::SimpleAssignmentTarget(
                self.ast.simple_assignment_target_identifier(ident),
            );
            let right = self.ast.copy(&logical_expr.left);
            assignment =
                self.ast.assignment_expression(span, AssignmentOperator::Assign, left, right);
        };

        let test = if self.assumptions.no_document_all {
            let null = self.ast.literal_null_expression(NullLiteral::new(span));
            self.ast.binary_expression(span, assignment, BinaryOperator::Inequality, null)
        } else {
            let op = BinaryOperator::StrictInequality;
            let null = self.ast.literal_null_expression(NullLiteral::new(span));
            let left = self.ast.binary_expression(span, self.ast.copy(&assignment), op, null);

            let right =
                self.ast.binary_expression(span, self.ast.copy(&reference), op, self.ast.void_0());

            self.ast.logical_expression(span, left, LogicalOperator::And, right)
        };

        let right = self.ast.move_expression(&mut logical_expr.right);

        *expr = self.ast.conditional_expression(span, test, reference, right);
    }
}
