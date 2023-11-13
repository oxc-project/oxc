use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode, Fix};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(number-literal-case): {1}")]
#[diagnostic(severity(warning), help("{2}"))]
struct NumberLiteralCaseDiagnostic(#[label] pub Span, &'static str, &'static str);

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
                (&ctx.source_text()[span.start as usize..span.end as usize], span)
            }
            _ => return,
        };

        if let Some((error_span, message, fixed_literal)) =
            check_number_literal(raw_literal, raw_span)
        {
            let (error, help) = message.details();
            ctx.diagnostic_with_fix(NumberLiteralCaseDiagnostic(error_span, error, help), || {
                Fix::new(fixed_literal, raw_span)
            });
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn check_number_literal(number_literal: &str, raw_span: Span) -> Option<(Span, Message, String)> {
    if number_literal.starts_with("0B") || number_literal.starts_with("0O") {
        return Some((
            Span { start: raw_span.start + 1, end: raw_span.start + 2 },
            Message::UppercasePrefix,
            number_literal.to_lowercase(),
        ));
    }
    if number_literal.starts_with("0X") || number_literal.starts_with("0x") {
        let has_uppercase_prefix = number_literal.starts_with("0X");
        let has_lowercase_digits = number_literal[2..].chars().any(|c| ('a'..='f').contains(&c));
        if has_uppercase_prefix && has_lowercase_digits {
            return Some((
                raw_span,
                Message::UppercasePrefixAndLowercaseHexadecimalDigits,
                "0x".to_owned() + &digits_to_uppercase(&number_literal[2..]),
            ));
        }
        if has_uppercase_prefix {
            return Some((
                Span { start: raw_span.start + 1, end: raw_span.start + 2 },
                Message::UppercasePrefix,
                "0x".to_owned() + &number_literal[2..],
            ));
        }
        if has_lowercase_digits {
            return Some((
                Span { start: raw_span.start + 2, end: raw_span.end },
                Message::LowercaseHexadecimalDigits,
                "0x".to_owned() + &digits_to_uppercase(&number_literal[2..]),
            ));
        }
        return None;
    }
    if let Some(index) = number_literal.find('E') {
        let char_position = raw_span.start + index as u32;
        return Some((
            Span { start: char_position, end: char_position + 1 },
            Message::UppercaseExponentialNotation,
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

enum Message {
    UppercasePrefix,
    UppercaseExponentialNotation,
    LowercaseHexadecimalDigits,
    UppercasePrefixAndLowercaseHexadecimalDigits,
}

impl Message {
    fn details(&self) -> (&'static str, &'static str) {
        match self {
            Self::UppercasePrefix => ("Unexpected number literal prefix in uppercase.", "Use lowercase for number literal prefixes (`0b`, `0o`, and `0x`)."),
            Self::UppercaseExponentialNotation => ("Unexpected exponential notation in uppercase.", "Use lowercase for `e` in exponential notations."),
            Self::LowercaseHexadecimalDigits => ("Unexpected hexadecimal digits in lowercase.", "Use uppercase for hexadecimal digits."),
            Self::UppercasePrefixAndLowercaseHexadecimalDigits => ("Unexpected number literal prefix in uppercase and hexadecimal digits in lowercase.", "Use lowercase for number literal prefixes (`0b`, `0o`, and `0x`) and uppercase for hexadecimal digits.")
        }
    }
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
