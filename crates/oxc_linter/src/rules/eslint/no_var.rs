use oxc_ast::{
    AstKind,
    ast::{BindingPattern, BindingPatternKind, VariableDeclarationKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_var_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected var, use let or const instead.")
        .with_help("Replace var with let or const")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoVar;

// doc: https://github.com/eslint/eslint/blob/v9.9.1/docs/src/rules/no-var.md
// code: https://github.com/eslint/eslint/blob/v9.9.1/lib/rules/no-var.js
// test: https://github.com/eslint/eslint/blob/v9.9.1/tests/lib/rules/no-var.js

declare_oxc_lint!(
    /// ### What it does
    ///
    /// ECMAScript 6 allows programmers to create variables with block scope
    /// instead of function scope using the `let` and `const` keywords.  Block
    /// scope is common in many other programming languages and helps
    /// programmers avoid mistakes.
    ///
    /// ### Why is this bad?
    ///
    /// Using `var` in an es6 environment triggers this error
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var x = "y";
    /// var CONFIG = {};
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// let x = "y";
    /// const CONFIG = {};
    /// ```
    NoVar,
    eslint,
    restriction,
    conditional_fix
);

impl Rule for NoVar {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::VariableDeclaration(dec) = node.kind()
            && dec.kind == VariableDeclarationKind::Var
        {
            let is_written_to = dec.declarations.iter().any(|v| is_written_to(&v.id, ctx));
            let span = Span::sized(dec.span.start, 3);
            ctx.diagnostic_with_fix(no_var_diagnostic(span), |fixer| {
                let parent_span = ctx.nodes().parent_kind(node.id()).span();
                if dec.declarations.iter().any(|decl| {
                    decl.id.get_binding_identifiers().iter().any(|ident| {
                        ctx.symbol_references(ident.symbol_id()).any(|id| {
                            !parent_span
                                .contains_inclusive(ctx.nodes().get_node(id.node_id()).span())
                        })
                    })
                }) {
                    return fixer.noop();
                }

                fixer.replace(span, if is_written_to { "let" } else { "const" })
            });
        }
    }
}

fn is_written_to(binding_pat: &BindingPattern, ctx: &LintContext) -> bool {
    match &binding_pat.kind {
        BindingPatternKind::BindingIdentifier(binding_ident) => ctx
            .semantic()
            .symbol_references(binding_ident.symbol_id())
            .any(oxc_semantic::Reference::is_write),
        BindingPatternKind::ObjectPattern(object_pat) => {
            if object_pat.properties.iter().any(|prop| is_written_to(&prop.value, ctx)) {
                return true;
            }

            if let Some(rest) = &object_pat.rest {
                is_written_to(&rest.argument, ctx)
            } else {
                false
            }
        }
        BindingPatternKind::AssignmentPattern(_) => true,
        BindingPatternKind::ArrayPattern(array_pat) => array_pat
            .elements
            .iter()
            .any(|elem| if let Some(elem) = elem { is_written_to(elem, ctx) } else { false }),
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

    let fix = vec![
        ("var foo", "const foo"),
        ("var foo; foo += 1", "let foo; foo += 1"),
        ("var foo,bar; bar = 'que'", "let foo,bar; bar = 'que'"),
        ("var { a } = {}; a = fn()", "let { a } = {}; a = fn()"),
        ("var { a } = {}; let b = a", "const { a } = {}; let b = a"),
        // TODO: implement a correct fixer for this case.
        // we need to add a `let a;` to the parent of both scopes
        // then change `var a = undefined` into `a = undefined`
        (
            "function play(index: number) { if (index > 1) { var a = undefined } else { var a = undefined } console.log(a) }",
            "function play(index: number) { if (index > 1) { var a = undefined } else { var a = undefined } console.log(a) }",
        ),
    ];

    Tester::new(NoVar::NAME, NoVar::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
