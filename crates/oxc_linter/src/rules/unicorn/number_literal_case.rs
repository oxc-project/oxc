use cow_utils::CowUtils;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn uppercase_prefix(span: Span, prefix: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected number literal prefix in uppercase.")
        .with_help(format!("Use lowercase for the number literal prefix `{prefix}`."))
        .with_label(span)
}

fn uppercase_exponential_notation(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected exponential notation in uppercase.")
        .with_help("Use lowercase for `e` in exponential notations.")
        .with_label(span)
}

fn wrong_case_hexadecimal_digits(span: Span, case: HexadecimalValue) -> OxcDiagnostic {
    match case {
        HexadecimalValue::Uppercase => {
            OxcDiagnostic::warn("Unexpected hexadecimal digits in lowercase.")
                .with_help("Use uppercase for hexadecimal digits.")
        }
        HexadecimalValue::Lowercase => {
            OxcDiagnostic::warn("Unexpected hexadecimal digits in uppercase.")
                .with_help("Use lowercase for hexadecimal digits.")
        }
    }
    .with_label(span)
}

fn uppercase_prefix_and_wrong_case_hexadecimal_digits(
    span: Span,
    prefix: &str,
    case: HexadecimalValue,
) -> OxcDiagnostic {
    let (message, expected_case) = match case {
        HexadecimalValue::Uppercase => (
            "Unexpected number literal prefix in uppercase and hexadecimal digits in lowercase.",
            "uppercase",
        ),
        HexadecimalValue::Lowercase => (
            "Unexpected number literal prefix in uppercase and hexadecimal digits in uppercase.",
            "lowercase",
        ),
    };
    OxcDiagnostic::warn(message)
        .with_help(format!(
            "Use lowercase for the number literal prefix `{prefix}` and {expected_case} for hexadecimal digits."
        ))
        .with_label(span)
}

#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum HexadecimalValue {
    #[default]
    Uppercase,
    Lowercase,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NumberLiteralCase {
    /// The case of hexadecimal digits. Prefixes and exponential notation always use lowercase.
    hexadecimal_value: HexadecimalValue,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces proper case for numeric literals.
    /// Hexadecimal digits use uppercase by default. Set `hexadecimalValue` to
    /// `"lowercase"` to match formatters that emit lowercase hexadecimal digits.
    /// This option applies to both numbers and bigints; numeric prefixes and
    /// exponential notation always use lowercase.
    ///
    /// ### Why is this bad?
    ///
    /// When both an identifier and a numeric literal are in
    /// lower case, it can be hard to differentiate between them.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// <!-- prettier-ignore-start -->
    /// ```javascript
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
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const foo = 0xFF;
    /// const foo = 0b10;
    /// const foo = 0o76;
    /// const foo = 0xFFn;
    /// const foo = 2e+5;
    /// ```
    /// <!-- prettier-ignore-end -->
    NumberLiteralCase,
    unicorn,
    style,
    fix,
    config = NumberLiteralCase,
    version = "0.0.18",
    short_description = "This rule enforces proper case for numeric literals.",
);

impl Rule for NumberLiteralCase {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (raw_literal, raw_span) = match node.kind() {
            AstKind::NumericLiteral(number) => (number.raw.as_ref().unwrap().as_str(), number.span),
            AstKind::BigIntLiteral(number) => {
                let span = number.span;
                (span.source_text(ctx.source_text()), span)
            }
            _ => return,
        };

        if let Some((diagnostic, fixed_literal)) =
            check_number_literal(raw_literal, raw_span, self.hexadecimal_value)
        {
            ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.replace(raw_span, fixed_literal));
        }
    }
}

#[expect(clippy::cast_possible_truncation)]
fn check_number_literal(
    number_literal: &str,
    raw_span: Span,
    hexadecimal_value: HexadecimalValue,
) -> Option<(OxcDiagnostic, String)> {
    if number_literal.starts_with("0B") || number_literal.starts_with("0O") {
        return Some((
            uppercase_prefix(
                Span::new(raw_span.start + 1, raw_span.start + 2),
                if number_literal.starts_with("0B") { "0b" } else { "0o" },
            ),
            number_literal.cow_to_ascii_lowercase().into_owned(),
        ));
    }
    if number_literal.starts_with("0X") || number_literal.starts_with("0x") {
        let has_uppercase_prefix = number_literal.starts_with("0X");
        let has_wrong_case_digits =
            number_literal[2..].bytes().any(|digit| match hexadecimal_value {
                HexadecimalValue::Uppercase => (b'a'..=b'f').contains(&digit),
                HexadecimalValue::Lowercase => (b'A'..=b'F').contains(&digit),
            });
        if !has_uppercase_prefix && !has_wrong_case_digits {
            return None;
        }
        let diagnostic = if has_uppercase_prefix && has_wrong_case_digits {
            uppercase_prefix_and_wrong_case_hexadecimal_digits(raw_span, "0x", hexadecimal_value)
        } else if has_uppercase_prefix {
            uppercase_prefix(Span::new(raw_span.start + 1, raw_span.start + 2), "0x")
        } else {
            wrong_case_hexadecimal_digits(
                Span::new(raw_span.start + 2, raw_span.end),
                hexadecimal_value,
            )
        };
        let mut fixed_literal = number_literal.to_owned();
        fixed_literal[1..2].make_ascii_lowercase();
        if has_wrong_case_digits {
            let digits_end = number_literal.len() - usize::from(number_literal.ends_with('n'));
            match hexadecimal_value {
                HexadecimalValue::Uppercase => fixed_literal[2..digits_end].make_ascii_uppercase(),
                HexadecimalValue::Lowercase => fixed_literal[2..digits_end].make_ascii_lowercase(),
            }
        }
        return Some((diagnostic, fixed_literal));
    }
    if let Some(index) = number_literal.find('E') {
        let char_position = raw_span.start + index as u32;
        return Some((
            uppercase_exponential_notation(Span::sized(char_position, 1)),
            number_literal.cow_to_ascii_lowercase().into_owned(),
        ));
    }
    None
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

    Tester::new(NumberLiteralCase::NAME, NumberLiteralCase::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn test_lowercase_hexadecimal_value() {
    use crate::tester::Tester;

    let config = Some(serde_json::json!([{ "hexadecimalValue": "lowercase" }]));
    let pass = vec![
        ("const foo = 0xabcdef", config.clone()),
        ("const foo = 0xdead_beefn", config.clone()),
        ("const foo = 0x0123n", config.clone()),
        ("const foo = 0b1010n", config.clone()),
        ("const foo = 0o76", config.clone()),
        ("const foo = 1.2e+3", config.clone()),
    ];
    let fail = vec![
        ("const foo = 0xABCDEF", config.clone()),
        ("const foo = 0Xabcdef", config.clone()),
        ("const foo = 0XaBcD_EFn", config.clone()),
    ];
    let fix = vec![
        ("const foo = 0xABCDEF", "const foo = 0xabcdef", config.clone()),
        ("const foo = 0Xabcdef", "const foo = 0xabcdef", config.clone()),
        ("const foo = 0XaBcD_EFn", "const foo = 0xabcd_efn", config.clone()),
        ("const foo = 0B10n", "const foo = 0b10n", config.clone()),
        ("const foo = 0O76", "const foo = 0o76", config.clone()),
        ("const foo = 1.2E+3", "const foo = 1.2e+3", config),
    ];
    Tester::new(NumberLiteralCase::NAME, NumberLiteralCase::PLUGIN, pass, fail)
        .with_snapshot_suffix("lowercase_hexadecimal_value")
        .expect_fix(fix)
        .test_and_snapshot();
}
