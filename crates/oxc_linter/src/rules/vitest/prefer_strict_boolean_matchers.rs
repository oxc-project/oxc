use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{PossibleJestNode, parse_expect_and_typeof_vitest_fn_call},
};

fn prefer_strict_boolean_matchers_diagnostic(span: Span, value: bool) -> OxcDiagnostic {
    let (matcher_name, to_be_literal) =
        if value { ("toBeTruthy", "toBe(true)") } else { ("toBeFalsy", "toBe(false)") };

    OxcDiagnostic::warn(format!("Use `{to_be_literal}` instead of `{matcher_name}`."))
        .with_help("Use strict boolean comparison instead of truthy/falsy coercion.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferStrictBooleanMatchers;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce using `toBe(true)` and `toBe(false)` over matchers that coerce types to boolean.
    ///
    /// ### Why is this bad?
    ///
    /// Truthy/falsy matchers coerce values to boolean and can hide type mistakes.
    /// Strict boolean assertions make intent explicit and avoid accidental coercion.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// expect(foo).toBeTruthy()
    /// expectTypeOf(foo).toBeTruthy()
    /// expect(foo).toBeFalsy()
    /// expectTypeOf(foo).toBeFalsy()
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// expect(foo).toBe(true)
    /// expectTypeOf(foo).toBe(true)
    /// expect(foo).toBe(false)
    /// expectTypeOf(foo).toBe(false)
    /// ```
    PreferStrictBooleanMatchers,
    vitest,
    style,
    fix,
    version = "1.57.0",
);

impl Rule for PreferStrictBooleanMatchers {
    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_vitest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let AstKind::CallExpression(call_expr) = possible_vitest_node.node.kind() else {
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

        let Some(value) = (if matcher.is_name_equal("toBeTruthy") {
            Some(true)
        } else if matcher.is_name_equal("toBeFalsy") {
            Some(false)
        } else {
            None
        }) else {
            return;
        };

        let is_cmp_mem_expr = match matcher.parent {
            Some(Expression::ComputedMemberExpression(_)) => true,
            Some(Expression::StaticMemberExpression(_) | Expression::PrivateFieldExpression(_)) => {
                false
            }
            _ => return,
        };

        let span = Span::new(matcher.span.start, call_expr.span.end);
        ctx.diagnostic_with_fix(prefer_strict_boolean_matchers_diagnostic(span, value), |fixer| {
            let call_name = if is_cmp_mem_expr { "\"toBe\"]" } else { "toBe" };
            fixer.replace(span, format!("{call_name}({value})"))
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "[].push(true)",
        "[].push(false)",
        r#"expect("something");"#,
        "expect(true).toBe(true);",
        "expect(true).toBe(false);",
        "expect(false).toBe(true);",
        "expect(false).toBe(false);",
        "expect(fal,se).toBe(true);",
        "expect(fal,se).toBe(false);",
        "expect(value).toEqual();",
        "expect(value).not.toBe(true);",
        "expect(value).not.toBe(false);",
        "expect(value).not.toEqual();",
        "expect(value).toBe(undefined);",
        "expect(value).not.toBe(undefined);",
        "expect(value).toBe();",
        "expect(true).toMatchSnapshot();",
        r#"expect("a string").toMatchSnapshot(true);"#,
        r#"expect("a string").toMatchSnapshot(false);"#,
        r#"expect("a string").not.toMatchSnapshot();"#,
        r#"expect(value)["toBe"](true);"#,
        r#"expect(value)["toBe"](false);"#,
        "expect(something).toEqual('a string');",
        "expect(true).toBe",
        "expectTypeOf(true).toBe()",
    ];

    let fail = vec![
        "expect(false).toBeTruthy();",
        "expect(false).toBeFalsy();",
        "expectTypeOf(false).toBeTruthy();",
        "expectTypeOf(false).toBeFalsy();",
        "expect(wasSuccessful).toBeTruthy();",
        "expect(wasSuccessful).toBeFalsy();",
        r#"expect("a string").not.toBeTruthy();"#,
        r#"expect("a string").not.toBeFalsy();"#,
        r#"expect(value)["toBeTruthy"]();"#,
        r#"expect(value)["toBeFalsy"]();"#,
        r#"expect(value).not["toBeTruthy"]();"#,
        r#"expect(value).not["toBeFalsy"]();"#,
    ];

    let fix = vec![
        ("expect(false).toBeTruthy();", "expect(false).toBe(true);"),
        ("expect(false).toBeFalsy();", "expect(false).toBe(false);"),
        ("expectTypeOf(false).toBeTruthy();", "expectTypeOf(false).toBe(true);"),
        ("expectTypeOf(false).toBeFalsy();", "expectTypeOf(false).toBe(false);"),
        ("expect(wasSuccessful).toBeTruthy();", "expect(wasSuccessful).toBe(true);"),
        ("expect(wasSuccessful).toBeFalsy();", "expect(wasSuccessful).toBe(false);"),
        (r#"expect("a string").not.toBeTruthy();"#, r#"expect("a string").not.toBe(true);"#),
        (r#"expect("a string").not.toBeFalsy();"#, r#"expect("a string").not.toBe(false);"#),
        (r#"expect(value)["toBeTruthy"]();"#, r#"expect(value)["toBe"](true);"#),
        (r#"expect(value)["toBeFalsy"]();"#, r#"expect(value)["toBe"](false);"#),
        (r#"expect(value).not["toBeTruthy"]();"#, r#"expect(value).not["toBe"](true);"#),
        (r#"expect(value).not["toBeFalsy"]();"#, r#"expect(value).not["toBe"](false);"#),
    ];

    Tester::new(PreferStrictBooleanMatchers::NAME, PreferStrictBooleanMatchers::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
