use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{PossibleJestNode, is_equality_matcher, parse_expect_and_typeof_vitest_fn_call},
};

pub fn prefer_to_be_simply_bool<'a>(
    possible_vitest_node: &PossibleJestNode<'a, '_>,
    ctx: &LintContext<'a>,
    value: bool,
) {
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
    if !is_equality_matcher(matcher) || vitest_expect_fn_call.args.is_empty() {
        return;
    }
    let Some(arg_expr) = vitest_expect_fn_call.args.first().and_then(Argument::as_expression)
    else {
        return;
    };

    if let Expression::BooleanLiteral(arg) = arg_expr.get_inner_expression()
        && arg.value == value
    {
        let span = Span::new(matcher.span.start, call_expr.span.end);

        let is_cmp_mem_expr = match matcher.parent {
            Some(Expression::ComputedMemberExpression(_)) => true,
            Some(Expression::StaticMemberExpression(_) | Expression::PrivateFieldExpression(_)) => {
                false
            }
            _ => return,
        };

        let call_name = if value { "toBeTruthy" } else { "toBeFalsy" };

        ctx.diagnostic_with_suggestion(
            OxcDiagnostic::warn(format!("Use `{call_name}` instead.")).with_label(span),
            |fixer| {
                let new_matcher = if is_cmp_mem_expr {
                    format!("[\"{call_name}\"]()")
                } else {
                    format!("{call_name}()")
                };
                fixer.replace(span, new_matcher)
            },
        );
    }
}

#[derive(Debug, Default, Clone)]
pub struct PreferToBeTruthy;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule warns when `toBe(true)` is used with `expect` or `expectTypeOf`.
    /// With `--fix-suggestions`, it will be replaced with `toBeTruthy()`.
    ///
    /// ### Why is this bad?
    ///
    /// When testing for truthiness, `toBeTruthy()` expresses that intent directly.
    /// Unlike `toBe(true)`, it also accepts non-boolean truthy values such as
    /// non-empty strings and objects. The replacement is a suggestion because
    /// it changes which values pass the assertion.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// expect(foo).toBe(true)
    /// expectTypeOf(foo).toBe(true)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// expect(foo).toBeTruthy()
    /// expectTypeOf(foo).toBeTruthy()
    /// ```
    PreferToBeTruthy,
    vitest,
    style,
    suggestion,
    version = "0.7.1",
    short_description = "Prefer `toBeTruthy()` over `toBe(true)`.",
);

impl Rule for PreferToBeTruthy {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        prefer_to_be_simply_bool(jest_node, ctx, true);
    }
}

#[test]
fn test() {
    use crate::{fixer::FixKind, tester::Tester};

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
        ("expect(false).toBe(true);", "expect(false).toBeTruthy();"),
        ("expectTypeOf(false).toBe(true);", "expectTypeOf(false).toBeTruthy();"),
        ("expect(wasSuccessful).toEqual(true);", "expect(wasSuccessful).toBeTruthy();"),
        (
            "expect(fs.existsSync('/path/to/file')).toStrictEqual(true);",
            "expect(fs.existsSync('/path/to/file')).toBeTruthy();",
        ),
        (r#"expect("a string").not.toBe(true);"#, r#"expect("a string").not.toBeTruthy();"#),
        (r#"expect("a string").not.toEqual(true);"#, r#"expect("a string").not.toBeTruthy();"#),
        (
            r#"expectTypeOf("a string").not.toStrictEqual(true);"#,
            r#"expectTypeOf("a string").not.toBeTruthy();"#,
        ),
    ];

    let mut fix = fix
        .into_iter()
        .map(|(source, expected)| (source, expected, None, FixKind::Suggestion))
        .collect::<Vec<_>>();
    fix.extend([
        ("expect(1).toBe(true);", "expect(1).toBe(true);", None, FixKind::Fix),
        ("expect(1).toBe(true);", "expect(1).toBeTruthy();", None, FixKind::Suggestion),
    ]);

    Tester::new(PreferToBeTruthy::NAME, PreferToBeTruthy::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
