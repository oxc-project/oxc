use oxc_ast::{
    ast::{Argument, CallExpression, FormalParameters, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    utils::is_promise,
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

fn is_within_promise_handler<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    if !matches!(node.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)) {
        return false;
    }

    let Some(parent) = ctx.nodes().parent_node(node.id()) else {
        return false;
    };
    if !matches!(ctx.nodes().kind(parent.id()), AstKind::Argument(_)) {
        return false;
    };

    let Some(AstKind::CallExpression(call_expr)) = ctx.nodes().parent_kind(parent.id()) else {
        return false;
    };

    matches!(call_expr.callee_name(), Some("then" | "catch"))
}

fn is_inside_promise(node: &AstNode, ctx: &LintContext) -> bool {
    if !matches!(node.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_))
        || !matches!(ctx.nodes().parent_kind(node.id()), Some(AstKind::Argument(_)))
    {
        return false;
    }

    ctx.nodes()
        .ancestors(node.id())
        .nth(2)
        .is_some_and(|node| node.kind().as_call_expression().is_some_and(has_promise_callback))
}

fn has_promise_callback(call_expr: &CallExpression) -> bool {
    matches!(
        call_expr.callee.as_member_expression().and_then(MemberExpression::static_property_name),
        Some("then" | "catch")
    )
}

fn is_promise_then_or_catch(call_expr: &CallExpression) -> Option<String> {
    let member_expr = call_expr.callee.get_member_expr()?;
    let prop_name = member_expr.static_property_name()?;

    // hello.then(), hello.catch()
    if matches!(prop_name, "then" | "catch") {
        return Some(prop_name.into());
    }

    None
}

/// Get closest callback function scope outside of current callback.
/// ```
/// doThing()
///  .then(a => getB(a) <---- get this scopes args
///    .then(b => getC(a, b)) <--- when here
///  )
/// ```
/// We don't want a violation of this rule in such cases
/// because we cannot unnest the above as `a` would be undefined.
/// Here is the unnested version where would be `a` `undefined`
/// in the second `then` callback:
/// ```
/// doThing()
///  .then(a => getB(a))
///  .then(b => getC(a, b))
/// ```
///
fn get_closest_promise_callback<'a, 'b>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> &'b Option<Vec<Argument<'a>>> {
    let closest_prom_cb_args = ctx.semantic().nodes().ancestors(node.id()).find_map(|node| {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return None;
        };

        if let Some(prop_name) = is_promise_then_or_catch(call_expr) {
            if prop_name == "then" {
                return Some(&call_expr.arguments);
            } else {
                return None;
            }
        } else {
            return None;
        };
    });

    closest_prom_cb_args
}

impl Rule for NoNesting {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(prop_name) = is_promise_then_or_catch(call_expr) else {
            return;
        };

        println!("yayyyyy  {call_expr:?}");

        let mut ancestors = ctx.nodes().ancestors(node.id());
        if ancestors.any(|node| is_inside_promise(node, ctx)) {
            ctx.diagnostic(no_nesting_diagnostic(call_expr.callee.span()));
        }
    }
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
