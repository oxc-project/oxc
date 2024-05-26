use oxc_ast::{ast::ModifierKind, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use rustc_hash::FxHashMap;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_restricted_syntax(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(x0).with_labels([span1.into()])
}

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedSyntax {
    selectors: Box<FxHashMap<String, String>>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows specified (that is, user-defined) syntax.
    ///
    /// ### Why is this bad?
    ///
    /// JavaScript has a lot of language features, and not everyone likes all of them. As a result,
    /// some projects choose to disallow the use of certain language features altogether.
    ///
    ///
    /// ### Example
    ///
    /// With options
    ///
    /// ```json
    /// {
    ///    "rules": {
    ///        "no-restricted-syntax": ["error", "AwaitExpression"]
    ///    }
    /// }
    /// ```
    ///
    /// The following patterns are considered problems:
    ///
    /// ```javascript
    /// await Foo() // error
    /// ```
    ///
    /// ### Cautions
    ///
    /// For now, this rule only supports the following selectors:
    ///
    /// - `ObjectPattern > RestElement`
    /// - `ObjectExpression > SpreadElement`
    /// - `AwaitExpression`
    /// - `TSEnumDeclaration[const=true]`
    ///
    NoRestrictedSyntax,
    restriction,
);

impl Rule for NoRestrictedSyntax {
    fn from_configuration(value: serde_json::Value) -> Self {
        let selector = match value {
            Value::Array(arr) => arr.iter().fold(FxHashMap::default(), |mut acc, v| match v {
                // "no-restricted-syntax": ["error", "FunctionExpression"]
                Value::String(name) => {
                    acc.insert(name.to_string(), String::new());
                    acc
                }
                // "no-restricted-syntax": ["error", { "selector": "FunctionExpression", "message": "custom error message." }]
                Value::Object(obj) => {
                    let name = obj.get("selector").and_then(Value::as_str).unwrap_or_default();
                    let message = obj.get("message").and_then(Value::as_str).unwrap_or_default();
                    acc.insert(name.to_string(), message.to_string());
                    acc
                }
                _ => acc,
            }),
            _ => FxHashMap::default(),
        };

        Self { selectors: Box::new(selector) }
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::AwaitExpression(await_expr) => {
                if let Some(message) = self.get_message("AwaitExpression") {
                    ctx.diagnostic(no_restricted_syntax(&message, await_expr.span));
                }
            }
            // "ObjectExpression > SpreadElement".
            AstKind::SpreadElement(spread_element) => {
                let Some(message) = self.get_message("ObjectExpression > SpreadElement") else {
                    return;
                };

                if let Some(AstKind::ObjectExpression(_)) = ctx.nodes().parent_kind(node.id()) {
                    ctx.diagnostic(no_restricted_syntax(&message, spread_element.span));
                };
            }
            // "ObjectPattern > RestElement".
            AstKind::BindingRestElement(rest_element) => {
                let Some(message) = self.get_message("ObjectPattern > RestElement") else {
                    return;
                };

                if let Some(AstKind::ObjectPattern(_)) = ctx.nodes().parent_kind(node.id()) {
                    ctx.diagnostic(no_restricted_syntax(&message, rest_element.span));
                };
            }
            // "TSEnumDeclaration[const=true]".
            AstKind::TSEnumDeclaration(enum_decl) => {
                let Some(message) = self.get_message("TSEnumDeclaration[const=true]") else {
                    return;
                };
                if enum_decl.modifiers.contains(ModifierKind::Const) {
                    ctx.diagnostic(no_restricted_syntax(
                        &message,
                        Span::new(enum_decl.span.start, enum_decl.span.start + 5), // 5 is the length of "const".
                    ));
                }
            }
            _ => {}
        }
    }
}

