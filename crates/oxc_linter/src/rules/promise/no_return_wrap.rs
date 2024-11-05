use oxc_ast::AstKind::{self, ParenthesizedExpression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;

use crate::utils::is_promise;
use crate::{context::LintContext, rule::Rule, AstNode};

fn no_return_wrap_diagnostic_resolve(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid wrapping return values in Promise.resolve").with_label(span)
}

fn no_return_wrap_diagnostic_reject(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected throw instead of Promise.reject").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoReturnWrap(Box<NoReturnWrapOptions>);

#[derive(Debug, Default, Clone)]
pub struct NoReturnWrapOptions {
    pub allow_reject: bool,
}

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
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for NoReturnWrap {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0);

        let allow_reject = config
            .and_then(|c| c.get("allowReject"))
            .and_then(serde_json::Value::as_bool)
            .map_or(false, |v| v);

        Self(Box::new(NoReturnWrapOptions { allow_reject }))
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        if call_expr.callee.as_member_expression().is_some()
            && is_promise(call_expr).is_some()
            && is_in_promise(node.id(), ctx)
        {
            if matches!(call_expr.callee_name(), Some("resolve")) {
                ctx.diagnostic(no_return_wrap_diagnostic_resolve(call_expr.span));
            }
            if !self.0.allow_reject && matches!(call_expr.callee_name(), Some("reject")) {
                ctx.diagnostic(no_return_wrap_diagnostic_reject(call_expr.span));
            }
        }
    }
}

fn is_in_promise(id: NodeId, ctx: &LintContext) -> bool {
    // find the innermost function node
    let mut function_node = ctx.nodes().ancestors(id).find(|node| {
        matches!(node.kind(), AstKind::ArrowFunctionExpression(_) | oxc_ast::AstKind::Function(_))
    });

    // move up to the parent node while the function is re-bound with "bind"
    while let Some(func_node) = function_node {
        let Some(mut parent) = ctx.nodes().parent_id((func_node).id()) else { break };
        if matches!(ctx.nodes().kind(parent), ParenthesizedExpression(_)) {
            parent = match ctx.nodes().parent_id(parent) {
                Some(id) => id,
                None => break,
            };
        }
        let oxc_ast::AstKind::MemberExpression(member_expr) = ctx.nodes().kind(parent) else {
            break;
        };
        if member_expr.static_property_name() != Some("bind") {
            break;
        };
        let Some(grandparent) = ctx.nodes().parent_id(parent) else { break };
        let oxc_ast::AstKind::CallExpression(call_expr) = ctx.nodes().kind(grandparent) else {
            break;
        };
        if call_expr.callee.as_member_expression().is_some() {
            function_node = Some(ctx.nodes().get_node(grandparent));
            continue;
        }

        break;
    }
    let Some(function_node) = function_node else {
        return false;
    };

    // judge parent node is promise
    let Some(parent) = ctx.nodes().parent_node(function_node.id()) else { return false };
    let parent_kind = parent.kind();
    let parent = match parent_kind {
        AstKind::ParenthesizedExpression(_) | AstKind::Argument(_) => parent,
        _ => return false,
    };

    let grandparent = match ctx.nodes().parent_node(parent.id()) {
        Some(node)
            if matches!(
                node.kind(),
                AstKind::ParenthesizedExpression(_) | AstKind::MemberExpression(_)
            ) =>
        {
            ctx.nodes().parent_node(node.id()).unwrap_or(node)
        }
        Some(node) => node,
        None => return false,
    };

    if let AstKind::CallExpression(call_expr) = ctx.nodes().kind(grandparent.id()) {
        return is_promise(call_expr).is_some();
    }

    false
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

    Tester::new(NoReturnWrap::NAME, pass, fail).test_and_snapshot();
}
