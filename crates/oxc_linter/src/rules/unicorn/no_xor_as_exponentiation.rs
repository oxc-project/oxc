use oxc_ast::{
    AstKind,
    ast::{BinaryExpression, BinaryOperator, Expression, NumberBase},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn no_xor_as_exponentiation_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Probable confusion of XOR operator with exponentiation")
        .with_help(
            "If not intended to be a XOR operation, replace the operator with '**' for exponentiation.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoXorAsExponentiation;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the bitwise XOR operator where exponentiation was likely intended.
    ///
    /// ### Why is this bad?
    ///
    /// In JavaScript, `^` is the bitwise XOR operator, not exponentiation.
    /// Developers coming from languages like Lua, Julia, R, or MATLAB, or from math notation, often expect `^` to mean “to the power of”, so `2 ^ 32` silently evaluates to `34` instead of `4294967296`.
    /// The actual exponentiation operator is `**`.
    ///
    /// This rule flags `^` between two decimal integer literals, which is almost always this mistake.
    /// Hexadecimal, octal, and binary literals (such as `0xFF ^ 8`) and any non-literal operands (such as `flags ^ MASK`) are ignored,
    /// since those are far more likely to be intentional bitwise XOR.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// 2 ^ 5
    /// x = 3 ^ 6
    /// y = 10 ^ 1000000
    /// bar(5 ^ 35)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// 2.5 ^ 3
    /// 3e5 ^ 6
    /// 0x1A ^ 0x2B
    /// 0x1A ^ 5
    /// (2 as number) ^ 8
    /// flags ^ MASK
    /// x = foo(2.5 ^ 0x3E)
    /// ```
    NoXorAsExponentiation,
    unicorn,
    suspicious,
    pending,
    version = "next",
    short_description = "Disallow the bitwise XOR operator where exponentiation was likely intended.",
);

fn is_int_literal<'a>(expr: &Expression<'a>) -> bool {
    let Expression::NumericLiteral(numeric) = expr else {
        return false;
    };

    if !matches!(numeric.base, NumberBase::Decimal) {
        return false;
    }

    !numeric.raw_str().to_owned().contains('e')
}

impl Rule for NoXorAsExponentiation {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BinaryExpression(BinaryExpression { left, operator, right, span, .. })
                if matches!(operator, BinaryOperator::BitwiseXOR) =>
            {
                if is_int_literal(left) && is_int_literal(right) {
                    ctx.diagnostic(no_xor_as_exponentiation_diagnostic(*span));
                }
            }
            _ => {}
        }
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

    Tester::new(NoXorAsExponentiation::NAME, NoXorAsExponentiation::PLUGIN, pass, fail)
        .test_and_snapshot();
}