impl NoRestrictedSyntax {
    /// Get the message for the given selector, return `None` if not exists.
    fn get_message(&self, selector: &str) -> Option<CompactStr> {
        let default_msg = CompactStr::from(format!("Using '{selector}' is not allowed."));

        self.selectors.get(selector).map(|msg| {
            if msg.is_empty() {
                default_msg
            } else {
                CompactStr::from(msg.as_str())
            }
        })
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("doSomething();", None),
        ("var foo = 42;", Some(serde_json::json!(["ConditionalExpression"]))),
        ("foo += 42;", Some(serde_json::json!(["VariableDeclaration", "FunctionExpression"]))),
        ("foo;", Some(serde_json::json!(["Identifier[name=\"bar\"]"]))),
        ("() => 5", Some(serde_json::json!(["ArrowFunctionExpression > BlockStatement"]))), // { "ecmaVersion": 6 },
        ("({ foo: 1, bar: 2 })", Some(serde_json::json!(["Property > Literal.key"]))),
        ("A: for (;;) break;", Some(serde_json::json!(["BreakStatement[label]"]))),
        (
            "function foo(bar, baz) {}",
            Some(serde_json::json!(["FunctionDeclaration[params.length>2]"])),
        ),
        ("var foo = 42;", Some(serde_json::json!([{ "selector": "ConditionalExpression" }]))),
        (
            "({ foo: 1, bar: 2 })",
            Some(serde_json::json!([{ "selector": "Property > Literal.key" }])),
        ),
        (
            "({ foo: 1, bar: 2 })",
            Some(
                serde_json::json!([{ "selector": "FunctionDeclaration[params.length>2]", "message": "custom error message." }]),
            ),
        ),
        ("console.log(/a/);", Some(serde_json::json!(["Literal[regex.flags=/./]"]))),
        // TSEnumDeclaration
        ("const enum Foo { A, B, C }", None),
        (" enum Foo { A, B, C }", Some(serde_json::json!(["TSEnumDeclaration[const=true]"]))),
        // AwaitExpression
        ("async function foo() { await bar(); }", None),
        // ObjectExpression > SpreadElement
        ("const foo = { a, b, ...c }", None),
        // ObjectPattern > RestElement
        ("const { a, b, ...c } = foo", None),
    ];

    let fail = vec![
        // ("var foo = 41;", Some(serde_json::json!(["VariableDeclaration"]))),
        // (";function lol(a) { return 42; }", Some(serde_json::json!(["EmptyStatement"]))),
        // (
        //     "try { voila(); } catch (e) { oops(); }",
        //     Some(serde_json::json!(["TryStatement", "CallExpression", "CatchClause"])),
        // ),
        // ("bar;", Some(serde_json::json!(["Identifier[name=\"bar\"]"]))),
        // ("bar;", Some(serde_json::json!(["Identifier", "Identifier[name=\"bar\"]"]))),
        // ("() => {}", Some(serde_json::json!(["ArrowFunctionExpression > BlockStatement"]))), // { "ecmaVersion": 6 },
        // ("({ foo: 1, 'bar': 2 })", Some(serde_json::json!(["Property > Literal.key"]))),
        // ("A: for (;;) break A;", Some(serde_json::json!(["BreakStatement[label]"]))),
        // (
        //     "function foo(bar, baz, qux) {}",
        //     Some(serde_json::json!(["FunctionDeclaration[params.length>2]"])),
        // ),
        // ("var foo = 41;", Some(serde_json::json!([{ "selector": "VariableDeclaration" }]))),
        // (
        //     "function foo(bar, baz, qux) {}",
        //     Some(serde_json::json!([{ "selector": "FunctionDeclaration[params.length>2]" }])),
        // ),
        // (
        //     "function foo(bar, baz, qux) {}",
        //     Some(
        //         serde_json::json!([{ "selector": "FunctionDeclaration[params.length>2]", "message": "custom error message." }]),
        //     ),
        // ),
        // (
        //     "function foo(bar, baz, qux) {}",
        //     Some(
        //         serde_json::json!([{ "selector": "FunctionDeclaration[params.length>2]", "message": "custom message with {{selector}}" }]),
        //     ),
        // ),
        // ("console.log(/a/i);", Some(serde_json::json!(["Literal[regex.flags=/./]"]))),
        // ("var foo = foo?.bar?.();", Some(serde_json::json!(["ChainExpression"]))), // { "ecmaVersion": 2020 },
        // ("var foo = foo?.bar?.();", Some(serde_json::json!(["[optional=true]"]))), // { "ecmaVersion": 2020 },
        // ("a?.b", Some(serde_json::json!([":nth-child(1)"]))), // { "ecmaVersion": 2020 },
        // ("const foo = [<div/>, <div/>]", Some(serde_json::json!(["* ~ *"]))), // { "ecmaVersion": 2020, "parserOptions": { "ecmaFeatures": { "jsx": true } } }
        // TSEnumDeclaration[const=true]
        ("const enum Foo { A, B, C }", Some(serde_json::json!(["TSEnumDeclaration[const=true]"]))),
        ("const enum Foo { A, B, C }", Some(serde_json::json!(["TSEnumDeclaration[const=true]"]))),
        (
            "const enum Foo { A, B, C }",
            Some(serde_json::json!([{
                "selector": "TSEnumDeclaration[const=true]",
                "message": "Please use non-const enums. This project automatically inlines enums"
            }])),
        ),
        // AwaitExpression
        ("async function foo() { await bar(); }", Some(serde_json::json!(["AwaitExpression"]))),
        // ObjectExpression > SpreadElement
        ("const foo = { ...bar }", Some(serde_json::json!(["ObjectExpression > SpreadElement"]))),
        // ObjectPattern > RestElement
        ("const { ...bar } = foo", Some(serde_json::json!(["ObjectPattern > RestElement"]))),
    ];

    Tester::new(NoRestrictedSyntax::NAME, pass, fail).test_and_snapshot();
}
