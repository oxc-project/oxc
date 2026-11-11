use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_same_expression};

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
    /// ### Examples
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
    recommended,
    fix,
    version = "0.7.0",
    short_description = "Disallow using `length` as the end argument of a `slice` call.",
);

impl Rule for NoLengthAsSliceEnd {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(call_expr) = node.kind()
            && !call_expr.optional
            && call_expr.arguments.len() == 2
            && let Some(member_expr) = call_expr.callee.get_member_expr()
            && let Some((method_span, method_name)) = member_expr.static_property_info()
            && method_name == "slice"
            && !call_expr.arguments.iter().any(Argument::is_spread)
            && let Some(MemberExpression::StaticMemberExpression(second_argument)) = call_expr
                .arguments[1]
                .as_expression()
                .map(Expression::without_parentheses)
                .and_then(|e| e.get_member_expr())
            && second_argument.property.name == "length"
            && is_same_expression(member_expr.object(), &second_argument.object, ctx)
        {
            ctx.diagnostic_with_fix(
                no_length_as_slice_end_diagnostic(method_span, second_argument.span),
                |fixer| {
                    let start = call_expr.arguments[0].span().end;
                    let end = call_expr.arguments[1].span().end;
                    let span = Span::new(start, end);
                    fixer.delete(&span)
                },
            );
        }
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
        "(a.slice)(0, b.length)",
        "(a.slice)?.(0, a.length)",
    ];

    let fail = vec![
        "foo.slice(1, foo.length)",
        "foo?.slice(1, foo.length)",
        "foo.slice(1, foo.length,)",
        "foo.slice(1, (( foo.length )))",
        "foo.slice(1, foo?.length)",
        "foo?.slice(1, foo?.length)",
        "(a.slice)(0, a.length)",
        "((a.slice))(0, a.length)",
        "(a?.slice)(0, a.length)",
        "(a[\"slice\"])(0, a.length)",
        "(a.slice!)(0, a.length)",
        "(a.slice as typeof a.slice)(0, a.length)",
    ];

    let fix = vec![
        ("foo.slice(1, foo.length)", "foo.slice(1)"),
        ("foo?.slice(1, foo.length)", "foo?.slice(1)"),
        ("foo.slice(1, foo.length,)", "foo.slice(1,)"),
        ("foo.slice(1, (( foo.length )))", "foo.slice(1)"),
        ("foo.slice(1, foo?.length)", "foo.slice(1)"),
        ("foo?.slice(1, foo?.length)", "foo?.slice(1)"),
        ("(a.slice)(0, a.length)", "(a.slice)(0)"),
        ("((a.slice))(0, a.length)", "((a.slice))(0)"),
        ("(a?.slice)(0, a.length)", "(a?.slice)(0)"),
        ("(a[\"slice\"])(0, a.length)", "(a[\"slice\"])(0)"),
        ("(a.slice!)(0, a.length)", "(a.slice!)(0)"),
        ("(a.slice as typeof a.slice)(0, a.length)", "(a.slice as typeof a.slice)(0)"),
    ];

    Tester::new(NoLengthAsSliceEnd::NAME, NoLengthAsSliceEnd::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
