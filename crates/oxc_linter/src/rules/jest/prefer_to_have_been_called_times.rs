use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    fixer::RuleFixer,
    rule::Rule,
    utils::{ParsedExpectFnCall, PossibleJestNode, parse_expect_jest_fn_call},
};
use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, MemberExpression},
    match_member_expression,
};

fn prefer_to_have_been_called_times_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Prefer `toHaveBeenCalledTimes()` over `toHaveLength()` when asserting mock call counts",
    )
    .with_help(
        "Use `toHaveBeenCalledTimes()` to assert the number of times a mock function was called",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferToHaveBeenCalledTimes;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// In order to have a better failure message, [`toHaveBeenCalledTimes` should be used
    /// instead of directly checking the length of `mock.calls`](https://github.com/jest-community/eslint-plugin-jest/blob/v29.5.0/docs/rules/prefer-to-have-been-called-times.md).
    ///
    /// ### Why is this bad?
    ///
    /// This rule triggers a warning if `toHaveLength` is used to assert the number of times a mock is called.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// expect(someFunction.mock.calls).toHaveLength(1);
    /// expect(someFunction.mock.calls).toHaveLength(0);
    /// expect(someFunction.mock.calls).not.toHaveLength(1);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// expect(someFunction).toHaveBeenCalledTimes(1);
    /// expect(someFunction).toHaveBeenCalledTimes(0);
    /// expect(someFunction).not.toHaveBeenCalledTimes(0);
    /// expect(uncalledFunction).not.toBeCalled();
    /// expect(method.mock.calls[0][0]).toStrictEqual(value);
    /// ```
    PreferToHaveBeenCalledTimes,
    jest,
    style,
    fix,
);

impl Rule for PreferToHaveBeenCalledTimes {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }
}
impl PreferToHaveBeenCalledTimes {
    fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(parsed_expect_call) =
            parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        Self::check_and_fix(&parsed_expect_call, call_expr, ctx);
    }
    fn check_and_fix<'a>(
        parsed_expect_call: &ParsedExpectFnCall<'a>,
        call_expr: &CallExpression<'a>,
        ctx: &LintContext<'a>,
    ) {
        let Some(matcher) = parsed_expect_call.matcher() else {
            return;
        };

        let is_wanted_matcher = matcher.is_name_equal("toHaveLength");
        if !is_wanted_matcher {
            return;
        }

        let matcher_argument = parsed_expect_call.matcher_arguments.and_then(|args| args.first());
        if matcher_argument.is_none() {
            return;
        }

        let expect_argument = parsed_expect_call.expect_arguments.and_then(|args| args.first());

        let expect_argument_mem_expr =
            expect_argument.and_then(|arg| arg.as_expression()).and_then(|arg| match arg {
                expr @ match_member_expression!(Expression) => Some(expr.to_member_expression()),
                _ => None,
            });

        let is_expect_argument_mock_calls = expect_argument_mem_expr.is_some_and(|mem_expr| {
            let is_last_member_calls = mem_expr.static_property_name() == Some("calls");

            let is_reversed_second_member_mock = match mem_expr.object() {
                expr_inner @ match_member_expression!(Expression) => {
                    let inner_mem_expr = expr_inner.to_member_expression();
                    inner_mem_expr.static_property_name() == Some("mock")
                }
                _ => false,
            };

            is_last_member_calls && is_reversed_second_member_mock
        });

        let should_fix =
            matcher_argument.is_some() && is_expect_argument_mock_calls && is_wanted_matcher;

        if !should_fix {
            return;
        }

        ctx.diagnostic_with_fix(
            prefer_to_have_been_called_times_diagnostic(call_expr.span),
            |fixer| {
                let matcher_arg_text = if let Some(arg) = matcher_argument {
                    fixer.source_range(arg.span())
                } else {
                    return fixer.noop();
                };

                let param_text = Self::build_expect_argument(expect_argument_mem_expr, fixer);

                let modifier_text = parsed_expect_call.modifiers().iter().fold(
                    String::new(),
                    |mut acc, modifier| {
                        use std::fmt::Write;
                        write!(&mut acc, ".{}", fixer.source_range(modifier.span)).unwrap();
                        acc
                    },
                );

                let method_text = "toHaveBeenCalledTimes";

                let code = format!(
                    "expect({param_text}){modifier_text}.{method_text}({matcher_arg_text})"
                );

                fixer.replace(call_expr.span, code)
            },
        );
    }
    fn build_expect_argument<'a>(
        expect_argument_mem_expr: Option<&MemberExpression<'_>>,
        fixer: RuleFixer<'_, 'a>,
    ) -> &'a str {
        if let Some(mem_expr) = expect_argument_mem_expr
            && mem_expr.static_property_name().unwrap().eq("calls")
            && let Some(expr) = mem_expr.object().as_member_expression()
            && expr.static_property_name() == Some("mock")
        {
            return fixer.source_range(expr.object().span());
        }
        ""
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "expect.assertions(1)",
        "expect(fn).toHaveBeenCalledTimes",
        "expect(fn.mock.calls).toHaveLength",
        "expect(fn.mock.values).toHaveLength(0)",
        "expect(fn.values.calls).toHaveLength(0)",
        "expect(fn).toHaveBeenCalledTimes(0)",
        "expect(fn).resolves.toHaveBeenCalledTimes(10)",
        "expect(fn).not.toHaveBeenCalledTimes(10)",
        "expect(fn).toHaveBeenCalledTimes(1)",
        "expect(fn).toBeCalledTimes(0);",
        "expect(fn).toHaveBeenCalledTimes(0);",
        "expect(fn);",
        "expect(method.mock.calls[0][0]).toStrictEqual(value);",
        "expect(fn.mock.length).toEqual(1);",
        "expect(fn.mock.calls).toEqual([]);",
        "expect(fn.mock.calls).toContain(1, 2, 3);",
    ];

    let fail = vec![
        "expect(method.mock.calls).toHaveLength(1);",
        "expect(method.mock.calls).resolves.toHaveLength(x);",
        r#"expect(method["mock"].calls).toHaveLength(0);"#,
        "expect(my.method.mock.calls).not.toHaveLength(0);",
    ];

    let fix = vec![
        (
            "expect(method.mock.calls).toHaveLength(1);",
            "expect(method).toHaveBeenCalledTimes(1);",
            None,
        ),
        (
            "expect(method.mock.calls).toHaveLength(
                1,
            );",
            "expect(method).toHaveBeenCalledTimes(1);",
            None,
        ),
        (
            "expect(method.mock.calls).toHaveLength(
                /* number of calls (one) */
                1,
            );",
            "expect(method).toHaveBeenCalledTimes(1);",
            None,
        ),
        (
            "expect(method.mock.calls).resolves.toHaveLength(x);",
            "expect(method).resolves.toHaveBeenCalledTimes(x);",
            None,
        ),
        (
            r#"expect(method["mock"].calls).toHaveLength(0);"#,
            "expect(method).toHaveBeenCalledTimes(0);",
            None,
        ),
        (
            "expect(my.method.mock.calls).not.toHaveLength(0);",
            "expect(my.method).not.toHaveBeenCalledTimes(0);",
            None,
        ),
    ];
    Tester::new(PreferToHaveBeenCalledTimes::NAME, PreferToHaveBeenCalledTimes::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
