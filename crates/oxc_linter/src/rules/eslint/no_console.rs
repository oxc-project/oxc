use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{AstKind, MemberExpressionKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use oxc_syntax::operator::BinaryOperator;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::{DefaultRuleConfig, Rule},
    utils::pad_fix_with_token_boundary,
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
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
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
    version = "0.0.13",
    short_description = "Disallow the use of console.",
);

impl Rule for NoConsole {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(member_expr) = node.kind().as_member_expression_kind() else {
            return;
        };

        let Expression::Identifier(ident) = member_expr.object() else {
            return;
        };

        // Not `ctx.is_reference_to_global_variable`: `globals: { console: "off" }` still reports.
        if ident.name != "console" || !ident.is_global_reference(ctx.scoping()) {
            return;
        }

        let (mem_span, prop_name) = match member_expr {
            MemberExpressionKind::Static(member_expr) => member_expr.static_property_info(),
            MemberExpressionKind::Computed(member_expr) => {
                match member_expr.static_property_info() {
                    Some(info) => info,
                    None => return,
                }
            }
            MemberExpressionKind::PrivateField(_) => return,
        };

        if self.allow.iter().any(|allowed_name| allowed_name.as_str() == prop_name) {
            return;
        }

        let diagnostic_span = ident.span().merge(mem_span);

        ctx.diagnostic_with_suggestion(
            no_console_diagnostic(diagnostic_span, &self.allow),
            |fixer| {
                let parent = ctx.nodes().parent_node(node.id());
                if let AstKind::CallExpression(call) = parent.kind()
                    && call.callee.span() == node.span()
                {
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
    let mut expression = node;
    let mut parent = ctx.nodes().parent_node(node.id());
    while matches!(parent.kind(), AstKind::ParenthesizedExpression(_) | AstKind::ChainExpression(_))
    {
        expression = parent;
        parent = ctx.nodes().parent_node(parent.id());
    }

    match parent.kind() {
        AstKind::ExpressionStatement(_) => {}
        AstKind::ArrowFunctionExpression(_) => return fixer.replace(expression.span(), "{}"),
        _ => {
            let span = expression.span();
            let mut replacement =
                if needs_parentheses(expression, parent) { "(void 0)" } else { "void 0" }
                    .to_string();
            pad_fix_with_token_boundary(ctx.source_text(), span, &mut replacement);
            return fixer.replace(span, replacement).dangerously();
        }
    }

    match ctx.nodes().parent_kind(parent.id()) {
        AstKind::IfStatement(_)
        | AstKind::WhileStatement(_)
        | AstKind::DoWhileStatement(_)
        | AstKind::ForStatement(_)
        | AstKind::ForInStatement(_)
        | AstKind::ForOfStatement(_)
        | AstKind::LabeledStatement(_)
        | AstKind::WithStatement(_) => fixer.replace(parent.span(), "{}"),
        _ => fixer.delete(parent),
    }
}

/// Whether `void 0` must be parenthesized to replace `expression` as a child of `parent`,
/// e.g. `console.log(foo).bar` -> `(void 0).bar`.
fn needs_parentheses(expression: &AstNode, parent: &AstNode) -> bool {
    let span = expression.span();
    match parent.kind() {
        AstKind::StaticMemberExpression(_)
        | AstKind::PrivateFieldExpression(_)
        | AstKind::TaggedTemplateExpression(_)
        | AstKind::Decorator(_) => true,
        AstKind::ComputedMemberExpression(member) => member.object.span() == span,
        AstKind::CallExpression(call) => call.callee.span() == span,
        AstKind::NewExpression(new) => new.callee.span() == span,
        AstKind::BinaryExpression(binary) => {
            binary.operator == BinaryOperator::Exponential && binary.left.span() == span
        }
        _ => false,
    }
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
        (
            "var c = console; function f(myLogger) { const console = myLogger; console.log(foo); }",
            None,
            None,
        ),
    ];

    let fail = vec![
        (r#"console["\uD800"]()"#, None, None),
        (r"console[`\uDC00`]()", None, None),
        (r#"console["a\uD800b"]()"#, Some(serde_json::json!([{ "allow": ["ab", "log"] }])), None),
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
        ("const x = () => (console.log(foo))", "const x = () => {}", None),
        ("const x = (a = console.log(foo)) => a", "const x = (a = void 0) => a", None),
        ("const x = () => { console.log(foo) }", "const x = () => {  }", None),
        ("const x = () => { console.log(foo); }", "const x = () => {  }", None),
        ("const x = () => { ((console.log(foo))); }", "const x = () => {  }", None),
        ("const x = () => { console.log(foo); return 5 }", "const x = () => {  return 5 }", None),
        ("if (foo) { console.log(foo) }", "if (foo) {  }", None),
        ("foo ? console.log(foo) : 5", "foo ? void 0 : 5", None),
        ("(console.log(foo), 5)", "(void 0, 5)", None),
        ("(5, console.log(foo))", "(5, void 0)", None),
        ("const x = { foo: console.log(bar) }", "const x = { foo: void 0 }", None),
        ("while (console.log(foo)) {}", "while (void 0) {}", None),
        ("if (console.log(foo)) bar();", "if (void 0) bar();", None),
        ("for (console.log(foo);;) {}", "for (void 0;;) {}", None),
        ("do {} while ((console.log(foo)))", "do {} while (void 0)", None),
        ("const result = console.log('debug');", "const result = void 0;", None),
        ("bar(console.log(foo))", "bar(void 0)", None),
        ("function f(){return(console.log())}", "function f(){return void 0}", None),
        ("(console.log())in obj", "void 0 in obj", None),
        ("console.log(foo).bar", "(void 0).bar", None),
        ("(console.log(foo)).bar", "(void 0).bar", None),
        ("console.log(foo)[bar]", "(void 0)[bar]", None),
        ("bar[console.log(foo)]", "bar[void 0]", None),
        ("console.log(foo)()", "(void 0)()", None),
        ("console.log(foo) ** 2", "(void 0) ** 2", None),
        ("2 ** console.log(foo)", "2 ** void 0", None),
        ("x = console.log?.(foo)", "x = void 0", None),
        ("console.log?.(foo);", "", None),
        ("do console.log(foo); while (bar)", "do {} while (bar)", None),
        ("foo(console.log)", "foo(console.log)", None),
        ("while (console.log) {}", "while (console.log) {}", None),
    ];

    Tester::new(NoConsole::NAME, NoConsole::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
