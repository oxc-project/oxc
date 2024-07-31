use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn zero_fraction(span0: Span, x1: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Don't use a zero fraction in the number.")
        .with_help(format!("Replace the number literal with `{x1}`"))
        .with_label(span0)
}

fn dangling_dot(span0: Span, x1: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Don't use a dangling dot in the number.")
        .with_help(format!("Replace the number literal with `{x1}`"))
        .with_label(span0)
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
    /// ```javascript
    /// // Bad
    /// const foo = 1.0;
    /// const foo = -1.0;
    /// const foo = 123_456.000_000;
    ///
    /// // Good
    /// const foo = 1;
    /// const foo = -1;
    /// const foo = 123456;
    /// const foo = 1.1;
    /// ```
    NoZeroFractions,
    style
);

impl Rule for NoZeroFractions {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a, '_>) {
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
            |fixer| fixer.replace(number_literal.span, fmt),
        );
    }
}

fn format_raw(raw: &str) -> Option<(String, bool)> {
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
    ];

    Tester::new(NoZeroFractions::NAME, pass, fail).test_and_snapshot();
}
