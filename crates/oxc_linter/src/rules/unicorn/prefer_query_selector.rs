use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::is_node_value, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(prefer-query-selector):")]
#[diagnostic(severity(warning), help(""))]
struct PreferQuerySelectorDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferQuerySelector;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `.querySelector()` over `.getElementById()`, `.querySelectorAll()` over `.getElementsByClassName()` and `.getElementsByTagName()`.
    ///
    /// ### Example
    /// ```javascript
    /// ```
    PreferQuerySelector,
    correctness
);

impl Rule for PreferQuerySelector {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };
        if is_node_value(&call_expr.callee) {
            return;
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "new document.getElementById(foo);",
        "getElementById(foo);",
        "document['getElementById'](bar);",
        "document[getElementById](bar);",
        "document.foo(bar);",
        "document.getElementById();",
        "document.getElementsByClassName(\"foo\", \"bar\");",
        "document.getElementById(...[\"id\"]);",
        "document.querySelector(\"#foo\");",
        "document.querySelector(\".bar\");",
        "document.querySelector(\"main #foo .bar\");",
        "document.querySelectorAll(\".foo .bar\");",
        "document.querySelectorAll(\"li a\");",
        "document.querySelector(\"li\").querySelectorAll(\"a\");",
    ];

    let fail = vec![
        "document.getElementById(\"foo\");",
        "document.getElementsByClassName(\"foo\");",
        "document.getElementsByClassName(\"foo bar\");",
        "document.getElementsByTagName(\"foo\");",
        "document.getElementById(\"\");",
        "document.getElementById('foo');",
        "document.getElementsByClassName('foo');",
        "document.getElementsByClassName('foo bar');",
        "document.getElementsByTagName('foo');",
        "document.getElementsByClassName('');",
        "document.getElementById(`foo`);",
        "document.getElementsByClassName(`foo`);",
        "document.getElementsByClassName(`foo bar`);",
        "document.getElementsByTagName(`foo`);",
        "document.getElementsByTagName(``);",
        "document.getElementsByClassName(`${fn()}`);",
        "document.getElementsByClassName(`foo ${undefined}`);",
        "document.getElementsByClassName(null);",
        "document.getElementsByTagName(null);",
        "document.getElementsByClassName(fn());",
        "document.getElementsByClassName(\"foo\" + fn());",
        "document.getElementsByClassName(foo + \"bar\");",
        "e.getElementById(3)",
    ];

    Tester::new_without_config(PreferQuerySelector::NAME, pass, fail).test_and_snapshot();
}
