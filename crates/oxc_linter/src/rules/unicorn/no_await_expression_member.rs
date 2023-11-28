use oxc_ast::{
    ast::{Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-await-expression-member): Disallow member access from await expression")]
#[diagnostic(severity(warning), help("Do not access a member directly from an await expression."))]
struct NoAwaitExpressionMemberDiagnostic(#[label] pub Span);

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
    correctness
);

impl Rule for NoAwaitExpressionMember {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MemberExpression(member_expr) = node.kind() else { return };

        let Expression::ParenthesizedExpression(paren_expr) = member_expr.object() else {
            return;
        };

        if matches!(paren_expr.expression, Expression::AwaitExpression(_)) {
            let node_span = match member_expr {
                MemberExpression::ComputedMemberExpression(expr) => expr.span,
                MemberExpression::PrivateFieldExpression(expr) => expr.span,
                MemberExpression::StaticMemberExpression(expr) => expr.span,
            };

            ctx.diagnostic(NoAwaitExpressionMemberDiagnostic(node_span));
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
    ];

    Tester::new(NoAwaitExpressionMember::NAME, pass, fail).test_and_snapshot();
}
