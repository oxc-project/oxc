use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    ast_util::{call_expr_method_callee_info, is_method_call},
    context::LintContext,
    rule::Rule,
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(prefer-dom-node-remove): Prefer `childNode.remove()` over `parentNode.removeChild(childNode)`.")]
#[diagnostic(
    severity(warning),
    help(
        "Replace `parentNode.removeChild(childNode)` with `childNode{{dotOrQuestionDot}}remove()`."
    )
)]
struct PreferDomNodeRemoveDiagnostic(#[label] pub Span);

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
    /// ### Example
    /// ```javascript
    /// // bad
    /// parentNode.removeChild(childNode);
    ///
    /// // good
    /// childNode.remove();
    /// ```
    PreferDomNodeRemove,
    pedantic
);

impl Rule for PreferDomNodeRemove {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        if call_expr.optional {
            return;
        }

        if !is_method_call(call_expr, None, Some(&["removeChild"]), Some(1), Some(1)) {
            return;
        }

        let Argument::Expression(expr) = &call_expr.arguments[0] else {
            return;
        };

        let expr = expr.without_parenthesized();
        if matches!(
            expr,
            Expression::ArrayExpression(_)
                | Expression::ArrowExpression(_)
                | Expression::ClassExpression(_)
                | Expression::FunctionExpression(_)
                | Expression::ObjectExpression(_)
                | Expression::TemplateLiteral(_)
        ) || expr.is_literal()
            || expr.is_null_or_undefined()
        {
            return;
        }

        ctx.diagnostic(PreferDomNodeRemoveDiagnostic(
            call_expr_method_callee_info(call_expr).unwrap().0,
        ));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"foo.remove()",
        r"this.remove()",
        r"remove()",
        r"foo.parentNode.removeChild('bar')",
        r"parentNode.removeChild(undefined)",
        r"new parentNode.removeChild(bar);",
        r"removeChild(foo);",
        r"parentNode[removeChild](bar);",
        r"parentNode.foo(bar);",
        r"parentNode.removeChild(bar, extra);",
        r"parentNode.removeChild();",
        r"parentNode.removeChild(...argumentsArray)",
        r"parentNode.removeChild?.(foo)",
    ];

    let fail = vec![
        r"parentNode.removeChild(foo)",
        r"parentNode.removeChild(this)",
        r"parentNode.removeChild(some.node)",
        r"parentNode.removeChild(getChild())",
        r"parentNode.removeChild(lib.getChild())",
        r"parentNode.removeChild((() => childNode)())",
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
        r"if (parentNode.removeChild(foo)) {}",
        r"var removed = parentNode.removeChild(child);",
        r"const foo = parentNode.removeChild(child);",
        r"foo.bar(parentNode.removeChild(child));",
        r#"parentNode.removeChild(child) || "foo";"#,
        r"parentNode.removeChild(child) + 0;",
        r"+parentNode.removeChild(child);",
        r#"parentNode.removeChild(child) ? "foo" : "bar";"#,
        r"if (parentNode.removeChild(child)) {}",
        r"const foo = [parentNode.removeChild(child)]",
        r"const foo = { bar: parentNode.removeChild(child) }",
        r"function foo() { return parentNode.removeChild(child); }",
        r"const foo = () => { return parentElement.removeChild(child); }",
        r"foo(bar = parentNode.removeChild(child))",
        r"foo().removeChild(child)",
        r"foo[doSomething()].removeChild(child)",
        r"parentNode?.removeChild(foo)",
        r"foo?.parentNode.removeChild(foo)",
        r"foo.parentNode?.removeChild(foo)",
        r"foo?.parentNode?.removeChild(foo)",
        r"foo.bar?.parentNode.removeChild(foo.bar)",
        r"a.b?.c.parentNode.removeChild(foo)",
        r"a[b?.c].parentNode.removeChild(foo)",
        r"a?.b.parentNode.removeChild(a.b)",
    ];

    Tester::new_without_config(PreferDomNodeRemove::NAME, pass, fail).test_and_snapshot();
}
