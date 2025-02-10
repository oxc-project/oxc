use oxc_ast::{
    ast::{Expression, Statement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_await_in_loop_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected `await` inside a loop.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAwaitInLoop;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows the use of `await` within loop bodies. (for, for-in, for-of, while, do-while).
    ///
    /// ### Why is this bad?
    ///
    /// It potentially indicates that the async operations are not being effectively parallelized.
    /// Instead, they are being run in series, which can lead to poorer performance.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// async function bad() {
    ///     for (const user of users) {
    ///       const userRecord = await getUserRecord(user);
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// async function good() {
    ///     await Promise.all(users.map(user => getUserRecord(user)));
    /// }
    /// ```
    NoAwaitInLoop,
    eslint,
    perf
);

impl Rule for NoAwaitInLoop {
    fn run(&self, node: &AstNode, ctx: &LintContext) {
        // if node is AwaitExpression or AwaitForOfStatement
        let span = match node.kind() {
            // if the await attr of ForOfStatement is false, return
            AstKind::ForOfStatement(for_of_stmt) => {
                if !for_of_stmt.r#await {
                    return;
                }

                // only highlight the 'await' keyword
                Span::new(for_of_stmt.span.start + 4, for_of_stmt.span.start + 9)
            }
            // only highlight the 'await' keyword
            AstKind::AwaitExpression(expr) => Span::new(expr.span.start, expr.span.start + 5),
            // other node type, return
            _ => return,
        };

        let nodes = ctx.semantic().nodes();
        // Perform validation for AwaitExpression and ForOfStatement that contains await
        let mut parent_node = nodes.parent_node(node.id());
        let mut is_in_loop = false;
        while let Some(parent) = parent_node {
            // Check if the current node is the boundary of the loop
            if Self::is_boundary(parent) {
                break;
            }

            // if AwaitExpression or AwaitForOfStatement are in loop, break and report error
            if Self::is_looped(span, parent) {
                is_in_loop = true;
                break;
            }

            parent_node = nodes.parent_node(parent.id());
        }

        if is_in_loop {
            ctx.diagnostic(no_await_in_loop_diagnostic(span));
        }
    }
}

impl NoAwaitInLoop {
    fn node_matches_stmt_span(span: Span, stmt: &Statement) -> bool {
        match stmt {
            Statement::BlockStatement(block) => Self::include_span(block.span, span),
            Statement::ExpressionStatement(expr_statement) => {
                Self::include_span(expr_statement.span, span)
            }
            _ => false,
        }
    }

    fn node_matches_expr_span(span: Span, expr: &Expression) -> bool {
        match expr {
            Expression::TemplateLiteral(expr) => Self::include_span(expr.span, span),
            Expression::ArrayExpression(expr) => Self::include_span(expr.span, span),
            Expression::ArrowFunctionExpression(expr) => Self::include_span(expr.span, span),
            Expression::AssignmentExpression(expr) => Self::include_span(expr.span, span),
            Expression::AwaitExpression(expr) => Self::include_span(expr.span, span),
            Expression::BinaryExpression(expr) => Self::include_span(expr.span, span),
            Expression::CallExpression(expr) => Self::include_span(expr.span, span),
            Expression::ChainExpression(expr) => Self::include_span(expr.span, span),
            Expression::ClassExpression(expr) => Self::include_span(expr.span, span),
            Expression::ConditionalExpression(expr) => Self::include_span(expr.span, span),
            Expression::FunctionExpression(expr) => Self::include_span(expr.span, span),
            Expression::ImportExpression(expr) => Self::include_span(expr.span, span),
            Expression::LogicalExpression(expr) => Self::include_span(expr.span, span),
            Expression::NewExpression(expr) => Self::include_span(expr.span, span),
            Expression::ObjectExpression(expr) => Self::include_span(expr.span, span),
            Expression::ParenthesizedExpression(expr) => Self::include_span(expr.span, span),
            Expression::SequenceExpression(expr) => Self::include_span(expr.span, span),
            Expression::TaggedTemplateExpression(expr) => Self::include_span(expr.span, span),
            Expression::ThisExpression(expr) => Self::include_span(expr.span, span),
            Expression::UnaryExpression(expr) => Self::include_span(expr.span, span),
            Expression::UpdateExpression(expr) => Self::include_span(expr.span, span),
            Expression::YieldExpression(expr) => Self::include_span(expr.span, span),
            Expression::PrivateInExpression(expr) => Self::include_span(expr.span, span),
            _ => false,
        }
    }

