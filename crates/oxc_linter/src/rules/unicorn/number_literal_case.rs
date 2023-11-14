use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode, Fix};

#[derive(Debug, Error, Diagnostic)]
enum NumberLiteralCaseDiagnostic {
    #[error("eslint-plugin-unicorn(number-literal-case): Unexpected number literal prefix in uppercase.")]
    #[diagnostic(severity(warning), help("Use lowercase for the number literal prefix `{1}`."))]
    UppercasePrefix(#[label] Span, &'static str),
    #[error(
        "eslint-plugin-unicorn(number-literal-case): Unexpected exponential notation in uppercase."
    )]
    #[diagnostic(severity(warning), help("Use lowercase for `e` in exponential notations."))]
    UppercaseExponentialNotation(#[label] Span),
    #[error(
        "eslint-plugin-unicorn(number-literal-case): Unexpected hexadecimal digits in lowercase."
    )]
    #[diagnostic(severity(warning), help("Use uppercase for hexadecimal digits."))]
    LowercaseHexadecimalDigits(#[label] Span),
    #[error(
        "eslint-plugin-unicorn(number-literal-case): Unexpected number literal prefix in uppercase and hexadecimal digits in lowercase."
    )]
    #[diagnostic(severity(warning), help("Use lowercase for the number literal prefix `{1}` and uppercase for hexadecimal digits."))]
    UppercasePrefixAndLowercaseHexadecimalDigits(#[label] Span, &'static str),
}

#[derive(Debug, Default, Clone)]
pub struct NumberLiteralCase;

declare_oxc_lint!(
    /// ### What it does
    /// This rule enforces proper case for numeric literals.
    ///
    /// ### Why is this bad?
    /// When both an identifier and a number literal are in lower case, it can be hard to differentiate between them.
    ///
    /// ### Example
    /// ```javascript
    /// // Fail
    /// const foo = 0XFF;
    /// const foo = 0xff;
    /// const foo = 0Xff;
    /// const foo = 0Xffn;
    ///
    /// const foo = 0B10;
    /// const foo = 0B10n;
    ///
    /// const foo = 0O76;
    /// const foo = 0O76n;
    ///
    /// const foo = 2E-5;
    ///
    /// // Pass
    /// const foo = 0xFF;
    /// const foo = 0b10;
    /// const foo = 0o76;
    /// const foo = 0xFFn;
    /// const foo = 2e+5;
    /// ```
    NumberLiteralCase,
    correctness
);

impl Rule for NumberLiteralCase {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (raw_literal, raw_span) = match node.kind() {
            AstKind::NumberLiteral(number) => (number.raw, number.span),
            AstKind::BigintLiteral(number) => {
                let span = number.span;
                (span.source_text(ctx.source_text()), span)
            }
            _ => return,
        };

