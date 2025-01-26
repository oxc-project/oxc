use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_undef_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is not defined.")).with_label(span)
}

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
    eslint,
    nursery
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

        for reference_id_list in ctx.scopes().root_unresolved_references_ids() {
            for reference_id in reference_id_list {
                let reference = symbol_table.get_reference(reference_id);

                if reference.is_type() {
                    return;
                }

                let name = ctx.semantic().reference_name(reference);

                if ctx.env_contains_var(name) {
                    continue;
                }

                if ctx.globals().is_enabled(name) {
                    continue;
                }

                let node = ctx.nodes().get_node(reference.node_id());
                if !self.type_of && has_typeof_operator(node, ctx) {
                    continue;
                }

                ctx.diagnostic(no_undef_diagnostic(name, node.kind().span()));
            }
        }
    }
}

fn has_typeof_operator(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    ctx.nodes().parent_node(node.id()).is_some_and(|parent| match parent.kind() {
        AstKind::UnaryExpression(expr) => expr.operator == UnaryOperator::Typeof,
        AstKind::ParenthesizedExpression(_) => has_typeof_operator(parent, ctx),
        _ => false,
    })
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var a = 1, b = 2; a;",
        // "/*global b*/ function f() { b; }",
        // { code: "function f() { b; }", globals: { b: false } },
        // "/*global b a:false*/  a;  function f() { b; a; }",
        "function a(){}  a();",
        "function f(b) { b; }",
        "var a; a = 1; a++;",
        "var a; function f() { a = 1; }",
        // "/*global b:true*/ b++;",
        // "/*eslint-env browser*/ window;",
        // "/*eslint-env node*/ require(\"a\");",
        "Object; isNaN();",
        "function evilEval(stuffToEval) { var ultimateAnswer; ultimateAnswer = 42; eval(stuffToEval); }",
        "typeof a",
        "typeof (a)",
        "var b = typeof a",
        "typeof a === 'undefined'",
        "if (typeof a === 'undefined') {}",
        "function foo() { var [a, b=4] = [1, 2]; return {a, b}; }",
        "var toString = 1;",
        "function myFunc(...foo) {  return foo;}",
        "var React, App, a=1; React.render(<App attr={a} />);",
        "var console; [1,2,3].forEach(obj => {\n  console.log(obj);\n});",
        "var Foo; class Bar extends Foo { constructor() { super();  }}",
        "import Warning from '../lib/warning'; var warn = new Warning('text');",
        "import * as Warning from '../lib/warning'; var warn = new Warning('text');",
        "var a; [a] = [0];",
        "var a; ({a} = {});",
        "var a; ({b: a} = {});",
        "var obj; [obj.a, obj.b] = [0, 1];",
        // "URLSearchParams;",
        // "Intl;",
        // "IntersectionObserver;",
        // "Credential;",
        // "requestIdleCallback;",
        // "customElements;",
        // "PromiseRejectionEvent;",
        "(foo, bar) => { foo ||= WeakRef; bar ??= FinalizationRegistry; }",
        // "/*global b:false*/ function f() { b = 1; }",
        // { code: "function f() { b = 1; }", globals: { b: false } },
        // "/*global b:false*/ function f() { b++; }",
        // "/*global b*/ b = 1;",
        // "/*global b:false*/ var b = 1;",
        "Array = 1;",
        "class A { constructor() { new.target; } }",
        // {
        //     code: "var {bacon, ...others} = stuff; foo(others)",
        //     parserOptions: {
        //         ecmaVersion: 2018
        //     },
        //     globals: { stuff: false, foo: false }
        // },
        "export * as ns from \"source\"",
        "import.meta",
        "let a; class C { static {} } a;",
        "var a; class C { static {} } a;",
        "a; class C { static {} } var a;",
        "class C { static { C; } }",
        "const C = class { static { C; } }",
        "class C { static { a; } } var a;",
        "class C { static { a; } } let a;",
        "class C { static { var a; a; } }",
        "class C { static { a; var a; } }",
        "class C { static { a; { var a; } } }",
        "class C { static { let a; a; } }",
        "class C { static { a; let a; } }",
        "class C { static { function a() {} a; } }",
        "class C { static { a; function a() {} } }",
        "String;Array;Boolean;",
        "function resolve<T>(path: string): T { return { path } as T; }",
        "let xyz: NodeListOf<HTMLElement>",
        "type Foo = Record<string, unknown>;",
    ];

    let fail = vec![
        "a = 1;",
        "var a = b;",
        "function f() { b; }",
        "window;",
        "require(\"a\");",
        "var React; React.render(<img attr={a} />);",
        "var React, App; React.render(<App attr={a} />);",
        "[a] = [0];",
        "({a} = {});",
        "({b: a} = {});",
        "[obj.a, obj.b] = [0, 1];",
        "const c = 0; const a = {...b, c};",
        "class C { static { a; } }",
        "class C { static { { let a; } a; } }",
        "class C { static { { function a() {} } a; } }",
        "class C { static { function foo() { var a; }  a; } }",
        "class C { static { var a; } static { a; } }",
        "class C { static { let a; } static { a; } }",
        "class C { static { function a(){} } static { a; } }",
        "class C { static { var a; } foo() { a; } }",
        "class C { static { let a; } foo() { a; } }",
        "class C { static { var a; } [a]; }",
        "class C { static { let a; } [a]; }",
        "class C { static { function a() {} } [a]; }",
        "class C { static { var a; } } a;",
        "toString()",
        "hasOwnProperty()",
    ];

    Tester::new(NoUndef::NAME, NoUndef::PLUGIN, pass, fail).test_and_snapshot();

    let pass = vec![];
    let fail = vec![(
        "if (typeof anUndefinedVar === 'string') {}",
        Some(serde_json::json!([{ "typeof": true }])),
    )];

    Tester::new(NoUndef::NAME, NoUndef::PLUGIN, pass, fail).test();

    let pass = vec![("foo", None, Some(serde_json::json!({ "globals": { "foo": "readonly" } })))];
    let fail = vec![("foo", None, Some(serde_json::json!({ "globals": { "foo": "off" } })))];

    Tester::new(NoUndef::NAME, NoUndef::PLUGIN, pass, fail).test();
}