    fn is_looped(span: Span, parent: &AstNode) -> bool {
        match parent.kind() {
            AstKind::ForStatement(stmt) => {
                let mut result = Self::node_matches_stmt_span(span, &stmt.body);
                if result {
                    return result;
                }

                if let Some(test) = &stmt.test {
                    result = Self::node_matches_expr_span(span, test);
                    if result {
                        return result;
                    }
                }

                if let Some(update) = &stmt.update {
                    result = Self::node_matches_expr_span(span, update);
                }

                result
            }
            AstKind::ForInStatement(stmt) => Self::node_matches_stmt_span(span, &stmt.body),
            AstKind::ForOfStatement(stmt) => Self::node_matches_stmt_span(span, &stmt.body),
            AstKind::WhileStatement(stmt) => {
                Self::node_matches_stmt_span(span, &stmt.body)
                    || Self::node_matches_expr_span(span, &stmt.test)
            }
            AstKind::DoWhileStatement(stmt) => {
                Self::node_matches_stmt_span(span, &stmt.body)
                    || Self::node_matches_expr_span(span, &stmt.test)
            }
            _ => false,
        }
    }

    // is span1 include span2
    fn include_span(span1: Span, span2: Span) -> bool {
        span1.start <= span2.start && span1.end >= span2.end
    }

    fn is_boundary(node: &AstNode) -> bool {
        match node.kind() {
            AstKind::Function(func) => func.is_declaration() || func.is_expression(),
            AstKind::ArrowFunctionExpression(_) => true,
            AstKind::ForOfStatement(for_of_stmt) => for_of_stmt.r#await,
            _ => false,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "async function foo() { await bar; }",
        "async function foo() { for (var bar in await baz) { } }",
        "async function foo() { for (var bar of await baz) { } }",
        "async function foo() { for await (var bar of await baz) { } }",
        "async function foo() { for (var bar = await baz in qux) {} }",
        // While loops
        "async function foo() { while (true) { async function foo() { await bar; } } }", // Blocked by a function declaration
        // For loops
        "async function foo() { for (var i = await bar; i < n; i++) {  } }",
        // Do while loops
        "async function foo() { do { } while (bar); }",
        // Blocked by a function expression
        "async function foo() { while (true) { var y = async function() { await bar; } } }",
        // Blocked by an arrow function
        "async function foo() { while (true) { var y = async () => await foo; } }",
        "async function foo() { while (true) { var y = async () => { await foo; } } }",
        // Blocked by a class method
        "async function foo() { while (true) { class Foo { async foo() { await bar; } } } }",
        // Asynchronous iteration intentionally
        "async function foo() { for await (var x of xs) { await f(x) } }",
    ];

    let fail = vec![
        // While loops
        "async function foo() { while (baz) { await bar; } }",
        "async function foo() { while (await foo()) {  } }",
        "async function foo() { while (baz) { for await (x of xs); } }",
        // For of loops
        "async function foo() { for (var bar of baz) { await bar; } }",
        "async function foo() { for (var bar of baz) await bar; }",
        // For in loops
        "async function foo() { for (var bar in baz) { await bar; } }",
        // For loops
        "async function foo() { for (var i; i < n; i++) { await bar; } }",
        "async function foo() { for (var i; await foo(i); i++) {  } }",
        "async function foo() { for (var i; i < n; i = await bar) {  } }",
        // Do while loops
        "async function foo() { do { await bar; } while (baz); }",
        "async function foo() { do { } while (await bar); }",
        // Deep in a loop body
        "async function foo() { while (true) { if (bar) { foo(await bar); } } }",
        // Deep in a loop condition
        "async function foo() { while (xyz || 5 > await x) {  } }",
        // In a nested loop of for-await-of
        "async function foo() { for await (var x of xs) { while (1) await f(x) } }",
    ];

    Tester::new(NoAwaitInLoop::NAME, NoAwaitInLoop::PLUGIN, pass, fail).test_and_snapshot();
}
