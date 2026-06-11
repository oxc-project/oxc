use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;
use schemars::JsonSchema;
use serde::Deserialize;
use std::fmt::Write as _;

use crate::{
    AstNode,
    ast_util::is_method_call,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::is_promise,
};

fn format_termination_method(method: &CompactStr, as_call: bool) -> String {
    if as_call { format!("`{method}()`") } else { format!("`{method}`") }
}

fn join_termination_methods(methods: &[CompactStr], as_call: bool) -> String {
    match methods {
        [] => String::new(),
        [method] => format_termination_method(method, as_call),
        [first, second] => {
            format!(
                "{} or {}",
                format_termination_method(first, as_call),
                format_termination_method(second, as_call)
            )
        }
        [methods @ .., last] => {
            let mut message = methods
                .iter()
                .map(|method| format_termination_method(method, as_call))
                .collect::<Vec<_>>()
                .join(", ");
            let _ = write!(message, ", or {}", format_termination_method(last, as_call));
            message
        }
    }
}

fn catch_or_return_diagnostic(methods: &[CompactStr], span: Span) -> OxcDiagnostic {
    let expected_methods = match methods {
        [] => "`return`".to_string(),
        [method] => format!("`{method}` or `return`"),
        [first, second] => format!("`{first}`, `{second}`, or `return`"),
        [methods @ .., last] => {
            let mut message =
                methods.iter().map(|method| format!("`{method}`")).collect::<Vec<_>>().join(", ");
            let _ = write!(message, ", `{last}`, or `return`");
            message
        }
    };

    let diagnostic = OxcDiagnostic::warn(format!("Expected {expected_methods}.")).with_label(span);

    match methods {
        [] => diagnostic.with_help("Return the promise."),
        [_] => {
            let chain_methods = join_termination_methods(methods, true);
            diagnostic.with_help(format!("Return the promise or chain a {chain_methods}."))
        }
        [_, ..] => {
            let chain_methods = join_termination_methods(methods, true);
            diagnostic.with_help(format!("Return the promise or chain {chain_methods}."))
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct CatchOrReturn(Box<CatchOrReturnConfig>);

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct CatchOrReturnConfig {
    /// Whether to allow `finally()` as a termination method.
    allow_finally: bool,
    /// Whether to allow `then()` with two arguments as a termination method.
    allow_then: bool,
    /// Whether to allow `then(null, handler)` as a termination method.
    allow_then_strict: bool,
    /// List of allowed termination methods (e.g., `catch`, `done`).
    #[serde(
        default = "default_termination_method",
        deserialize_with = "deserialize_termination_method"
    )]
    termination_method: Vec<CompactStr>,
}

impl Default for CatchOrReturnConfig {
    fn default() -> Self {
        Self {
            allow_finally: false,
            allow_then: false,
            allow_then_strict: false,
            termination_method: default_termination_method(),
        }
    }
}

fn default_termination_method() -> Vec<CompactStr> {
    vec![CompactStr::new("catch")]
}

fn deserialize_termination_method<'de, D>(deserializer: D) -> Result<Vec<CompactStr>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TerminationMethod {
        Single(CompactStr),
        Multiple(Vec<CompactStr>),
    }

    Ok(match TerminationMethod::deserialize(deserializer)? {
        TerminationMethod::Single(method) => vec![method],
        TerminationMethod::Multiple(methods) => methods,
    })
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
    /// const arrowFunc = () => myPromise.then(doSomething)
    /// ```
    CatchOrReturn,
    promise,
    restriction,
    config = CatchOrReturnConfig,
    version = "0.9.2",
);

impl Rule for CatchOrReturn {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
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

        if is_arrow_function_expression_return(node, ctx) {
            return;
        }

        if self.is_allowed_promise_termination(call_expr) {
            return;
        }

