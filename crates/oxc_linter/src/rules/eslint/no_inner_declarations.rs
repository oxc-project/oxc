use oxc_ast::{ast::ModuleDeclaration, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint(no-inner-declarations): Variable or `function` declarations are not allowed in nested blocks"
)]
#[diagnostic(severity(warning), help("Move {0} declaration to {1} root"))]
struct NoInnerDeclarationsDiagnostic(&'static str, &'static str, #[label] pub Span);

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
    ///
    /// ### Example
    /// ```javascript
    /// if (test) {
    ///     function doSomethingElse () { }
    /// }
    /// ```
    NoInnerDeclarations,
    correctness
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
            AstKind::VariableDeclaration(decl)
                if decl.kind.is_var() && self.config == NoInnerDeclarationsConfig::Both => {}
            AstKind::Function(func) if func.is_function_declaration() => {}
            _ => return,
        }

        let mut parent = ctx.nodes().parent_node(node.id());
        if let Some(parent_node) = parent {
            let parent_kind = parent_node.kind();
            if let AstKind::FunctionBody(_) = parent_kind {
                if let Some(grandparent) = ctx.nodes().parent_node(parent_node.id()) && grandparent.kind().is_function_like() {
                    return
                }
            }

            if matches!(
                parent_kind,
                AstKind::Program(_)
                    | AstKind::StaticBlock(_)
                    | AstKind::ModuleDeclaration(
                        ModuleDeclaration::ExportNamedDeclaration(_)
                            | ModuleDeclaration::ExportDefaultDeclaration(_)
                    )
                    | AstKind::ForStatementInit(_)
                    | AstKind::ForInStatement(_)
                    | AstKind::ForOfStatement(_)
            ) {
                return;
            }
        }

        let decl_type = match node.kind() {
            AstKind::VariableDeclaration(_) => "variable",
            AstKind::Function(_) => "function",
            _ => unreachable!(),
        };

        let mut body = "program";
        while let Some(parent_node) = parent {
            let parent_kind = parent_node.kind();
            match parent_kind {
                AstKind::StaticBlock(_) => {
                    body = "class static block body";
                    break;
                }
                AstKind::Function(_) => {
                    body = "function body";
                    break;
                }
                _ => parent = ctx.nodes().parent_node(parent_node.id()),
            }
        }

        ctx.diagnostic(NoInnerDeclarationsDiagnostic(decl_type, body, node.kind().span()));
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
    ];

    Tester::new(NoInnerDeclarations::NAME, pass, fail).test_and_snapshot();
}
