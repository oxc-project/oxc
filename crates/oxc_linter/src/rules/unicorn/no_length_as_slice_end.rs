use oxc_ast::{ast::MemberExpression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    ast_util::is_method_call, context::LintContext, rule::Rule, utils::is_same_expression, AstNode,
};

fn no_length_as_slice_end_diagnostic(call_span: Span, arg_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Passing `length` as the end argument of a `slice` call is unnecessary.")
        .with_help("Remove the second argument.")
        .with_labels([
            call_span.label("`.slice` called here."),
            arg_span.label("Invalid argument here"),
        ])
}

#[derive(Debug, Default, Clone)]
pub struct NoLengthAsSliceEnd;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow using `length` as the end argument of a `slice` call.
    ///
    /// ### Why is this bad?
    ///
    /// Passing `length` as the end argument of a `slice` call is unnecessary and can be confusing.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// foo.slice(1, foo.length)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// foo.slice(1)
    /// ```
    NoLengthAsSliceEnd,
    unicorn,
    restriction,
    fix
);

impl Rule for NoLengthAsSliceEnd {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_method_call(call_expr, None, Some(&["slice"]), Some(2), Some(2))
            || call_expr.optional
        {
            return;
        }

        if call_expr.arguments.iter().any(oxc_ast::ast::Argument::is_spread) {
            return;
        }

        let Some(MemberExpression::StaticMemberExpression(second_argument)) = call_expr.arguments
            [1]
        .as_expression()
        .map(oxc_ast::ast::Expression::without_parentheses)
        .and_then(|e| e.get_member_expr()) else {
            return;
        };

        if second_argument.property.name != "length" {
            return;
        }

        if !is_same_expression(
            call_expr.callee.as_member_expression().unwrap().object(),
            &second_argument.object,
            ctx,
        ) {
            return;
        }

        ctx.diagnostic_with_fix(
            no_length_as_slice_end_diagnostic(
                call_expr.callee.get_member_expr().unwrap().static_property_info().unwrap().0,
                second_argument.span,
            ),
            |fixer| {
                let start = call_expr.arguments[0].span().end;
                let end = call_expr.arguments[1].span().end;
                let span = Span::new(start, end);
                fixer.delete(&span)
            },
        );
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo.slice?.(1, foo.length)",
        "foo.slice(foo.length, 1)",
        "foo.slice()",
        "foo.slice(1)",
        "foo.slice(1, foo.length - 1)",
        "foo.slice(1, foo.length, extraArgument)",
        "foo.slice(...[1], foo.length)",
        "foo.notSlice(1, foo.length)",
        "new foo.slice(1, foo.length)",
        "slice(1, foo.length)",
        "foo.slice(1, foo.notLength)",
        "foo.slice(1, length)",
        "foo[slice](1, foo.length)",
        "foo.slice(1, foo[length])",
        "foo.slice(1, bar.length)",
        "foo().slice(1, foo().length)",
    ];

    let fail = vec![
        "foo.slice(1, foo.length)",
        "foo?.slice(1, foo.length)",
        "foo.slice(1, foo.length,)",
        "foo.slice(1, (( foo.length )))",
        "foo.slice(1, foo?.length)",
        "foo?.slice(1, foo?.length)",
    ];

    let fix = vec![
        ("foo.slice(1, foo.length)", "foo.slice(1)"),
        ("foo?.slice(1, foo.length)", "foo?.slice(1)"),
        ("foo.slice(1, foo.length,)", "foo.slice(1,)"),
        ("foo.slice(1, (( foo.length )))", "foo.slice(1)"),
        ("foo.slice(1, foo?.length)", "foo.slice(1)"),
        ("foo?.slice(1, foo?.length)", "foo?.slice(1)"),
    ];

    Tester::new(NoLengthAsSliceEnd::NAME, NoLengthAsSliceEnd::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
