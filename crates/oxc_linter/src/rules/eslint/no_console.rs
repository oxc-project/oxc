use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-console): Unexpected console statement.")]
#[diagnostic(severity(warning))]
struct NoConsoleDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoConsole {
    /// A list of methods allowed to be used.
    ///
    /// ```javascript
    /// // allowed: ['info']
    /// console.log('foo'); // will error
    /// console.info('bar'); // will not error
    /// ```
    pub allow: Vec<String>,
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallows using the global console object.
    ///
    /// ### Why is this bad?
    /// In JavaScript that is designed to be executed in the browser,
    /// it’s considered a best practice to avoid using methods on console.
    /// Such messages are considered to be for debugging purposes and therefore
    /// not suitable to ship to the client.
    ///
    /// ### Example
    /// ```javascript
    /// console.log('here');
    /// ```
    NoConsole,
    restriction
);

impl Rule for NoConsole {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self {
            allow: value
                .get(0)
                .and_then(|v| v.get("allow"))
                .and_then(serde_json::Value::as_array)
                .map(|v| {
                    v.iter()
                        .filter_map(serde_json::Value::as_str)
                        .map(ToString::to_string)
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(call_expr) = node.kind() {
            if let Expression::MemberExpression(mem) = &call_expr.callee {
                if let Expression::Identifier(ident) = mem.object() {
                    if ctx.semantic().is_reference_to_global_variable(ident)
                        && ident.name == "console"
                        && !self
                            .allow
                            .iter()
                            .any(|s| mem.static_property_name().is_some_and(|f| f == s))
                    {
                        if let Some(mem) = mem.static_property_info() {
                            ctx.diagnostic(NoConsoleDiagnostic(mem.0));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("Console.info(foo)", None),
        ("console.info(foo)", Some(serde_json::json!([{ "allow": ["info"] }]))),
        ("console.warn(foo)", Some(serde_json::json!([{ "allow": ["warn"] }]))),
        ("console.error(foo)", Some(serde_json::json!([{ "allow": ["error"] }]))),
        ("console.log(foo)", Some(serde_json::json!([{ "allow": ["log"] }]))),
        ("console.info(foo)", Some(serde_json::json!([{ "allow": ["warn", "info"] }]))),
        ("console.warn(foo)", Some(serde_json::json!([{ "allow": ["error", "warn"] }]))),
        ("console.error(foo)", Some(serde_json::json!([{ "allow": ["log", "error"] }]))),
        ("console.log(foo)", Some(serde_json::json!([{ "allow": ["info", "log", "warn"] }]))),
        ("var console = require('myconsole'); console.log(foo)", None),
        ("import console from 'myconsole'; console.log(foo)", None),
    ];

    let fail = vec![
        ("console.log()", None),
        ("console.log(foo)", None),
        ("console.error(foo)", None),
        ("console.info(foo)", None),
        ("console.warn(foo)", None),
        ("console.log(foo)", Some(serde_json::json!([{ "allow": ["error"] }]))),
        ("console.error(foo)", Some(serde_json::json!([{ "allow": ["warn"] }]))),
        ("console.info(foo)", Some(serde_json::json!([{ "allow": ["log"] }]))),
        ("console.warn(foo)", Some(serde_json::json!([{ "allow": ["error"] }]))),
        ("console.log(foo)", Some(serde_json::json!([{ "allow": ["warn", "info"] }]))),
        ("console.error(foo)", Some(serde_json::json!([{ "allow": ["warn", "info", "log"] }]))),
        ("console.info(foo)", Some(serde_json::json!([{ "allow": ["warn", "error", "log"] }]))),
        ("console.warn(foo)", Some(serde_json::json!([{ "allow": ["info", "log"] }]))),
    ];

    Tester::new(NoConsole::NAME, pass, fail).test_and_snapshot();
}
