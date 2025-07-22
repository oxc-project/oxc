use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeId;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_promise};

fn no_nesting_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid nesting promises.")
        .with_help("Refactor so that promises are chained in a flat manner.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNesting;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow nested then() or catch() statements.
    ///
    /// ### Why is this bad?
    ///
    /// Nesting promises makes code harder to read and understand.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// doThing().then(() => a.then())
    ///
    /// doThing().then(function() { a.then() })
    ///
    /// doThing().then(() => { b.catch() })
    ///
    /// doThing().catch((val) => doSomething(val).catch(errors))
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// doThing().then(() => 4)
    ///
    /// doThing().then(function() { return 4 })
    ///
    /// doThing().catch(() => 4)
    /// ```
    ///
    /// ```javascript
    /// doThing()
    ///   .then(() => Promise.resolve(1))
    ///   .then(() => Promise.resolve(2))
    /// ```
    ///
    /// This example is not a rule violation as unnesting here would
    /// result in `a` being undefined in the expression `getC(a, b)`.
    /// ```javascript
    /// doThing()
    ///	  .then(a => getB(a)
    ///      .then(b => getC(a, b))
    ///    )
    /// ```
    NoNesting,
    promise,
    style,
    pending
);

fn is_inside_promise(node: &AstNode, ctx: &LintContext) -> bool {
    if !matches!(node.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_))
        || !matches!(ctx.nodes().parent_kind(node.id()), AstKind::Argument(_))
    {
        return false;
    }

    ctx.nodes().ancestors(node.id()).nth(1).is_some_and(|node| {
        node.kind().as_call_expression().is_some_and(|a| {
            is_promise(a).is_some_and(|prop_name| prop_name == "then" || prop_name == "catch")
        })
    })
}

/// Gets the closest promise callback function of the nested promise.
fn closest_promise_cb<'a, 'b>(
    node: &'a AstNode<'b>,
    ctx: &'a LintContext<'b>,
) -> Option<&'a CallExpression<'b>> {
    ctx.nodes().ancestors(node.id()).filter_map(|node| node.kind().as_call_expression()).find(
        |ancestor| {
            is_promise(ancestor)
                .is_some_and(|prop_name| prop_name == "then" || prop_name == "catch")
        },
    )
}

/// Checks if we can safely unnest the promise callback.
///
/// 1. This function gets variable bindings defined in closest parent promise callback function
///    scope.
///
/// 2. Checks if the argument callback of the nested promise call uses any of these variables
///    and if so returns `false` to denote that the promises cannot be safely unnested.
///
/// Here is an example of a nested promise which isn't safe to nest without further refactoring.
/// ```javascript
/// doThing()
///  .then(a => getB(a) <---- 1. Get this scopes bound variables
///    .then(b => getC(a, b)) <--- 2. Check for references to the bound variables from 1.
///  )
/// ```
///
/// In this case unnesting is not safe as doing so would result in `a` being undefined in the
/// expression `getC(a, b)`, as seen below in the unnested version of the example:
/// ```javascript
/// doThing()
///  .then(a => getB(a))
///  .then(b => getC(a, b))
/// ```
fn can_safely_unnest(
    cb_call_expr: &CallExpression,
    closest: &CallExpression,
    ctx: &LintContext,
) -> bool {
    let Some(cb_span) = cb_call_expr.arguments.first().map(GetSpan::span) else {
        return true;
    };

    // Loop through the args of closest parent callback which contains this child callback.
    for new_expr in &closest.arguments {
        let Some(arg_expr) = new_expr.as_expression() else {
            continue;
        };
        // Check if our nested child callback references one of these args and return early if so.
        match arg_expr {
            Expression::ArrowFunctionExpression(arrow_expr) => {
                let scope = arrow_expr.scope_id();
                if uses_closest_cb_vars(scope, cb_span, ctx) {
                    return false; // Not safe to unnest.
                }
            }
            Expression::FunctionExpression(func_expr) => {
                let scope = func_expr.scope_id();
                if uses_closest_cb_vars(scope, cb_span, ctx) {
                    return false; // Not safe to unnest.
                }
            }
            _ => {}
        }
    }

    // Didn't return false early, so it is safe to unnest the child callback as the child doesn't reference
    // variables bound in the closest parent callback.
    true
}

/// Check for references in cb_span to variables defined in the closest parent cb scope
/// and returns true if the nested promise callback uses references that are bound in
/// the closest parent callback scope.
///
/// In the given example we would loop through all bindings in the closest
/// parent scope a,b,c,d.
///
///  .then((a,b,c) => { // closest_cb_scope_id
///    const d = 5;
///    getB(a).then(d => getC(a, b)) });
///                // ^^^^^^^^^^^^^^ <- `cb_span`
fn uses_closest_cb_vars(closest_cb_scope_id: ScopeId, cb_span: Span, ctx: &LintContext) -> bool {
    for (_, binding_symbol_id) in ctx.scoping().get_bindings(closest_cb_scope_id) {
        for usage in ctx.semantic().symbol_references(*binding_symbol_id) {
            let usage_span: Span = ctx.reference_span(usage);
            if cb_span.contains_inclusive(usage_span) {
                // Cannot unnest this nested promise as the nested cb refers to a variable
                // defined in the parent promise callback scope. Unnesting would result in
                // reference to an undefined variable.
                return true;
            }
        }
    }

    false
}

impl Rule for NoNesting {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_promise(call_expr)
            .is_some_and(|prop_name| prop_name == "then" || prop_name == "catch")
        {
            return;
        }

        let mut ancestors = ctx.nodes().ancestors(node.id());
        if ancestors.any(|node| is_inside_promise(node, ctx)) {
            match closest_promise_cb(node, ctx) {
                Some(closest) => {
                    if can_safely_unnest(call_expr, closest, ctx) {
                        ctx.diagnostic(no_nesting_diagnostic(call_expr.callee.span()));
                    }
                }
                None => ctx.diagnostic(no_nesting_diagnostic(call_expr.callee.span())),
            }
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
        "doThing()
          .then(() => Promise.resolve(1))
          .then(() => Promise.resolve(2))",
        "doThing().then(() => Promise.all([a]))",
        "doThing()
		  .then(a => getB(a)
			.then(b => getC(a, b))
		  )",
        "doThing()
		  .then(a => getB(a)
		    .then(function(b) { getC(a, b) })
		  )",
        "doThing()
          .then(a => {
            const c = a * 2;
            return getB(c).then(b => getC(c, b))
          })",
        "doThing()
          .then(function (a) {
            const c = a * 2;
            return getB(c).then(function () { getC(c, b) } )
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
        "doThing().then((val) => doSomething(val).catch(errors))",
        "doThing().catch((val) => doSomething(val).catch(errors))",
        "doThing()
          .then(() =>
            a.then(() => Promise.resolve(1)))",
        "doThing()
		  .then(a => getB(a)
		    .then(b => getC(b))
		  )",
        "doThing()
		  .then(a => getB(a)
		    .then(b => getC(a, b)
		      .then(c => getD(a, c))
			)
		  )",
    ];

    Tester::new(NoNesting::NAME, NoNesting::PLUGIN, pass, fail).test_and_snapshot();
}
