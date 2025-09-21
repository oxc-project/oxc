use oxc_allocator::Box as OBox;
use oxc_ast::{
    AstKind,
    ast::{ArrowFunctionExpression, CallExpression, Expression, FunctionBody, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_promise};

fn no_return_wrap_diagnostic(span: Span, issue: &ReturnWrapper) -> OxcDiagnostic {
    let warn_msg = match issue {
        ReturnWrapper::Resolve => "Avoid wrapping return values in Promise.resolve",
        ReturnWrapper::Reject => "Expected throw instead of Promise.reject",
    };

    let help_msg = match issue {
        ReturnWrapper::Resolve => "Return the value being passed into Promise.resolve instead",
        ReturnWrapper::Reject => "Throw the value being passed into Promise.reject instead",
    };

    OxcDiagnostic::warn(warn_msg).with_help(help_msg).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoReturnWrap {
    allow_reject: bool,
}

#[derive(Debug, PartialEq)]
enum ReturnWrapper {
    Resolve,
    Reject,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents unnecessary wrapping of return values in promises with either `Promise.resolve`
    /// or `Promise.reject`.
    ///
    /// This rule enforces the following stances:
    ///
    /// 1. When a promise is to be resolved, instead of returning `Promise.resolve(value)` it is
    /// better to return the raw value with `return value` instead.
    ///
    /// 2. When a promise is to be rejected, instead of returning `Promise.reject(error)`, instead
    /// the raw error value should be thrown as in `throw error`.
    ///
    /// There is an option to turn off the enforcing of 2, see the options section below.
    ///
    /// ### Why is this bad?
    ///
    /// It is unnecessary to use `Promise.resolve` and `Promise.reject` for converting raw values
    /// to promises in the return statements of `then` and `catch` callbacks. Using these
    /// operations to convert raw values to promises is unnecessary as simply returning the raw
    /// value for the success case and throwing the raw error value in the failure case have the
    /// same effect. This is why some take the opinion that returning values such as
    /// `Promise.resolve(1)` or `Promise.reject(err)` is syntactic noise.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// myPromise().then(() => Promise.resolve(4))
    /// myPromise().then(function() { return Promise.resolve(4) })
    ///
    /// myPromise().then(() => Promise.reject("err"))
    /// myPromise().then(function() { return Promise.reject("err") })
    /// ```
    ///
    /// ```js
    /// myPromise().catch(
    ///   function() {
    ///     return Promise.reject("err")
    /// })
    /// ```
    ///
    /// ```js
    /// myPromise().finally(
    ///   function() {
    ///     return Promise.reject("err")
    /// })
    /// ```
    ///
    /// ```js
    /// myPromise().finally(() => Promise.resolve(4))
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// myPromise().then(() => 4)
    /// myPromise().then(function() { return 4 })
    ///
    /// myPromise().then(() => throw "err")
    /// myPromise().then(function() { throw "err" })
    /// ```
    ///
    /// ```js
    /// myPromise().catch(
    ///   function() {
    ///     throw "err"
    /// })
    /// ```
    ///
    /// ```js
    /// myPromise().finally(() => 4)
    /// ```
    ///
    /// ### Options
    ///
    /// #### allowReject
    ///
    /// `{ type: boolean, default: false }`
    ///
    /// The `allowReject` turns off the checking of returning a call `Promise.reject` inside a
    /// promise handler.
    ///
    /// With `allowReject` set to `true` the following are examples of correct code:
    ///
    /// ```js
    /// myPromise().then(
    ///   function() {
    ///     return Promise.reject(0)
    /// })
    /// ```
    ///
    /// ```js
    /// myPromise().then().catch(() => Promise.reject("err"))
    /// ```
    NoReturnWrap,
    promise,
    style,
    pending
);

impl Rule for NoReturnWrap {
    fn from_configuration(value: serde_json::Value) -> Self {
        let allow_reject = value
            .get(0)
            .and_then(|v| v.get("allowReject"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Self { allow_reject }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if is_promise(call_expr).is_none() && !inside_promise_cb(node, ctx) {
            return;
        }

        for argument in &call_expr.arguments {
            let Some(arg_expr) = argument.as_expression().map(Expression::without_parentheses)
            else {
                continue;
            };

            match arg_expr {
                Expression::ArrowFunctionExpression(arrow) => {
                    check_arrow_cb_arg(ctx, self.allow_reject, arrow);
                }
                Expression::FunctionExpression(func_expr) => {
                    let Some(func_body) = &func_expr.body else {
                        continue;
                    };
                    check_first_return_statement(ctx, func_body, self.allow_reject);
                }
                Expression::CallExpression(call) => {
                    let Expression::StaticMemberExpression(static_memb_expr) =
                        call.callee.get_inner_expression()
                    else {
                        continue;
                    };

                    // `.bind(this)` is true but `.bind(foo)` is false.
                    let is_this_arg = call.arguments.first().is_some_and(|arg| {
                        matches!(arg.as_expression(), Some(Expression::ThisExpression(_)))
                    });

                    let property_name = static_memb_expr.property.name;

                    if is_this_arg && property_name == "bind" {
                    } else {
                        // We only examine the return statement inside func when the call expression on
                        // the func is a `this` binding for example `func.bind.this()` or
                        // `func.bind.this().bind.this()`.
                        continue;
                    }

                    let inner_obj = &static_memb_expr.object.get_inner_expression();

                    if let Expression::CallExpression(nested_call) = inner_obj {
                        // if not a chained .bind(this) then skip
                        let Expression::StaticMemberExpression(nested_expr) =
                            nested_call.callee.get_inner_expression()
                        else {
                            continue;
                        };
                        check_callback_fn(
                            ctx,
                            self.allow_reject,
                            nested_expr.object.without_parentheses(),
                        );
                    } else {
                        check_callback_fn(ctx, self.allow_reject, inner_obj);
                    }
                }
                _ => {}
            }
        }
    }
}

/// Look for issues in the arrow callback `cb` in `myProm().then(cb)`.
fn check_arrow_cb_arg<'a>(
    ctx: &LintContext<'a>,
    allow_reject: bool,
    arrow_expr: &ArrowFunctionExpression<'a>,
) {
    if arrow_expr.body.statements.len() == 1 {
        let Some(only_stmt) = &arrow_expr.body.statements.first() else {
            return;
        };

        if let Statement::BlockStatement(_) = only_stmt {
            check_first_return_statement(ctx, &arrow_expr.body, allow_reject);
        }

        if let Statement::ReturnStatement(r) = only_stmt
            && let Some(Expression::CallExpression(returned_call_expr)) = &r.argument
        {
            check_for_resolve_reject(ctx, allow_reject, returned_call_expr);
        }

        let Statement::ExpressionStatement(expr_stmt) = only_stmt else {
            return;
        };

        let Expression::CallExpression(ref returned_call_expr) = expr_stmt.expression else {
            return;
        };
        check_for_resolve_reject(ctx, allow_reject, returned_call_expr);
    } else {
        check_first_return_statement(ctx, &arrow_expr.body, allow_reject);
    }
}

fn check_callback_fn<'a>(ctx: &LintContext<'a>, allow_reject: bool, expr: &Expression<'a>) {
    match expr {
        Expression::ArrowFunctionExpression(arrow_expr) => {
            check_first_return_statement(ctx, &arrow_expr.body, allow_reject);
        }
        Expression::FunctionExpression(func_expr) => {
            let Some(func_body) = &func_expr.body else {
                return;
            };
            check_first_return_statement(ctx, func_body, allow_reject);
        }
        _ => (),
    }
}

/// Checks for `return` at top level statements and
/// will look inside if no return is found as a top level statement in the function body.
fn check_first_return_statement<'a>(
    ctx: &LintContext<'a>,
    func_body: &OBox<'_, FunctionBody<'a>>,
    allow_reject: bool,
) {
    let top_level_statements = func_body
        .statements
        .iter()
        .find(|stmt| matches!(stmt, Statement::ReturnStatement(_) | Statement::IfStatement(_)));

    let maybe_return_stmt = match top_level_statements {
        Some(Statement::ReturnStatement(r)) => Some(r),
        Some(Statement::IfStatement(if_stmt)) => match &if_stmt.consequent {
            Statement::BlockStatement(block_stmt) => {
                // Find first return statement in `if { // here } else { }`
                let res = block_stmt.body.iter().find_map(|stmt| {
                    if let Statement::ReturnStatement(r) = stmt { Some(r) } else { None }
                });

                match res {
                    None => {
                        // No return found so now look `if {  } else { // here }`
                        block_stmt.body.iter().find_map(|stmt| {
                            if let Statement::ReturnStatement(r) = stmt { Some(r) } else { None }
                        })
                    }
                    res => res,
                }
            }
            Statement::ReturnStatement(r) => Some(r),
            _ => None,
        },
        _ => None,
    };

    let Some(return_stmt) = maybe_return_stmt else {
        return;
    };

    let Some(Expression::CallExpression(returned_call_expr)) = &return_stmt.argument else {
        return;
    };

    check_for_resolve_reject(ctx, allow_reject, returned_call_expr);
}

/// Checks for `return Promise.resolve()` or `return Promise.reject()`
fn check_for_resolve_reject(ctx: &LintContext, allow_reject: bool, call_expr: &CallExpression) {
    let Expression::StaticMemberExpression(stat_expr) = &call_expr.callee else {
        return;
    };

    let Some(obj_call_ident) = stat_expr.object.get_identifier_reference() else {
        return;
    };

    if !ctx.semantic().is_reference_to_global_variable(obj_call_ident) {
        return;
    }

    if !(obj_call_ident.name == "Promise") {
        return;
    }

    if stat_expr.property.name == "resolve" {
        ctx.diagnostic(no_return_wrap_diagnostic(call_expr.span, &ReturnWrapper::Resolve));
    } else if stat_expr.property.name == "reject" && !allow_reject {
        ctx.diagnostic(no_return_wrap_diagnostic(call_expr.span, &ReturnWrapper::Reject));
    }
}

/// Return true if this node is inside a `then` or `catch` or `finally` promise callback.
fn inside_promise_cb<'a, 'b>(node: &'a AstNode<'b>, ctx: &'a LintContext<'b>) -> bool {
    ctx.nodes().ancestors(node.id()).any(|node| {
        node.kind().as_call_expression().is_some_and(|call_expr| is_promise(call_expr).is_some())
    })
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
        ("doThing().then(() => {}).finally(() => 4)", None),
        (r#"doThing().then(() => {}).finally(() => { throw "err" })"#, None),
        ("doThing().then(function() { return Promise.all([a,b,c]) })", None),
        ("doThing().then(() => 4)", None),
        ("doThing().then(() => { throw 4 })", None),
        ("doThing().then(() => {}, () => 4)", None),
        ("doThing().then(() => {}, () => { throw 4 })", None),
        ("doThing().catch(() => 4)", None),
        ("doThing().catch(() => { throw 4 })", None),
        ("var x = function() { return Promise.resolve(4) }", None),
        ("function y() { return Promise.resolve(4) }", None),
        ("function then() { return Promise.reject() }", None),
        ("doThing(function(x) { return Promise.reject(x) })", None),
        ("doThing().then(function() { return })", None),
        (
            "doThing().then(function() { return Promise.reject(0) })",
            Some(serde_json::json!([{ "allowReject": true }])),
        ),
        (r#"doThing().then(function () {}).finally(function () { Promise.reject("err") })"#, None),
        (
            r#"doThing().then().catch(() => Promise.reject("err"))"#,
            Some(serde_json::json!([{ "allowReject": true }])),
        ),
        (
            r#"doThing().then(function () {}).finally(function () { return Promise.reject("err") })"#,
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
        (
            "class Promise { constructor(){} resolve(){} };
             doThing().then(function() { return Promise.resolve(4) })",
            None,
        ),
    ];

    let fail = vec![
        ("doThing().then(function() { return Promise.resolve(4) })", None),
        ("doThing().then(null, function() { return Promise.resolve(4) })", None),
        ("doThing().catch(function() { return Promise.resolve(4) })", None),
        ("doThing().then(function() { return Promise.reject(4) })", None),
        ("doThing().then(null, function() { return Promise.reject(4) })", None),
        ("doThing().catch(function() { return Promise.reject(4) })", None),
        ("doThing().finally(() => Promise.resolve(4))", None),
        (
            r#"doThing().then(function () {}).finally(function () { return Promise.reject("err") })"#,
            None,
        ),
        (
            r#"doThing().then(
                 function(x) {
                   if (x>1) {
                     return Promise.resolve(4)
                   } else {
                     throw "bad"
                   }
                })"#,
            None,
        ),
        ("doThing().then(function(x) { if (x>1) { return Promise.reject(4) } })", None),
        (
            "doThing().then(null, function() { if (true && false) { return Promise.resolve() } })",
            None,
        ),
        (
            "doThing().catch(
              function(x) {
                if (x) {
                  return Promise.resolve(4)
                } else {
                  return Promise.reject()
                }
             })",
            None,
        ),
        (
            "fn(function() {
			   doThing().then(function() {
			     return Promise.resolve(4)
			   })
			   return
			 })",
            None,
        ),
        (
            "fn(function() {
			   doThing().then(function nm() {
			     return Promise.resolve(4)
			   })
			   return
			 })",
            None,
        ),
        (
            "fn(function() {
			   fn2(function() {
			     doThing().then(function() {
			       return Promise.resolve(4)
			     })
			   })
			 })",
            None,
        ),
        (
            "fn(function() {
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
            "const o = {
			   fn: function() {
			     return doThing().then(function() {
			       return Promise.resolve(5);
			     });
			   },
			 }",
            None,
        ),
        (
            "fn(
			   doThing().then(function() {
			     return Promise.resolve(5);
			   })
			 );",
            None,
        ),
        ("doThing().then((function() { return Promise.resolve(4) }).bind(this))", None),
        ("doThing().then((function() { return Promise.resolve(4) }).bind(this).bind(this))", None),
        ("doThing().then(null, (function() { return Promise.resolve(4) }).bind(this))", None),
        ("doThing().then(() => { return Promise.resolve(4) })", None),
        (
            "function a () {
		       return p.then(function(val) {
			     return Promise.resolve(val * 4)
			   })
			 }",
            None,
        ),
        ("doThing().then(() => Promise.resolve(4))", None),
        ("doThing().then(() => Promise.reject(4))", None),
        (
            "fn((() => {
			   fn2(function() {
			     doThing().then(function() {
			       fn3(function() {
			         return Promise.resolve(4)
			       })
			       return Promise.resolve(4)
			     })
			   }).bind(this)
			 }))",
            None,
        ),
    ];

    Tester::new(NoReturnWrap::NAME, NoReturnWrap::PLUGIN, pass, fail).test_and_snapshot();
}
