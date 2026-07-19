use oxc_ast::{
    AstKind,
    ast::{AwaitExpression, Expression, ExpressionKind, ExpressionTag},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNodes;
use oxc_span::Span;
use oxc_syntax::operator::{UnaryOperator, UpdateOperator};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_unnecessary_await_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected `await` on a non-Promise value")
        .with_help("Consider removing the `await`")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnnecessaryAwait;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow awaiting on non-promise values.
    ///
    /// ### Why is this bad?
    ///
    /// The `await` operator should only be used on `Promise` values.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// async function bad() {
    ///     await await promise;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// async function bad() {
    ///     await promise;
    /// }
    /// ```
    NoUnnecessaryAwait,
    unicorn,
    correctness,
    conditional_fix,
    version = "0.0.12",
    short_description = "Disallow awaiting non-promise values.",
);

impl Rule for NoUnnecessaryAwait {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::AwaitExpression(expr) = node.kind() {
            if !not_promise(&expr.argument) {
                return;
            }
            if is_fixable(expr, ctx.nodes()) {
                ctx.diagnostic_with_fix(
                    no_unnecessary_await_diagnostic(Span::sized(expr.span.start, 5)),
                    |fixer| fixer.replace_with(expr, &expr.argument),
                );
            } else {
                ctx.diagnostic(no_unnecessary_await_diagnostic(Span::sized(expr.span.start, 5)));
            }
        }
    }
}

fn not_promise(expr: &Expression) -> bool {
    match expr.kind() {
        ExpressionKind::ArrayExpression(_)
        | ExpressionKind::ArrowFunctionExpression(_)
        | ExpressionKind::AwaitExpression(_)
        | ExpressionKind::BinaryExpression(_)
        | ExpressionKind::ClassExpression(_)
        | ExpressionKind::FunctionExpression(_)
        | ExpressionKind::JSXElement(_)
        | ExpressionKind::JSXFragment(_)
        | ExpressionKind::BooleanLiteral(_)
        | ExpressionKind::NullLiteral(_)
        | ExpressionKind::NumericLiteral(_)
        | ExpressionKind::BigIntLiteral(_)
        | ExpressionKind::RegExpLiteral(_)
        | ExpressionKind::StringLiteral(_)
        | ExpressionKind::TemplateLiteral(_)
        | ExpressionKind::UnaryExpression(_)
        | ExpressionKind::UpdateExpression(_) => true,
        ExpressionKind::SequenceExpression(expr) => not_promise(expr.expressions.last().unwrap()),
        ExpressionKind::ParenthesizedExpression(expr) => not_promise(&expr.expression),
        _ => false,
    }
}

fn is_fixable(expr: &AwaitExpression, nodes: &AstNodes<'_>) -> bool {
    // Removing `await` may change them to a declaration, if there is no `id` will cause SyntaxError
    if matches!(
        expr.argument.tag(),
        ExpressionTag::FunctionExpression | ExpressionTag::ClassExpression
    ) {
        return false;
    }

    // Removing `await` would paste the parent unary operator onto the argument's
    // leading operator and change tokenization into a syntax error:
    // `+await +1` -> `++1`, `+await ++a` -> `+++a`, `-await --a` -> `---a`.
    // Skip the fix in those cases (the diagnostic is still reported).
    let parent = nodes.parent_node(expr.node_id());
    match (parent.kind(), expr.argument.kind()) {
        (AstKind::UnaryExpression(parent_unary), ExpressionKind::UnaryExpression(inner_unary)) => {
            parent_unary.operator != inner_unary.operator
        }
        (
            AstKind::UnaryExpression(parent_unary),
            ExpressionKind::UpdateExpression(inner_update),
        ) => {
            !(inner_update.prefix
                && matches!(
                    (parent_unary.operator, inner_update.operator),
                    (UnaryOperator::UnaryPlus, UpdateOperator::Increment)
                        | (UnaryOperator::UnaryNegation, UpdateOperator::Decrement)
                ))
        }
        _ => true,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "await {then}",
        "await a ? b : c",
        "await a || b",
        "await a && b",
        "await a ?? b",
        "await new Foo()",
        "await tagged``",
        "class A { async foo() { await this }}",
        "async function * foo() {await (yield bar);}",
        "await (1, Promise.resolve())",
    ];

    let fail = vec![
        "await []",
        "await [Promise.resolve()]",
        "await (() => {})",
        "await (() => Promise.resolve())",
        "await (a === b)",
        "await (a instanceof Promise)",
        "await (a > b)",
        "await class {}",
        "await class extends Promise {}",
        "await function() {}",
        "await function name() {}",
        "await function() { return Promise.resolve() }",
        "await (<></>)",
        "await (<a></a>)",
        "await 0",
        "await 1",
        "await \"\"",
        "await \"string\"",
        "await true",
        "await false",
        "await null",
        "await 0n",
        "await 1n",
        "await `${Promise.resolve()}`",
        "await !Promise.resolve()",
        "await void Promise.resolve()",
        "await +Promise.resolve()",
        "await ~1",
        "await ++foo",
        "await foo--",
        "await (Promise.resolve(), 1)",
        "async function foo() {+await +1}",
        "async function foo() {-await-1}",
        "async function foo() {+await -1}",
        // https://github.com/oxc-project/oxc/issues/1718
        "await await this.assertTotalDocumentCount(expectedFormattedTotalDocCount);",
        "async function foo() {+await ++a}",
        "async function foo() {-await --a}",
    ];

    let fix = vec![
        ("await []", "[]"),
        ("await (a == b)", "(a == b)"),
        ("+await -1", "+-1"),
        ("-await +1", "-+1"),
        ("await function() {}", "await function() {}"), // no autofix
        ("await class {}", "await class {}"),           // no autofix
        ("+await +1", "+await +1"),                     // no autofix
        ("-await -1", "-await -1"),                     // no autofix
        ("+await ++a", "+await ++a"), // no autofix: `+++a` would be a syntax error
        ("-await --a", "-await --a"), // no autofix: `---a` would be a syntax error
        ("+await --a", "+--a"),       // safe: `+--a` parses as `+(--a)`
        ("-await ++a", "-++a"),       // safe: `-++a` parses as `-(++a)`
    ];

    Tester::new(NoUnnecessaryAwait::NAME, NoUnnecessaryAwait::PLUGIN, pass, fail)
        .change_rule_path_extension("mjs")
        .expect_fix(fix)
        .test_and_snapshot();
}
