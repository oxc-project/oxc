use oxc_ast::ast::{Expression, ExpressionStatement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::GetSpan;
use oxc_syntax::operator::UnaryOperator;

use oxc_types::TypeId;

use crate::Checker;

impl Checker<'_> {
    /// Check an expression statement.
    /// Equivalent to tsgo's `checkExpressionStatement`.
    pub(crate) fn check_expression_statement(&mut self, stmt: &ExpressionStatement<'_>) {
        self.check_expression(&stmt.expression, None);
    }

    /// Check an expression — the primary entry point for all expression checking.
    /// Equivalent to tsgo's `checkExpression`.
    ///
    /// Returns the type of the expression. All expression-level diagnostics
    /// (TS2695, TS2322, TS2339, TS2345, TS2349, TS2554, etc.) are emitted
    /// from within `get_type_of_expression` / `get_type_of_expression_inner`.
    ///
    /// Use this for user-written expressions (statement-level, variable
    /// initializers, return values, etc.). For type-only contexts (CFA
    /// narrowing, declaration resolution), use `get_type_of_expression`
    /// directly — in the future, that path will pass `CheckMode::TypeOnly`
    /// to suppress certain diagnostics.
    pub(crate) fn check_expression(
        &mut self,
        expr: &Expression<'_>,
        contextual_type: Option<TypeId>,
    ) -> TypeId {
        self.get_type_of_expression(expr, contextual_type)
    }

    /// Check a sequence (comma) expression.
    ///
    /// Emits TS2695 for non-last elements that are side-effect-free.
    /// Returns the type of the last element.
    /// Mirrors tsgo's comma-token case in `checkBinaryLikeExpression`.
    ///
    /// Called from `get_type_of_expression_inner` so diagnostics fire
    /// regardless of how the expression is reached.
    pub(crate) fn check_sequence_expression(
        &mut self,
        seq: &oxc_ast::ast::SequenceExpression<'_>,
        contextual_type: Option<TypeId>,
    ) -> TypeId {
        let exprs = &seq.expressions;
        let mut result = self.undefined_type;
        for (i, expr) in exprs.iter().enumerate() {
            let is_last = i == exprs.len() - 1;
            if !is_last
                && self.allow_unreachable_code != Some(true)
                && Self::is_side_effect_free(expr)
                && !Self::is_indirect_call(expr, exprs.get(i + 1))
            {
                self.diagnostics.push(
                    OxcDiagnostic::error(
                        "Left side of comma operator is unused and has no side effects.",
                    )
                    .with_error_code("ts", "2695")
                    .with_label(expr.span()),
                );
            }
            // Check each sub-expression (the last one gets the contextual type)
            let ctx = if is_last { contextual_type } else { None };
            result = self.get_type_of_expression(expr, ctx);
        }
        result
    }

    /// Determines if an expression is side-effect-free (i.e., evaluating it
    /// produces no observable change). Mirrors tsgo's `isSideEffectFree`.
    fn is_side_effect_free(expr: &Expression<'_>) -> bool {
        match expr.without_parentheses() {
            // Literals and identifiers
            Expression::Identifier(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::TaggedTemplateExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ClassExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::ObjectExpression(_)
            | Expression::TSNonNullExpression(_) => true,

            // Unary ~, !, +, -, typeof are side-effect-free (tsgo does not
            // recurse into the operand).
            Expression::UnaryExpression(unary) => matches!(
                unary.operator,
                UnaryOperator::Typeof
                    | UnaryOperator::LogicalNot
                    | UnaryOperator::UnaryPlus
                    | UnaryOperator::UnaryNegation
                    | UnaryOperator::BitwiseNot
            ),

            // Conditional: both branches must be side-effect-free
            Expression::ConditionalExpression(cond) => {
                Self::is_side_effect_free(&cond.consequent)
                    && Self::is_side_effect_free(&cond.alternate)
            }

            // Binary: both sides must be side-effect-free and not an assignment
            Expression::BinaryExpression(bin) => {
                Self::is_side_effect_free(&bin.left) && Self::is_side_effect_free(&bin.right)
            }

            _ => false,
        }
    }

    /// Detect the indirect call pattern `(0, x.f)()` or `(0, eval)()`.
    /// When a comma expression is used for indirect invocation, TS2695 is suppressed.
    ///
    /// Note: tsgo also checks that the parent of the comma expression is a
    /// parenthesized call expression. We don't have parent access here, so this
    /// is a shape-only heuristic that may suppress TS2695 for bare `0, eval;`
    /// statements. This is a harmless false negative (missing a diagnostic in
    /// a rare edge case).
    // TODO: check parent is a call expression once node_id is available here
    fn is_indirect_call(left: &Expression<'_>, right: Option<&Expression<'_>>) -> bool {
        // Left must be the numeric literal `0`
        let is_zero = matches!(left, Expression::NumericLiteral(n) if n.value == 0.0);
        if !is_zero {
            return false;
        }
        // Right must be a property access or `eval`
        let Some(right) = right else { return false };
        matches!(
            right,
            Expression::StaticMemberExpression(_) | Expression::ComputedMemberExpression(_)
        ) || matches!(right, Expression::Identifier(id) if id.name == "eval")
    }

    /// Check an assignment expression and return its type.
    ///
    /// For simple `=` assignments to identifiers, validates that the RHS type
    /// is assignable to the LHS declared type (TS2322). Always returns the
    /// RHS type as the expression result.
    ///
    /// Called from `get_type_of_expression_inner` so diagnostics fire
    /// regardless of how the expression is reached.
    pub(crate) fn check_assignment_expression(
        &mut self,
        assign: &oxc_ast::ast::AssignmentExpression<'_>,
        contextual_type: Option<TypeId>,
    ) -> TypeId {
        use oxc_ast::ast::AssignmentTarget;
        use oxc_syntax::operator::AssignmentOperator;

        if assign.operator == AssignmentOperator::Assign {
            if let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign.left {
                let target_type = self.get_type_of_identifier(ident);
                let value_type = self.get_type_of_expression(&assign.right, Some(target_type));
                self.check_type_assignable_to_and_report(
                    value_type,
                    target_type,
                    ident.span(),
                    "2322",
                    |s, t| format!("Type '{s}' is not assignable to type '{t}'."),
                );
                return value_type;
            }
        }

        self.get_type_of_expression(&assign.right, contextual_type)
    }
}
