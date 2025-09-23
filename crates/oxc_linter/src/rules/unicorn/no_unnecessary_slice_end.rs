use std::borrow::Cow;

use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, MemberExpression, StaticMemberExpression},
    match_member_expression,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode, ast_util::is_method_call, context::LintContext, rule::Rule, utils::is_same_expression,
};

fn no_unnecessary_slice_end_diagnostic(span: Span, arg_str: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Passing `{arg_str}` as the `end` argument is unnecessary."))
        .with_help("Consider omitting the unnecessary end argument.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnnecessarySliceEnd;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Omitting the end argument defaults it to the object's .length.
    /// Passing it explicitly or using Infinity is unnecessary
    ///
    /// ### Why is this bad?
    ///
    /// In JavaScript, omitting the end index already causes .slice() to run to the end of the target,
    /// so explicitly passing its length or Infinity is redundant.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const foo = string.slice(1, string.length);
    /// const foo = string.slice(1, Infinity);
    /// const foo = string.slice(1, Number.POSITIVE_INFINITY);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const foo = string.slice(1);
    /// ```
    NoUnnecessarySliceEnd,
    unicorn,
    pedantic,
    fix,
);

impl Rule for NoUnnecessarySliceEnd {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        // ignore optional chaining, e.g. "foo.slice?.()"
        if call_expr.optional
            || !is_method_call(call_expr, None, Some(&["slice"]), Some(2), Some(2))
        {
            return;
        }
        let Some(MemberExpression::StaticMemberExpression(member_expr)) =
            call_expr.callee.as_member_expression()
        else {
            return;
        };
        if matches!(member_expr.object, Expression::CallExpression(_))
            || call_expr.arguments.iter().any(|arg| matches!(arg, Argument::SpreadElement(_)))
        {
            return;
        }
        let [first_arg, second_arg] = call_expr.arguments.as_slice() else {
            return;
        };
        let Some(arg_expr) = second_arg.as_expression().map(Expression::without_parentheses) else {
            return;
        };
        match arg_expr {
            Expression::Identifier(ident) if ident.name.as_str() == "Infinity" => {
                ctx.diagnostic_with_fix(
                    no_unnecessary_slice_end_diagnostic(second_arg.span(), "Infinity"),
                    |fixer| {
                        fixer.delete_range(Span::new(first_arg.span().end, second_arg.span().end))
                    },
                );
            }
            Expression::ChainExpression(chain_expr) => {
                if let Some(expr) = chain_expr.expression.as_member_expression()
                    && let Some(msg) =
                        check_expression_and_get_diagnostic(member_expr, expr, true, ctx)
                {
                    ctx.diagnostic_with_fix(
                        no_unnecessary_slice_end_diagnostic(second_arg.span(), &msg),
                        |fixer| {
                            fixer.delete_range(Span::new(
                                first_arg.span().end,
                                second_arg.span().end,
                            ))
                        },
                    );
                }
            }
            match_member_expression!(Expression) => {
                let expr = arg_expr.to_member_expression();
                if let Some(msg) =
                    check_expression_and_get_diagnostic(member_expr, expr, false, ctx)
                {
                    ctx.diagnostic_with_fix(
                        no_unnecessary_slice_end_diagnostic(second_arg.span(), &msg),
                        |fixer| {
                            fixer.delete_range(Span::new(
                                first_arg.span().end,
                                second_arg.span().end,
                            ))
                        },
                    );
                }
            }
            _ => {}
        }
    }
}

fn check_expression_and_get_diagnostic<'a>(
    left: &StaticMemberExpression,
    right: &MemberExpression,
    is_chain_expr: bool,
    ctx: &'a LintContext,
) -> Option<Cow<'a, str>> {
    let MemberExpression::StaticMemberExpression(right) = right else {
        return None;
    };
    let property = right.property.name.as_str();
    if ctx.source_range(right.span()) == "Number.POSITIVE_INFINITY" {
        return Some("Number.POSITIVE_INFINITY".into());
    }
    if property == "length" && is_same_expression(&left.object, &right.object, ctx) {
        if matches!(left.object, Expression::Identifier(_)) {
            return Some(ctx.source_range(right.span()).into());
        }
        let operator = if is_chain_expr { "?." } else { "." };
        return Some(format!("…{operator}length").into());
    }
    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("const foo = string.slice(1)"),
        ("foo.slice?.(1, foo.length)"),
        ("foo.slice()"),
        ("foo.slice(1)"),
        ("foo.slice()"),
        ("foo.slice(1, foo.length - 1)"),
        ("foo.slice(1, foo.length, extraArgument)"),
        ("foo.slice(...[1], foo.length)"),
        ("foo.not_slice(1, foo.length)"),
        ("new foo.slice(1, foo.length)"),
        ("slice(1, foo.length)"),
        ("foo.slice(1, foo.notLength)"),
        ("foo.slice(1, length)"),
        ("foo[slice](1, foo.length)"),
        ("foo.slice(1, foo[length])"),
        ("foo.slice(1, bar.length)"),
        ("foo?.slice(1, NotInfinity)"),
        ("foo?.slice(1, Number.NOT_POSITIVE_INFINITY)"),
        ("foo?.slice(1, Not_Number.POSITIVE_INFINITY)"),
        ("foo?.slice(1, Number?.POSITIVE_INFINITY)"),
        ("foo().slice(1, foo().length)"),
    ];

    let fail = vec![
        ("const foo = string.slice(1, string.length)"),
        ("a[b].slice(1, a[b].length)"),
        ("a?.[b].slice(1, a[b]?.length)"),
        ("foo.slice(1, foo.length,)"),
        ("foo.slice(1, (( foo.length )))"),
        ("foo.slice(1, foo?.length)"),
        ("foo?.slice(1, foo?.length)"),
        ("foo?.slice(1, Infinity)"),
        ("foo?.slice(1, Number.POSITIVE_INFINITY)"),
        ("foo.bar.slice(1, foo.bar.length)"),
        ("foo?.slice(1, (Number.POSITIVE_INFINITY))"),
        ("a?.b?.slice(1, a.b.length)"),
        ("a?.b?.slice(1, a?.b.length)"),
        ("a?.slice(1, a?.length)"),
        ("Array.prototype.slice(1, Infinity)"),
    ];

    let fix = vec![
        ("foo.slice(1, foo.length)", "foo.slice(1)"),
        ("a[b].slice(1, a[b].length)", "a[b].slice(1)"),
        ("foo.slice(1, foo.length,)", "foo.slice(1,)"),
        ("foo.slice(1, (( foo.length )))", "foo.slice(1)"),
        ("foo.slice(1, foo?.length)", "foo.slice(1)"),
        ("foo?.slice(1, foo?.length)", "foo?.slice(1)"),
        ("foo?.slice(1, Infinity)", "foo?.slice(1)"),
        ("foo?.slice(1, Number.POSITIVE_INFINITY)", "foo?.slice(1)"),
        ("foo.bar.slice(1, foo.bar.length)", "foo.bar.slice(1)"),
        ("foo?.slice(1, (Number.POSITIVE_INFINITY))", "foo?.slice(1)"),
        ("a?.slice(1, a?.length /** comments */)", "a?.slice(1 /** comments */)"),
    ];

    Tester::new(NoUnnecessarySliceEnd::NAME, NoUnnecessarySliceEnd::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
