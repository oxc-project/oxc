use oxc_ast::ast::Expression;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_nested_ternary_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not nest ternary expressions.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNestedTernary;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows nested ternary expressions to improve code readability and maintainability.
    ///
    /// ### Why is this bad?
    ///
    /// Nested ternary expressions make code harder to read and understand. They can lead to complex, difficult-to-debug logic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const result = condition1 ? (condition2 ? "a" : "b") : "c";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// let result;
    /// if (condition1) {
    ///   result = condition2 ? "a" : "b";
    /// } else {
    ///   result = "c";
    /// }
    /// ```
    NoNestedTernary,
    eslint,
    style,
);

impl Rule for NoNestedTernary {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ConditionalExpression(node) = node.kind() {
            if matches!(
                node.consequent.get_inner_expression(),
                Expression::ConditionalExpression(_)
            ) || matches!(
                node.alternate.get_inner_expression(),
                Expression::ConditionalExpression(_)
            ) {
                ctx.diagnostic(no_nested_ternary_diagnostic(node.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo ? doBar() : doBaz();",
        "var foo = bar === baz ? qux : quxx;",
        "var result = foo && bar ? baz : qux || quux;",
        "var result = foo ? bar : baz === qux;",
        "foo ? doSomething(a, b) : doSomethingElse(c, d);",
        // Parenthesized Expressions
        "var result = (foo ? bar : baz) || qux;",
        "var result = (foo ? bar : baz) && qux;",
        "var result = foo === bar ? (baz || qux) : quux;",
        "var result = (foo ? bar : baz) ? qux : quux;",
        // TypeScript
        "var result = foo! ? bar : baz;",
        "var result = foo ? bar! : baz;",
        "var result = (foo as boolean) ? bar : baz;",
        "var result = foo ? (bar as string) : baz;",
    ];

    let fail = vec![
        "foo ? bar : baz === qux ? quxx : foobar;",
        "foo ? baz === qux ? quxx : foobar : bar;",
        // Parenthesized Expressions
        "var result = foo ? (bar ? baz : qux) : quux;",
        "var result = foo ? (bar === baz ? qux : quux) : foobar;",
        "doSomething(foo ? bar : baz ? qux : quux);",
        // Comment
        "var result = foo /* comment */ ? bar : baz ? qux : quux;",
        // TypeScript
        "var result = foo! ? bar : baz! ? qux : quux;",
        "var result = foo ? bar! : (baz! ? qux : quux);",
        "var result = (foo as boolean) ? bar : (baz as string) ? qux : quux;",
        "var result = foo ? (bar as string) : (baz as number ? qux : quux);",
    ];

    Tester::new(NoNestedTernary::NAME, NoNestedTernary::PLUGIN, pass, fail).test_and_snapshot();
}
