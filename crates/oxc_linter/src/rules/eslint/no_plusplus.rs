use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    AstNode,
};

#[derive(Debug, Default, Clone)]
pub struct NoPlusplus;

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
    NoPlusplus,
    restriction,
);

impl Rule for NoPlusplus {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var foo = 0; foo=+1;", None),
        ("var foo = 0; foo=+1;", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        (
            "for (i = 0; i < l; i++) { console.log(i); }",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (var i = 0, j = i + 1; j < example.length; i++, j++) {}",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        ("for (;; i--, foo());", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        ("for (;; foo(), --i);", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        (
            "for (;; foo(), ++i, bar);",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (;; i++, (++j, k--));",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (;; foo(), (bar(), i++), baz());",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (;; (--i, j += 2), bar = j + 1);",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (;; a, (i--, (b, ++j, c)), d);",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
    ];

    let fail = vec![
        ("var foo = 0; foo++;", None),
        ("var foo = 0; foo--;", None),
        ("for (i = 0; i < l; i++) { console.log(i); }", None),
        ("for (i = 0; i < l; foo, i++) { console.log(i); }", None),
        ("var foo = 0; foo++;", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        (
            "for (i = 0; i < l; i++) { v++; }",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        ("for (i++;;);", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        ("for (;--i;);", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        ("for (;;) ++i;", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        ("for (;; i = j++);", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        ("for (;; i++, f(--j));", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        (
            "for (;; foo + (i++, bar));",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
    ];

    Tester::new(NoPlusplus::NAME, pass, fail).test_and_snapshot();
}
