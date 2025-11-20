use lazy_regex::{Lazy, Regex, lazy_regex};
use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::number::NumberBase;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_bigint_literals_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer bigint literals over `BigInt(...)`.")
        .with_help("Use a bigint literal (e.g. `123n`) instead of calling `BigInt` with a literal argument.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferBigintLiterals;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires using BigInt literals (e.g. `123n`) instead of calling the `BigInt()` constructor
    /// with literal arguments such as numbers or numeric strings
    ///
    /// ### Why is this bad?
    ///
    /// Using `BigInt(…)` with literal values is unnecessarily verbose and less idiomatic than using
    /// a BigInt literal.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// BigInt(0);
    /// BigInt(123);
    /// BigInt(0xFF);
    /// BigInt(1e3);
    /// BigInt("42");
    /// BigInt("0x10");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// 0n;
    /// 123n;
    /// 0xFFn;
    /// 1000n;
    /// // Non-integer, dynamic, or non-literal input:
    /// BigInt(x);
    /// BigInt("not-a-number");
    /// BigInt("1.23");
    /// ```
    PreferBigintLiterals,
    unicorn,
    style,
    fix
);

static BINARY_INTEGER_RE: Lazy<Regex> = lazy_regex!(r"^0[bB][01]+$");
static OCTAL_INTEGER_RE: Lazy<Regex> = lazy_regex!(r"^0[oO][0-7]+$");
static HEX_INTEGER_RE: Lazy<Regex> = lazy_regex!(r"^0[xX][0-9A-Fa-f]+$");
static DECIMAL_INTEGER_RE: Lazy<Regex> = lazy_regex!(r"^[0-9]+$");

fn matches_js_integer_literal(s: &str) -> bool {
    let s = s.trim();
    !s.is_empty()
        && (DECIMAL_INTEGER_RE.is_match(s)
            || HEX_INTEGER_RE.is_match(s)
            || BINARY_INTEGER_RE.is_match(s)
            || OCTAL_INTEGER_RE.is_match(s))
}

fn bigint_literal_from_string(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if !matches_js_integer_literal(trimmed) {
        return None;
    }

    if trimmed.starts_with("0b") || trimmed.starts_with("0B") {
        return Some(format!("{trimmed}n"));
    }

    if trimmed.starts_with("0o") || trimmed.starts_with("0O") {
        return Some(format!("{trimmed}n"));
    }

    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        return Some(format!("{trimmed}n"));
    }

    // Decimal: normalize leading zeros (e.g. "0777" -> "777n").
    Some(format!("{}n", trim_leading_zeros(trimmed)))
}

fn trim_leading_zeros(raw: &str) -> &str {
    let trimmed = raw.trim_start_matches('0');
    if trimmed.is_empty() { "0" } else { trimmed }
}

fn bigint_literal_from_numeric(raw: &str, base: NumberBase) -> Option<String> {
    let literal = match base {
        NumberBase::Binary | NumberBase::Hex => format!("{raw}n"),
        NumberBase::Octal => {
            if raw.starts_with("0o") || raw.starts_with("0O") {
                format!("{raw}n")
            } else {
                // Legacy octal like `0777` is invalid as a BigInt `0777n`, so normalize to `0o`.
                format!("0o{}n", trim_leading_zeros(raw))
            }
        }
        NumberBase::Decimal => format!("{}n", trim_leading_zeros(raw)),
        NumberBase::Float => return None,
    };
    Some(literal)
}

impl Rule for PreferBigintLiterals {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(call) = node.kind() {
            let Some(reference) = call.callee.get_identifier_reference() else {
                return;
            };

            if reference.name != "BigInt" {
                return;
            }

            if call.optional {
                return;
            }

            if call.arguments.len() != 1 {
                return;
            }

            let Some(arg) = call.arguments.first() else {
                return;
            };
            let diagnostic_span = arg.span();

            let Some(argument_expression) = arg.as_expression() else {
                return;
            };

