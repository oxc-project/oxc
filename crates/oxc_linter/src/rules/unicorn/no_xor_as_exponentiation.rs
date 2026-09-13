use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{number::NumberBase, operator::BinaryOperator};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_xor_as_exponentiation_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Unexpected bitwise XOR operator `^`. Did you mean the exponentiation operator `**`?",
    )
    .with_help("Replace `^` with `**`.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoXorAsExponentiation;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the bitwise XOR operator between two decimal integer literals,
    /// where exponentiation was likely intended.
    ///
    /// ### Why is this bad?
    ///
    /// JavaScript uses `**` for exponentiation. The `^` operator performs a
    /// bitwise XOR, which can produce an unexpected result when it is mistaken
    /// for exponentiation.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const value = 2 ^ 8;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const value = 2 ** 8;
    /// const flags = value ^ mask;
    /// ```
    NoXorAsExponentiation,
    unicorn,
    suspicious,
    suggestion,
    version = "next",
    short_description = "Disallow the bitwise XOR operator where exponentiation was likely intended.",
);

impl Rule for NoXorAsExponentiation {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(binary_expression) = node.kind() else {
            return;
        };

        if binary_expression.operator != BinaryOperator::BitwiseXOR
            || !is_decimal_integer_literal(&binary_expression.left)
            || !is_decimal_integer_literal(&binary_expression.right)
        {
            return;
        }

        // Search by token so a `^` inside a comment between the operands is not selected.
        let left_end = binary_expression.left.span().end;
        let right_start = binary_expression.right.span().start;
        let operator_offset = ctx.find_next_token_within(left_end, right_start, "^").unwrap();
        let operator_span = Span::sized(left_end + operator_offset, 1);

        ctx.diagnostic_with_suggestion(
            no_xor_as_exponentiation_diagnostic(operator_span),
            |fixer| fixer.replace(operator_span, "**").with_message("Replace `^` with `**`."),
        );
    }
}

fn is_decimal_integer_literal(expression: &Expression<'_>) -> bool {
    let Expression::NumericLiteral(literal) = expression.without_parentheses() else {
        return false;
    };

    // Integral exponent literals such as `2e3` also use `NumberBase::Decimal`.
    literal.base == NumberBase::Decimal
        && literal.raw.as_ref().is_some_and(|raw| is_decimal_integer_raw(raw.as_bytes()))
}

fn is_decimal_integer_raw(raw: &[u8]) -> bool {
    match raw {
        b"0" => true,
        [b'0', rest @ ..] => {
            // `8` or `9` distinguishes a leading-zero decimal from legacy octal.
            let Some(index) = rest.iter().position(|byte| matches!(byte, b'8' | b'9')) else {
                return false;
            };
            rest[..index].iter().all(|byte| matches!(byte, b'0'..=b'7'))
                && rest[index + 1..].iter().all(u8::is_ascii_digit)
        }
        [b'1'..=b'9', rest @ ..] => {
            let mut previous_was_separator = false;
            for byte in rest {
                match byte {
                    b'0'..=b'9' => previous_was_separator = false,
                    b'_' if !previous_was_separator => previous_was_separator = true,
                    _ => return false,
                }
            }
            !previous_was_separator
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::{fixer::FixKind, tester::Tester};

    let pass = vec![
        "2 ** 32",
        "0xFF ^ 8",
        "2 ^ 0x10",
        "0b100 ^ 2",
        "0o20 ^ 2",
        "2 ^ 0o20",
        "01 ^ 07",
        "a ^ b",
        "x ^ 2",
        "2 ^ y",
        "flags ^ MASK",
        "2.5 ^ 3",
        "2 ^ 3.5",
        "2e3 ^ 2",
        "2 ^ 3e2",
        "2n ^ 32n",
        "2 ^ -3",
        "-2 ^ 3",
        "2 ^ +3",
        "2 | 8",
        "2 & 8",
        "2 << 8",
        "(2 as number) ^ 8",
    ];

    let fail = vec![
        "2 ^ 32",
        "3 ^ 3",
        "10 ^ 6",
        "0 ^ 0",
        "2 ^ 8",
        "2  ^  8",
        "const x = 2 ^ 8;",
        "foo(2 ^ 8)",
        "10 ^ 1_000",
        "2 ^ 8 ^ 2",
        "2 /* comment */ ^ 8",
        "(1) ^ (((2)))",
        "1 /* ^ */ ^ /* ^ */ 2",
        "08 ^ 09",
        "018 ^ 019",
    ];

    let fix = vec![
        ("1 ^ 2", "1 ** 2", None, FixKind::Suggestion),
        ("1 /*a*/ ^ /*b*/ 2", "1 /*a*/ ** /*b*/ 2", None, FixKind::Suggestion),
    ];

    Tester::new(NoXorAsExponentiation::NAME, NoXorAsExponentiation::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
