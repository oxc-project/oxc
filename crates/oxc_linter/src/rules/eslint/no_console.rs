use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::{DefaultRuleConfig, Rule},
};

fn no_console_diagnostic(span: Span, allow: &[CompactStr]) -> OxcDiagnostic {
    let only_msg = if allow.is_empty() {
        String::from("Delete this console statement.")
    } else {
        format!("Supported methods are: {}.", allow.join(", "))
    };

    OxcDiagnostic::warn("Unexpected console statement.").with_label(span).with_help(only_msg)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoConsole(Box<NoConsoleConfig>);

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoConsoleConfig {
    /// The `allow` option permits the given list of console methods to be used as exceptions to
    /// this rule.
    ///
    /// Say the option was configured as `{ "allow": ["info"] }` then the rule would behave as
    /// follows:
    ///
    /// Example of **incorrect** code for this option:
    /// ```javascript
    /// console.log('foo');
    /// ```
    ///
    /// Example of **correct** code for this option:
    /// ```javascript
    /// console.info('foo');
    /// ```
    pub allow: Vec<CompactStr>,
}

impl std::ops::Deref for NoConsole {
    type Target = NoConsoleConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the use of console.
    ///
    /// ### Why is this bad?
    ///
    /// In JavaScript that is designed to be executed in the browser, it’s considered a best
    /// practice to avoid using methods on console. Such messages are considered to be for
    /// debugging purposes and therefore not suitable to ship to the client. In general, calls
    /// using console should be stripped before being pushed to production.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// console.log("Log a debug level message.");
    /// console.warn("Log a warn level message.");
    /// console.error("Log an error level message.");
    /// console.log = foo();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///  ```javascript
    /// // custom console
    /// Console.log("Hello world!");
    /// ```
    NoConsole,
    eslint,
    restriction,
    conditional_suggestion,
    config = NoConsoleConfig,
);

impl Rule for NoConsole {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoConsole>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let object = match node.kind() {
            AstKind::StaticMemberExpression(member_expr) => &member_expr.object,
            AstKind::ComputedMemberExpression(member_expr) => &member_expr.object,
            _ => return,
        };

        let Expression::Identifier(ident) = object else {
            return;
        };

        if ident.name != "console"
            || !ctx.scoping().root_unresolved_references().contains_key("console")
        {
            return;
        }

        let (mem_span, prop_name) = match node.kind() {
            AstKind::StaticMemberExpression(member_expr) => member_expr.static_property_info(),
            AstKind::ComputedMemberExpression(member_expr) => {
                match member_expr.static_property_info() {
                    Some(info) => info,
                    None => return,
                }
            }
            _ => unreachable!(),
        };

        if self.allow.iter().any(|allowed_name| allowed_name == prop_name) {
            return;
        }

        let diagnostic_span = ident.span().merge(mem_span);

        ctx.diagnostic_with_suggestion(
            no_console_diagnostic(diagnostic_span, &self.allow),
            |fixer| {
                let parent = ctx.nodes().parent_node(node.id());
                if let AstKind::CallExpression(_) = parent.kind() {
                    return remove_console(fixer, ctx, parent);
                }
                fixer.noop()
            },
        );
    }
}

