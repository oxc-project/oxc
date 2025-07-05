use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_inner_declarations_diagnostic(decl_type: &str, body: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Variable or `function` declarations are not allowed in nested blocks")
        .with_help(format!("Move {decl_type} declaration to {body} root"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoInnerDeclarations {
    config: NoInnerDeclarationsConfig,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
enum NoInnerDeclarationsConfig {
    /// Disallows function declarations in nested blocks
    #[default]
    Functions,
    /// Disallows function and var declarations in nested blocks
    Both,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow variable or function declarations in nested blocks
    ///
    /// ### Why is this bad?
    ///
    /// A variable declaration is permitted anywhere a statement can go, even nested deeply inside other blocks.
    /// This is often undesirable due to variable hoisting, and moving declarations to the root of the program or function body can increase clarity.
    /// Note that block bindings (let, const) are not hoisted and therefore they are not affected by this rule.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (test) {
    ///     function doSomethingElse () { }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// function doSomethingElse () { }
    /// if (test) {
    ///   // your code here
    /// }
    /// ```
    NoInnerDeclarations,
    eslint,
    pedantic
);

impl Rule for NoInnerDeclarations {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0).and_then(serde_json::Value::as_str).map_or_else(
            NoInnerDeclarationsConfig::default,
            |value| match value {
                "functions" => NoInnerDeclarationsConfig::Functions,
                _ => NoInnerDeclarationsConfig::Both,
            },
        );
        Self { config }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::VariableDeclaration(decl) => {
                if self.config == NoInnerDeclarationsConfig::Functions || !decl.kind.is_var() {
                    return;
                }
            }
            AstKind::Function(func) => {
                if !func.is_function_declaration() {
                    return;
                }
            }
            _ => return,
        }

        let parent_node = ctx.nodes().parent_node(node.id());
        if matches!(
            parent_node.kind(),
            AstKind::Program(_)
                | AstKind::FunctionBody(_)
                | AstKind::StaticBlock(_)
                | AstKind::ExportNamedDeclaration(_)
                | AstKind::ExportDefaultDeclaration(_)
        ) {
            return;
        }

        let mut body = "program";
        let mut parent = ctx.nodes().parent_node(parent_node.id());
        loop {
            match parent.kind() {
                AstKind::Program(_) => break,
                AstKind::StaticBlock(_) => {
                    body = "class static block body";
                    break;
                }
                AstKind::Function(_) => {
                    body = "function body";
                    break;
                }
                _ => parent = ctx.nodes().parent_node(parent.id()),
            }
        }

        let (decl_type, span) = match node.kind() {
            AstKind::VariableDeclaration(decl) => {
                let span = Span::sized(decl.span.start, 3); // 3 for "var".len()
                ("variable", span)
            }
            AstKind::Function(func) => {
                let span = Span::sized(func.span.start, 8); // 8 for "function".len()
                ("function", span)
            }
            _ => unreachable!(),
        };

        ctx.diagnostic(no_inner_declarations_diagnostic(decl_type, body, span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function doSomething() { }", None),
        ("function doSomething() { function somethingElse() { } }", None),
        ("(function() { function doSomething() { } }());", None),
        ("if (test) { var fn = function() { }; }", None),
        ("if (test) { var fn = function expr() { }; }", None),
        ("function decl() { var fn = function expr() { }; }", None),
        ("function decl(arg) { var fn; if (arg) { fn = function() { }; } }", None),
        ("var x = {doSomething() {function doSomethingElse() {}}}", None),
        ("function decl(arg) { var fn; if (arg) { fn = function expr() { }; } }", None),
        ("function decl(arg) { var fn; if (arg) { fn = function expr() { }; } }", None),
        ("if (test) { var foo; }", None),
        ("if (test) { let x = 1; }", Some(serde_json::json!(["both"]))),
        ("if (test) { const x = 1; }", Some(serde_json::json!(["both"]))),
        ("function doSomething() { while (test) { var foo; } }", None),
        ("var foo;", Some(serde_json::json!(["both"]))),
        ("var foo = 42;", Some(serde_json::json!(["both"]))),
        ("function doSomething() { var foo; }", Some(serde_json::json!(["both"]))),
        ("(function() { var foo; }());", Some(serde_json::json!(["both"]))),
        ("foo(() => { function bar() { } });", None),
        ("var fn = () => {var foo;}", Some(serde_json::json!(["both"]))),
        ("var x = {doSomething() {var foo;}}", Some(serde_json::json!(["both"]))),
        ("export var foo;", Some(serde_json::json!(["both"]))),
        ("export function bar() {}", Some(serde_json::json!(["both"]))),
        ("export default function baz() {}", Some(serde_json::json!(["both"]))),
        ("exports.foo = () => {}", Some(serde_json::json!(["both"]))),
        ("exports.foo = function(){}", Some(serde_json::json!(["both"]))),
        ("module.exports = function foo(){}", Some(serde_json::json!(["both"]))),
        ("class C { method() { function foo() {} } }", Some(serde_json::json!(["both"]))),
        ("class C { method() { var x; } }", Some(serde_json::json!(["both"]))),
        ("class C { static { function foo() {} } }", Some(serde_json::json!(["both"]))),
        ("class C { static { var x; } }", Some(serde_json::json!(["both"]))),
        ("for (const x in {}) { let y = 5; }", Some(serde_json::json!(["both"]))),
        ("for (const x of []) { let y = 5; }", Some(serde_json::json!(["both"]))),
        ("for (const x = 1; a < 10; a++) { let y = 5; }", Some(serde_json::json!(["both"]))),
    ];

    let fail = vec![
        ("if (test) { function doSomething() { } }", Some(serde_json::json!(["both"]))),
        ("if (foo) var a; ", Some(serde_json::json!(["both"]))),
        ("if (foo) /* some comments */ var a; ", Some(serde_json::json!(["both"]))),
        ("if (foo){ function f(){ if(bar){ var a; } } }", Some(serde_json::json!(["both"]))),
        ("if (foo) function f(){ if(bar) var a; } ", Some(serde_json::json!(["both"]))),
        ("if (foo) { var fn = function(){} } ", Some(serde_json::json!(["both"]))),
        ("if (foo)  function f(){} ", None),
        ("function bar() { if (foo) function f(){}; }", Some(serde_json::json!(["both"]))),
        ("function bar() { if (foo) var a; }", Some(serde_json::json!(["both"]))),
        ("if (foo){ var a; }", Some(serde_json::json!(["both"]))),
        ("function doSomething() { do { function somethingElse() { } } while (test); }", None),
        ("(function() { if (test) { function doSomething() { } } }());", None),
        ("while (test) { var foo; }", Some(serde_json::json!(["both"]))),
        (
            "function doSomething() { if (test) { var foo = 42; } }",
            Some(serde_json::json!(["both"])),
        ),
        ("(function() { if (test) { var foo; } }());", Some(serde_json::json!(["both"]))),
        (
            "const doSomething = () => { if (test) { var foo = 42; } }",
            Some(serde_json::json!(["both"])),
        ),
        ("class C { method() { if(test) { var foo; } } }", Some(serde_json::json!(["both"]))),
        (
            "class C { static { if (test) { function foo() {} } } }",
            Some(serde_json::json!(["both"])),
        ),
        ("class C { static { if (test) { var foo; } } }", Some(serde_json::json!(["both"]))),
        (
            "class C { static { if (test) { if (anotherTest) { var foo; } } } }",
            Some(serde_json::json!(["both"])),
        ),
        ("for (const x in {}) { var y = 5; }", Some(serde_json::json!(["both"]))),
        ("for (const x of []) { var y = 5; }", Some(serde_json::json!(["both"]))),
        ("for (const x = 1; a < 10; a++) { var y = 5; }", Some(serde_json::json!(["both"]))),
        ("for (const x in {}) var y = 5;", Some(serde_json::json!(["both"]))),
        ("for (const x of []) var y = 5;", Some(serde_json::json!(["both"]))),
        ("for (const x = 1; a < 10; a++) var y = 5;", Some(serde_json::json!(["both"]))),
        ("for (var x in {}) {}", Some(serde_json::json!(["both"]))),
        ("for (var x of []) {}", Some(serde_json::json!(["both"]))),
        ("for (var x = 1; a < 10; a++) {}", Some(serde_json::json!(["both"]))),
    ];

    Tester::new(NoInnerDeclarations::NAME, NoInnerDeclarations::PLUGIN, pass, fail)
        .test_and_snapshot();
}
