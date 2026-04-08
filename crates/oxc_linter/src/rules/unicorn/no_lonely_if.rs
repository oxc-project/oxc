use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_lonely_if_diagnostic(if_stmt_span: Span, parent_if_stmt_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected `if` as the only statement in a `if` block without `else`.")
        .with_help("Move the inner `if` test to the outer `if` test.")
        .with_labels([if_stmt_span, parent_if_stmt_span])
}

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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (foo) {
    ///     if (bar) {
    ///     }
    /// }
    /// if (foo) if (bar) baz();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// if (foo && bar) {
    /// }
    /// if (foo && bar) baz();
    /// ```
    NoLonelyIf,
    unicorn,
    pedantic,
    fix
);

impl Rule for NoLonelyIf {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IfStatement(if_stmt) = node.kind() else {
            return;
        };

        if if_stmt.alternate.is_some() {
            return;
        }

        let parent = ctx.nodes().parent_node(node.id());

        let parent_if_stmt_span = match parent.kind() {
            AstKind::BlockStatement(block_stmt) => {
                if block_stmt.body.len() != 1 {
                    return;
                }

                let AstKind::IfStatement(parent_if_stmt) = ctx.nodes().parent_kind(parent.id())
                else {
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

        let inner_test = &if_stmt.test;
        let inner_consequent = &if_stmt.consequent;

        ctx.diagnostic_with_fix(
            no_lonely_if_diagnostic(
                Span::sized(if_stmt.span.start, 2),
                Span::sized(parent_if_stmt_span.start, 2),
            ),
            |fixer| {
                let inner_test_text = fixer.source_range(inner_test.span());
                let inner_test_needs_parens = needs_parentheses(inner_test);
                let inner_test_str = if inner_test_needs_parens {
                    format!("({inner_test_text})")
                } else {
                    inner_test_text.to_string()
                };

                // Get the outer if's test
                let outer_test_span = match parent.kind() {
                    AstKind::BlockStatement(_) => {
                        let AstKind::IfStatement(parent_if_stmt) =
                            ctx.nodes().parent_kind(parent.id())
                        else {
                            return fixer.noop();
                        };
                        parent_if_stmt.test.span()
                    }
                    AstKind::IfStatement(parent_if_stmt) => parent_if_stmt.test.span(),
                    _ => return fixer.noop(),
                };

                let outer_test_text = fixer.source_range(outer_test_span);

                // The outer test also needs parentheses if it has lower precedence than &&
                let outer_test_expr = match parent.kind() {
                    AstKind::BlockStatement(_) => {
                        let AstKind::IfStatement(parent_if_stmt) =
                            ctx.nodes().parent_kind(parent.id())
                        else {
                            return fixer.noop();
                        };
                        &parent_if_stmt.test
                    }
                    AstKind::IfStatement(parent_if_stmt) => &parent_if_stmt.test,
                    _ => return fixer.noop(),
                };
                let outer_test_str = if needs_parentheses(outer_test_expr) {
                    format!("({outer_test_text})")
                } else {
                    outer_test_text.to_string()
                };

                let consequent_text = fixer.source_range(inner_consequent.span());

                let combined =
                    format!("if ({outer_test_str} && {inner_test_str}) {consequent_text}");
                fixer.replace(parent_if_stmt_span, combined)
            },
        );
    }
}

fn needs_parentheses(expr: &Expression) -> bool {
    matches!(
        expr.get_inner_expression(),
        Expression::LogicalExpression(e) if matches!(e.operator, oxc_syntax::operator::LogicalOperator::Or | oxc_syntax::operator::LogicalOperator::Coalesce)
    ) || matches!(
        expr.get_inner_expression(),
        Expression::ConditionalExpression(_)
            | Expression::AssignmentExpression(_)
            | Expression::SequenceExpression(_)
            | Expression::YieldExpression(_)
    )
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"if (a) { if (b) { } } else { }",
        r"if (a) { if (b) { } foo(); } else {}",
        r"if (a) { } else { if (y) { } }",
        r"if (a) { b ? c() : d()}",
    ];

    let fail = vec![
        r"
        if (a) {
            if (b) {
            }
        }
    ",
        // Inner one is `BlockStatement`
        r"
        if (a) if (b) {
            foo();
        }
    ",
        // Outer one is `BlockStatement`
        r"
        if (a) {
            if (b) foo();
        }
    ",
        // No `BlockStatement`
        r"if (a) if (b) foo();",
        r"
        if (a) {
            if (b) foo()
        }
    ",
        // `EmptyStatement`
        r"if (a) if (b);",
        // Nested
        r"
        if (a) {
            if (b) {
                // Should not report
            }
        } else if (c) {
            if (d) {
            }
        }
    ",
        // Need parenthesis
        r"
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
    ",
        // Should not add parenthesis
        r"
        async function foo() {
            if (a)
            if (await a)
            if (a.b)
            if (a && b);
        }
    ",
        // Don't case parenthesis in outer test
        // 'if (((a || b))) if (((c || d)));',
        // Comments
        r"
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
    ",
        // Semicolon
        r"
        if (a) {
            if (b) foo()
        }
        [].forEach(bar)
    ",
        r"
        if (a)
            if (b) foo()
        ;[].forEach(bar)
    ",
        r"
        if (a) {
            if (b) foo()
        }
        ;[].forEach(bar)
    ",
    ];

    let fix = vec![
        ("if (a) { if (b) { } }", "if (a && b) { }"),
        ("if (a) if (b) foo();", "if (a && b) foo();"),
        // Inner test needs parens
        ("if (a) { if (b || c) { } }", "if (a && (b || c)) { }"),
        // Outer test needs parens
        ("if (a || b) { if (c) { } }", "if ((a || b) && c) { }"),
        // Both need parens
        ("if (a || b) { if (c || d) { } }", "if ((a || b) && (c || d)) { }"),
    ];

    Tester::new(NoLonelyIf::NAME, NoLonelyIf::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
