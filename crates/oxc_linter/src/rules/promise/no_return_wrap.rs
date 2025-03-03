use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn no_return_wrap_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoReturnWrap;

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
    NoReturnWrap,
    promise,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for NoReturnWrap {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("Promise.resolve(4).then(function(x) { return x })", None),
        ("Promise.reject(4).then(function(x) { return x })", None),
        ("Promise.resolve(4).then(function() {})", None),
        ("Promise.reject(4).then(function() {})", None),
        ("doThing().then(function() { return 4 })", None),
        ("doThing().then(function() { throw 4 })", None),
        ("doThing().then(null, function() { return 4 })", None),
        ("doThing().then(null, function() { throw 4 })", None),
        ("doThing().catch(null, function() { return 4 })", None),
        ("doThing().catch(null, function() { throw 4 })", None),
        ("doThing().then(function() { return Promise.all([a,b,c]) })", None),
        ("doThing().then(() => 4)", None),
        ("doThing().then(() => { throw 4 })", None),
        ("doThing().then(()=>{}, () => 4)", None),
        ("doThing().then(()=>{}, () => { throw 4 })", None),
        ("doThing().catch(() => 4)", None),
        ("doThing().catch(() => { throw 4 })", None),
        ("var x = function() { return Promise.resolve(4) }", None),
        ("function y() { return Promise.resolve(4) }", None),
        ("function then() { return Promise.reject() }", None),
        ("doThing(function(x) { return Promise.reject(x) })", None),
        ("doThing().then(function() { return })", None),
        (
            "doThing().then(function() { return Promise.reject(4) })",
            Some(serde_json::json!([{ "allowReject": true }])),
        ),
        ("doThing().then((function() { return Promise.resolve(4) }).toString())", None),
        (
            "doThing().then(() => Promise.reject(4))",
            Some(serde_json::json!([{ "allowReject": true }])),
        ),
        ("doThing().then(function() { return a() })", None),
        ("doThing().then(function() { return Promise.a() })", None),
        ("doThing().then(() => { return a() })", None),
        ("doThing().then(() => { return Promise.a() })", None),
        ("doThing().then(() => a())", None),
        ("doThing().then(() => Promise.a())", None),
    ];

    let fail = vec![
        ("doThing().then(function() { return Promise.resolve(4) })", None),
        ("doThing().then(null, function() { return Promise.resolve(4) })", None),
        ("doThing().catch(function() { return Promise.resolve(4) })", None),
        ("doThing().then(function() { return Promise.reject(4) })", None),
        ("doThing().then(null, function() { return Promise.reject(4) })", None),
        ("doThing().catch(function() { return Promise.reject(4) })", None),
        (
            r#"doThing().then(function(x) { if (x>1) { return Promise.resolve(4) } else { throw "bad" } })"#,
            None,
        ),
        ("doThing().then(function(x) { if (x>1) { return Promise.reject(4) } })", None),
        (
            "doThing().then(null, function() { if (true && false) { return Promise.resolve() } })",
            None,
        ),
        (
            "doThing().catch(function(x) {if (x) { return Promise.resolve(4) } else { return Promise.reject() } })",
            None,
        ),
        (
            "
			      fn(function() {
			        doThing().then(function() {
			          return Promise.resolve(4)
			        })
			        return
			      })",
            None,
        ),
        (
            "
			      fn(function() {
			        doThing().then(function nm() {
			          return Promise.resolve(4)
			        })
			        return
			      })",
            None,
        ),
        (
            "
			      fn(function() {
			        fn2(function() {
			          doThing().then(function() {
			            return Promise.resolve(4)
			          })
			        })
			      })",
            None,
        ),
        (
            "
			      fn(function() {
			        fn2(function() {
			          doThing().then(function() {
			            fn3(function() {
			              return Promise.resolve(4)
			            })
			            return Promise.resolve(4)
			          })
			        })
			      })",
            None,
        ),
        (
            "
			      const o = {
			        fn: function() {
			          return doThing().then(function() {
			            return Promise.resolve(5);
			          });
			        },
			      }
			      ",
            None,
        ),
        (
            "
			      fn(
			        doThing().then(function() {
			          return Promise.resolve(5);
			        })
			      );
			      ",
            None,
        ),
        ("doThing().then((function() { return Promise.resolve(4) }).bind(this))", None),
        ("doThing().then((function() { return Promise.resolve(4) }).bind(this).bind(this))", None),
        ("doThing().then(() => { return Promise.resolve(4) })", None),
        (
            "
			      function a () {
			        return p.then(function(val) {
			          return Promise.resolve(val * 4)
			        })
			      }
			      ",
            None,
        ),
        ("doThing().then(() => Promise.resolve(4))", None),
        ("doThing().then(() => Promise.reject(4))", None),
    ];

    Tester::new(NoReturnWrap::NAME, NoReturnWrap::PLUGIN, pass, fail).test_and_snapshot();
}