        if let Some((diagnostic, fixed_literal)) = check_number_literal(raw_literal, raw_span) {
            ctx.diagnostic_with_fix(diagnostic, || Fix::new(fixed_literal, raw_span));
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn check_number_literal(
    number_literal: &str,
    raw_span: Span,
) -> Option<(NumberLiteralCaseDiagnostic, String)> {
    if number_literal.starts_with("0B") || number_literal.starts_with("0O") {
        return Some((
            NumberLiteralCaseDiagnostic::UppercasePrefix(
                Span { start: raw_span.start + 1, end: raw_span.start + 2 },
                if number_literal.starts_with("0B") { "0b" } else { "0o" },
            ),
            number_literal.to_lowercase(),
        ));
    }
    if number_literal.starts_with("0X") || number_literal.starts_with("0x") {
        let has_uppercase_prefix = number_literal.starts_with("0X");
        let has_lowercase_digits = number_literal[2..].chars().any(|c| ('a'..='f').contains(&c));
        if has_uppercase_prefix && has_lowercase_digits {
            return Some((
                NumberLiteralCaseDiagnostic::UppercasePrefixAndLowercaseHexadecimalDigits(
                    raw_span, "0x",
                ),
                "0x".to_owned() + &digits_to_uppercase(&number_literal[2..]),
            ));
        }
        if has_uppercase_prefix {
            return Some((
                NumberLiteralCaseDiagnostic::UppercasePrefix(
                    Span { start: raw_span.start + 1, end: raw_span.start + 2 },
                    "0x",
                ),
                "0x".to_owned() + &number_literal[2..],
            ));
        }
        if has_lowercase_digits {
            return Some((
                NumberLiteralCaseDiagnostic::LowercaseHexadecimalDigits(Span {
                    start: raw_span.start + 2,
                    end: raw_span.end,
                }),
                "0x".to_owned() + &digits_to_uppercase(&number_literal[2..]),
            ));
        }
        return None;
    }
    if let Some(index) = number_literal.find('E') {
        let char_position = raw_span.start + index as u32;
        return Some((
            NumberLiteralCaseDiagnostic::UppercaseExponentialNotation(Span {
                start: char_position,
                end: char_position + 1,
            }),
            number_literal.to_lowercase(),
        ));
    }
    None
}

fn digits_to_uppercase(digits: &str) -> String {
    let mut result = digits.to_uppercase();
    if result.ends_with('N') {
        result.truncate(result.len() - 1);
        result.push('n');
    }
    result
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var foo = 0777",
        "var foo = 0888",
        "const foo = 1234",
        "const foo = 0b10",
        "const foo = 0o1234567",
        "const foo = 0xABCDEF",
        "const foo = 1234n",
        "const foo = 0b10n",
        "const foo = 0o1234567n",
        "const foo = 0xABCDEFn",
        "const foo = NaN",
        "const foo = +Infinity",
        "const foo = -Infinity",
        "const foo = 1.2e3",
        "const foo = 1.2e-3",
        "const foo = 1.2e+3",
        "const foo = '0Xff'",
        "const foo = '0Xffn'",
        "const foo = 123_456",
        "const foo = 0b10_10",
        "const foo = 0o1_234_567",
        "const foo = 0xDEED_BEEF",
        "const foo = 123_456n",
        "const foo = 0b10_10n",
        "const foo = 0o1_234_567n",
        "const foo = 0xDEED_BEEFn",
    ];

    let fail = vec![
        "const foo = 0B10",
        "const foo = 0O1234567",
        "const foo = 0XaBcDeF",
        "const foo = 0B10n",
        "const foo = 0O1234567n",
        "const foo = 0XaBcDeFn",
        "const foo = 0B0n",
        "const foo = 0O0n",
        "const foo = 0X0n",
        "const foo = 1.2E3",
        "const foo = 1.2E-3",
        "const foo = 1.2E+3",
        "
            const foo = 255;

            if (foo === 0xff) {
                console.log('invalid');
            }
        ",
        "const foo = 0XdeEd_Beefn",
        "console.log(BigInt(0B10 + 1.2E+3) + 0XdeEd_Beefn)",
    ];

    let fix = vec![
        ("const foo = 0B10", "const foo = 0b10", None),
        ("const foo = 0O1234567", "const foo = 0o1234567", None),
        ("const foo = 0XaBcDeF", "const foo = 0xABCDEF", None),
        ("const foo = 0B10n", "const foo = 0b10n", None),
        ("const foo = 0O1234567n", "const foo = 0o1234567n", None),
        ("const foo = 0XaBcDeFn", "const foo = 0xABCDEFn", None),
        ("const foo = 0B0n", "const foo = 0b0n", None),
        ("const foo = 0O0n", "const foo = 0o0n", None),
        ("const foo = 0X0n", "const foo = 0x0n", None),
        ("const foo = 1.2E3", "const foo = 1.2e3", None),
        ("const foo = 1.2E-3", "const foo = 1.2e-3", None),
        ("const foo = 1.2E+3", "const foo = 1.2e+3", None),
        (
            "
            const foo = 255;

            if (foo === 0xff) {
                console.log('invalid');
            }
            ",
            "
            const foo = 255;

            if (foo === 0xFF) {
                console.log('invalid');
            }
            ",
            None,
        ),
        ("const foo = 0XdeEd_Beefn", "const foo = 0xDEED_BEEFn", None),
        (
            "console.log(BigInt(0B10 + 1.2E+3) + 0XdeEd_Beefn)",
            "console.log(BigInt(0b10 + 1.2e+3) + 0xDEED_BEEFn)",
            None,
        ),
    ];

    Tester::new_without_config(NumberLiteralCase::NAME, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
