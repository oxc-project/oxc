use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_await_expression_member_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint-plugin-unicorn(no-await-expression-member): Disallow member access from await expression")
        .with_help("When accessing a member from an await expression, the await expression has to be parenthesized, which is not readable.")
        .with_labels([span0.into()])
}

#[derive(Debug, Default, Clone)]
pub struct NoAwaitExpressionMember;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows member access from await expression
    ///
    /// ### Why is this bad?
    ///
    /// When accessing a member from an await expression,
    /// the await expression has to be parenthesized, which is not readable.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// const secondElement = (await getArray())[1];
    ///
    /// // Good
    /// const [, secondElement] = await getArray();
    /// ```
    NoAwaitExpressionMember,
    style
);

impl Rule for NoAwaitExpressionMember {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MemberExpression(member_expr) = node.kind() else {
            return;
        };

        let Expression::ParenthesizedExpression(paren_expr) = member_expr.object() else {
            return;
        };

        if matches!(paren_expr.expression, Expression::AwaitExpression(_)) {
            let node_span = member_expr.span();
            ctx.diagnostic(no_await_expression_member_diagnostic(node_span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"const foo = await promise", None),
        (r"const {foo: bar} = await promise", None),
        (r"const foo = !await promise", None),
        (r"const foo = typeof await promise", None),
        (r"const foo = await notPromise.method()", None),
        (r"const foo = foo[await promise]", None),
        // These await expression need parenthesized, but rarely used
        (r"new (await promiseReturnsAClass)", None),
        (r"(await promiseReturnsAFunction)()", None),
        // typescript
        (r"async function foo () {return (await promise) as string;}", None),
    ];

    let fail = vec![
        (r"(await promise)[0]", None),
        (r"(await promise).property", None),
        (r"const foo = (await promise).bar()", None),
        (r"const foo = (await promise).bar?.()", None),
        (r"const foo = (await promise)?.bar()", None),
        (r"const firstElement = (await getArray())[0]", None),
        (r"const secondElement = (await getArray())[1]", None),
        (r"const thirdElement = (await getArray())[2]", None),
        (r"const optionalFirstElement = (await getArray())?.[0]", None),
        (r"const {propertyOfFirstElement} = (await getArray())[0]", None),
        (r"const [firstElementOfFirstElement] = (await getArray())[0]", None),
        (r"let foo, firstElement = (await getArray())[0]", None),
        (r"var firstElement = (await getArray())[0], bar", None),
        (r"const property = (await getObject()).property", None),
        (r"const renamed = (await getObject()).property", None),
        (r"const property = (await getObject())[property]", None),
        (r"const property = (await getObject())?.property", None),
        (r"const {propertyOfProperty} = (await getObject()).property", None),
        (r"const {propertyOfProperty} = (await getObject()).propertyOfProperty", None),
        (r"const [firstElementOfProperty] = (await getObject()).property", None),
        (r"const [firstElementOfProperty] = (await getObject()).firstElementOfProperty", None),
        (r"firstElement = (await getArray())[0]", None),
        (r"property = (await getArray()).property", None),
        // typescript
        (r"const foo: Type = (await promise)[0]", None),
        (r"const foo: Type | A = (await promise).foo", None),
    ];

    Tester::new(NoAwaitExpressionMember::NAME, pass, fail).test_and_snapshot();
}
