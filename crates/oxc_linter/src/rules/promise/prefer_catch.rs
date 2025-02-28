use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn prefer_catch_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferCatch;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    PreferCatch,
    promise,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for PreferCatch {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "prom.then()",
        "prom.then(fn)",
        "prom.then(fn1).then(fn2)",
        "prom.then(() => {})",
        "prom.then(function () {})",
        "prom.catch()",
        "prom.catch(handleErr).then(handle)",
        "prom.catch(handleErr)",
    ];

    let fail = vec![
        "hey.then(fn1, fn2)",
        "hey.then(fn1, (fn2))",
        "hey.then(null, fn2)",
        "hey.then(undefined, fn2)",
        "function foo() { hey.then(x => {}, () => {}) }",
        "
			        function foo() {
			          hey.then(function a() { }, function b() {}).then(fn1, fn2)
			        }
			      ",
    ];

    let fix = vec![
        ("hey.then(fn1, fn2)", "hey.catch(fn2).then(fn1)", None),
        ("hey.then(fn1, (fn2))", "hey.catch(fn2).then(fn1)", None),
        ("hey.then(null, fn2)", "hey.catch(fn2)", None),
        ("hey.then(undefined, fn2)", "hey.catch(fn2)", None),
        (
            "function foo() { hey.then(x => {}, () => {}) }",
            "function foo() { hey.catch(() => {}).then(x => {}) }",
            None,
        ),
        (
            "
			        function foo() {
			          hey.then(function a() { }, function b() {}).then(fn1, fn2)
			        }
			      ",
            "
			        function foo() {
			          hey.catch(function b() {}).then(function a() { }).catch(fn2).then(fn1)
			        }
			      ",
            None,
        ),
    ];
    Tester::new(PreferCatch::NAME, PreferCatch::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
