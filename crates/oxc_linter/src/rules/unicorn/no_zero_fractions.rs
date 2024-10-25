use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn zero_fraction(span: Span, lit: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Don't use a zero fraction in the number.")
        .with_help(format!("Replace the number literal with `{lit}`"))
        .with_label(span)
}

fn dangling_dot(span: Span, lit: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Don't use a dangling dot in the number.")
        .with_help(format!("Replace the number literal with `{lit}`"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoZeroFractions;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents the use of zero fractions.
    ///
    /// ### Why is this bad?
    ///
    /// There is no difference in JavaScript between, for example, `1`, `1.0` and `1.`, so prefer the former for consistency and brevity.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = 1.0;
    /// const foo = -1.0;
    /// const foo = 123_456.000_000;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const foo = 1;
    /// const foo = -1;
    /// const foo = 123456;
    /// const foo = 1.1;
    /// ```
    NoZeroFractions,
    style,
    fix
);

impl Rule for NoZeroFractions {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NumericLiteral(number_literal) = node.kind() else {
            return;
        };

        let Some((fmt, is_dangling_dot)) = format_raw(number_literal.raw) else {
            return;
        };
        if fmt == number_literal.raw {
            return;
        };

        ctx.diagnostic_with_fix(
            if is_dangling_dot {
                dangling_dot(number_literal.span, &fmt)
            } else {
                zero_fraction(number_literal.span, &fmt)
            },
            |fixer| {
                let mut fixed = fmt.clone();
                let is_decimal_integer = fmt.parse::<i64>().is_ok();
                let is_member_expression =
                    ctx.nodes().parent_node(node.id()).map_or(false, |parent_node| {
                        matches!(parent_node.kind(), AstKind::MemberExpression(_))
                    });

                if is_member_expression && is_decimal_integer {
                    fixed = format!("({fixed})");
                    // TODO: checks the type and value of tokenBefore:
                    // If tokenBefore is a Punctuator (e.g., a symbol like ;, ], or )), it determines whether a semicolon is necessary based on the context (e.g., the type of the last block node).
                    // If the token type is in tokenTypesNeedsSemicolon, it returns true (semicolon needed).
                    // Special cases like Template strings, ObjectExpression blocks, and certain Identifier cases are handled explicitly.
                    // https://github.com/sindresorhus/eslint-plugin-unicorn/blob/77f32e5a6b2df542cf50dfbd371054f2cd8ce2d6/rules/no-zero-fractions.js#L56
                }

                // Handle special cases where a space is needed after certain keywords
                // to prevent the number from being interpreted as a property access
                let end = number_literal.span.start;
                let token = ctx.source_range(oxc_span::Span::new(0, end));
                if token.ends_with("return")
                    || token.ends_with("throw")
                    || token.ends_with("typeof")
                    || token.ends_with("void")
                {
                    fixed = format!(" {fixed}");
                }

                fixer.replace(number_literal.span, fixed)
            },
        );
    }
}

fn format_raw(raw: &str) -> Option<(String, bool)> {
    // Check if the string contains 'e' or 'E' (scientific notation)
    if let Some((base, exp)) = raw.split_once(['e', 'E']) {
        // Process the base part
        let (formatted_base, has_fraction) = format_raw(base)?;
        // Recombine the scientific notation
        return Some((format!("{formatted_base}e{exp}"), has_fraction));
    }
    let (before, after_and_dot) = raw.split_once('.')?;
    let mut after_parts = after_and_dot.splitn(2, |c: char| !c.is_ascii_digit() && c != '_');
    let dot_and_fractions = after_parts.next()?;
    let after = after_parts.next().unwrap_or("");

    let fixed_dot_and_fractions = dot_and_fractions.trim_end_matches(['0', '.', '_']);
    let formatted = format!(
        "{}{}{}{}",
        if before.is_empty() && fixed_dot_and_fractions.is_empty() { "0" } else { before },
        if fixed_dot_and_fractions.is_empty() { "" } else { "." },
        fixed_dot_and_fractions,
        after
    );

    Some((formatted, dot_and_fractions.is_empty()))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"const foo = "123.1000""#,
        r#"foo("123.1000")"#,
        r"const foo = 1",
        r"const foo = 1 + 2",
        r"const foo = -1",
        r"const foo = 123123123",
        r"const foo = 1.1",
        r"const foo = -1.1",
        r"const foo = 123123123.4",
        r"const foo = 1e3",
        r"1 .toString()",
    ];

    let fail = vec![
        r"const foo = 1.0",
        r"const foo = 1.0 + 1",
        r"foo(1.0 + 1)",
        r"const foo = 1.00",
        r"const foo = 1.00000",
        r"const foo = -1.0",
        r"const foo = 123123123.0",
        r"const foo = 123.11100000000",
        r"const foo = 1.",
        r"const foo = +1.",
        r"const foo = -1.",
        r"const foo = 1.e10",
        r"const foo = +1.e-10",
        r"const foo = -1.e+10",
        r"const foo = (1.).toString()",
        r"1.00.toFixed(2)",
        r"1.00 .toFixed(2)",
        r"(1.00).toFixed(2)",
        r"1.00?.toFixed(2)",
        r"a = .0;",
        r"a = .0.toString()",
        r"function foo(){return.0}",
        r"function foo(){return.0.toString()}",
        r"function foo(){return.0+.1}",
        "ôTest(0.)",
    ];

    let fix = vec![
        (r"const foo = 1.0", r"const foo = 1"),
        (r"const foo = 1.0 + 1", r"const foo = 1 + 1"),
        (r"foo(1.0 + 1)", r"foo(1 + 1)"),
        (r"const foo = 1.00", r"const foo = 1"),
        (r"const foo = 1.00000", r"const foo = 1"),
        (r"const foo = -1.0", r"const foo = -1"),
        (r"const foo = 123123123.0", r"const foo = 123123123"),
        (r"const foo = 123.11100000000", r"const foo = 123.111"),
        (r"const foo = 1.", r"const foo = 1"),
        (r"const foo = +1.", r"const foo = +1"),
        (r"const foo = -1.", r"const foo = -1"),
        (r"const foo = 1.e10", r"const foo = 1e10"),
        (r"const foo = +1.e-10", r"const foo = +1e-10"),
        (r"const foo = -1.e+10", r"const foo = -1e+10"),
        (r"const foo = (1.).toString()", r"const foo = (1).toString()"),
        (r"1.00.toFixed(2)", r"(1).toFixed(2)"),
        (r"1.010.toFixed(2)", r"1.01.toFixed(2)"),
        (r"1.00 .toFixed(2)", r"(1) .toFixed(2)"),
        (r"(1.00).toFixed(2)", r"(1).toFixed(2)"),
        (r"1.00?.toFixed(2)", r"(1)?.toFixed(2)"),
        (r"a = .0;", r"a = 0;"),
        (r"a = .0.toString()", r"a = (0).toString()"),
        (r"function foo(){return.0}", r"function foo(){return 0}"),
        (r"function foo(){return.0.toString()}", r"function foo(){return (0).toString()}"),
        (r"function foo(){return.0+.1}", r"function foo(){return 0+.1}"),
        (r"typeof.0", r"typeof 0"),
        (r"function foo(){typeof.0.toString()}", r"function foo(){typeof (0).toString()}"),
        (r"typeof.0+.1", r"typeof 0+.1"),
        (r"function foo(){throw.0;}", r"function foo(){throw 0;}"),
        (r"function foo(){typeof.0.toString()}", r"function foo(){typeof (0).toString()}"),
        (r"function foo(){throw.0+.1;}", r"function foo(){throw 0+.1;}"),
        (r"void.0", r"void 0"),
        (r"function foo(){void.0.toString()}", r"function foo(){void (0).toString()}"),
        (r"function foo(){void.0+.1;}", r"function foo(){void 0+.1;}"),
        ("ôTest(0.)", "ôTest(0)"),
    ];

    Tester::new(NoZeroFractions::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