            if argument_expression.is_big_int_literal() {
                return;
            }

            if let Expression::StringLiteral(string_literal) = argument_expression {
                if let Some(replacement) = bigint_literal_from_string(string_literal.value.as_str())
                {
                    ctx.diagnostic_with_fix(
                        prefer_bigint_literals_diagnostic(diagnostic_span),
                        |fixer| fixer.replace(call.span, replacement),
                    );
                }
                return;
            }

            if let Expression::NumericLiteral(numeric_literal) = argument_expression {
                if numeric_literal.value.fract() != 0.0 {
                    return;
                }

                let raw_text = numeric_literal.raw.as_ref().map_or_else(
                    || ctx.source_range(numeric_literal.span).to_string(),
                    |raw| raw.as_str().to_string(),
                );

                if let Some(replacement) =
                    bigint_literal_from_numeric(&raw_text, numeric_literal.base)
                {
                    ctx.diagnostic_with_fix(
                        prefer_bigint_literals_diagnostic(diagnostic_span),
                        |fixer| fixer.replace(call.span, replacement),
                    );
                } else {
                    ctx.diagnostic(prefer_bigint_literals_diagnostic(diagnostic_span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        r"1n",
        r"BigInt()",
        r"BigInt(1, 1)",
        r"BigInt(...[1])",
        r"BigInt(true)",
        r"BigInt(null)",
        r"new BigInt(1)",
        r"Not_BigInt(1)",
        r#"BigInt("1.0")"#,
        r#"BigInt("1.1")"#,
        r#"BigInt("1e3")"#,
        r"BigInt(`1`)",
        r#"BigInt("1" + "2")"#,
        r"BigInt?.(1)",
        r"BigInt(1.1)",
        r"typeof BigInt",
        r"BigInt(1n)",
        r#"BigInt("not-number")"#,
        r#"BigInt("1_2")"#,
        r#"BigInt("1\\\n2")"#,
        r#"String.raw`BigInt("\u{31}")`"#,
    ];
    let fail: Vec<&str> = vec![
        r#"BigInt("0")"#,
        r#"BigInt("  0  ")"#,
        r#"BigInt("9007199254740993")"#,
        r#"BigInt("0B11")"#,
        r#"BigInt("0O777")"#,
        r#"BigInt("0XFe")"#,
        r"BigInt(0)",
        r"BigInt(0B11_11)",
        r"BigInt(0O777_777)",
        r"BigInt(0XFe_fE)",
        r"BigInt(0777)",
        r"BigInt(0888)",
        r"BigInt(1.0)",
        r"BigInt(1e2)",
        r"BigInt(/* comment */1)",
        r"BigInt(9007199254740993)",
        r"BigInt(0x20000000000001)",
        r"BigInt(9_007_199_254_740_993)",
        r"BigInt(0x20_00_00_00_00_00_01)",
    ];
    let fix = vec![
        (r"BigInt('42')", "42n"),
        (r"BigInt('  0xFF  ')", "0xFFn"),
        (r"BigInt(0)", "0n"),
        (r"BigInt(0B11_11)", "0B11_11n"),
        (r"BigInt(0O777_777)", "0O777_777n"),
        (r"BigInt(0777)", "0o777n"),
        (r"BigInt(0888)", "888n"),
        (r#"BigInt("0777")"#, "777n"),
        (r#"BigInt("0888")"#, "888n"),
        (r#"BigInt("0b1010")"#, "0b1010n"),
        (r#"BigInt("0B0011")"#, "0B0011n"),
        (r#"BigInt("0O123")"#, "0O123n"),
        (r#"BigInt(" 0001 ")"#, "1n"),
        (
            r"BigInt('9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999')",
            "9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999n",
        ),
        (
            r"BigInt(9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999)",
            "9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999n",
        ),
    ];

    Tester::new(PreferBigintLiterals::NAME, PreferBigintLiterals::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
