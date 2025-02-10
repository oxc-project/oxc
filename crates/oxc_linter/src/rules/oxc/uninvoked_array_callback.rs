use oxc_ast::{
    ast::{Argument, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn uninvoked_array_callback_diagnostic(cb_span: Span, arr_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Uninvoked array callback")
        .with_help(
            "consider filling the array with `undefined` values using `Array.prototype.fill()`",
        )
        .with_labels([
            cb_span.label("this callback will not be invoked"),
            arr_span.label("because this is an array with only empty slots"),
        ])
}

#[derive(Debug, Default, Clone)]
pub struct UninvokedArrayCallback;

declare_oxc_lint!(
    /// ### What it does
    /// This rule applies when an Array function has a callback argument used for an array with empty slots.
    ///
    /// ### Why is this bad?
    /// When the Array constructor is called with a single number argument, an array with the specified number of empty slots (not actual undefined values) is constructed.
    /// If a callback function is passed to the function of this array, the callback function is never invoked because the array has no actual elements.
    ///
    /// ### Example
    /// ```javascript
    ///   const list = new Array(5).map(_ => createElement());
    /// ```
    UninvokedArrayCallback,
    oxc,
    correctness
);

impl Rule for UninvokedArrayCallback {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };
        if !new_expr.callee.is_specific_id("Array") {
            return;
        }
        if new_expr.arguments.len() != 1 {
            return;
        }
        if !matches!(new_expr.arguments.first(), Some(Argument::NumericLiteral(_))) {
            return;
        }

        let Some(member_expr_node) = ctx.nodes().parent_node(node.id()) else {
            return;
        };

        let AstKind::MemberExpression(member_expr) = member_expr_node.kind() else {
            return;
        };

        let Some(AstKind::CallExpression(call_expr)) =
            ctx.nodes().parent_kind(member_expr_node.id())
        else {
            return;
        };
        if !matches!(
            call_expr.arguments.first(),
            Some(Argument::FunctionExpression(_) | Argument::ArrowFunctionExpression(_))
        ) {
            return;
        }

        let property_span = match member_expr {
            MemberExpression::ComputedMemberExpression(expr) => expr.expression.span(),
            MemberExpression::StaticMemberExpression(expr) => expr.property.span,
            MemberExpression::PrivateFieldExpression(expr) => expr.field.span,
        };
        ctx.diagnostic(uninvoked_array_callback_diagnostic(property_span, new_expr.span));
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

    Tester::new(UninvokedArrayCallback::NAME, UninvokedArrayCallback::PLUGIN, pass, fail)
        .test_and_snapshot();
}
