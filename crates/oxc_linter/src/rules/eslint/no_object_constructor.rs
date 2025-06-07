use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_object_constructor_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow calls to the `Object` constructor without an argument")
        .with_help("Use object literal notation {} instead")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoObjectConstructor;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow calls to the Object constructor without an argument
    ///
    /// ### Why is this bad?
    ///
    /// Use of the Object constructor to construct a new empty object is generally discouraged in favor of object literal notation because of conciseness and because the Object global may be redefined. The exception is when the Object constructor is used to intentionally wrap a specified value which is passed as an argument.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// Object();
    /// new Object();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// Object("foo");
    /// const obj = { a: 1, b: 2 };
    /// const isObject = value => value === Object(value);
    /// const createObject = Object => new Object();
    /// ```
    NoObjectConstructor,
    eslint,
    pedantic,
    pending
);

impl Rule for NoObjectConstructor {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (span, callee, arguments, type_parameters) = match node.kind() {
            AstKind::CallExpression(call_expr) => {
                (call_expr.span, &call_expr.callee, &call_expr.arguments, &call_expr.type_arguments)
            }
            AstKind::NewExpression(new_expr) => {
                (new_expr.span, &new_expr.callee, &new_expr.arguments, &new_expr.type_arguments)
            }
            _ => return,
        };

        let Expression::Identifier(ident) = &callee else {
            return;
        };

        if ident.name == "Object"
            && ctx.is_reference_to_global_variable(ident)
            && arguments.is_empty()
            && type_parameters.is_none()
        {
            ctx.diagnostic(no_object_constructor_diagnostic(span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("new Object(x)", None, None),
        ("Object(x)", None, None),
        ("new globalThis.Object", None, None),
        ("const createObject = Object => new Object()", None, None),
        ("var Object; new Object;", None, None),
        ("new Object()", None, Some(serde_json::json!({"globals": {"Object": "off"} }))),
    ];

    let fail = vec![
        ("new Object", None, None),
        ("Object()", None, None),
        ("const fn = () => Object();", None, None),
        ("Object() instanceof Object;", None, None),
        ("const obj = Object?.();", None, None),
        ("(new Object() instanceof Object);", None, None),
        // Semicolon required before `({})` to compensate for ASI
        ("Object()", None, None),
        (
            "foo()
        Object()",
            None,
            None,
        ),
        (
            "var yield = bar.yield
        Object()",
            None,
            None,
        ),
        (
            "var foo = { bar: baz }
        Object()",
            None,
            None,
        ),
        (
            "<foo />
        Object()",
            None,
            None,
        ),
        (
            "<foo></foo>
        Object()",
            None,
            None,
        ),
        // No semicolon required before `({})` because ASI does not occur
        ("Object()", None, None),
        (
            "{}
        Object()",
            None,
            None,
        ),
        (
            "function foo() {}
        Object()",
            None,
            None,
        ),
        (
            "class Foo {}
        Object()",
            None,
            None,
        ),
        ("foo: Object();", None, None),
        ("foo();Object();", None, None),
        ("{ Object(); }", None, None),
        ("if (a) Object();", None, None),
        ("if (a); else Object();", None, None),
        ("while (a) Object();", None, None),
        ("do Object(); while (a);", None, None),
        ("for (let i = 0; i < 10; i++) Object();", None, None),
        ("for (const prop in obj) Object();", None, None),
        ("for (const element of iterable) Object();", None, None),
        ("with (obj) Object();", None, None),
        // No semicolon required before `({})` because ASI still occurs
        (
            "const foo = () => {}
        Object()",
            None,
            None,
        ),
        (
            "a++
        Object()",
            None,
            None,
        ),
        (
            "a--
        Object()",
            None,
            None,
        ),
        (
            "function foo() {
            return
            Object();
        }",
            None,
            None,
        ),
        (
            "function * foo() {
            yield
            Object();
        }",
            None,
            None,
        ),
        ("do {} while (a) Object()", None, None),
        (
            "debugger
        Object()",
            None,
            None,
        ),
        (
            "for (;;) {
            break
            Object()
        }",
            None,
            None,
        ),
        (
            r"for (;;) {
            continue
            Object()
        }",
            None,
            None,
        ),
        (
            "foo: break foo
        Object()",
            None,
            None,
        ),
        (
            "foo: while (true) continue foo
        Object()",
            None,
            None,
        ),
        (
            "const foo = bar
        export { foo }
        Object()",
            None,
            None,
        ),
        (
            "export { foo } from 'bar'
        Object()",
            None,
            None,
        ),
        (
            r"export * as foo from 'bar'
        Object()",
            None,
            None,
        ),
        (
            "import foo from 'bar'
         Object()",
            None,
            None,
        ),
        (
            "var yield = 5;
        yield: while (foo) {
            if (bar)
                break yield
            new Object();
        }",
            None,
            None,
        ),
    ];

    Tester::new(NoObjectConstructor::NAME, NoObjectConstructor::PLUGIN, pass, fail)
        .test_and_snapshot();
}