fn remove_console<'c, 'a: 'c>(
    fixer: RuleFixer<'c, 'a>,
    ctx: &'c LintContext<'a>,
    node: &AstNode<'a>,
) -> RuleFix {
    let mut node_to_delete = node;
    for parent in ctx.nodes().ancestors(node.id()) {
        match parent.kind() {
            AstKind::ParenthesizedExpression(_)
            | AstKind::ExpressionStatement(_)
            => node_to_delete = parent,
            AstKind::IfStatement(_)
            | AstKind::WhileStatement(_)
            | AstKind::ForStatement(_)
            | AstKind::ForInStatement(_)
            | AstKind::ForOfStatement(_)
            | AstKind::ArrowFunctionExpression(_) => {
                return fixer.replace(node_to_delete.span(), "{}")
            }
            // Arrow function AST nodes do not say whether they have brackets or
            // not, so we need to check manually.
            // e.g: const x = () => { console.log(foo) }
            // vs:  const x = () => console.log(foo)
            | AstKind::FunctionBody(body) if !fixer.source_range(body.span).starts_with('{') => {
                return fixer.replace(node_to_delete.span(), "{}")
            }
            // Marked as dangerous until we're sure this is safe
            AstKind::ConditionalExpression(_)
            // from: const x = (console.log("foo"), 5);
            // to:   const x = (undefined, 5);
            | AstKind::SequenceExpression(_)
            | AstKind::ObjectProperty(_)
            => {
                return fixer.replace(node_to_delete.span(), "undefined").dangerously()
            }
            _ => break,
        }
    }
    fixer.delete(node_to_delete)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("Console.info(foo)", None, None),
        ("console.info(foo)", Some(serde_json::json!([{ "allow": ["info"] }])), None),
        (
            "console.info(foo)",
            Some(serde_json::json!([{ "allow": ["info"] }])),
            Some(serde_json::json!({ "env": { "browser": true }})),
        ),
        (
            "console.info(foo)",
            Some(serde_json::json!([{ "allow": ["info"] }])),
            Some(serde_json::json!({ "globals": { "console": "readonly" }})),
        ),
        ("console.warn(foo)", Some(serde_json::json!([{ "allow": ["warn"] }])), None),
        ("console.error(foo)", Some(serde_json::json!([{ "allow": ["error"] }])), None),
        ("console.log(foo)", Some(serde_json::json!([{ "allow": ["log"] }])), None),
        ("console.info(foo)", Some(serde_json::json!([{ "allow": ["warn", "info"] }])), None),
        ("console.warn(foo)", Some(serde_json::json!([{ "allow": ["error", "warn"] }])), None),
        ("console.error(foo)", Some(serde_json::json!([{ "allow": ["log", "error"] }])), None),
        ("console.log(foo)", Some(serde_json::json!([{ "allow": ["info", "log", "warn"] }])), None),
        ("var console = require('myconsole'); console.log(foo)", None, None),
        ("import console from 'myconsole'; console.log(foo)", None, None),
    ];

    let fail = vec![
        ("console.log()", None, None),
        ("foo(console.log)", None, None),
        ("console.log(foo)", None, None),
        ("console.error(foo)", None, None),
        ("console.info(foo)", None, None),
        ("console.warn(foo)", None, None),
        (
            "console
               .warn(foo)",
            None,
            None,
        ),
        (
            "console
               /* comment */
               .warn(foo);",
            None,
            None,
        ),
        ("console.warn(foo)", Some(serde_json::json!([{ "allow": [] }])), None),
        ("console['log'](foo)", None, None),
        ("console[`log`](foo)", None, None),
        ("console['lo\\x67'](foo)", Some(serde_json::json!([{ "allow": ["lo\\x67"] }])), None),
        ("console[`lo\\x67`](foo)", Some(serde_json::json!([{ "allow": ["lo\\x67"] }])), None),
        ("console.log()", None, Some(serde_json::json!({ "env": { "browser": true }}))),
        ("console.log()", None, Some(serde_json::json!({ "globals": { "console": "off" }}))),
        ("console.log(foo)", Some(serde_json::json!([{ "allow": ["error"] }])), None),
        ("console.error(foo)", Some(serde_json::json!([{ "allow": ["warn"] }])), None),
        ("console.info(foo)", Some(serde_json::json!([{ "allow": ["log"] }])), None),
        ("console.warn(foo)", Some(serde_json::json!([{ "allow": ["error"] }])), None),
        ("console.log(foo)", Some(serde_json::json!([{ "allow": ["warn", "info"] }])), None),
        (
            "console.error(foo)",
            Some(serde_json::json!([{ "allow": ["warn", "info", "log"] }])),
            None,
        ),
        (
            "console.info(foo)",
            Some(serde_json::json!([{ "allow": ["warn", "error", "log"] }])),
            None,
        ),
        ("console.warn(foo)", Some(serde_json::json!([{ "allow": ["info", "log"] }])), None),
    ];

    let fix = vec![
        ("function foo() { console.log(bar); }", "function foo() {  }", None),
        ("function foo() { console.log(bar) }", "function foo() {  }", None),
        ("const x = () => console.log(foo)", "const x = () => {}", None),
        ("const x = () => { console.log(foo) }", "const x = () => {  }", None),
        ("const x = () => { console.log(foo); }", "const x = () => {  }", None),
        ("const x = () => { ((console.log(foo))); }", "const x = () => {  }", None),
        ("const x = () => { console.log(foo); return 5 }", "const x = () => {  return 5 }", None),
        ("if (foo) { console.log(foo) }", "if (foo) {  }", None),
        ("foo ? console.log(foo) : 5", "foo ? undefined : 5", None),
        ("(console.log(foo), 5)", "(undefined, 5)", None),
        ("(5, console.log(foo))", "(5, undefined)", None),
        ("const x = { foo: console.log(bar) }", "const x = { foo: undefined }", None),
    ];

    Tester::new(NoConsole::NAME, NoConsole::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
