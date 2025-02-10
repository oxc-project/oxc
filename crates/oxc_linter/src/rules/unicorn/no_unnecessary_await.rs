use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_unnecessary_await_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected `await` on a non-Promise value")
        .with_help("Consider removing the `await`")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnnecessaryAwait;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow awaiting on non-promise values.
    ///
    /// ### Why is this bad?
    /// The `await` operator should only be used on `Promise` values.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// async function bad() {
    ///     await await promise;
    /// }
    /// ```
    NoUnnecessaryAwait,
    unicorn,
    correctness,
    conditional_fix
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
                // `+await +1` -> `++1`
                ctx.nodes().parent_node(node.id()).is_some_and(|parent| {
                    if let (
                        AstKind::UnaryExpression(parent_unary),
                        Expression::UnaryExpression(inner_unary),
                    ) = (parent.kind(), &expr.argument)
                    {
                        parent_unary.operator == inner_unary.operator
                    } else {
                        false
                    }
                })
            } {
                ctx.diagnostic(no_unnecessary_await_diagnostic(Span::new(
                    expr.span.start,
                    expr.span.start + 5,
                )));
            } else {
                ctx.diagnostic_with_fix(
                    no_unnecessary_await_diagnostic(Span::new(
                        expr.span.start,
                        expr.span.start + 5,
                    )),
                    |fixer| fixer.replace(expr.span, fixer.source_range(expr.argument.span())),
                );
            };
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
        ("await {then}", None),
        ("await a ? b : c", None),
        ("await a || b", None),
        ("await a && b", None),
        ("await a ?? b", None),
        ("await new Foo()", None),
        ("await tagged``", None),
        ("class A { async foo() { await this }}", None),
        ("async function * foo() {await (yield bar);}", None),
        ("await (1, Promise.resolve())", None),
    ];

    let fail = vec![
        ("await []", None),
        ("await [Promise.resolve()]", None),
        ("await (() => {})", None),
        ("await (() => Promise.resolve())", None),
        ("await (a === b)", None),
        ("await (a instanceof Promise)", None),
        ("await (a > b)", None),
        ("await class {}", None),
        ("await class extends Promise {}", None),
        ("await function() {}", None),
        ("await function name() {}", None),
        ("await function() { return Promise.resolve() }", None),
        ("await (<></>)", None),
        ("await (<a></a>)", None),
        ("await 0", None),
        ("await 1", None),
        ("await \"\"", None),
        ("await \"string\"", None),
        ("await true", None),
        ("await false", None),
        ("await null", None),
        ("await 0n", None),
        ("await 1n", None),
        ("await `${Promise.resolve()}`", None),
        ("await !Promise.resolve()", None),
        ("await void Promise.resolve()", None),
        ("await +Promise.resolve()", None),
        ("await ~1", None),
        ("await ++foo", None),
        ("await foo--", None),
        ("await (Promise.resolve(), 1)", None),
        ("async function foo() {+await +1}", None),
        ("async function foo() {-await-1}", None),
        ("async function foo() {+await -1}", None),
        // https://github.com/oxc-project/oxc/issues/1718
        ("await await this.assertTotalDocumentCount(expectedFormattedTotalDocCount);", None),
    ];

    let fix = vec![
        ("await []", "[]", None),
        ("await (a == b)", "(a == b)", None),
        ("+await -1", "+-1", None),
        ("-await +1", "-+1", None),
        ("await function() {}", "await function() {}", None), // no autofix
        ("await class {}", "await class {}", None),           // no autofix
        ("+await +1", "+await +1", None),                     // no autofix
        ("-await -1", "-await -1", None),                     // no autofix
    ];

    Tester::new(NoUnnecessaryAwait::NAME, NoUnnecessaryAwait::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
