use oxc_ast::{
    ast::{CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{
    ast_util::is_method_call, context::LintContext, rule::Rule, utils::is_promise, AstNode,
};

fn catch_or_return_diagnostic(method_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "eslint-plugin-promise(catch-or-return): Expected {method_name} or return"
    ))
    .with_help(format!("Return the promise or chain a {method_name}()"))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct CatchOrReturn(Box<CatchOrReturnConfig>);

#[derive(Debug, Clone)]
pub struct CatchOrReturnConfig {
    allow_finally: bool,
    allow_then: bool,
    termination_method: Vec<CompactStr>,
}

impl Default for CatchOrReturnConfig {
    fn default() -> Self {
        Self {
            allow_finally: false,
            allow_then: false,
            termination_method: vec![CompactStr::new("catch")],
        }
    }
}

impl std::ops::Deref for CatchOrReturn {
    type Target = CatchOrReturnConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensure that each time a `then()` is applied to a promise, a `catch()`
    /// must be applied as well. Exceptions are made for promises returned from
    /// a function.
    ///
    /// ### Why is this bad?
    ///
    /// Not catching errors in a promise can cause hard to debug problems or
    /// missing handling of error conditions. In the worst case, unhandled
    /// promise rejections can cause your application to crash.
    ///
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// myPromise.then(doSomething)
    /// myPromise.then(doSomething, catchErrors) // catch() may be a little better
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// myPromise.then(doSomething).catch(errors)
    /// function doSomethingElse() {
    ///  return myPromise.then(doSomething)
    /// }
    /// ```
    CatchOrReturn,
    promise,
    restriction,
);

impl Rule for CatchOrReturn {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut config = CatchOrReturnConfig::default();

        if let Some(termination_array_config) = value
            .get(0)
            .and_then(|v| v.get("terminationMethod"))
            .and_then(serde_json::Value::as_array)
        {
            config.termination_method = termination_array_config
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(CompactStr::from)
                .collect();
        }

        if let Some(termination_string_config) = value
            .get(0)
            .and_then(|v| v.get("terminationMethod"))
            .and_then(serde_json::Value::as_str)
        {
            config.termination_method = vec![CompactStr::new(termination_string_config)];
        }

        if let Some(allow_finally_config) =
            value.get(0).and_then(|v| v.get("allowFinally")).and_then(serde_json::Value::as_bool)
        {
            config.allow_finally = allow_finally_config;
        }

        if let Some(allow_then_config) =
            value.get(0).and_then(|v| v.get("allowThen")).and_then(serde_json::Value::as_bool)
        {
            config.allow_then = allow_then_config;
        }

        Self(Box::new(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExpressionStatement(expr_stmt) = node.kind() else {
            return;
        };

        let Expression::CallExpression(call_expr) = &expr_stmt.expression else {
            return;
        };

        // Check for a promise or a method call at the end of a promise for example:
        // foo().catch().randomFunc()
        if is_promise(call_expr).is_none() && !is_part_of_promise(call_expr) {
            return;
        }

        if self.is_allowed_promise_termination(call_expr) {
            return;
        }

        let termination_method = &self.termination_method[0];
        ctx.diagnostic(catch_or_return_diagnostic(termination_method, call_expr.span));
    }
}

impl CatchOrReturn {
    fn is_allowed_promise_termination(&self, call_expr: &CallExpression) -> bool {
        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return false;
        };

        let Some(prop_name) = member_expr.static_property_name() else {
            return false;
        };

        // somePromise.then(a, b)
        if self.allow_then && prop_name == "then" && call_expr.arguments.len() == 2 {
            return true;
        }

        // somePromise.catch().finally(fn)
        if self.allow_finally && prop_name == "finally" {
            let Expression::CallExpression(object_call_expr) = member_expr.object() else {
                return false;
            };

            if is_promise(object_call_expr).is_some()
                && self.is_allowed_promise_termination(object_call_expr)
            {
                return true;
            }
        }

        // somePromise.catch()
        if self.termination_method.contains(&CompactStr::from(prop_name)) {
            return true;
        }

        // somePromise['catch']()
        if prop_name == "catch"
            && matches!(call_expr.callee.get_inner_expression(), Expression::StringLiteral(_))
        {
            return true;
        }

        let Expression::CallExpression(object_call_expr) = member_expr.object() else {
            return false;
        };

        // cy.get().then(a, b)
        is_cypress_call(object_call_expr)
    }
}

fn is_part_of_promise(call_expr: &CallExpression) -> bool {
    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return false;
    };

    let Expression::CallExpression(object_call_expr) = member_expr.object() else {
        return false;
    };

    is_promise(object_call_expr).is_some()
}

