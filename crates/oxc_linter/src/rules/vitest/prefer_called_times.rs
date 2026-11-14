use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{ParsedExpectFnCall, PossibleJestNode, parse_expect_jest_fn_call},
};
use oxc_ast::{AstKind, ast::CallExpression};

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
    conditional_fix,
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

        ctx.diagnostic_with_fix(prefer_called_times_diagnostic(call_expr.span), |fixer| {
            if !call_expr.arguments.is_empty() {
                return fixer.noop();
            }

            let method_text = if matcher.is_name_equal("toBeCalledOnce") {
                "toBeCalledTimes"
            } else {
                "toHaveBeenCalledTimes"
            };

            let replacement = match fixer.source_range(matcher.span).chars().next() {
                Some(quote @ ('\'' | '"' | '`')) => format!("{quote}{method_text}{quote}"),
                _ => method_text.to_string(),
            };
            let fixer = fixer.for_multifix();
            let mut fix = fixer.new_fix_with_capacity(2);
            fix.push(fixer.replace(matcher.span, replacement));
            fix.push(fixer.insert_text_before_range(Span::empty(call_expr.span.end - 1), "1"));
            fix.with_message(format!("Replace with `{method_text}(1)`."))
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
            "import { expect as check } from 'vitest'; check(fn, message()).toHaveBeenCalledOnce();",
            "import { expect as check } from 'vitest'; check(fn, message()).toHaveBeenCalledTimes(1);",
        ),
        ("expect(fn)['not'].toBeCalledOnce();", "expect(fn)['not'].toBeCalledTimes(1);"),
        ("expect(fn)['toHaveBeenCalledOnce']();", "expect(fn)['toHaveBeenCalledTimes'](1);"),
        (
            "expect(fn)[`toBeCalledOnce`](/* once */);",
            "expect(fn)[`toBeCalledTimes`](/* once */1);",
        ),
        ("expect(fn).toBeCalledOnce(sideEffect());", "expect(fn).toBeCalledOnce(sideEffect());"),
    ];

    Tester::new(PreferCalledTimes::NAME, PreferCalledTimes::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
