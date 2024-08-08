use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, is_equality_matcher,
        parse_expect_and_typeof_vitest_fn_call, PossibleJestNode,
    },
};

fn use_to_be_truthy(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `toBeTruthy` instead.").with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct PreferToBeTruthy;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule warns when `toBe(true)` is used with `expect` or `expectTypeOf`. With `--fix`, it will be replaced with `toBeTruthy()`.
    ///
    /// ### Examples
    ///
    /// ```javascript
    /// // bad
    /// expect(foo).toBe(true)
    /// expectTypeOf(foo).toBe(true)
    ///
    /// // good
    /// expect(foo).toBeTruthy()
    /// expectTypeOf(foo).toBeTruthy()
    /// ```
    PreferToBeTruthy,
    style,
    fix
);

impl Rule for PreferToBeTruthy {
    fn run_once(&self, ctx: &LintContext) {
        for possible_vitest_node in &collect_possible_jest_call_node(ctx) {
            Self::run(possible_vitest_node, ctx);
        }
    }
}

impl PreferToBeTruthy {
    fn run<'a>(possible_vitest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_vitest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(vitest_expect_fn_call) =
            parse_expect_and_typeof_vitest_fn_call(call_expr, possible_vitest_node, ctx)
        else {
            return;
        };
        let Some(matcher) = vitest_expect_fn_call.matcher() else {
            return;
        };

        if !is_equality_matcher(matcher) || vitest_expect_fn_call.args.len() == 0 {
            return;
        }

        let Some(arg_expr) = vitest_expect_fn_call.args.first().and_then(Argument::as_expression)
        else {
            return;
        };

        if let Expression::BooleanLiteral(arg) = arg_expr.get_inner_expression() {
            if arg.value {
                let span = Span::new(matcher.span.start, call_expr.span.end);

                let is_cmp_mem_expr = match matcher.parent {
                    Some(Expression::ComputedMemberExpression(_)) => true,
                    Some(
                        Expression::StaticMemberExpression(_)
                        | Expression::PrivateFieldExpression(_),
                    ) => false,
                    _ => return,
                };

                ctx.diagnostic_with_fix(use_to_be_truthy(span), |fixer| {
                    let new_matcher =
                        if is_cmp_mem_expr { "[\"toBeTruthy\"]()" } else { "toBeTruthy()" };

                    fixer.replace(span, new_matcher)
                });
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "[].push(true)",
        r#"expect("something");"#,
        "expect(true).toBeTrue();",
        "expect(false).toBeTrue();",
        "expect(fal,se).toBeFalse();",
        "expect(true).toBeFalse();",
        "expect(value).toEqual();",
        "expect(value).not.toBeTrue();",
        "expect(value).not.toEqual();",
        "expect(value).toBe(undefined);",
        "expect(value).not.toBe(undefined);",
        "expect(true).toBe(false)",
        "expect(value).toBe();",
        "expect(true).toMatchSnapshot();",
        r#"expect("a string").toMatchSnapshot(true);"#,
        r#"expect("a string").not.toMatchSnapshot();"#,
        "expect(something).toEqual('a string');",
        "expect(true).toBe",
        "expectTypeOf(true).toBe()",
    ];

    let fail = vec![
        "expect(false).toBe(true);",
        "expectTypeOf(false).toBe(true);",
        "expect(wasSuccessful).toEqual(true);",
        "expect(fs.existsSync('/path/to/file')).toStrictEqual(true);",
        r#"expect("a string").not.toBe(true);"#,
        r#"expect("a string").not.toEqual(true);"#,
        r#"expectTypeOf("a string").not.toStrictEqual(true);"#,
    ];

    let fix = vec![
        ("expect(false).toBe(true);", "expect(false).toBeTruthy();", None),
        ("expectTypeOf(false).toBe(true);", "expectTypeOf(false).toBeTruthy();", None),
        ("expect(wasSuccessful).toEqual(true);", "expect(wasSuccessful).toBeTruthy();", None),
        (
            "expect(fs.existsSync('/path/to/file')).toStrictEqual(true);",
            "expect(fs.existsSync('/path/to/file')).toBeTruthy();",
            None,
        ),
        (r#"expect("a string").not.toBe(true);"#, r#"expect("a string").not.toBeTruthy();"#, None),
        (
            r#"expect("a string").not.toEqual(true);"#,
            r#"expect("a string").not.toBeTruthy();"#,
            None,
        ),
        (
            r#"expectTypeOf("a string").not.toStrictEqual(true);"#,
            r#"expectTypeOf("a string").not.toBeTruthy();"#,
            None,
        ),
    ];
    Tester::new(PreferToBeTruthy::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
