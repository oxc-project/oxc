use oxc_ast::{
    AstKind,
    ast::{Expression, NumberBase},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::BinaryOperator;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_xor_as_exponentiation_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Unexpected bitwise XOR operator `^` where exponentiation was likely intended.",
    )
    .with_help("Replace `^` with the exponentiation operator `**`.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoXorAsExponentiation;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the bitwise XOR operator `^` between two decimal integer
    /// literals, where the exponentiation operator `**` was likely intended.
    ///
    /// ### Why is this bad?
    ///
    /// In JavaScript, `^` is the bitwise XOR operator, not exponentiation.
    /// Developers coming from languages like Lua, Julia, R, or MATLAB, or from
    /// math notation, often expect `^` to mean "to the power of", so `2 ^ 32`
    /// silently evaluates to `34` instead of `4294967296`. The actual
    /// exponentiation operator is `**`.
    ///
    /// This rule only flags `^` between two decimal integer literals, which is
    /// almost always this mistake. Hexadecimal, octal, and binary literals
    /// (such as `0xFF ^ 8`) and any non-literal operands (such as `flags ^ MASK`)
    /// are ignored, since those are far more likely to be intentional bitwise XOR.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const kibibyte = 2 ^ 10; // 8, not 1024
    /// const cube = 3 ^ 3; // 0, not 27
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const kibibyte = 2 ** 10;
    /// const cube = 3 ** 3;
    /// ```
    NoXorAsExponentiation,
    unicorn,
    suspicious,
    suggestion,
    version = "next",
    short_description = "Disallow the bitwise XOR operator where exponentiation was likely intended.",
);

fn is_decimal_integer_literal(expr: &Expression) -> bool {
    let Expression::NumericLiteral(literal) = expr else {
        return false;
    };

    // `NumberBase::Decimal` already excludes hexadecimal, octal, binary, and
    // literals with a fractional part (those use `NumberBase::Float`). Exponent
    // notation such as `2e3` is still `Decimal` (and its `value` is an integer),
    // so reject it explicitly by looking at the raw source text.
    literal.base == NumberBase::Decimal && !literal.raw_str().contains(['e', 'E'])
}

impl Rule for NoXorAsExponentiation {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(binary_expr) = node.kind() else {
            return;
        };

        if binary_expr.operator != BinaryOperator::BitwiseXOR
            || !is_decimal_integer_literal(&binary_expr.left)
            || !is_decimal_integer_literal(&binary_expr.right)
        {
            return;
        }

        let operator_start = binary_expr.left.span().end;
        let operator_end = binary_expr.right.span().start;
        let Some(operator_offset) = ctx.find_next_token_within(operator_start, operator_end, "^")
        else {
            return;
        };
        let operator_span = Span::sized(operator_start + operator_offset, 1);

        ctx.diagnostic_with_suggestion(
            no_xor_as_exponentiation_diagnostic(operator_span),
            |fixer| fixer.replace(operator_span, "**"),
        );
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "2 ** 32",
        "0xFF ^ 8",
        "2 ^ 0x10",
        "0b100 ^ 2",
        "0o20 ^ 2",
        "2 ^ 0o20",
        "a ^ b",
        "x ^ 2",
        "2 ^ y",
        "flags ^ MASK",
        "2.5 ^ 3",
        "2 ^ 3.5",
        "2e3 ^ 2",
        "2n ^ 32n",
        "2 ^ -3",
        "-2 ^ 3",
        "2 ^ +3",
        "2 | 8",
        "2 & 8",
        "2 << 8",
        "2 ** 8",
        "(2 as number) ^ 8", // {"parser": parsers.typescript}
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
        "2 ^ 8", // {"parser": parsers.typescript}
    ];

    let fix = vec![
        ("2 ^ 32", "2 ** 32"),
        ("3 ^ 3", "3 ** 3"),
        ("10 ^ 6", "10 ** 6"),
        ("0 ^ 0", "0 ** 0"),
        ("2 ^ 8", "2 ** 8"),
        ("2  ^  8", "2  **  8"),
        ("const x = 2 ^ 8;", "const x = 2 ** 8;"),
        ("foo(2 ^ 8)", "foo(2 ** 8)"),
        ("10 ^ 1_000", "10 ** 1_000"),
        ("2 ^ 8 ^ 2", "2 ** 8 ^ 2"),
        ("2 /* comment */ ^ 8", "2 /* comment */ ** 8"),
    ];

    Tester::new(NoXorAsExponentiation::NAME, NoXorAsExponentiation::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
