use oxc_ast::{AstKind, ast::Argument};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{PossibleJestNode, parse_expect_jest_fn_call},
};

fn prefer_called_once_diagnostic(span: Span, new_matcher_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "The use of `toBeCalledTimes(1)` and `toHaveBeenCalledTimes(1)` is discouraged.",
    )
    .with_help(format!("Prefer `{new_matcher_name}()`."))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferCalledOnce;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Substitute `toBeCalledTimes(1)` and `toHaveBeenCalledTimes(1)` with
    /// `toBeCalledOnce()` and `toHaveBeenCalledOnce()` respectively.
    ///
    /// ### Why is this bad?
    ///
    /// The *Times method required to read the arguments to know how many times
    /// is expected a spy to be called. Most of the times you expecting a method is called
    /// once.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// test('foo', () => {
    ///   const mock = vi.fn()
    ///   mock('foo')
    ///   expect(mock).toBeCalledTimes(1)
    ///   expect(mock).toHaveBeenCalledTimes(1)
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// test('foo', () => {
    ///   const mock = vi.fn()
    ///   mock('foo')
    ///   expect(mock).toBeCalledOnce()
    ///   expect(mock).toHaveBeenCalledOnce()
    /// })
    /// ```
    PreferCalledOnce,
    vitest,
    style,
    fix,
);

impl Rule for PreferCalledOnce {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &crate::utils::PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }
}

impl PreferCalledOnce {
    fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(parsed_expect) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        if parsed_expect.matcher_arguments.is_some_and(|arguments| arguments.len() != 1) {
            return;
        }

        let Some(Argument::NumericLiteral(called_times_value)) =
            parsed_expect.matcher_arguments.and_then(|arguments| arguments.first())
        else {
            return;
        };

        let Some(matcher_to_be_fixed) = parsed_expect.members.iter().find(|member| {
            member.is_name_equal("toBeCalledTimes") || member.is_name_equal("toHaveBeenCalledTimes")
        }) else {
            return;
        };

        if called_times_value.raw.is_some_and(|value| value.as_ref() == "1") {
            let new_matcher_name = {
                let span_matcher_without_suffix =
                    Span::new(matcher_to_be_fixed.span.start, matcher_to_be_fixed.span.end - 5);

                format!("{}Once", ctx.source_range(span_matcher_without_suffix))
            };

            let matcher_and_args_span =
                Span::new(matcher_to_be_fixed.span.start, call_expr.span.end);

            ctx.diagnostic_with_fix(
                prefer_called_once_diagnostic(matcher_and_args_span, new_matcher_name.as_ref()),
                |fixer| {
                    let argument_without_parenthesis_span =
                        get_inside_parenthesis_span(called_times_value.span, ctx);

                    let multi_fix = fixer.for_multifix();
                    let mut fixes = multi_fix.new_fix_with_capacity(2);

                    fixes.push(fixer.replace(matcher_to_be_fixed.span, new_matcher_name));
                    fixes.push(fixer.delete(&argument_without_parenthesis_span));

                    fixes.with_message("Replace API with preferOnce instead of Times")
                },
            );
        }
    }
}

fn is_open_parenthesis(source_text: &str) -> bool {
    source_text.starts_with('(')
}

fn is_close_parenthesis(source_text: &str) -> bool {
    source_text.ends_with(')')
}

fn get_inside_parenthesis_span(argument_span: Span, ctx: &LintContext<'_>) -> Span {
    let mut inside_parenthesis_span = Span::new(argument_span.start, argument_span.end);

    while !is_open_parenthesis(ctx.source_range(inside_parenthesis_span.expand_left(1))) {
        inside_parenthesis_span = inside_parenthesis_span.expand_left(1);
    }

    while !is_close_parenthesis(ctx.source_range(inside_parenthesis_span.expand_right(1))) {
        inside_parenthesis_span = inside_parenthesis_span.expand_right(1);
    }

    inside_parenthesis_span
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "expect(fn).toBeCalledOnce();",
        "expect(fn).toHaveBeenCalledOnce();",
        "expect(fn).toBeCalledTimes(2);",
        "expect(fn).toHaveBeenCalledTimes(2);",
        "expect(fn).toBeCalledTimes(expect.anything());",
        "expect(fn).toHaveBeenCalledTimes(expect.anything());",
        "expect(fn).not.toBeCalledOnce();",
        "expect(fn).rejects.not.toBeCalledOnce();",
        "expect(fn).not.toHaveBeenCalledOnce();",
        "expect(fn).resolves.not.toHaveBeenCalledOnce();",
        "expect(fn).toBeCalledTimes(0);",
        "expect(fn).toHaveBeenCalledTimes(0);",
        "expect(fn);",
    ];

    let fail = vec![
        "expect(fn).toBeCalledTimes(1);",
        "expect(fn).toHaveBeenCalledTimes(1);",
        "expect(fn).not.toBeCalledTimes(1);",
        "expect(fn).not.toHaveBeenCalledTimes(1);",
        "expect(fn).resolves.toBeCalledTimes(1);",
        "expect(fn).resolves.toHaveBeenCalledTimes(1);",
        "expect(fn).resolves.toHaveBeenCalledTimes(/*comment*/1);",
        "expect(window.HTMLElement.prototype.scrollIntoView).toHaveBeenCalledTimes(
              1,
            );",
    ];

    let fix = vec![
        ("expect(fn).toBeCalledTimes(1);", "expect(fn).toBeCalledOnce();"),
        ("expect(fn).toHaveBeenCalledTimes(1);", "expect(fn).toHaveBeenCalledOnce();"),
        ("expect(fn).not.toBeCalledTimes(1);", "expect(fn).not.toBeCalledOnce();"),
        ("expect(fn).not.toHaveBeenCalledTimes(1);", "expect(fn).not.toHaveBeenCalledOnce();"),
        ("expect(fn).resolves.toBeCalledTimes(1);", "expect(fn).resolves.toBeCalledOnce();"),
        (
            "expect(fn).resolves.toHaveBeenCalledTimes(1);",
            "expect(fn).resolves.toHaveBeenCalledOnce();",
        ),
        (
            "expect(window.HTMLElement.prototype.scrollIntoView).toHaveBeenCalledTimes(
            1,
            );",
            "expect(window.HTMLElement.prototype.scrollIntoView).toHaveBeenCalledOnce();",
        ),
        (
            "expect(window.HTMLElement.prototype.scrollIntoView).toHaveBeenCalledTimes(1,);",
            "expect(window.HTMLElement.prototype.scrollIntoView).toHaveBeenCalledOnce();",
        ),
        (
            "expect(window.HTMLElement.prototype.scrollIntoView).toHaveBeenCalledTimes(/* comment (because why not) */1,);",
            "expect(window.HTMLElement.prototype.scrollIntoView).toHaveBeenCalledOnce();",
        ),
        (
            "expect(window.HTMLElement.prototype.scrollIntoView).toHaveBeenCalledTimes(1/* comment (because why not) */,);",
            "expect(window.HTMLElement.prototype.scrollIntoView).toHaveBeenCalledOnce();",
        ),
        (
            "expect(window.HTMLElement.prototype.scrollIntoView).toHaveBeenCalledTimes(
                /* I only want to call this function 1 (ONE) time, please. */
                1,
            );",
            "expect(window.HTMLElement.prototype.scrollIntoView).toHaveBeenCalledOnce();",
        ),
    ];
    Tester::new(PreferCalledOnce::NAME, PreferCalledOnce::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
