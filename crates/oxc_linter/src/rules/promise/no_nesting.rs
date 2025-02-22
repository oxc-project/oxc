use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    AstNode,
};

fn no_nesting_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNesting;

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
    NoNesting,
    promise,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for NoNesting {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Promise.resolve(4).then(function(x) { return x })",
        "Promise.reject(4).then(function(x) { return x })",
        "Promise.resolve(4).then(function() {})",
        "Promise.reject(4).then(function() {})",
        "doThing().then(function() { return 4 })",
        "doThing().then(function() { throw 4 })",
        "doThing().then(null, function() { return 4 })",
        "doThing().then(null, function() { throw 4 })",
        "doThing().catch(null, function() { return 4 })",
        "doThing().catch(null, function() { throw 4 })",
        "doThing().then(() => 4)",
        "doThing().then(() => { throw 4 })",
        "doThing().then(()=>{}, () => 4)",
        "doThing().then(()=>{}, () => { throw 4 })",
        "doThing().catch(() => 4)",
        "doThing().catch(() => { throw 4 })",
        "var x = function() { return Promise.resolve(4) }",
        "function y() { return Promise.resolve(4) }",
        "function then() { return Promise.reject() }",
        "doThing(function(x) { return Promise.reject(x) })",
        "doThing().then(function() { return Promise.all([a,b,c]) })",
        "doThing().then(function() { return Promise.resolve(4) })",
        "doThing().then(() => Promise.resolve(4))",
        "doThing().then(() => Promise.all([a]))",
        "doThing()
			      .then(a => getB(a)
			        .then(b => getC(a, b))
			      )",
        "doThing()
			      .then(a => {
			        const c = a * 2;
			        return getB(c).then(b => getC(c, b))
			      })",
    ];

    let fail = vec![
        "doThing().then(function() { a.then() })",
        "doThing().then(function() { b.catch() })",
        "doThing().then(function() { return a.then() })",
        "doThing().then(function() { return b.catch() })",
        "doThing().then(() => { a.then() })",
        "doThing().then(() => { b.catch() })",
        "doThing().then(() => a.then())",
        "doThing().then(() => b.catch())",
        "
			      doThing()
			        .then(a => getB(a)
			          .then(b => getC(b))
			        )",
        "
			      doThing()
			        .then(a => getB(a)
			          .then(b => getC(a, b)
			            .then(c => getD(a, c))
			          )
			        )",
    ];

    Tester::new(NoNesting::NAME, NoNesting::PLUGIN, pass, fail).test_and_snapshot();
}
