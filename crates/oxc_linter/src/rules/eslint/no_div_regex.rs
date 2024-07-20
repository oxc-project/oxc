use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_div_regex_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("A regular expression literal can be confused with '/='.")
        .with_help("Rewrite `/=` into `/[=]`")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct NoDivRegex;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow equal signs explicitly at the beginning of regular expressions.
    ///
    /// ### Why is this bad?
    ///
    /// Characters /= at the beginning of a regular expression literal can be confused with a
    /// division assignment operator.
    ///
    /// ### Example
    /// ```javascript
    /// function bar() { return /=foo/; }
    /// ```
    NoDivRegex,
    restriction,
);

impl Rule for NoDivRegex {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::RegExpLiteral(lit) = node.kind() {
            if lit.regex.pattern.starts_with('=') {
                ctx.diagnostic_with_fix(no_div_regex_diagnostic(lit.span), |fixer| {
                    let span = Span::sized(lit.span.start + 1, 1);
                    fixer.replace(span, "[=]")
                });
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var f = function() { return /foo/ig.test('bar'); };",
        "var f = function() { return /\\=foo/; };",
    ];

    let fail = vec!["var f = function() { return /=foo/; };"];

    let fix = vec![(
        "var f = function() { return /=foo/; };",
        "var f = function() { return /[=]foo/; };",
        None,
    )];

    Tester::new(NoDivRegex::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
