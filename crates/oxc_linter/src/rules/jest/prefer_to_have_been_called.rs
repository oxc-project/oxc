use crate::{
    context::LintContext,
    rule::Rule,
    utils::{ParsedExpectFnCall, PossibleJestNode, parse_expect_jest_fn_call},
};
use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

fn prefer_to_have_been_called_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `toHaveBeenCalled()` over `toHaveBeenCalledTimes(0)`")
        .with_help("Use `toHaveBeenCalled()` to check if function was called, or `not.toHaveBeenCalled()` to check if it wasn't called")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferToHaveBeenCalled;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Suggests using `toHaveBeenCalled()` or `not.toHaveBeenCalled()` over `toHaveBeenCalledTimes(0)` or `toBeCalledTimes(0)`.
    ///
    /// ### Why is this bad?
    ///
    /// `toHaveBeenCalled()` is more explicit and readable than `toHaveBeenCalledTimes(0)`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// expect(mock).toHaveBeenCalledTimes(0);
    /// expect(mock).toBeCalledTimes(0);
    /// expect(mock).not.toHaveBeenCalledTimes(0);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// expect(mock).not.toHaveBeenCalled();
    /// expect(mock).toHaveBeenCalled();
    /// expect(mock).toHaveBeenCalledTimes(1);
    /// ```
    PreferToHaveBeenCalled,
    jest,
    style,
    fix,
);

impl Rule for PreferToHaveBeenCalled {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }
}
impl PreferToHaveBeenCalled {
    fn run<'a, 'c>(jest_node: &PossibleJestNode<'a, 'c>, ctx: &'c LintContext<'a>) {
        let node = jest_node.node;

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(parsed_expect_call) = parse_expect_jest_fn_call(call_expr, jest_node, ctx) else {
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

        if matcher.is_name_unequal("toHaveBeenCalledTimes")
            && matcher.is_name_unequal("toBeCalledTimes")
        {
            return;
        }

        // check if first argument is 0
        let Some(arg) = parsed_expect_call.args.first() else { return };
        let Some(arg_expr) = arg.as_expression() else {
            return;
        };
        if !is_zero_arg(arg_expr) {
            return;
        }

        ctx.diagnostic_with_fix(prefer_to_have_been_called_diagnostic(call_expr.span), |fixer| {
            // check if there is a `not` modifier
            let binding = parsed_expect_call.modifiers();
            let not_modifier = binding.iter().find(|modifier| modifier.is_name_equal("not"));

            if let Some(not_modifier) = not_modifier {
                // if has `not` modifier, remove not and replace with toHaveBeenCalled()
                // need to find the position of not and replace to the end of the method call
                let not_start = not_modifier.span.start;

                let call_end = call_expr.span.end;
                let replace_span = Span::new(not_start, call_end);

                fixer.replace(replace_span, "toHaveBeenCalled()")
            } else {
                // if does not have `not` modifier, add `not.` before method name
                let method_start = matcher.span.start;
                let call_end = call_expr.span.end;
                let replace_span = Span::new(method_start, call_end);

                fixer.replace(replace_span, "not.toHaveBeenCalled()")
            }
        });
    }
}

fn is_zero_arg(expr: &Expression<'_>) -> bool {
    match expr.get_inner_expression() {
        Expression::NumericLiteral(lit) => lit.value == 0.0,
        Expression::BigIntLiteral(lit) => lit.value == "0",
        _ => false,
    }
}
#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "expect(method.mock.calls).toHaveLength;",
        "expect(method.mock.calls).toHaveLength(0);",
        "expect(method).toHaveBeenCalledTimes(1)",
        "expect(method).not.toHaveBeenCalledTimes(x)",
        "expect(method).not.toHaveBeenCalledTimes(1)",
        "expect(method).not.toHaveBeenCalledTimes(...x)",
        "expect(a);",
        "expect(method).not.resolves.toHaveBeenCalledTimes(0);",
        "expect(method).toBe([])",
        "expect(fn.mock.calls).toEqual([])",
        "expect(fn.mock.calls).toContain(1, 2, 3)",
    ];

    let fail: Vec<&str> = vec![
        "expect(method).toBeCalledTimes(0);",
        "expect(method).not.toBeCalledTimes(0);",
        "expect(method).toHaveBeenCalledTimes(0);",
        "expect(method).not.toHaveBeenCalledTimes(0);",
        "expect(method).not.toHaveBeenCalledTimes(0, 1, 2);",
        "expect(method).resolves.toHaveBeenCalledTimes(0);",
        "expect(method).rejects.not.toHaveBeenCalledTimes(0);",
        "expect(method).toBeCalledTimes(0 as number);",
    ];

    let fix = vec![
        ("expect(method).toBeCalledTimes(0);", "expect(method).not.toHaveBeenCalled();", None),
        ("expect(method).not.toBeCalledTimes(0);", "expect(method).toHaveBeenCalled();", None),
        (
            "expect(method).toHaveBeenCalledTimes(0);",
            "expect(method).not.toHaveBeenCalled();",
            None,
        ),
        (
            "expect(method).not.toHaveBeenCalledTimes(0);",
            "expect(method).toHaveBeenCalled();",
            None,
        ),
        (
            "expect(method).not.toHaveBeenCalledTimes(0, 1, 2);",
            "expect(method).toHaveBeenCalled();",
            None,
        ),
        (
            "expect(method).resolves.toHaveBeenCalledTimes(0);",
            "expect(method).resolves.not.toHaveBeenCalled();",
            None,
        ),
        (
            "expect(method).rejects.not.toHaveBeenCalledTimes(0);",
            "expect(method).rejects.toHaveBeenCalled();",
            None,
        ),
        (
            "expect(method).toBeCalledTimes(0 as number);",
            "expect(method).not.toHaveBeenCalled();",
            None,
        ),
    ];
    Tester::new(PreferToHaveBeenCalled::NAME, PreferToHaveBeenCalled::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
