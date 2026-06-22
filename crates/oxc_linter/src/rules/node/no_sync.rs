use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;

use crate::{
    AstNode,
    ast_util::get_enclosing_function,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_sync_diagnostic(span: Span, property_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected sync method: '{property_name}'.")).with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct NoSyncConfig {
    /// Whether synchronous methods should be allowed at the top level of a file.
    allow_at_root_level: bool,
    /// Function names to ignore.
    ignores: FxHashSet<CompactStr>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoSync(Box<NoSyncConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows synchronous methods from being called in Node.js code.
    ///
    /// ### Why is this bad?
    ///
    /// In Node.js, most I/O is done through asynchronous methods. However, there are often
    /// synchronous versions of the asynchronous methods. For example, `fs.exists()` and
    /// `fs.existsSync()`. In some contexts, using synchronous operations is okay (if, as with
    /// ESLint, you are writing a command line utility). However, in other contexts the use of
    /// synchronous operations is considered a bad practice that should be avoided.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// fs.existsSync(somePath);
    ///
    /// function foo() {
    ///   var contents = fs.readFileSync(somePath).toString();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// obj.sync();
    ///
    /// async(function() {
    ///     // ...
    /// });
    /// ```
    NoSync,
    node,
    style,
    config = NoSyncConfig,
    version = "1.71.0",
    short_description = "Disallow synchronous methods.",
);

impl Rule for NoSync {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(property_name) = get_sync_property_name(&call_expr.callee) else {
            return;
        };

        if self.0.ignores.contains(property_name) {
            return;
        }

        if self.0.allow_at_root_level && get_enclosing_function(node, ctx).is_none() {
            return;
        }

        ctx.diagnostic(no_sync_diagnostic(call_expr.span, property_name));
    }
}

fn get_sync_property_name<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match expr.get_inner_expression() {
        Expression::Identifier(ident) if ident.name.as_str().ends_with("Sync") => {
            Some(ident.name.as_str())
        }
        Expression::StaticMemberExpression(member) => {
            if member.property.name.as_str().ends_with("Sync") {
                Some(member.property.name.as_str())
            } else {
                get_sync_property_name(&member.object)
            }
        }
        Expression::ComputedMemberExpression(member) => {
            if let Some(name) = member.static_property_name()
                && name.as_str().ends_with("Sync")
            {
                return Some(name.as_str());
            }
            get_sync_property_name(&member.object)
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var foo = fs.foo.foo();", None),
        ("fs.fooSync;", None),
        ("fooSync;", None),
        ("() => fooSync;", None),
        ("var foo = fs.fooSync;", Some(serde_json::json!([{ "allowAtRootLevel": true }]))),
        ("var foo = fooSync;", Some(serde_json::json!([{ "allowAtRootLevel": true }]))),
        ("if (true) {fs.fooSync();}", Some(serde_json::json!([{ "allowAtRootLevel": true }]))),
        ("if (true) {fooSync();}", Some(serde_json::json!([{ "allowAtRootLevel": true }]))),
        ("fooSync();", Some(serde_json::json!([{ "ignores": ["fooSync"] }]))),
    ];

    let fail = vec![
        ("var foo = fs.fooSync();", None),
        ("var foo = fs.fooSync.apply();", None),
        ("var foo = fooSync();", None),
        ("var foo = fooSync.apply();", None),
        ("var foo = fs.fooSync();", Some(serde_json::json!([{ "allowAtRootLevel": false }]))),
        ("if (true) {fs.fooSync();}", None),
        ("function someFunction() {fs.fooSync();}", None),
        (
            "function someFunction() {fs.fooSync();}",
            Some(serde_json::json!([{ "allowAtRootLevel": true }])),
        ),
        (
            "var a = function someFunction() {fs.fooSync();}",
            Some(serde_json::json!([{ "allowAtRootLevel": true }])),
        ),
        (
            "() => {fs.fooSync();}",
            Some(serde_json::json!([{ "allowAtRootLevel": true, "ignores": ["barSync"] }])),
        ),
    ];

    Tester::new(NoSync::NAME, NoSync::PLUGIN, pass, fail).test_and_snapshot();
}
