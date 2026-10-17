use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    operator::{BinaryOperator, UnaryOperator},
    precedence::Precedence,
};

use crate::{AstNode, ast_util, context::LintContext, rule::Rule, utils::get_precedence};

fn prefer_unary_minus_diagnostic(span: Span, operation: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer the unary minus operator over {operation} by `-1`."))
        .with_help("Replace the binary expression with a unary minus expression.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferUnaryMinus;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers the unary minus operator over multiplying or dividing by `-1`.
    ///
    /// ### Why is this bad?
    ///
    /// Unary negation expresses the intent directly and avoids an unnecessary
    /// arithmetic operation.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const negative = value * -1;
    /// const alsoNegative = value / -1;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const negative = -value;
    /// ```
    PreferUnaryMinus,
    unicorn,
    style,
    conditional_fix,
    version = "next",
    short_description = "Prefer unary negation over multiplying or dividing by `-1`.",
);

impl Rule for PreferUnaryMinus {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(binary) = node.kind() else { return };

        let operand = match binary.operator {
            BinaryOperator::Multiplication => {
                let left_is_negative_one = is_negative_one(binary.left.without_parentheses());
                let right_is_negative_one = is_negative_one(binary.right.without_parentheses());
                match (left_is_negative_one, right_is_negative_one) {
                    (true, false) => &binary.right,
                    (false, true) => &binary.left,
                    _ => return,
                }
            }
            BinaryOperator::Division
                if is_negative_one(binary.right.without_parentheses())
                    && !is_negative_one(binary.left.without_parentheses()) =>
            {
                &binary.left
            }
            _ => return,
        };

        let operation = if binary.operator == BinaryOperator::Multiplication {
            "multiplying"
        } else {
            "dividing"
        };
        let diagnostic = prefer_unary_minus_diagnostic(binary.span, operation);
        if ctx.has_comments_between(binary.span) {
            ctx.diagnostic(diagnostic);
            return;
        }

        ctx.diagnostic_with_fix(diagnostic, |fixer| {
            let operand = operand.without_parentheses();
            let source = fixer.source_range(operand.span());
            let needs_parentheses = get_precedence(operand)
                .is_some_and(|precedence| precedence < Precedence::Prefix)
                || matches!(
                    operand,
                    Expression::UpdateExpression(_)
                        | Expression::TSNonNullExpression(_)
                        | Expression::TSTypeAssertion(_)
                )
                || matches!(operand, Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::UnaryNegation);
            let mut replacement =
                if needs_parentheses { format!("-({source})") } else { format!("-{source}") };

            if ast_util::could_be_asi_hazard(node, ctx) {
                replacement.insert(0, ';');
            } else if binary.span.start > 0
                && ctx.source_text().as_bytes()[binary.span.start as usize - 1] == b'-'
            {
                replacement.insert(0, ' ');
            }

            fixer.replace(binary.span, replacement)
        });
    }
}

fn is_negative_one(expression: &Expression) -> bool {
    let Expression::UnaryExpression(unary) = expression else { return false };
    unary.operator == UnaryOperator::UnaryNegation
        && matches!(&unary.argument, Expression::NumericLiteral(literal) if literal.value.to_bits() == 1.0_f64.to_bits())
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "x * 2;",
        "x / 2;",
        "x * -2;",
        "x / -2;",
        "-1 * -1;",
        "-1 / -1;",
        "-1 / x;",
        "-1 / (a + b);",
        "x * -1n;",
        "x / -1n;",
        "-1n * x;",
        "-1n / x;",
        "x ** -1;",
        "x - 1;",
        "x + -1;",
        "x % -1;",
        "x & -1;",
    ];

    let fail = vec![
        "x * -1;",
        "-1 * x;",
        "x / -1;",
        "a.b * -1;",
        "foo() * -1;",
        "5 * -1;",
        "-1 * 5;",
        "foo?.bar * -1;",
        "tag`str` * -1;",
        "--x * -1;",
        "x++ * -1;",
        "-y * -1;",
        "+x * -1;",
        "(a + b) * -1;",
        "-1 * (a + b);",
        "(a ? b : c) * -1;",
        "(a, b) * -1;",
        "(a = b) * -1;",
        "(a + b) / -1;",
        "(x) * -1;",
        "(x * -1) ** 2;",
        "a-x*-1;",
        "a - x * -1;",
        "foo
            x * -1;",
        "function f() { return x * -1; }",
        "x * -1 * -1;",
        "a + b * -1;",
        "x /* c */ * -1;",
        "x * -1 /* c */;",
        "(x as number) * -1;", // {"parser": parsers.typescript},
        "-1 * (x as number);", // {"parser": parsers.typescript},
        "x! * -1;",            // {"parser": parsers.typescript}
    ];

    let fix = vec![
        ("x * -1;", "-x;"),
        ("-1 * x;", "-x;"),
        ("x / -1;", "-x;"),
        ("--x * -1;", "-(--x);"),
        ("x++ * -1;", "-(x++);"),
        ("-y * -1;", "-(-y);"),
        ("+x * -1;", "-+x;"),
        ("(a + b) * -1;", "-(a + b);"),
        ("-1 * (a ? b : c);", "-(a ? b : c);"),
        ("(x) * -1;", "-x;"),
        ("a-x*-1;", "a- -x;"),
        ("foo\nx * -1;", "foo\n;-x;"),
        ("x /* c */ * -1;", "x /* c */ * -1;"),
        ("x * -1 /* c */;", "-x /* c */;"),
        ("x! * -1;", "-(x!);"),
    ];

    Tester::new(PreferUnaryMinus::NAME, PreferUnaryMinus::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
