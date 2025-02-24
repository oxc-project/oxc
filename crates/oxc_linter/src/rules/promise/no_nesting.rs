use oxc_allocator::{Allocator, HashMap};
use oxc_ast::{
    ast::{CallExpression, Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_nesting_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
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
    style,
    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

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

fn closest_promise_callback_def_vars<'a, 'b>(
    node: &'a AstNode<'b>,
    ctx: &'a LintContext<'b>,
) -> Option<&'a CallExpression<'b>> {
    ctx.nodes()
        .ancestors(node.id())
        .filter_map(|node| node.kind().as_call_expression())
        .filter(|a| has_promise_callback(a))
        .nth(1)
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

    // For example: hello.then(), hello.catch()
    if matches!(prop_name, "then" | "catch") {
        return Some(prop_name.into());
    }

    None
}

/// Checks if we can safely unnest the promise callback.
///
/// 1. Gets names of variables defined in closest parent promise callback function scope.
/// 2. Checks if the argument callback of the nested promise call uses any of these variables from 1.
///
/// ```javascript
/// doThing()
///  .then(a => getB(a) <---- 1. Get this scopes bound variables
///    .then(b => getC(a, b)) <--- 2. Check for references to the bound variables from 1.
///  )
/// ```
///
/// We don't want a violation of this rule in the above case as unnesting would
/// result in the following code where `getC(a, b` would be referencing an
/// undefined `a`.
///
/// ```javascript
/// doThing()
///  .then(a => getB(a))
///  .then(b => getC(a, b))
/// ```
///
/// We then see that both `a` and `b` has a reference
/// in the nested promise callback. Because of this reference, this nesting
/// isn't a rule violation.
fn can_safely_unnest<'a>(
    call_expr: &CallExpression,
    closest: &CallExpression,
    ctx: &LintContext<'a>,
    alloc: &Allocator,
) -> bool {
    let mut closest_cb_scope_bindings: &HashMap<'_, &str, SymbolId> = &HashMap::new_in(alloc);

    closest.arguments.iter().for_each(|new_expr| {
        let Some(arg_expr) = new_expr.as_expression() else {
            return;
        };

        match arg_expr {
            Expression::ArrowFunctionExpression(arrow_expr) => {
                let func_scope = arrow_expr.scope_id();
                let bound_vars_for_scope = ctx.scopes().get_bindings(func_scope);
                closest_cb_scope_bindings = bound_vars_for_scope;
            }
            Expression::FunctionExpression(func_expr) => {
                let func_scope = func_expr.scope_id();
                let bound_vars_for_scope = ctx.scopes().get_bindings(func_scope);
                closest_cb_scope_bindings = bound_vars_for_scope;
            }
            _ => {}
        }
    });

    if let Some(cb_span) = call_expr.arguments.get(0).map(|a| a.span()) {
        // Now check for references in cb_span to variables defined in the closest parent cb scope.
        // In the given example we would loop through all bindings in the closest
        // parent scope a,b,c,d.
        //
        // ```javascript
        //  .then((a,b,c) => {
        //    const d = 5;
        //    getB(a).then(d => getC(a, b))
        //             // ^^^^^^^^^^^^^^^^ <- cb_span
        // };
        // ```
        for (_, binding_symbol_id) in closest_cb_scope_bindings {
            for usage in ctx.semantic().symbol_references(*binding_symbol_id) {
                let usage_span: Span = ctx.reference_span(usage);
                if cb_span.contains_inclusive(usage_span) {
                    // Cannot unnest this nested promise as the nested cb refers to a variable
                    // defined in the parent promise callback scope. Unnesting would result in
                    // reference to an undefined variable.
                    return false;
                };
            }
        }
    }

    return true;
}

impl Rule for NoNesting {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if is_promise_then_or_catch(call_expr).is_none() {
            return;
        };

        let allocator = Allocator::default();

        let mut ancestors = ctx.nodes().ancestors(node.id());
        if ancestors.any(|node| is_inside_promise(node, ctx)) {
            match closest_promise_callback_def_vars(node, ctx) {
                Some(closest) => {
                    if can_safely_unnest(call_expr, closest, ctx, &allocator) {
                        ctx.diagnostic(no_nesting_diagnostic(call_expr.callee.span()));
                    } else {
                        return;
                    }
                }
                None => ctx.diagnostic(no_nesting_diagnostic(call_expr.callee.span())),
            }
        };
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
