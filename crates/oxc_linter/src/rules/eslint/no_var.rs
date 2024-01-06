use oxc_ast::{ast::VariableDeclarationKind, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-var): Unexpected var, use let or const instead.")]
#[diagnostic(severity(warning), help("Replace var with let or const"))]
struct NoVarDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoVar;

// doc: https://github.com/eslint/eslint/blob/main/docs/src/rules/no-var.md
// code: https://github.com/eslint/eslint/blob/main/lib/rules/no-var.js
// test: https://github.com/eslint/eslint/blob/main/tests/lib/rules/no-var.js

declare_oxc_lint!(
    /// ### What it does
    /// ECMAScript 6 allows programmers to create variables with block scope instead of function scope using the `let` and `const` keywords.
    /// Block scope is common in many other programming languages and helps programmers avoid mistakes
    ///
    /// ### Why is this bad?
    /// Using `var` in an es6 environment triggers this error
    ///
    /// ### Example
    /// ```javascript
    /// // error
    /// var x = "y";
    /// var CONFIG = {};
    ///
    /// // success
    /// let x = "y";
    /// const CONFIG = {};
    /// ```
    NoVar,
    restriction
);

impl Rule for NoVar {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::VariableDeclaration(dec) = node.kind() {
            if dec.kind == VariableDeclarationKind::Var {
                ctx.diagnostic(NoVarDiagnostic(Span::new(dec.span.start, dec.span.start + 3)));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![("const JOE = 'schmoe';", None), ("let moo = 'car';", None)];

    let fail = vec![
        ("var foo = bar;", None),
        ("var foo = bar, toast = most;", None),
        ("var foo = bar; let toast = most;", None),
        ("for (var a of b) { console.log(a); }", None),
        ("for (var a in b) { console.log(a); }", None),
        ("for (let a of b) { var c = 1; console.log(c); }", None),
        ("for (var i = 0; i < list.length; ++i) { foo(i) }", None),
        ("for (var i = 0, i = 0; false;);", None),
        ("var i = 0; for (var i = 1; false;); console.log(i);", None),
        ("var a, b, c; var a;", None),
        ("var a; if (b) { var a; }", None),
        ("if (foo) { var a, b, c; } a;", None),
        ("for (var i = 0; i < 10; ++i) {} i;", None),
        ("for (var a in obj) {} a;", None),
        ("for (var a of list) {} a;", None),
        ("switch (a) { case 0: var b = 1 }", None),
        ("for (var a of b) { arr.push(() => a); }", None),
        ("for (let a of b) { var c; console.log(c); c = 'hello'; }", None),
        ("var a = a", None),
        ("var {a = a} = {}", None),
        ("var {a = b, b} = {}", None),
        ("var {a, b = a} = {}", None),
        ("var a = b, b = 1", None),
        ("var a = b; var b = 1", None),
        ("function foo() { a } var a = 1; foo()", None),
        ("if (foo) var bar = 1;", None),
        ("var foo = 1", None),
        ("{ var foo = 1 }", None),
        ("if (true) { var foo = 1 }", None),
        ("var foo = 1", None),
        ("declare var foo = 2;", None),
        ("function foo() { var let; }", None),
        ("function foo() { var { let } = {}; }", None),
        (
            "var fx = function (i = 0) { if (i < 5) { return fx(i + 1); } console.log(i); }; fx();",
            None,
        ),
        ("var foo = function () { foo() };", None),
        ("var foo = () => foo();", None),
        ("var foo = (function () { foo(); })();", None),
        ("var foo = bar(function () { foo(); });", None),
        ("var bar = foo, foo = function () { foo(); };", None),
        ("var bar = foo; var foo = function () { foo(); };", None),
        ("var { foo = foo } = function () { foo(); };", None),
        ("var { bar = foo, foo } = function () { foo(); };", None),
        ("var bar = function () { foo(); }; var foo = function() {};", None),
    ];

    Tester::new(NoVar::NAME, pass, fail).test_and_snapshot();
}
