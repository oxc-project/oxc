use oxc_ast::ast::{AssignmentExpression, Expression, ExpressionStatement};
use oxc_span::GetSpan;
use oxc_syntax::operator::AssignmentOperator;

use oxc_diagnostics::OxcDiagnostic;

use crate::Checker;

impl Checker<'_> {
    /// Check an expression statement.
    /// Equivalent to tsgo's `checkExpressionStatement`.
    pub(crate) fn check_expression_statement(&mut self, stmt: &ExpressionStatement<'_>) {
        self.check_expression(&stmt.expression);
    }

    /// Check an expression, dispatching by kind.
    /// Equivalent to tsgo's `checkExpression` / `checkExpressionWorker`.
    ///
    /// For most expressions, evaluating the type via `get_type_of_expression`
    /// recursively walks sub-expressions and emits diagnostics (TS2339, TS2345,
    /// TS2349, TS2554, etc.) along the way. Assignment expressions need
    /// special handling for LHS type checking.
    pub(crate) fn check_expression(&mut self, expr: &Expression<'_>) {
        match expr {
            Expression::AssignmentExpression(assign) => {
                self.check_assignment_expression(assign);
            }
            _ => {
                self.get_type_of_expression(expr);
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
        let value_type = self.get_type_of_expression(&assign.right);

        if !self.is_type_assignable_to(value_type, target_type) {
            let source_str = self.type_to_string(value_type);
            let target_str = self.type_to_string(target_type);

            self.diagnostics.push(
                OxcDiagnostic::error(format!(
                    "Type '{source_str}' is not assignable to type '{target_str}'."
                ))
                .with_error_code("ts", "2322")
                .with_label(ident.span()),
            );
        }
    }
}
