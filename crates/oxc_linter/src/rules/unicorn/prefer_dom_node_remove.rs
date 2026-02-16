use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    ast_util::{call_expr_method_callee_info, is_method_call},
    context::LintContext,
    rule::Rule,
};

fn prefer_dom_node_remove_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `childNode.remove()` over `parentNode.removeChild(childNode)`.")
        .with_help("Replace `parentNode.removeChild(childNode)` with `childNode{dotOrQuestionDot}remove()`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferDomNodeRemove;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers the use of `child.remove()` over `parentNode.removeChild(child)`.
    ///
    /// ### Why is this bad?
    ///
    /// The DOM function [`Node#remove()`](https://developer.mozilla.org/en-US/docs/Web/API/ChildNode/remove) is preferred over the indirect removal of an object with [`Node#removeChild()`](https://developer.mozilla.org/en-US/docs/Web/API/Node/removeChild).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// parentNode.removeChild(childNode);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// childNode.remove();
    /// ```
    PreferDomNodeRemove,
    unicorn,
    pedantic,
    pending
);

impl Rule for PreferDomNodeRemove {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional {
            return;
        }

        if !is_method_call(call_expr, None, Some(&["removeChild"]), Some(1), Some(1)) {
            return;
        }

        let Some(expr) = call_expr.arguments[0].as_expression() else {
            return;
        };

        let expr = expr.without_parentheses();
        if matches!(
            expr,
            Expression::ArrayExpression(_)
                | Expression::ArrowFunctionExpression(_)
                | Expression::ClassExpression(_)
                | Expression::FunctionExpression(_)
                | Expression::ObjectExpression(_)
                | Expression::TemplateLiteral(_)
        ) || expr.is_literal()
            || expr.is_null_or_undefined()
        {
            return;
        }

        ctx.diagnostic(prefer_dom_node_remove_diagnostic(
            call_expr_method_callee_info(call_expr).unwrap().0,
        ));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo.remove()",
        "this.remove()",
        "remove()",
        "foo.parentNode.removeChild('bar')",
        "parentNode.removeChild(undefined)",
        "new parentNode.removeChild(bar);",
        "removeChild(foo);",
        "parentNode[removeChild](bar);",
        "parentNode.foo(bar);",
        "parentNode.removeChild(bar, extra);",
        "parentNode.removeChild();",
        "parentNode.removeChild(...argumentsArray)",
        "parentNode.removeChild?.(foo)",
    ];

    let fail = vec![
        "parentNode.removeChild(foo)",
        "parentNode.removeChild(this)",
        "parentNode.removeChild(some.node)",
        "parentNode.removeChild(getChild())",
        "parentNode.removeChild(lib.getChild())",
        "parentNode.removeChild((() => childNode)())",
        r"
                async function foo () {
                    parentNode.removeChild(
                        await getChild()
                    );
                }
        ",
        r"
        async function foo () {
            parentNode.removeChild(
                (await getChild())
            );
        }
        ",
        r"parentNode.removeChild((0, child))",
        r"parentNode.removeChild( (  (new Image)) )",
        r"parentNode.removeChild( new Audio )",
        r"
        const array = []
        parentNode.removeChild([a, b, c].reduce(child => child, child))
        ",
        r"
        async function foo () {
            const array = []
            parentNode.removeChild(
                await getChild()
            );
        }
        ",
        r"
        async function foo () {
            const array = []
            parentNode.removeChild(
                (0, childNode)
            );
        }
        ",
        r"
        async function foo () {
            const array = []
            parentNode.removeChild(
                (0, childNode)
            );
        }
        ",
        r"
        async function foo () {
            const array = []
            parentNode.removeChild(
                (0, childNode)
            );
        }
        ",
        "if (parentNode.removeChild(foo)) {}",
        "var removed = parentNode.removeChild(child);",
        "const foo = parentNode.removeChild(child);",
        "foo.bar(parentNode.removeChild(child));",
        r#"parentNode.removeChild(child) || "foo";"#,
        "parentNode.removeChild(child) + 0;",
        "+parentNode.removeChild(child);",
        r#"parentNode.removeChild(child) ? "foo" : "bar";"#,
        "if (parentNode.removeChild(child)) {}",
        "const foo = [parentNode.removeChild(child)]",
        "const foo = { bar: parentNode.removeChild(child) }",
        "function foo() { return parentNode.removeChild(child); }",
        "const foo = () => { return parentElement.removeChild(child); }",
        "foo(bar = parentNode.removeChild(child))",
        "foo().removeChild(child)",
        "foo[doSomething()].removeChild(child)",
        "parentNode?.removeChild(foo)",
        "foo?.parentNode.removeChild(foo)",
        "foo.parentNode?.removeChild(foo)",
        "foo?.parentNode?.removeChild(foo)",
        "foo.bar?.parentNode.removeChild(foo.bar)",
        "a.b?.c.parentNode.removeChild(foo)",
        "a[b?.c].parentNode.removeChild(foo)",
        "a?.b.parentNode.removeChild(a.b)",
        "a.removeChild!(k)",
    ];

    Tester::new(PreferDomNodeRemove::NAME, PreferDomNodeRemove::PLUGIN, pass, fail)
        .test_and_snapshot();
}
