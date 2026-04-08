use oxc_ast::AstKind;
use oxc_ast::ast::{BinaryExpression, Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_string_case_mismatch_diagnostic(span: Span, method: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Comparison with `{method}()` will always be false due to case mismatch."))
        .with_help(format!("The string being compared contains characters that don't match the case transformation from `{method}()`. This comparison will never be true."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoStringCaseMismatch;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects comparisons where a case-transforming method like `toLowerCase()`
    /// or `toUpperCase()` is compared against a string that contains characters
    /// in the wrong case, making the comparison always false.
    ///
    /// ### Why is this bad?
    ///
    /// Comparing `str.toLowerCase()` against a string with uppercase characters
    /// (or `str.toUpperCase()` against lowercase) will always evaluate to false.
    /// This is almost certainly a bug.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if (str.toLowerCase() === "ABC") {}
    /// if (str.toUpperCase() === "abc") {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// if (str.toLowerCase() === "abc") {}
    /// if (str.toUpperCase() === "ABC") {}
    /// ```
    NoStringCaseMismatch,
    eslint,
    correctness
);

impl Rule for NoStringCaseMismatch {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(expr) = node.kind() else {
            return;
        };

        if !matches!(
            expr.operator,
            BinaryOperator::Equality
                | BinaryOperator::StrictEquality
                | BinaryOperator::Inequality
                | BinaryOperator::StrictInequality
        ) {
            return;
        }

        check_case_mismatch(expr, &expr.left, &expr.right, ctx);
        check_case_mismatch(expr, &expr.right, &expr.left, ctx);
    }
}

fn check_case_mismatch(
    expr: &BinaryExpression,
    call_side: &Expression,
    literal_side: &Expression,
    ctx: &LintContext,
) {
    // Check if one side is a call to toLowerCase/toUpperCase
    let Expression::CallExpression(call) = call_side else {
        return;
    };

    let Expression::StaticMemberExpression(member) = &call.callee else {
        return;
    };

    let method_name = member.property.name.as_str();
    let is_to_lower = method_name == "toLowerCase" || method_name == "toLocaleLowerCase";
    let is_to_upper = method_name == "toUpperCase" || method_name == "toLocaleUpperCase";

    if !is_to_lower && !is_to_upper {
        return;
    }

    // Call should have no arguments
    if !call.arguments.is_empty() {
        return;
    }

    // The other side should be a string literal
    let Expression::StringLiteral(lit) = literal_side else {
        return;
    };

    let value = lit.value.as_str();

    // Check for case mismatch
    let has_mismatch = if is_to_lower {
        value.chars().any(|c| c.is_uppercase())
    } else {
        value.chars().any(|c| c.is_lowercase())
    };

    if has_mismatch {
        ctx.diagnostic(no_string_case_mismatch_diagnostic(expr.span, method_name));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"if (str.toLowerCase() === "abc") {}"#,
        r#"if (str.toUpperCase() === "ABC") {}"#,
        r#"if (str.toLowerCase() === "abc123") {}"#,
        r#"if (str.toUpperCase() === "ABC123") {}"#,
        r#"if (str.toLowerCase() === "") {}"#,
        "if (str.toLowerCase() === variable) {}",
        r#"if (str.toLowerCase() === "123") {}"#,
        r#"if (str.toUpperCase() === "123") {}"#,
    ];

    let fail = vec![
        r#"if (str.toLowerCase() === "ABC") {}"#,
        r#"if (str.toUpperCase() === "abc") {}"#,
        r#"if ("ABC" === str.toLowerCase()) {}"#,
        r#"if (str.toLowerCase() === "AbC") {}"#,
        r#"if (str.toUpperCase() === "AbC") {}"#,
        r#"if (str.toLowerCase() !== "ABC") {}"#,
        r#"if (str.toLocaleLowerCase() === "ABC") {}"#,
        r#"if (str.toLocaleUpperCase() === "abc") {}"#,
    ];

    Tester::new(NoStringCaseMismatch::NAME, NoStringCaseMismatch::PLUGIN, pass, fail)
        .test_and_snapshot();
}
