use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

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
    /// Disallows passing an argument into the second parameter of `then` calls, favouring the use
    /// of `catch` for handling promise errors instead.
    ///
    /// ### Why is this bad?
    ///
    /// A then call with two arguments can make it more difficult to recognize that a catch error handler is present.
    /// Also, using the second argument in `then` calls makes the ordering of promise error handling les
    /// obvious.
    ///
    /// For example on first glance it may appear that `prom.then(fn1, fn2)` is equivalent to
    /// `prom.then(fn1).catch(fn2)`. However they aren't equivalent. In fact
    /// `prom.catch(fn2).then(fn1)` is the equivalent.
    ///
    /// This easy confusion is a good reason for prefering explicit `catch` calls over passing an argument
    /// to the second parameter of `then` calls.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// prom.then(fn1, fn2)
    ///
    /// prom.then(null, fn2)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// prom.catch(fn2).then(fn1)
    ///
    /// prom.catch(fn2)
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
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExpressionStatement(expr) = node.kind() else {
            return;
        };

        let Expression::CallExpression(ref call_expr) = expr.expression else {
            return;
        };

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        let is_promise_then_call = member_expr
            .static_property_name()
            .map_or_else(|| false, |prop_name| matches!(prop_name, "then"));

        if !is_promise_then_call {
            println!("_______");
            let s = node.span().source_text(ctx.source_text());
            println!("{s:?}");


            println!("not a promise then: {node:?}");
            println!("_______");
        }
        // todo is arg count geq to 2? if so then flag violation

        //println!("aa {is_promise_then_call:?}");
        //println!("aa {node:?}");
        //println!("call {call_expr:?}");
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // not promise related
        "foo()",
        "a.foo()",
        "var a = new Foo()",
        "foo().then()",
        // I added these ^^
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
        "prom().then()",
        // I added ^
        "prom.then(fn1, fn2)",
        "prom.then(fn1, (fn2))",
        "prom.then(null, fn2)",
        "prom.then(undefined, fn2)",
        "function foo() { prom.then(x => {}, () => {}) }",
        "function foo() {
		   prom.then(function a() { }, function b() {}).then(fn1, fn2)
	     }",
    ];

    let fix = vec![
        ("prom.then(fn1, fn2)", "prom.catch(fn2).then(fn1)", None),
        ("prom.then(fn1, (fn2))", "prom.catch(fn2).then(fn1)", None),
        ("prom.then(null, fn2)", "prom.catch(fn2)", None),
        ("prom.then(undefined, fn2)", "prom.catch(fn2)", None),
        (
            "function foo() { prom.then(x => {}, () => {}) }",
            "function foo() { prom.catch(() => {}).then(x => {}) }",
            None,
        ),
        (
            "
			        function foo() {
			          prom.then(function a() { }, function b() {}).then(fn1, fn2)
			        }
			      ",
            "
			        function foo() {
			          prom.catch(function b() {}).then(function a() { }).catch(fn2).then(fn1)
			        }
			      ",
            None,
        ),
    ];
    Tester::new(PreferCatch::NAME, PreferCatch::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
