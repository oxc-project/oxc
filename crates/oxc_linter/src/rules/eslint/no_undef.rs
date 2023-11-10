use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};
use oxc_syntax::operator::UnaryOperator;

use crate::{context::LintContext, globals::BUILTINS, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-undef): Disallow the use of undeclared variables")]
#[diagnostic(severity(warning), help("'{0}' is not defined."))]
struct NoUndefDiagnostic(Atom, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoUndef {
    #[allow(dead_code)]
    type_of: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the use of undeclared variables.
    ///
    /// ### Why is this bad?
    ///
    /// It is most likely a potential ReferenceError caused by a misspelling of a variable or parameter name.
    ///
    /// ### Example
    /// ```javascript
    /// var foo = someFunction();
    /// var bar = a + 1;
    /// ```
    NoUndef,
    nursery // https://github.com/oxc-project/oxc/issues/732
);

impl Rule for NoUndef {
    fn from_configuration(value: serde_json::Value) -> Self {
        let type_of = value
            .get(0)
            .and_then(|config| config.get("typeof"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_default();
        Self { type_of }
    }
    fn run_once(&self, ctx: &LintContext) {
        let symbol_table = ctx.symbols();

        for reference_id_list in ctx.scopes().root_unresolved_references().values() {
            for &reference_id in reference_id_list {
                let reference = symbol_table.get_reference(reference_id);
                if BUILTINS.contains_key(reference.name().as_str()) {
                    return;
                }

                let node = ctx.nodes().get_node(reference.node_id());
                if !self.type_of && has_typeof_operator(node, ctx) {
                    return;
                }

                ctx.diagnostic(NoUndefDiagnostic(reference.name().clone(), reference.span()));
            }
        }
    }
}

fn has_typeof_operator(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    ctx.nodes().parent_node(node.id()).map_or(false, |parent| match parent.kind() {
        AstKind::UnaryExpression(expr) => expr.operator == UnaryOperator::Typeof,
        AstKind::ParenthesizedExpression(_) => has_typeof_operator(parent, ctx),
        _ => false,
    })
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var a = 1, b = 2; a;", None),
        // ("/*global b*/ function f() { b; }", None),
        // { code: "function f() { b; }", globals: { b: false } },
        // ("/*global b a:false*/  a;  function f() { b; a; }", None),
        ("function a(){}  a();", None),
        ("function f(b) { b; }", None),
        ("var a; a = 1; a++;", None),
        ("var a; function f() { a = 1; }", None),
        // ("/*global b:true*/ b++;", None),
        // ("/*eslint-env browser*/ window;", None),
        // ("/*eslint-env node*/ require(\"a\");", None),
        ("Object; isNaN();", None),
        ("toString()", None),
        ("hasOwnProperty()", None),
        ("function evilEval(stuffToEval) { var ultimateAnswer; ultimateAnswer = 42; eval(stuffToEval); }", None),
        ("typeof a", None),
        ("typeof (a)", None),
        ("var b = typeof a", None),
        ("typeof a === 'undefined'", None),
        ("if (typeof a === 'undefined') {}", None),
        ("function foo() { var [a, b=4] = [1, 2]; return {a, b}; }", None),
        ("var toString = 1;", None),
        ("function myFunc(...foo) {  return foo;}", None),
        ("var React, App, a=1; React.render(<App attr={a} />);", None),
        ("var console; [1,2,3].forEach(obj => {\n  console.log(obj);\n});", None),
        ("var Foo; class Bar extends Foo { constructor() { super();  }}", None),
        ("import Warning from '../lib/warning'; var warn = new Warning('text');", None),
        ("import * as Warning from '../lib/warning'; var warn = new Warning('text');", None),
        ("var a; [a] = [0];", None),
        ("var a; ({a} = {});", None),
        ("var a; ({b: a} = {});", None),
        ("var obj; [obj.a, obj.b] = [0, 1];", None),
        // ("URLSearchParams;", None),
        // ("Intl;", None),
        // ("IntersectionObserver;", None),
        // ("Credential;", None),
        // ("requestIdleCallback;", None),
        // ("customElements;", None),
        // ("PromiseRejectionEvent;", None),
        ("(foo, bar) => { foo ||= WeakRef; bar ??= FinalizationRegistry; }", None),
        // ("/*global b:false*/ function f() { b = 1; }", None),
        // { code: "function f() { b = 1; }", globals: { b: false } },
        // ("/*global b:false*/ function f() { b++; }", None),
        // ("/*global b*/ b = 1;", None),
        // ("/*global b:false*/ var b = 1;", None),
        ("Array = 1;", None),
        ("class A { constructor() { new.target; } }", None),
        // {
        //     code: "var {bacon, ...others} = stuff; foo(others)",
        //     parserOptions: {
        //         ecmaVersion: 2018
        //     },
        //     globals: { stuff: false, foo: false }
        // },
        ("export * as ns from \"source\"", None),
        ("import.meta", None),
        ("let a; class C { static {} } a;", None),
        ("var a; class C { static {} } a;", None),
        ("a; class C { static {} } var a;", None),
        ("class C { static { C; } }", None),
        ("const C = class { static { C; } }", None),
        ("class C { static { a; } } var a;", None),
        ("class C { static { a; } } let a;", None),
        ("class C { static { var a; a; } }", None),
        ("class C { static { a; var a; } }", None),
        ("class C { static { a; { var a; } } }", None),
        ("class C { static { let a; a; } }", None),
        ("class C { static { a; let a; } }", None),
        ("class C { static { function a() {} a; } }", None),
        ("class C { static { a; function a() {} } }", None)
    ];

    let fail = vec![
        ("a = 1;", None),
        (
            "if (typeof anUndefinedVar === 'string') {}",
            Some(serde_json::json!([{ "typeof": true }])),
        ),
        ("var a = b;", None),
        ("function f() { b; }", None),
        ("window;", None),
        ("require(\"a\");", None),
        ("var React; React.render(<img attr={a} />);", None),
        ("var React, App; React.render(<App attr={a} />);", None),
        ("[a] = [0];", None),
        ("({a} = {});", None),
        ("({b: a} = {});", None),
        ("[obj.a, obj.b] = [0, 1];", None),
        ("const c = 0; const a = {...b, c};", None),
        ("class C { static { a; } }", None),
        ("class C { static { { let a; } a; } }", None),
        ("class C { static { { function a() {} } a; } }", None),
        ("class C { static { function foo() { var a; }  a; } }", None),
        ("class C { static { var a; } static { a; } }", None),
        ("class C { static { let a; } static { a; } }", None),
        ("class C { static { function a(){} } static { a; } }", None),
        ("class C { static { var a; } foo() { a; } }", None),
        ("class C { static { let a; } foo() { a; } }", None),
        ("class C { static { var a; } [a]; }", None),
        ("class C { static { let a; } [a]; }", None),
        ("class C { static { function a() {} } [a]; }", None),
        ("class C { static { var a; } } a;", None),
    ];

    Tester::new(NoUndef::NAME, pass, fail).test_and_snapshot();
}
