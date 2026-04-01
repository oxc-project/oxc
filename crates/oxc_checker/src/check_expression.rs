use oxc_ast::ast::{AssignmentExpression, Expression, ExpressionStatement};
use oxc_span::GetSpan;
use oxc_syntax::operator::AssignmentOperator;

use oxc_types::TypeId;

use crate::Checker;

impl Checker<'_> {
    /// Check an expression statement.
    /// Equivalent to tsgo's `checkExpressionStatement`.
    pub(crate) fn check_expression_statement(&mut self, stmt: &ExpressionStatement<'_>) {
        self.check_expression(&stmt.expression, None);
    }

    /// Check an expression, dispatching by kind.
    /// Equivalent to tsgo's `checkExpression` / `checkExpressionWorker`.
    ///
    /// For most expressions, evaluating the type via `get_type_of_expression`
    /// recursively walks sub-expressions and emits diagnostics (TS2339, TS2345,
    /// TS2349, TS2554, etc.) along the way. Assignment expressions need
    /// special handling for LHS type checking.
    pub(crate) fn check_expression(
        &mut self,
        expr: &Expression<'_>,
        contextual_type: Option<TypeId>,
    ) {
        match expr {
            Expression::AssignmentExpression(assign) => {
                self.check_assignment_expression(assign);
            }
            _ => {
                self.get_type_of_expression(expr, contextual_type);
            }
        }
    }

    /// Check an assignment expression (`x = value`).
    ///
    /// For simple `=` assignments to identifiers, validates that the RHS type
    /// is assignable to the LHS declared type.
    fn check_assignment_expression(&mut self, assign: &AssignmentExpression<'_>) {
        // Only handle simple `=` for now
        if assign.operator != AssignmentOperator::Assign {
            return;
        }

        use oxc_ast::ast::AssignmentTarget;

        // Only handle simple identifier targets for now
        let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign.left else {
            return;
        };

        let target_type = self.get_type_of_identifier(ident);
        let value_type = self.get_type_of_expression(&assign.right, Some(target_type));

        self.check_type_assignable_to_and_report(
            value_type, target_type, ident.span(), "2322",
            |s, t| format!("Type '{s}' is not assignable to type '{t}'."),
        );
    }
}
