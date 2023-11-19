use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode, Fix};

#[derive(Debug, Error, Diagnostic)]
enum NoNestedTernaryDiagnostic {
    #[error("eslint-plugin-unicorn(no-nested-ternary): Unexpected nested ternary expression without parentheses.")]
    #[diagnostic(severity(warning), help("Add parentheses around the nested ternary expression."))]
    UnparenthesizedNestedTernary(#[label] Span),
    #[error(
        "eslint-plugin-unicorn(no-nested-ternary): Unexpected deeply nested ternary expression."
    )]
    #[diagnostic(
        severity(warning),
        help("Avoid nesting ternary expressions for more than one level.")
    )]
    DeeplyNestedTernary(#[label] Span),
}

#[derive(Debug, Default, Clone)]
pub struct NoNestedTernary;

declare_oxc_lint!(
    /// ### What it does
    /// This rule disallows deeply nested ternary expressions.
    /// Nested ternary expressions that are only one level deep and wrapped in parentheses are allowed.
    ///
    /// ### Why is this bad?
    /// Nesting ternary expressions can make code more difficult to understand.
    ///
    /// ### Example
    /// ```javascript
    /// // Fail
    /// const foo = i > 5 ? i < 100 ? true : false : true;
    /// const foo = i > 5 ? true : (i < 100 ? true : (i < 1000 ? true : false));
    ///
    /// // Pass
    /// const foo = i > 5 ? (i < 100 ? true : false) : true;
    /// const foo = i > 5 ? (i < 100 ? true : false) : (i < 100 ? true : false);
    /// ```
    NoNestedTernary,
    restriction
);

impl Rule for NoNestedTernary {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ConditionalExpression(cond_expr) = node.kind() else { return };

        if matches!(&cond_expr.test.get_inner_expression(), Expression::ConditionalExpression(_))
            || matches!(
                &cond_expr.consequent.get_inner_expression(),
                Expression::ConditionalExpression(_)
            )
            || matches!(
                &cond_expr.alternate.get_inner_expression(),
                Expression::ConditionalExpression(_)
            )
        {
            return;
        }

        let mut nested_level = 0;
        let mut current_node = node;
        while let Some(parent_node) = ctx.nodes().parent_node(current_node.id()) {
            match parent_node.kind() {
                AstKind::ConditionalExpression(_) => {
                    nested_level += 1;
                }
                AstKind::ParenthesizedExpression(_) => {}
                _ => break,
            }
            if nested_level == 2 {
                break;
            }
            current_node = parent_node;
        }

        match nested_level {
            0 => {}
            1 => {
                let Some(parent_node) = ctx.nodes().parent_node(node.id()) else { unreachable!() };
                if let AstKind::ParenthesizedExpression(_) = parent_node.kind() {
                    return;
                }
                ctx.diagnostic_with_fix(
                    NoNestedTernaryDiagnostic::UnparenthesizedNestedTernary(cond_expr.span),
                    || {
                        Fix::new(
                            format!("({})", cond_expr.span.source_text(ctx.source_text())),
                            cond_expr.span,
                        )
                    },
                );
            }
            2 => {
                ctx.diagnostic(NoNestedTernaryDiagnostic::DeeplyNestedTernary(cond_expr.span));
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const foo = i > 5 ? true : false;",
        r"const foo = i > 5 ? true : (i < 100 ? true : false);",
        r"const foo = i > 5 ? (i < 100 ? true : false) : true;",
        r"const foo = i > 5 ? (i < 100 ? true : false) : (i < 100 ? true : false);",
        r"const foo = i > 5 ? true : (i < 100 ? FOO(i > 50 ? false : true) : false);",
        r"foo ? doBar() : doBaz();",
        r"var foo = bar === baz ? qux : quxx;",
        r"
            const pluginName = isAbsolute ?
                pluginPath.slice(pluginPath.lastIndexOf('/') + 1) :
                (
                    isNamespaced ?
                    pluginPath.split('@')[1].split('/')[1] :
                    pluginPath
                );
        ",
    ];

    let fail = vec![
        r"const foo = i > 5 ? true : (i < 100 ? true : (i < 1000 ? true : false));",
        r"const foo = i > 5 ? true : (i < 100 ? (i > 50 ? false : true) : false);",
        r"const foo = i > 5 ? i < 100 ? true : false : true;",
        r"const foo = i > 5 ? i < 100 ? true : false : i < 100 ? true : false;",
        r"const foo = i > 5 ? true : i < 100 ? true : false;",
        r"foo ? bar : baz === qux ? quxx : foobar;",
        r"foo ? baz === qux ? quxx : foobar : bar;",
        r"
        const foo = a ?
            b :
            (
                c ?
                    d :
                    (
                        e ?
                            f :
                            (g ? h : i)
                    )
            )
        ",
    ];

    let fix = vec![
        (
            "const foo = i > 5 ? i < 100 ? true : false : true;",
            "const foo = i > 5 ? (i < 100 ? true : false) : true;",
            None,
        ),
        (
            "const foo = i > 5 ? i < 100 ? true : false : i < 100 ? true : false;",
            "const foo = i > 5 ? (i < 100 ? true : false) : (i < 100 ? true : false);",
            None,
        ),
        (
            "const foo = i > 5 ? true : i < 100 ? true : false;",
            "const foo = i > 5 ? true : (i < 100 ? true : false);",
            None,
        ),
        (
            "foo ? bar : baz === qux ? quxx : foobar;",
            "foo ? bar : (baz === qux ? quxx : foobar);",
            None,
        ),
        (
            "foo ? baz === qux ? quxx : foobar : bar;",
            "foo ? (baz === qux ? quxx : foobar) : bar;",
            None,
        ),
    ];

    Tester::new_without_config(NoNestedTernary::NAME, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
