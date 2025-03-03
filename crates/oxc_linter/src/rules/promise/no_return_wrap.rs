use oxc_allocator::Box as OBox;
use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, FunctionBody, MemberExpression, ReturnStatement, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{Span, format_compact_str};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    utils::is_promise,
};

fn no_return_wrap_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoReturnWrap {
    allow_reject: bool,
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
    /// It is unnecessary to use `Promise.resolve` and Promise.reject` for converting raw values
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
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// ### Options
    ///
    NoReturnWrap,
    promise,
    nursery,
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

        let d = ctx.source_text();
        //let in_prom_cb =
        if !inside_then_or_catch(node, ctx) {
            return;
        }

        //  let args = &call_expr.arguments;
        //     let resolve_cb = todo!();
        //     let reject_cb = todo!();

        // let ret = get_return(state);
        //        println!("return then or catch: {0:?}, {1:?}", args , d);

        for argument in &call_expr.arguments {
            let Some(arg_expr) = argument.as_expression().map(|a| a.without_parentheses()) else {
                println!("noe");

                continue;
            };

            match arg_expr {
                Expression::ArrowFunctionExpression(arrow_expr) => {
                    check_first_return_statement(&arrow_expr.body, ctx, self.allow_reject);
                }
                Expression::FunctionExpression(func_expr) => {
                    let Some(func_body) = &func_expr.body else {
                        continue;
                    };
                    check_first_return_statement(func_body, ctx, self.allow_reject);
                }
                //  Expression::CallExpression(call) => {
                //      let Expression::StaticMemberExpression(s) = call.callee.get_inner_expression()
                //      else {
                //          continue;
                //      };
                //      match &s.object.get_inner_expression() {
                //          Expression::ArrowFunctionExpression(arrow_expr) => {
                //              check_first_return_statement(&arrow_expr.body, ctx, self.allow_reject);
                //          }
                //          Expression::FunctionExpression(func_expr) => {
                //              let Some(func_body) = &func_expr.body else {
                //                  continue;
                //              };
                //              check_first_return_statement(func_body, ctx, self.allow_reject);
                //          }
                //          _ => continue,
                //      }
                //      continue;
                //  }
                _ => continue,
            }
        }
    }
}

/// Checks for `return Promise.resolve()` or `return Promise.reject()` at top level statements and
/// will look inside if no return is found as a top level statement in the function body.
fn check_first_return_statement<'a>(
    func_body: &OBox<'_, FunctionBody<'a>>,
    ctx: &LintContext<'a>,
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
                    if let Statement::ReturnStatement(r) = stmt {
                        return Some(r);
                    } else {
                        return None;
                    }
                });

                match res {
                    None => {
                        // No return found so now look `if {  } else { // here }`
                        block_stmt.body.iter().find_map(|stmt| {
                            if let Statement::ReturnStatement(r) = stmt {
                                return Some(r);
                            } else {
                                return None;
                            }
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

    let Some(Expression::CallExpression(call_expr)) = &return_stmt.argument else {
        return;
    };

    let Expression::StaticMemberExpression(stat_expr) = &call_expr.callee else {
        return;
    };

    let Some(obj_call_ident) = stat_expr.object.get_identifier_reference() else {
        return;
    };

    if !ctx.semantic().is_reference_to_global_variable(obj_call_ident) {
        return;
    }

    if obj_call_ident.name == "Promise" {
        if stat_expr.property.name == "resolve" {
            ctx.diagnostic(no_return_wrap_diagnostic(call_expr.span));
        } else if stat_expr.property.name == "reject" && !allow_reject {
            ctx.diagnostic(no_return_wrap_diagnostic(call_expr.span));
        }
    }
}

/// Return true if this node is inside a `then` or `catch` promise callback. Will return `true`
/// for `node` in both `prom.then(null, () => node)` and `prom.then(() => node)`.
fn inside_then_or_catch<'a, 'b>(node: &'a AstNode<'b>, ctx: &'a LintContext<'b>) -> bool {
    ctx.nodes().ancestors(node.id()).any(|node| {
        node.kind().as_call_expression().is_some_and(|call_expr| {
            matches!(
                call_expr
                    .callee
                    .as_member_expression()
                    .and_then(MemberExpression::static_property_name),
                Some("then" | "catch")
            )
        })
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
    ];

    Tester::new(NoReturnWrap::NAME, NoReturnWrap::PLUGIN, pass, fail).test_and_snapshot();
}
