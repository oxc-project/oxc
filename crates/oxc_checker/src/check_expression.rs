use oxc_ast::ast::{Expression, ExpressionStatement, TSType, TSTypeName};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::GetSpan;
use oxc_syntax::operator::UnaryOperator;

use oxc_types::TypeId;

use crate::checker::CheckMode;
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
    /// Uses `CheckMode::NORMAL` so all diagnostics are enabled.
    /// For type-only contexts (CFA narrowing, declaration resolution),
    /// call `get_type_of_expression` with `CheckMode::TYPE_ONLY` directly.
    pub(crate) fn check_expression(
        &mut self,
        expr: &Expression<'_>,
        contextual_type: Option<TypeId>,
    ) -> TypeId {
        self.get_type_of_expression(expr, contextual_type, CheckMode::NORMAL)
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
        check_mode: CheckMode,
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
            result = self.get_type_of_expression(expr, ctx, check_mode);
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
    /// For simple `=` assignments, resolves the LHS type (triggering TS2339
    /// for non-existent properties) and validates that the RHS type is
    /// assignable to the LHS declared type (TS2322).
    ///
    /// For compound assignments (`+=`, `-=`, etc.), resolves both LHS and
    /// RHS types, computes the binary operation result type, and checks
    /// that it's assignable back to the LHS.
    ///
    /// For logical assignments (`||=`, `&&=`, `??=`), checks that the RHS
    /// type is assignable to the LHS type directly.
    ///
    /// Called from `get_type_of_expression_inner` so diagnostics fire
    /// regardless of how the expression is reached.
    pub(crate) fn check_assignment_expression(
        &mut self,
        assign: &oxc_ast::ast::AssignmentExpression<'_>,
        contextual_type: Option<TypeId>,
        check_mode: CheckMode,
    ) -> TypeId {
        use oxc_syntax::operator::AssignmentOperator;

        // Resolve the LHS type — this triggers TS2339 for member expressions
        // where the property doesn't exist on the object type.
        let target_type = self.get_type_of_assignment_target(&assign.left);
        let label_span = assign.left.span();

        if assign.operator == AssignmentOperator::Assign {
            // Simple assignment: check RHS assignable to LHS.
            // Skip assignability check when target is `any` (e.g. property
            // not found already reported TS2339 — don't pile on TS2322).
            let value_type =
                self.get_type_of_expression(&assign.right, Some(target_type), check_mode);
            if target_type != self.any_type {
                self.check_type_assignable_to_and_report(
                    value_type,
                    target_type,
                    label_span,
                    "2322",
                    |s, t| format!("Type '{s}' is not assignable to type '{t}'."),
                );
            }
            return value_type;
        }

        if assign.operator.is_logical() {
            // Logical assignments (||=, &&=, ??=): RHS must be assignable to LHS
            let value_type =
                self.get_type_of_expression(&assign.right, Some(target_type), check_mode);
            self.check_type_assignable_to_and_report(
                value_type,
                target_type,
                label_span,
                "2322",
                |s, t| format!("Type '{s}' is not assignable to type '{t}'."),
            );
            return value_type;
        }

        // Compound assignment (+=, -=, *=, etc.): compute the binary result
        // type and check it's assignable back to the LHS.
        if let Some(bin_op) = assign.operator.to_binary_operator() {
            let rhs_type = self.get_type_of_expression(&assign.right, None, check_mode);
            let result_type =
                self.get_result_type_of_binary_operation(bin_op, target_type, rhs_type);
            self.check_type_assignable_to_and_report(
                result_type,
                target_type,
                label_span,
                "2322",
                |s, t| format!("Type '{s}' is not assignable to type '{t}'."),
            );
            return result_type;
        }

        // Fallback for any unhandled operator variant
        self.get_type_of_expression(&assign.right, contextual_type, check_mode)
    }

    /// Check a type assertion (`expr as T` or `<T>expr`).
    ///
    /// For const assertions (`as const` / `<const>`), validates that the
    /// expression is a valid const assertion target (TS1355) and returns
    /// the expression's type. For regular assertions, returns the asserted type.
    ///
    /// Mirrors tsgo's `checkAssertion`.
    pub(crate) fn check_assertion(
        &mut self,
        expression: &Expression<'_>,
        type_annotation: &TSType<'_>,
        contextual_type: Option<TypeId>,
        check_mode: CheckMode,
    ) -> TypeId {
        if Self::is_const_type_reference(type_annotation) {
            let expr_type =
                self.get_type_of_expression(expression, contextual_type, check_mode);
            if !self.is_valid_const_assertion_argument(expression) {
                self.diagnostics.push(
                    OxcDiagnostic::error(
                        "A 'const' assertion can only be applied to references to enum members, or string, number, boolean, array, or object literals."
                    )
                    .with_error_code("ts", "1355")
                    .with_label(expression.span()),
                );
            }
            return expr_type;
        }
        self.get_type_from_type_node(type_annotation)
    }

    /// Returns `true` if `type_node` is a `TSTypeReference` to the bare
    /// identifier `const` with no type arguments — i.e. the `const` in
    /// `as const` or `<const>`.
    fn is_const_type_reference(type_node: &TSType<'_>) -> bool {
        let TSType::TSTypeReference(type_ref) = type_node else {
            return false;
        };
        if type_ref.type_arguments.is_some() {
            return false;
        }
        let TSTypeName::IdentifierReference(ident) = &type_ref.type_name else {
            return false;
        };
        ident.name == "const"
    }

    /// Returns `true` if `expr` is a valid target for a const assertion.
    ///
    /// Valid targets: string/number/bigint/boolean literals, template literals,
    /// array literals, object literals, parenthesized valid targets, unary
    /// +/- on numeric/bigint literals, and enum member references.
    ///
    /// Mirrors tsgo's `isValidConstAssertionArgument`.
    fn is_valid_const_assertion_argument(&self, expr: &Expression<'_>) -> bool {
        match expr {
            Expression::StringLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::ArrayExpression(_)
            | Expression::ObjectExpression(_) => true,

            Expression::ParenthesizedExpression(paren) => {
                self.is_valid_const_assertion_argument(&paren.expression)
            }

            Expression::UnaryExpression(unary) => {
                match unary.operator {
                    UnaryOperator::UnaryNegation => matches!(
                        unary.argument,
                        Expression::NumericLiteral(_) | Expression::BigIntLiteral(_)
                    ),
                    UnaryOperator::UnaryPlus => {
                        matches!(unary.argument, Expression::NumericLiteral(_))
                    }
                    _ => false,
                }
            }

            // Enum member references: `E.Member` or `E["Member"]`
            Expression::StaticMemberExpression(member) => {
                self.is_enum_member_expression(&member.object)
            }
            Expression::ComputedMemberExpression(member) => {
                self.is_enum_member_expression(&member.object)
            }

            _ => false,
        }
    }

    /// Returns `true` if the expression resolves to an enum symbol.
    /// Used for validating enum member references in const assertions.
    fn is_enum_member_expression(&self, expr: &Expression<'_>) -> bool {
        let expr = expr.without_parentheses();
        if let Expression::Identifier(ident) = expr {
            if let Some(reference_id) = ident.reference_id.get() {
                let reference = self.semantic().scoping().get_reference(reference_id);
                if let Some(symbol_id) = reference.symbol_id() {
                    let flags = self.semantic().scoping().symbol_flags(symbol_id);
                    return flags.is_enum();
                }
            }
        }
        false
    }

    /// Get the type of an object expression in an assignment target position.
    ///
    /// For identifiers, returns the declared type directly (via `get_type_of_symbol`)
    /// to avoid firing TS2454 — the object of `x.prop = value` is in write context,
    /// so TSC doesn't treat `x` as "used before assigned" here.
    /// For non-identifier expressions, falls through to `get_type_of_expression`.
    pub(crate) fn get_assignment_target_object_type(&mut self, object: &Expression<'_>) -> TypeId {
        if let Expression::Identifier(ident) = object {
            let Some(reference_id) = ident.reference_id.get() else {
                return self.any_type;
            };
            let reference = self.semantic().scoping().get_reference(reference_id);
            let Some(symbol_id) = reference.symbol_id() else {
                return self.get_type_of_global_identifier(&ident.name);
            };
            return self.get_type_of_symbol(symbol_id);
        }
        self.get_type_of_expression(object, None, CheckMode::NORMAL)
    }
}
