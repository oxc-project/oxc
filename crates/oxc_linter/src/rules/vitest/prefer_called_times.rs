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
    ast::{Argument, CallExpression},
};

fn prefer_called_times_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `toBeCalledTimes(1)` or `toHaveBeenCalledTimes(1)` instead of `toBeCalledOnce()` or `toHaveBeenCalledOnce()`")
        .with_help("Replace with `toBeCalledTimes(1)` or `toHaveBeenCalledTimes(1)` for clarity and consistency")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferCalledTimes;

// See <https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/docs/rules/prefer-called-times.md> for rule details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule aims to enforce the use of `toBeCalledTimes(1)` or `toHaveBeenCalledTimes(1)` over `toBeCalledOnce()` or `toHaveBeenCalledOnce()`.
    ///
    /// ### Why is this bad?
    ///
    /// This rule aims to enforce the use of `toBeCalledTimes(1)` or `toHaveBeenCalledTimes(1)` over `toBeCalledOnce()` or `toHaveBeenCalledOnce()`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// test('foo', () => {
    ///   const mock = vi.fn()
    ///   mock('foo')
    ///   expect(mock).toBeCalledOnce()
    ///   expect(mock).toHaveBeenCalledOnce()
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// test('foo', () => {
    ///   const mock = vi.fn()
    ///   mock('foo')
    ///   expect(mock).toBeCalledTimes(1)
    ///   expect(mock).toHaveBeenCalledTimes(1)
    /// })
    /// ```
    PreferCalledTimes,
    vitest,
    style,
    fix,
);

impl Rule for PreferCalledTimes {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = jest_node.node;

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(parsed_expect_call) = parse_expect_jest_fn_call(call_expr, jest_node, ctx) else {
            return;
        };

        Self::check_and_fix(&parsed_expect_call, call_expr, ctx);
    }
}

impl PreferCalledTimes {
    fn check_and_fix<'a>(
        parsed_expect_call: &ParsedExpectFnCall<'a>,
        call_expr: &CallExpression<'a>,
        ctx: &LintContext<'a>,
    ) {
        let Some(matcher) = parsed_expect_call.matcher() else {
            return;
        };

        let is_wanted_matcher = matcher.is_name_equal("toBeCalledOnce")
            || matcher.is_name_equal("toHaveBeenCalledOnce");
        if !is_wanted_matcher {
            return;
        }

        let expect_argument = parsed_expect_call.expect_arguments.and_then(|args| args.first());

        ctx.diagnostic_with_fix(prefer_called_times_diagnostic(call_expr.span), |fixer| {
            let param_text = Self::build_expect_argument(expect_argument, fixer);

            let modifier_text =
                parsed_expect_call.modifiers().iter().fold(String::new(), |mut acc, modifier| {
                    use std::fmt::Write;
                    write!(&mut acc, ".{}", fixer.source_range(modifier.span)).unwrap();
                    acc
                });

            let method_text = if matcher.is_name_equal("toBeCalledOnce") {
                "toBeCalledTimes"
            } else {
                "toHaveBeenCalledTimes"
            };

            let code = format!("expect({param_text}){modifier_text}.{method_text}(1)");

            fixer.replace(call_expr.span, code)
        });
    }

    fn build_expect_argument<'a>(
        expect_argument: Option<&Argument<'_>>,
        fixer: RuleFixer<'_, 'a>,
    ) -> &'a str {
        if let Some(arg) = expect_argument {
            return fixer.source_range(arg.span());
        }
        ""
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "expect(fn).toBeCalledTimes(1);",
        "expect(fn).toHaveBeenCalledTimes(1);",
        "expect(fn).toBeCalledTimes(2);",
        "expect(fn).toHaveBeenCalledTimes(2);",
        "expect(fn).toBeCalledTimes(expect.anything());",
        "expect(fn).toHaveBeenCalledTimes(expect.anything());",
        "expect(fn).not.toBeCalledTimes(2);",
        "expect(fn).rejects.not.toBeCalledTimes(1);",
        "expect(fn).not.toHaveBeenCalledTimes(1);",
        "expect(fn).resolves.not.toHaveBeenCalledTimes(1);",
        "expect(fn).toBeCalledTimes(0);",
        "expect(fn).toHaveBeenCalledTimes(0);",
        "expect(fn);",
    ];

    let fail = vec![
        "expect(fn).toBeCalledOnce();",
        "expect(fn).toHaveBeenCalledOnce();",
        "expect(fn).not.toBeCalledOnce();",
        "expect(fn).not.toHaveBeenCalledOnce();",
        "expect(fn).resolves.toBeCalledOnce();",
        "expect(fn).resolves.toHaveBeenCalledOnce();",
    ];

    let fix = vec![
        ("expect(fn).toBeCalledOnce();", "expect(fn).toBeCalledTimes(1);"),
        ("expect(fn).toHaveBeenCalledOnce();", "expect(fn).toHaveBeenCalledTimes(1);"),
        ("expect(fn).not.toBeCalledOnce();", "expect(fn).not.toBeCalledTimes(1);"),
        ("expect(fn).not.toHaveBeenCalledOnce();", "expect(fn).not.toHaveBeenCalledTimes(1);"),
        ("expect(fn).resolves.toBeCalledOnce();", "expect(fn).resolves.toBeCalledTimes(1);"),
        (
            "expect(fn).resolves.toHaveBeenCalledOnce();",
            "expect(fn).resolves.toHaveBeenCalledTimes(1);",
        ),
    ];

    Tester::new(PreferCalledTimes::NAME, PreferCalledTimes::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
