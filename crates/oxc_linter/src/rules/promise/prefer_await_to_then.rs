use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

fn prefer_wait_to_then_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer await to then()/catch()/finally()").with_label(span)
}

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_promise};

#[derive(Debug, Default, Clone)]
pub struct PreferAwaitToThen(PreferAwaitToThenConfig);

impl std::ops::Deref for PreferAwaitToThen {
    type Target = PreferAwaitToThenConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone)]
pub struct PreferAwaitToThenConfig {
    strict: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `await` to `then()`/`catch()`/`finally()` for reading Promise values
    ///
    /// ### Why is this bad?
    ///
    /// Async/await syntax can be seen as more readable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function foo() { hey.then(x => {}) }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// async function hi() { await thing() }
    /// ```
    ///
    /// ### Example with strict mode
    /// Examples of **incorrect** code with `{ strict: true }`:
    /// ```javascript
    /// async function hi() {
    ///   await thing().then(x => {})
    /// }
    /// ```
    PreferAwaitToThen,
    promise,
    style,
);

fn is_inside_yield_or_await(node: &AstNode) -> bool {
    matches!(node.kind(), AstKind::YieldExpression(_) | AstKind::AwaitExpression(_))
}

impl Rule for PreferAwaitToThen {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0);
        let strict = config.and_then(|v| v.get("strict")).and_then(Value::as_bool).unwrap_or(false);

        Self(PreferAwaitToThenConfig { strict })
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if is_promise(call_expr).is_none_or(|v| v == "withResolvers") {
            return;
        }

        if matches!(ctx.nodes().parent_kind(node.id()), AstKind::ReturnStatement(_)) {
            return;
        }

        if !self.strict {
            // Already inside a yield or await
            if ctx.nodes().ancestors(node.id()).any(is_inside_yield_or_await) {
                return;
            }
        }

        let span = call_expr
            .callee
            .as_member_expression()
            .and_then(oxc_ast::ast::MemberExpression::static_property_info)
            .map_or(call_expr.span, |(span, _)| span);
        ctx.diagnostic(prefer_wait_to_then_diagnostic(span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("async function hi() { await thing() }", None),
        ("async function hi() { await thing().then() }", None),
        ("async function hi() { await thing().catch() }", None),
        ("a = async () => (await something())", None),
        (
            "a = async () => {
                try { await something() } catch (error) { somethingElse() }
            }",
            None,
        ),
        // <https://github.com/tc39/proposal-top-level-await>
        // Top level await is allowed now, so comment this out
        // ("something().then(async () => await somethingElse())", None),
        ("function foo() { hey.somethingElse(x => {}) }", None),
        (
            "const isThenable = (obj) => {
                return obj && typeof obj.then === 'function';
            };",
            None,
        ),
        (
            "function isThenable(obj) {
                return obj && typeof obj.then === 'function';
            }",
            None,
        ),
        (
            "async function hi() { await thing().then() }",
            Some(serde_json::json!([{ "strict": false }])),
        ),
        ("const { promise, resolve } = Promise.withResolvers()", None),
        ("function x () { return Promise.all() } ", None),
        ("function foo() { return hey.then(x => x) }", None),
        ("async function foo() { return thing().then(x => x) }", None),
    ];

    let fail = vec![
        ("function foo() { hey.then(x => {}) }", None),
        ("function foo() { hey.then(function() { }).then() }", None),
        ("function foo() { hey.then(function() { }).then(x).catch() }", None),
        ("async function a() { hey.then(function() { }).then(function() { }) }", None),
        ("function foo() { hey.catch(x => {}) }", None),
        ("function foo() { hey.finally(x => {}) }", None),
        ("something().then(async () => await somethingElse())", None),
        (
            "async function foo() { await thing().then() }",
            Some(serde_json::json!([{ "strict": true }])),
        ),
        ("async function foo() { thing().then() }", Some(serde_json::json!([{ "strict": false }]))),
        (
            "async function hi() { await thing().then(x => {}) }",
            Some(serde_json::json!([{ "strict": true }])),
        ),
    ];

    Tester::new(PreferAwaitToThen::NAME, PreferAwaitToThen::PLUGIN, pass, fail).test_and_snapshot();
}
