use std::fmt::Debug;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::BinaryOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_eq_null_diagnostic(span: Span, suggested_operator: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `null` comparisons without type-checking operators.")
        .with_help(format!("Use '{suggested_operator}' to compare with null"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEqNull;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow `null` comparisons without type-checking operators.
    ///
    /// ### Why is this bad?
    /// Comparing to `null` without a type-checking operator (`==` or `!=`), can
    /// have unintended results as the comparison will evaluate to `true` when
    /// comparing to not just a `null`, but also an `undefined` value.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if (foo == null) {
    ///     bar();
    /// }
    /// if (baz != null) {
    ///     bar();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// if (foo === null) {
    ///     bar();
    /// }
    ///
    /// if (baz !== null) {
    ///     bar();
    /// }
    ///
    /// if (bang === undefined) {
    ///     bar();
    /// }
    /// ```
    NoEqNull,
    eslint,
    restriction,
    fix_dangerous
);

impl Rule for NoEqNull {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::BinaryExpression(binary_expression) = node.kind() {
            let bad_operator = matches!(
                binary_expression.operator,
                BinaryOperator::Equality | BinaryOperator::Inequality
            );

            if binary_expression.right.is_literal()
                & binary_expression.right.is_null()
                & bad_operator
                | binary_expression.left.is_literal()
                    & binary_expression.left.is_null()
                    & bad_operator
            {
                let suggested_operator = if binary_expression.operator == BinaryOperator::Equality {
                    " === "
                } else {
                    " !== "
                };
                ctx.diagnostic_with_dangerous_fix(
                    no_eq_null_diagnostic(
                        //     Span::new(
                        //     binary_expression.span.start,
                        //     binary_expression.span.end,
                        // )
                        binary_expression.span,
                        suggested_operator.trim(),
                    ),
                    |fixer| {
                        let start = binary_expression.left.span().end;
                        let end = binary_expression.right.span().start;
                        let span = Span::new(start, end);
                        fixer.replace(span, suggested_operator)
                    },
                );
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["if (x === null) { }", "if (null === f()) { }"];

    let fail = vec!["if (x == null) { }", "if (x != null) { }", "do {} while (null == x)"];

    let fix = vec![
        ("if (x == null) { }", "if (x === null) { }"),
        ("if (x != null) { }", "if (x !== null) { }"),
        ("do {} while (null == x)", "do {} while (null === x)"),
    ];

    Tester::new(NoEqNull::NAME, NoEqNull::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
