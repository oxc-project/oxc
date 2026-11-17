use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{
    context::{ContextHost, LintContext},
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
    /// This rule warns when `toBeTruthy()` or `toBeFalsy()` is used with `expect` or `expectTypeOf`.
    /// With `--fix-suggestions`, it will be replaced with `toBe(true)` or `toBe(false)`.
    ///
    /// ### Why is this bad?
    ///
    /// Truthy/falsy matchers coerce values to boolean and can hide type mistakes.
    /// Strict boolean assertions make intent explicit and avoid accidental coercion.
    /// The replacement is a suggestion because it changes which values pass the assertion.
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
    suggestion,
    version = "1.57.0",
    short_description =
        "Enforce using `toBe(true)` and `toBe(false)` over matchers that coerce types to boolean.",
);

impl Rule for PreferStrictBooleanMatchers {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(possible_vitest_node) =
            PossibleJestNode::from_call_expression(node, call_expr, ctx)
        else {
            return;
        };
        let Some(vitest_expect_fn_call) =
            parse_expect_and_typeof_vitest_fn_call(call_expr, &possible_vitest_node, ctx)
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
        ctx.diagnostic_with_suggestion(
            prefer_strict_boolean_matchers_diagnostic(span, value),
            |fixer| {
                let call_name = if is_cmp_mem_expr { "\"toBe\"]" } else { "toBe" };
                fixer.replace(span, format!("{call_name}({value})"))
            },
        );
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.frameworks().is_test()
    }
}

#[test]
fn test() {
    use crate::{fixer::FixKind, tester::Tester};

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
        ("expect(false).toBeTruthy();", "expect(false).toBe(true);", None, FixKind::Suggestion),
        ("expect(false).toBeFalsy();", "expect(false).toBe(false);", None, FixKind::Suggestion),
        (
            "expectTypeOf(false).toBeTruthy();",
            "expectTypeOf(false).toBe(true);",
            None,
            FixKind::Suggestion,
        ),
        (
            "expectTypeOf(false).toBeFalsy();",
            "expectTypeOf(false).toBe(false);",
            None,
            FixKind::Suggestion,
        ),
        (
            "expect(wasSuccessful).toBeTruthy();",
            "expect(wasSuccessful).toBe(true);",
            None,
            FixKind::Suggestion,
        ),
        (
            "expect(wasSuccessful).toBeFalsy();",
            "expect(wasSuccessful).toBe(false);",
            None,
            FixKind::Suggestion,
        ),
        (
            r#"expect("a string").not.toBeTruthy();"#,
            r#"expect("a string").not.toBe(true);"#,
            None,
            FixKind::Suggestion,
        ),
        (
            r#"expect("a string").not.toBeFalsy();"#,
            r#"expect("a string").not.toBe(false);"#,
            None,
            FixKind::Suggestion,
        ),
        (
            r#"expect(value)["toBeTruthy"]();"#,
            r#"expect(value)["toBe"](true);"#,
            None,
            FixKind::Suggestion,
        ),
        (
            r#"expect(value)["toBeFalsy"]();"#,
            r#"expect(value)["toBe"](false);"#,
            None,
            FixKind::Suggestion,
        ),
        (
            r#"expect(value).not["toBeTruthy"]();"#,
            r#"expect(value).not["toBe"](true);"#,
            None,
            FixKind::Suggestion,
        ),
        (
            r#"expect(value).not["toBeFalsy"]();"#,
            r#"expect(value).not["toBe"](false);"#,
            None,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(PreferStrictBooleanMatchers::NAME, PreferStrictBooleanMatchers::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
