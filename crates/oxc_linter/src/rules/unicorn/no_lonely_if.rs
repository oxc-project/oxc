use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(no-lonely-if): Unexpected `if` as the only statement in a `if` block without `else`.")]
#[diagnostic(severity(warning), help("Move the inner `if` test to the outer `if` test."))]
struct NoLonelyIfDiagnostic(#[label] pub Span, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoLonelyIf;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `if` statements as the only statement in `if` blocks without `else`.
    ///
    /// ### Why is this bad?
    ///
    /// It can be confusing to have an `if` statement without an `else` clause as the only statement in an `if` block.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// if (foo) {
    ///         if (bar) {
    ///     }
    /// }
    /// if (foo) if (bar) baz();
    ///
    /// // Good
    /// if (foo && bar) {
    /// }
    /// if (foo && bar) baz();
    /// ```
    NoLonelyIf,
    pedantic
);

impl Rule for NoLonelyIf {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IfStatement(if_stmt) = node.kind() else { return };

        if if_stmt.alternate.is_some() {
            return;
        }

        let Some(parent) = ctx.nodes().parent_node(node.id()) else {
            return;
        };

        let parent_if_stmt_span = match parent.kind() {
            AstKind::BlockStatement(block_stmt) => {
                if block_stmt.body.len() != 1 {
                    return;
                }

                let Some(parent) = ctx.nodes().parent_node(parent.id()) else {
                    return;
                };

                let AstKind::IfStatement(parent_if_stmt) = parent.kind() else {
                    return;
                };

                if parent_if_stmt.alternate.is_some() {
                    return;
                }
                parent_if_stmt.span
            }
            AstKind::IfStatement(parent_if_stmt) => {
                if parent_if_stmt.alternate.is_some() {
                    return;
                }

                parent_if_stmt.span
            }
            _ => return,
        };

        ctx.diagnostic(NoLonelyIfDiagnostic(
            Span { start: if_stmt.span.start, end: if_stmt.span.start + 2 },
            Span { start: parent_if_stmt_span.start, end: parent_if_stmt_span.start + 2 },
        ));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"if (a) { if (b) { } } else { }"#,
        r#"if (a) { if (b) { } foo(); } else {}"#,
        r#"if (a) { } else { if (y) { } }"#,
        r#"if (a) { b ? c() : d()}"#,
    ];

    let fail = vec![
        r#"
        if (a) {
            if (b) {
            }
        }
    "#,
        // Inner one is `BlockStatement`
        r#"
        if (a) if (b) {
            foo();
        }
    "#,
        // Outer one is `BlockStatement`
        r#"
        if (a) {
            if (b) foo();
        }
    "#,
        // No `BlockStatement`
        r#"if (a) if (b) foo();"#,
        r#"
        if (a) {
            if (b) foo()
        }
    "#,
        // `EmptyStatement`
        r#"if (a) if (b);"#,
        // Nested
        r#"
        if (a) {
            if (b) {
                // Should not report
            }
        } else if (c) {
            if (d) {
            }
        }
    "#,
        // Need parenthesis
        r#"
        function * foo() {
            if (a || b)
            if (a ?? b)
            if (a ? b : c)
            if (a = b)
            if (a += b)
            if (a -= b)
            if (a &&= b)
            if (yield a)
            if (a, b);
        }
    "#,
        // Should not add parenthesis
        r#"
        async function foo() {
            if (a)
            if (await a)
            if (a.b)
            if (a && b);
        }
    "#,
        // Don't case parenthesis in outer test
        // 'if (((a || b))) if (((c || d)));',
        // Comments
        r#"
        if // 1
        (
            // 2
            a // 3
                .b // 4
        ) // 5
        {
            // 6
            if (
                // 7
                c // 8
                    .d // 9
            ) {
                // 10
                foo();
                // 11
            }
            // 12
        }
    "#,
        // Semicolon
        r#"
        if (a) {
            if (b) foo()
        }
        [].forEach(bar)
    "#,
        r#"
        if (a)
            if (b) foo()
        ;[].forEach(bar)
    "#,
        r#"
        if (a) {
            if (b) foo()
        }
        ;[].forEach(bar)
    "#,
    ];

    Tester::new_without_config(NoLonelyIf::NAME, pass, fail).test_and_snapshot();
}