fn is_cypress_call(call_expr: &CallExpression) -> bool {
    if is_method_call(call_expr, Some(&["cy"]), None, None, None) {
        return true;
    }

    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return false;
    };

    let Expression::CallExpression(object_call_expr) = member_expr.object() else {
        return false;
    };

    is_cypress_call(object_call_expr)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("frank().then(go).catch(doIt)", None),
        ("frank().then(go).then().then().then().catch(doIt)", None),
        ("frank().then(go).then().catch(function() { /* why bother */ })", None),
        ("frank.then(go).then(to).catch(jail)", None),
        ("Promise.resolve(frank).catch(jail)", None),
        (r#"Promise.resolve(frank)["catch"](jail)"#, None),
        ("frank.then(to).finally(fn).catch(jail)", None),
        (
            r#"postJSON("/smajobber/api/reportJob.json")
				.then(()=>this.setState())
				.catch(()=>this.setState())"#,
            None,
        ),
        ("function a() { return frank().then(go) }", None),
        ("function a() { return frank().then(go).then().then().then() }", None),
        ("function a() { return frank().then(go).then()}", None),
        ("function a() { return frank.then(go).then(to) }", None),
        ("frank().then(go).then(null, doIt)", Some(serde_json::json!([{ "allowThen": true }]))),
        (
            "frank().then(go).then().then().then().then(null, doIt)",
            Some(serde_json::json!([{ "allowThen": true }])),
        ),
        (
            "frank().then(go).then().then(null, function() { /* why bother */ })",
            Some(serde_json::json!([{ "allowThen": true }])),
        ),
        (
            "frank.then(go).then(to).then(null, jail)",
            Some(serde_json::json!([{ "allowThen": true }])),
        ),
        ("frank().then(a, b)", Some(serde_json::json!([{ "allowThen": true }]))),
        ("frank().then(a).then(b).then(null, c)", Some(serde_json::json!([{ "allowThen": true }]))),
        ("frank().then(a).then(b).then(c, d)", Some(serde_json::json!([{ "allowThen": true }]))),
        (
            "frank().then(a).then(b).then().then().then(null, doIt)",
            Some(serde_json::json!([{ "allowThen": true }])),
        ),
        (
            "frank().then(a).then(b).then(null, function() { /* why bother */ })",
            Some(serde_json::json!([{ "allowThen": true }])),
        ),
        ("frank().then(go).then(zam, doIt)", Some(serde_json::json!([{ "allowThen": true }]))),
        (
            "frank().then(go).then().then().then().then(wham, doIt)",
            Some(serde_json::json!([{ "allowThen": true }])),
        ),
        (
            "frank().then(go).then().then(function() {}, function() { /* why bother */ })",
            Some(serde_json::json!([{ "allowThen": true }])),
        ),
        (
            "frank.then(go).then(to).then(pewPew, jail)",
            Some(serde_json::json!([{ "allowThen": true }])),
        ),
        (
            "frank().then(go).catch(doIt).finally(fn)",
            Some(serde_json::json!([{ "allowFinally": true }])),
        ),
        (
            "frank().then(go).then().then().then().catch(doIt).finally(fn)",
            Some(serde_json::json!([{ "allowFinally": true }])),
        ),
        (
            "frank().then(go).then().catch(function() { /* why bother */ }).finally(fn)",
            Some(serde_json::json!([{ "allowFinally": true }])),
        ),
        ("frank().then(goo).done()", Some(serde_json::json!([{ "terminationMethod": "done" }]))),
        (
            "frank().then(go).catch()",
            Some(serde_json::json!([{ "terminationMethod": ["catch", "done"] }])),
        ),
        (
            "frank().then(go).done()",
            Some(serde_json::json!([{ "terminationMethod": ["catch", "done"] }])),
        ),
        (
            "frank().then(go).finally()",
            Some(serde_json::json!([{ "terminationMethod": ["catch", "finally"] }])),
        ),
        ("nonPromiseExpressionStatement();", None),
        ("frank().then(go)['catch']", None),
        ("await foo().then(bar)", None),
        // Cypress
        ("cy.get('.myClass').then(go)", None),
        ("cy.get('button').click().then()", None),
    ];

    let fail = vec![
        ("function callPromise(promise, cb) { promise.then(cb) }", None),
        (r#"fetch("http://www.yahoo.com").then(console.log.bind(console))"#, None),
        (r#"a.then(function() { return "x"; }).then(function(y) { throw y; })"#, None),
        ("Promise.resolve(frank)", None),
        ("Promise.all([])", None),
        ("Promise.allSettled([])", None),
        ("Promise.any([])", None),
        ("Promise.race([])", None),
        ("frank().then(to).catch(fn).then(foo)", None),
        ("frank().finally(fn)", None),
        ("frank().then(to).finally(fn)", None),
        ("frank().then(go).catch(doIt).finally(fn)", None),
        ("frank().then(go).then().then().then().catch(doIt).finally(fn)", None),
        ("frank().then(go).then().catch(function() { /* why bother */ }).finally(fn)", None),
        ("function a() { frank().then(go) }", None),
        ("function a() { frank().then(go).then().then().then() }", None),
        ("function a() { frank().then(go).then()}", None),
        ("function a() { frank.then(go).then(to) }", None),
        (
            "frank().then(go).catch(doIt).finally(fn).then(foo)",
            Some(serde_json::json!([{ "allowFinally": true }])),
        ),
        (
            "frank().then(go).catch(doIt).finally(fn).foobar(foo)",
            Some(serde_json::json!([{ "allowFinally": true }])),
        ),
        ("frank().then(go)", Some(serde_json::json!([{ "terminationMethod": "done" }]))),
        ("frank().catch(go)", Some(serde_json::json!([{ "terminationMethod": "done" }]))),
        ("frank().catch(go).someOtherMethod()", None),
        ("frank()['catch'](go).someOtherMethod()", None),
    ];

    Tester::new(CatchOrReturn::NAME, CatchOrReturn::PLUGIN, pass, fail).test_and_snapshot();
}
