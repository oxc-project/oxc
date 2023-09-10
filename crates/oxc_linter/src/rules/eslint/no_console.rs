use oxc_ast::{AstKind, ast::MemberExpression};
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
    /// A list of allowed methods to be used.
    /// 
    /// ```
    /// // allowed: ["info"]
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
    correctness
);

impl Rule for NoConsole {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self {
             allow: value.get(0)
             .and_then(|v| v.get("allow"))
             .and_then(serde_json::Value::as_array)
             .map(|v| {
                v.iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToString::to_string)
                .collect()
             })
             .unwrap_or_default()
            }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let module_record = ctx.semantic().module_record();
        
        for import_entry in &module_record.import_entries {
            if import_entry.local_name.name().as_str() == "console" {
                return;
            }
        }

        if let AstKind::MemberExpression(MemberExpression::StaticMemberExpression(expr)) = node.kind() {
            if expr.object.without_parenthesized().get_identifier_reference().is_some_and(|x| x.name == "console") {

                if !allowed_method(&self.allow,expr.property.name.as_str()) {
                    ctx.diagnostic(NoConsoleDiagnostic(expr.property.span));
                }

            }
        }
    }
}

fn allowed_method(allow: &[String], method: &str) -> bool {
    allow.iter().any(|s| s == method)
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
    ];

    let fail = vec![
        ("console.log(foo)", None),
        ("console.error(foo)", None),
        ("console.log = foo()", None),
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
