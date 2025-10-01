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

fn no_unnecessary_array_splice_count_diagnostic(span: Span, arg_str: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Passing `{arg_str}` as the `deleteCount` argument is unnecessary."
    ))
    .with_help("Omit the argument to delete all elements after the start index.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnnecessaryArraySpliceCount;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows passing `.length` or `Infinity` as the `deleteCount` or `skipCount` argument of `Array#splice()` or `Array#toSpliced()`.
    ///
    /// ### Why is this bad?
    ///
    /// When calling `Array#splice(start, deleteCount)` or `Array#toSpliced(start, skipCount)`,
    /// omitting the `deleteCount` or `skipCount` argument will delete or skip all elements after `start`.
    /// Using `.length` or `Infinity` is unnecessary and makes the code more verbose.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// array.splice(1, array.length);
    /// array.splice(1, Infinity);
    /// array.splice(1, Number.POSITIVE_INFINITY);
    /// array.toSpliced(1, array.length);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// array.splice(1);
    /// array.toSpliced(1);
    /// ```
    NoUnnecessaryArraySpliceCount,
    unicorn,
    pedantic,
    fix
);

impl Rule for NoUnnecessaryArraySpliceCount {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional
            || !is_method_call(call_expr, None, Some(&["splice", "toSpliced"]), Some(2), Some(2))
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
                    no_unnecessary_array_splice_count_diagnostic(second_arg.span(), "Infinity"),
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
                        no_unnecessary_array_splice_count_diagnostic(second_arg.span(), &msg),
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
                        no_unnecessary_array_splice_count_diagnostic(second_arg.span(), &msg),
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
        "foo.splice?.(1, foo.length)",
        "foo.splice(foo.length, 1)",
        "foo.splice()",
        "foo.splice(1)",
        "foo.splice(1, foo.length - 1)",
        "foo.splice(1, foo.length, extraArgument)",
        "foo.splice(...[1], foo.length)",
        "foo.not_splice(1, foo.length)",
        "new foo.splice(1, foo.length)",
        "splice(1, foo.length)",
        "foo.splice(1, foo.notLength)",
        "foo.splice(1, length)",
        "foo[splice](1, foo.length)",
        "foo.splice(1, foo[length])",
        "foo.splice(1, bar.length)",
        "foo?.splice(1, NotInfinity)",
        "foo?.splice(1, Number.NOT_POSITIVE_INFINITY)",
        "foo?.splice(1, Not_Number.POSITIVE_INFINITY)",
        "foo?.splice(1, Number?.POSITIVE_INFINITY)",
        "foo().splice(1, foo().length)",
        "foo.toSpliced?.(1, foo.length)",
        "foo.toSpliced(foo.length, 1)",
        "foo.toSpliced()",
        "foo.toSpliced(1)",
        "foo.toSpliced(1, foo.length - 1)",
        "foo.toSpliced(1, foo.length, extraArgument)",
        "foo.toSpliced(...[1], foo.length)",
        "foo.not_toSpliced(1, foo.length)",
        "new foo.toSpliced(1, foo.length)",
        "toSpliced(1, foo.length)",
        "foo.toSpliced(1, foo.notLength)",
        "foo.toSpliced(1, length)",
        "foo[toSpliced](1, foo.length)",
        "foo.toSpliced(1, foo[length])",
        "foo.toSpliced(1, bar.length)",
        "foo?.toSpliced(1, NotInfinity)",
        "foo?.toSpliced(1, Number.NOT_POSITIVE_INFINITY)",
        "foo?.toSpliced(1, Not_Number.POSITIVE_INFINITY)",
        "foo?.toSpliced(1, Number?.POSITIVE_INFINITY)",
        "foo().toSpliced(1, foo().length)",
    ];

    let fail = vec![
        "foo.splice(1, foo.length)",
        "foo?.splice(1, foo.length)",
        "foo.splice(1, foo.length,)",
        "foo.splice(1, (( foo.length )))",
        "foo.splice(1, foo?.length)",
        "foo?.splice(1, foo?.length)",
        "foo?.splice(1, Infinity)",
        "foo?.splice(1, Number.POSITIVE_INFINITY)",
        "foo.bar.splice(1, foo.bar.length)",
        "foo.toSpliced(1, foo.length)",
        "foo?.toSpliced(1, foo.length)",
        "foo.toSpliced(1, foo.length,)",
        "foo.toSpliced(1, (( foo.length )))",
        "foo.toSpliced(1, foo?.length)",
        "foo?.toSpliced(1, foo?.length)",
        "foo?.toSpliced(1, Infinity)",
        "foo?.toSpliced(1, Number.POSITIVE_INFINITY)",
        "foo.bar.toSpliced(1, foo.bar.length)",
    ];

    let fix = vec![
        ("foo.splice(1, foo.length)", "foo.splice(1)"),
        ("foo?.splice(1, foo.length)", "foo?.splice(1)"),
        ("foo.splice(1, foo.length,)", "foo.splice(1,)"),
        ("foo.splice(1, (( foo.length )))", "foo.splice(1)"),
        ("foo.splice(1, foo?.length)", "foo.splice(1)"),
        ("foo?.splice(1, foo?.length)", "foo?.splice(1)"),
        ("foo?.splice(1, Infinity)", "foo?.splice(1)"),
        ("foo?.splice(1, Number.POSITIVE_INFINITY)", "foo?.splice(1)"),
        ("foo.bar.splice(1, foo.bar.length)", "foo.bar.splice(1)"),
        ("foo.toSpliced(1, foo.length)", "foo.toSpliced(1)"),
        ("foo?.toSpliced(1, foo.length)", "foo?.toSpliced(1)"),
        ("foo.toSpliced(1, foo.length,)", "foo.toSpliced(1,)"),
        ("foo.toSpliced(1, (( foo.length )))", "foo.toSpliced(1)"),
        ("foo.toSpliced(1, foo?.length)", "foo.toSpliced(1)"),
        ("foo?.toSpliced(1, foo?.length)", "foo?.toSpliced(1)"),
        ("foo?.toSpliced(1, Infinity)", "foo?.toSpliced(1)"),
        ("foo?.toSpliced(1, Number.POSITIVE_INFINITY)", "foo?.toSpliced(1)"),
        ("foo.bar.toSpliced(1, foo.bar.length)", "foo.bar.toSpliced(1)"),
    ];

    Tester::new(
        NoUnnecessaryArraySpliceCount::NAME,
        NoUnnecessaryArraySpliceCount::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
