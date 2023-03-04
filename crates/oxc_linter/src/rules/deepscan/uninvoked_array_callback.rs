use oxc_ast::{
    ast::{Argument, Expression, MemberExpression},
    AstKind, GetSpan, Span,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("Uninvoked array callback")]
#[diagnostic(
    severity(warning),
    help("consider filling the array with `undefined` values using `Array.prototype.fill()`")
)]
struct UninvokedArrayCallbackDiagnostic(
    #[label("this callback function will not be invoked")] Span,
    #[label("because this is an array with only empty slots")] Span,
);

/// `https://deepscan.io/docs/rules/uninvoked-array-callback`
#[derive(Debug, Default, Clone)]
pub struct UninvokedArrayCallback;

declare_oxc_lint!(
    /// ### What it does
    /// This rule applies when the Array functions having callback argument are used for an array with empty slots.
    ///
    /// ### Why is this bad?
    /// When the Array constructor is called with a single number argument, an array with the specified number of empty slots (not actual undefined values) is constructed.
    /// If a callback function is passed to the function of this array, the callback function is never invoked because the array has no actual elements.
    ///
    ///
    /// ### Example
    /// ```javascript
    ///   const list = new Array(5).map(_ => createElement());
    /// ```
    UninvokedArrayCallback
);

impl Rule for UninvokedArrayCallback {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let new_expr = if let AstKind::NewExpression(new_expr) = node.get().kind()
            && new_expr.callee.is_specific_id("Array")
            && new_expr.arguments.len() == 1
            && let Some(Argument::Expression(arg_expr)) = new_expr.arguments.iter().next()
            && matches!(arg_expr, Expression::NumberLiteral(_))
            { new_expr } else { return };

        let Some(member_expr_node) = ctx.parent_node(node) else { return };

        let AstKind::MemberExpression(member_expr) = member_expr_node.get().kind() else {
            return
        };

        if let AstKind::CallExpression(call_expr) = ctx.parent_kind(member_expr_node)
            && let Some(argument) = call_expr.arguments.iter().next()
            && let Argument::Expression(arg_expr) = argument
            && arg_expr.is_function() {
            let property_span = match member_expr {
                MemberExpression::ComputedMemberExpression(expr) => expr.expression.span(),
                MemberExpression::StaticMemberExpression(expr) => expr.property.span,
                MemberExpression::PrivateFieldExpression(expr) => expr.field.span,
            };
            ctx.diagnostic(UninvokedArrayCallbackDiagnostic(
                property_span,
                new_expr.span,
            ));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("const list = new Array(5).fill().map(_ => {})", None),
        ("const list = new Array(5).flat()", None),
        ("const list = new Array(5).concat()", None),
        ("const list = new Array('x').forEach((x) => console.log(x))", None),
        ("const list = new Array(1, 2).forEach((x) => console.log(x))", None),
        ("const list = new Array(...[1, 2, 3]).forEach((x) => console.log(x))", None),
    ];

    let fail = vec![
        ("const list = new Array(5).map(_ => {})", None),
        ("const list = new Array(5).filter(function(_) {})", None),
        ("const list = new Array(5)['every'](function(_) {})", None),
    ];

    Tester::new(UninvokedArrayCallback::NAME, pass, fail).test_and_snapshot();
}
