use oxc_ast::{AstKind, ast::CallExpression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{ParsedExpectFnCall, PossibleJestNode, parse_expect_jest_fn_call},
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
    version = "1.35.0",
    short_description = "This rule aims to enforce the use of `toBeCalledTimes(1)` or `toHaveBeenCalledTimes(1)` over `toBeCalledOnce()` or `toHaveBeenCalledOnce()`.",
);

impl Rule for PreferCalledTimes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(jest_node) = PossibleJestNode::from_call_expression(node, call_expr, ctx) else {
            return;
        };
        let Some(parsed_expect_call) = parse_expect_jest_fn_call(call_expr, &jest_node, ctx) else {
            return;
        };

        Self::check_and_fix(&parsed_expect_call, call_expr, ctx);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.frameworks().is_test()
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

        let Some(member_expr) = call_expr.callee.as_member_expression() else {
            return;
        };
        if member_expr.static_property_info().is_none_or(|(span, _)| span != matcher.span) {
            return;
        }

        ctx.diagnostic_with_fix(prefer_called_times_diagnostic(call_expr.span), |fixer| {
            let method_text = if matcher.is_name_equal("toBeCalledOnce") {
                "toBeCalledTimes"
            } else {
                "toHaveBeenCalledTimes"
            };

            fixer.replace(
                Span::new(member_expr.object().span().end, call_expr.span.end),
                format!(".{method_text}(1)"),
            )
        });
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
        (
            "import { expect as check } from 'vitest'; check(spy).toHaveBeenCalledOnce();",
            "import { expect as check } from 'vitest'; check(spy).toHaveBeenCalledTimes(1);",
        ),
        (
            "expect(spy, 'spy should run once').toHaveBeenCalledOnce();",
            "expect(spy, 'spy should run once').toHaveBeenCalledTimes(1);",
        ),
        (
            "import { expect as check } from 'vitest'; check(spy, 'message').resolves.not.toBeCalledOnce();",
            "import { expect as check } from 'vitest'; check(spy, 'message').resolves.not.toBeCalledTimes(1);",
        ),
        ("expect(spy)['toHaveBeenCalledOnce']();", "expect(spy).toHaveBeenCalledTimes(1);"),
        (
            "expect(spy, 'message')[`resolves`]['not'][`toBeCalledOnce`]();",
            "expect(spy, 'message')[`resolves`]['not'].toBeCalledTimes(1);",
        ),
        (
            "expect(Promise.resolve(spy)).resolves.toHaveBeenCalledOnce().then(done).catch(fail);",
            "expect(Promise.resolve(spy)).resolves.toHaveBeenCalledTimes(1).then(done).catch(fail);",
        ),
        (
            "expect(spy).resolves.toBeCalledOnce().then.call(Promise.resolve(), done);",
            "expect(spy).resolves.toBeCalledTimes(1).then.call(Promise.resolve(), done);",
        ),
    ];

    Tester::new(PreferCalledTimes::NAME, PreferCalledTimes::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
