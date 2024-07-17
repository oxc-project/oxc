use oxc_ast::{
    ast::{CallExpression, Expression, Statement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    ast_util::{get_enclosing_function, is_method_call, is_promise},
    context::LintContext,
    rule::Rule,
    AstNode,
};

fn no_return_wrap_diagnostic(span0: Span, x0: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("eslint-plugin-promise(no-return-wrap): {x0}")).with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct NoReturnWrap {
    //
    allow_reject: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow wrapping values in Promise.resolve or Promise.reject when not needed
    /// (promise/no-return-wrap).
    ///
    /// ### Why is this bad?
    ///
    /// Ensure that inside a then() or a catch() we always return or throw a raw value instead of
    /// wrapping in Promise.resolve or Promise.reject
    ///
    /// ### Example
    /// ```javascript
    /// myPromise.then(function (val) {
    ///  return Promise.resolve(val * 2)
    /// })
    /// myPromise.then(function (val) {
    ///  return Promise.reject('bad thing')
    /// })
    /// ```
    NoReturnWrap,
    correctness,
);

impl Rule for NoReturnWrap {
    fn from_configuration(value: serde_json::Value) -> Self {
        let allow_reject = value
            .get(0)
            .and_then(|config| config.get("allowReject"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Self { allow_reject }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ArrowFunctionExpression(arrowfunc_expr) => {
                if arrowfunc_expr.body.statements.len() != 1 {
                    return;
                }

                let Statement::ExpressionStatement(expr_stmt) = &arrowfunc_expr.body.statements[0]
                else {
                    return;
                };

                let Expression::CallExpression(call_expr) = &expr_stmt.expression else {
                    return;
                };

                if !self.is_promise_call(call_expr) {
                    return;
                }

                is_in_promise(call_expr, node, call_expr.span, ctx);
            }
            AstKind::ReturnStatement(stmt) => {
                let Some(Expression::CallExpression(call_expr)) = &stmt.argument else {
                    return;
                };
                if !self.is_promise_call(call_expr) {
                    return;
                }

                is_in_promise(call_expr, node, stmt.span, ctx);
            }
            _ => {}
        }
    }
}

impl NoReturnWrap {
    fn is_promise_call(&self, call_expr: &CallExpression) -> bool {
        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return false;
        };

        if !member_expr.object().is_specific_id("Promise") {
            return false;
        }

        let Some(prop_name) = member_expr.static_property_name() else {
            return false;
        };

        if prop_name != "resolve" && prop_name != "reject" {
            return false;
        }

        if self.allow_reject && prop_name == "reject" {
            return false;
        }

        true
    }
}

fn is_in_promise<'a>(
    call_expr: &CallExpression,
    node: &AstNode<'a>,
    span: Span,
    ctx: &LintContext<'a>,
) {
    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return;
    };

    let Some(prop_name) = member_expr.static_property_name() else {
        return;
    };

    let Some(func_node) = get_enclosing_function(node, ctx) else { return };

    // Rename to get_enclosing_call_expr??
    // We are only interested in the first CallExpression from the enclosing function scope but
    // not in
    for node_id in ctx.nodes().ancestors(func_node.id()) {
        let kind = ctx.nodes().kind(node_id);
        let AstKind::CallExpression(outer_call_expr) = kind else { continue };

        // Ignore .bind(this)
        if !call_expr.optional && is_method_call(outer_call_expr, None, Some(&["bind"]), None, None)
        {
            continue;
        }

        if is_promise(outer_call_expr) {
            if prop_name == "resolve" {
                ctx.diagnostic(no_return_wrap_diagnostic(
                    span,
                    "Avoid wrapping return values in Promise.resolve",
                ));
            }
            if prop_name == "reject" {
                ctx.diagnostic(no_return_wrap_diagnostic(
                    span,
                    "Expected throw instead of Promise.reject",
                ));
            }
        }

        return;
    }
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
(r#"doThing().then(function(x) { if (x>1) { return Promise.resolve(4) } else { throw "bad" } })"#, None),
("doThing().then(function(x) { if (x>1) { return Promise.reject(4) } })", None),
("doThing().then(null, function() { if (true && false) { return Promise.resolve() } })", None),
("doThing().catch(function(x) {if (x) { return Promise.resolve(4) } else { return Promise.reject() } })", None),
("
			      fn(function() {
			        doThing().then(function() {
			          return Promise.resolve(4)
			        })
			        return
			      })", None),
("
			      fn(function() {
			        doThing().then(function nm() {
			          return Promise.resolve(4)
			        })
			        return
			      })", None),
("
			      fn(function() {
			        fn2(function() {
			          doThing().then(function() {
			            return Promise.resolve(4)
			          })
			        })
			      })", None),
("
			      fn(function() {
			        fn2(function() {
			          doThing().then(function() {
			            fn3(function() {
			              return Promise.resolve(4)
			            })
			            return Promise.resolve(4)
			          })
			        })
			      })", None),
("
			      const o = {
			        fn: function() {
			          return doThing().then(function() {
			            return Promise.resolve(5);
			          });
			        },
			      }
			      ", None),
("
			      fn(
			        doThing().then(function() {
			          return Promise.resolve(5);
			        })
			      );
			      ", None),
("doThing().then((function() { return Promise.resolve(4) }).bind(this))", None),
("doThing().then((function() { return Promise.resolve(4) }).bind(this).bind(this))", None),
("doThing().then(() => { return Promise.resolve(4) })", None),
("
			      function a () {
			        return p.then(function(val) {
			          return Promise.resolve(val * 4)
			        })
			      }
			      ", None),
("doThing1().then(() => Promise.resolve(9))", None),
("doThing().then(() => Promise.reject(4))", None)
    ];

    Tester::new(NoReturnWrap::NAME, pass, fail).test_and_snapshot();
}
