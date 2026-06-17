use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
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
            if {
                // Removing `await` may change them to a declaration, if there is no `id` will cause SyntaxError
                matches!(expr.argument, Expression::FunctionExpression(_))
                    || matches!(expr.argument, Expression::ClassExpression(_))
            } || {
                // Removing `await` would paste the parent unary operator onto the argument's
                // leading operator and change tokenization into a syntax error:
                // `+await +1` -> `++1`, `+await ++a` -> `+++a`, `-await --a` -> `---a`.
                // Skip the fix in those cases (the diagnostic is still reported).
                let parent = ctx.nodes().parent_node(node.id());
                match (parent.kind(), &expr.argument) {
                    (
                        AstKind::UnaryExpression(parent_unary),
                        Expression::UnaryExpression(inner_unary),
                    ) => parent_unary.operator == inner_unary.operator,
                    (
                        AstKind::UnaryExpression(parent_unary),
                        Expression::UpdateExpression(inner_update),
                    ) => {
                        inner_update.prefix
                            && matches!(
                                (parent_unary.operator, inner_update.operator),
                                (UnaryOperator::UnaryPlus, UpdateOperator::Increment)
                                    | (UnaryOperator::UnaryNegation, UpdateOperator::Decrement)
                            )
                    }
                    _ => false,
                }
            } {
                ctx.diagnostic(no_unnecessary_await_diagnostic(Span::sized(expr.span.start, 5)));
            } else {
                ctx.diagnostic_with_fix(
                    no_unnecessary_await_diagnostic(Span::sized(expr.span.start, 5)),
                    |fixer| fixer.replace_with(expr, &expr.argument),
                );
            }
        }
    }
}

fn not_promise(expr: &Expression) -> bool {
    match expr {
        Expression::ArrayExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::BinaryExpression(_)
        | Expression::ClassExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::JSXElement(_)
        | Expression::JSXFragment(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::TemplateLiteral(_)
        | Expression::UnaryExpression(_)
        | Expression::UpdateExpression(_) => true,
        Expression::SequenceExpression(expr) => not_promise(expr.expressions.last().unwrap()),
        Expression::ParenthesizedExpression(expr) => not_promise(&expr.expression),
        _ => false,
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
        // `+await ++a` / `-await --a`: removing `await` would paste `+`+`++` into `+++` / `-`+`--` into `---`
        "async function foo() {+await ++a}",
        "async function foo() {-await --a}",
        // https://github.com/oxc-project/oxc/issues/1718
        "await await this.assertTotalDocumentCount(expectedFormattedTotalDocCount);",
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