        ctx.diagnostic(catch_or_return_diagnostic(&self.termination_method, call_expr.span));
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
        if prop_name == "then"
            && call_expr.arguments.len() == 2
            && (self.allow_then
                || (self.allow_then_strict
                    && matches!(call_expr.arguments.first(), Some(Argument::NullLiteral(_)))))
        {
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
        if self.termination_method.iter().any(|method| method == prop_name) {
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

fn is_arrow_function_expression_return(node: &AstNode, ctx: &LintContext) -> bool {
    let parent = ctx.nodes().parent_node(node.id());

    if !matches!(parent.kind(), AstKind::FunctionBody(_)) {
        return false;
    }

    if let AstKind::ArrowFunctionExpression(arrow_func) = ctx.nodes().parent_kind(parent.id()) {
        return arrow_func.expression;
    }

    false
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
            "frank().then(go).then(null, doIt)",
            Some(serde_json::json!([{ "allowThenStrict": true }])),
        ),
        (
            "frank().then(go).then().then().then().then(null, doIt)",
            Some(serde_json::json!([{ "allowThenStrict": true }])),
        ),
        (
            "frank().then(go).then().then(null, function() { /* why bother */ })",
            Some(serde_json::json!([{ "allowThenStrict": true }])),
        ),
        (
            "frank.then(go).then(to).then(null, jail)",
            Some(serde_json::json!([{ "allowThenStrict": true }])),
        ),
        (
            "frank().then(a).then(b).then(null, c)",
            Some(serde_json::json!([{ "allowThenStrict": true }])),
        ),
        (
            "frank().then(a).then(b).then().then().then(null, doIt)",
            Some(serde_json::json!([{ "allowThenStrict": true }])),
        ),
        (
            "frank().then(a).then(b).then(null, function() { /* why bother */ })",
            Some(serde_json::json!([{ "allowThenStrict": true }])),
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
        ("const a = () => Promise.resolve(null)", None),
        ("const b = () => Promise.resolve({ id: '' })", None),
        ("const obj = { method: () => Promise.resolve(null) }", None),
        ("const obj = { openLinkModalPrompt: () => Promise.resolve(null) }", None),
        ("const arr = [() => Promise.resolve(null)]", None),
        ("foo(() => Promise.resolve(null))", None),
        ("const a = () => { return Promise.resolve(null); }", None),
        ("function a() { const b = () => Promise.resolve(null); return b; }", None),
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
        ("frank().then(go)", Some(serde_json::json!([{ "terminationMethod": ["catch", "done"] }]))),
        ("frank().then(go)", Some(serde_json::json!([{ "terminationMethod": [] }]))),
        ("frank().catch(go).someOtherMethod()", None),
        ("frank()['catch'](go).someOtherMethod()", None),
        ("frank().then(a).then(b).then(null, c)", None),
        ("frank().then(a).then(b).then().then().then(null, doIt)", None),
        ("frank().then(a).then(b).then(null, function() { /* why bother */ })", None),
        ("frank().then(a, b)", None),
        ("frank().then(go).then(zam, doIt)", None),
        ("frank().then(a).then(b).then(c, d)", None),
        ("frank().then(go).then().then().then().then(wham, doIt)", None),
        ("frank().then(go).then().then(function() {}, function() { /* why bother */ })", None),
        ("frank.then(go).then(to).then(pewPew, jail)", None),
        ("frank().then(a, b)", Some(serde_json::json!([{ "allowThenStrict": true }]))),
        (
            "frank().then(go).then(zam, doIt)",
            Some(serde_json::json!([{ "allowThenStrict": true }])),
        ),
        (
            "frank().then(a).then(b).then(c, d)",
            Some(serde_json::json!([{ "allowThenStrict": true }])),
        ),
        (
            "frank().then(go).then().then().then().then(wham, doIt)",
            Some(serde_json::json!([{ "allowThenStrict": true }])),
        ),
        (
            "frank().then(go).then().then(function() {}, function() { /* why bother */ })",
            Some(serde_json::json!([{ "allowThenStrict": true }])),
        ),
        (
            "frank.then(go).then(to).then(pewPew, jail)",
            Some(serde_json::json!([{ "allowThenStrict": true }])),
        ),
        ("const a = () => { Promise.resolve(null); }", None),
        ("function a() { const b = () => { Promise.resolve(null); }; return b; }", None),
    ];

    Tester::new(CatchOrReturn::NAME, CatchOrReturn::PLUGIN, pass, fail)
        .change_rule_path_extension("mjs")
        .test_and_snapshot();
}
