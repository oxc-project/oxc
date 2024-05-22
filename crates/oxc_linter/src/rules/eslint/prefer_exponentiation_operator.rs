use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct PreferExponentiationOperator;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    PreferExponentiationOperator,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc-project.github.io/docs/contribute/linter.html#rule-category> for details
);

impl Rule for PreferExponentiationOperator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        "Object.pow(a, b)",
        "Math.max(a, b)",
        "Math",
        "Math(a, b)",
        "pow",
        "pow(a, b)",
        "Math.pow",
        "Math.Pow(a, b)",
        "math.pow(a, b)",
        "foo.Math.pow(a, b)",
        "new Math.pow(a, b)",
        "Math[pow](a, b)",
        "/* globals Math:off*/ Math.pow(a, b)",
        "let Math; Math.pow(a, b);",
        "if (foo) { const Math = 1; Math.pow(a, b); }",
        "var x = function Math() { Math.pow(a, b); }",
        "function foo(Math) { Math.pow(a, b); }",
        "function foo() { Math.pow(a, b); var Math; }",
        "class C { #pow; foo() { Math.#pow(a, b); } }",
    ];

    let fail = vec![
        "Math.pow(a, b) + Math.pow(c,
			 d)",
        "Math.pow(Math.pow(a, b), Math.pow(c, d))",
        "Math.pow(a, b)**Math.pow(c, d)",
        "Math.pow(a, b as any)",
        "Math.pow(a as any, b)",
        "Math.pow(a, b) as any",
    ];

    Tester::new(PreferExponentiationOperator::NAME, pass, fail).test_and_snapshot();
}
