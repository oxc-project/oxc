use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::ast::{CharacterKind, Term};
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_div_regex_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("A regular expression literal can be confused with '/='.")
        .with_help("Rewrite `/=` into `/[=]`")
        .with_label(span)
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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function bar() { return /=foo/; }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// function bar() { return /[=]foo/; }
    /// ```
    NoDivRegex,
    eslint,
    restriction,
    fix,
    version = "0.4.2",
    short_description = "Disallow equal signs explicitly at the beginning of regular expressions.",
);

impl Rule for NoDivRegex {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::RegExpLiteral(lit) = node.kind() {
            let Some(pattern) = &lit.regex.pattern.pattern else { return };
            if pattern
                .body
                .body
                .first()
                .and_then(|it| it.body.first())
                .is_some_and(is_leading_equals)
            {
                ctx.diagnostic_with_fix(no_div_regex_diagnostic(lit.span), |fixer| {
                    let span = Span::sized(lit.span.start + 1, 1);
                    fixer.replace(span, "[=]")
                });
            }
        }
    }
}

/// Whether `term` starts the pattern with a literal `=`, also when it is
/// quantified (`/=+/`, `/=*x/`, `/={2}/`).
fn is_leading_equals(term: &Term) -> bool {
    match term {
        Term::Character(ch) => ch.kind == CharacterKind::Symbol && ch.value == '=' as u32,
        Term::Quantifier(quantifier) => is_leading_equals(&quantifier.body),
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var f = function() { return /foo/ig.test('bar'); };",
        "var f = function() { return /\\=foo/; };",
        "var f = function() { return /[=]+$/; };",
        "var f = function() { return /a=+/; };",
    ];

    let fail = vec![
        "var f = function() { return /=foo/; };",
        "var f = function() { return /=+$/; };",
        "var f = function() { return /=*x/; };",
        "var f = function() { return /={2}/; };",
    ];

    let fix = vec![
        ("var f = function() { return /=foo/; };", "var f = function() { return /[=]foo/; };"),
        ("var f = function() { return /=+$/; };", "var f = function() { return /[=]+$/; };"),
        ("var f = function() { return /={2}/; };", "var f = function() { return /[=]{2}/; };"),
    ];

    Tester::new(NoDivRegex::NAME, NoDivRegex::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
