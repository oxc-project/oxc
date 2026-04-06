use oxc_ast::{
    AstKind,
    ast::{Argument, BinaryExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::{
    context::LintContext,
    fixer::RuleFixer,
    rule::Rule,
    utils::{
        KnownMemberExpressionProperty, PossibleJestNode, is_equality_matcher,
        parse_expect_jest_fn_call,
    },
};

fn use_equality_matcher_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using the built-in equality matchers.")
        .with_help("Prefer using one of the equality matchers instead")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferEqualityMatcher;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Jest has built-in matchers for expecting equality, which allow for more readable
    /// tests and error messages if an expectation fails.
    ///
    /// ### Why is this bad?
    ///
    /// Testing equality expressions with generic matchers like `toBe(true)`
    /// makes tests harder to read and understand. When tests fail, the error
    /// messages are less helpful because they don't show what the actual values
    /// were. Using specific equality matchers provides clearer test intent and
    /// better debugging information.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// expect(x === 5).toBe(true);
    /// expect(name === 'Carl').not.toEqual(true);
    /// expect(myObj !== thatObj).toStrictEqual(true);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// expect(x).toBe(5);
    /// expect(name).not.toEqual('Carl');
    /// expect(myObj).toStrictEqual(thatObj);
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/docs/rules/prefer-equality-matcher.md),
    /// to use it, add the following configuration to your `.oxlintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/prefer-equality-matcher": "error"
    ///   }
    /// }
    /// ```
    PreferEqualityMatcher,
    jest,
    style,
    suggestion
);

impl Rule for PreferEqualityMatcher {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }
}

impl PreferEqualityMatcher {
    pub fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(matcher_call_expr) = node.kind() else {
            return;
        };
        let Some(jest_fn_call) =
            parse_expect_jest_fn_call(matcher_call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        let Some(expect_parent) = jest_fn_call.head.parent else {
            return;
        };
        let expr = expect_parent.get_inner_expression();
        let Expression::CallExpression(expect_call_expr) = expr else {
            return;
        };
        let Some(argument) = expect_call_expr.arguments.first() else {
            return;
        };

        let Argument::BinaryExpression(binary_expr) = argument else {
            return;
        };

        if binary_expr.operator != BinaryOperator::StrictEquality
            && binary_expr.operator != BinaryOperator::StrictInequality
        {
            return;
        }

        let Some(matcher) = jest_fn_call.matcher() else {
            return;
        };

        if !is_equality_matcher(matcher) {
            return;
        }
        let Some(first_matcher_arg) = jest_fn_call.args.first().and_then(Argument::as_expression)
        else {
            return;
        };
        let Expression::BooleanLiteral(matcher_arg_value) =
            first_matcher_arg.get_inner_expression()
        else {
            return;
        };

        let modifiers = jest_fn_call.modifiers();
        let has_not_modifier = modifiers.iter().any(|modifier| modifier.is_name_equal("not"));
        let add_not_modifier = (if binary_expr.operator == BinaryOperator::StrictInequality {
            !matcher_arg_value.value
        } else {
            matcher_arg_value.value
        }) == has_not_modifier;

        let fixer = RuleFixer::new(FixKind::Suggestion, ctx);
        let suggestions = ["toBe", "toEqual", "toStrictEqual"].into_iter().map(|eq_matcher| {
            // Preserve trailing commas: expect(a === b,).toBe(true,) -> expect(a,).toBe(b,)
            let call_span_end =
                fixer.source_range(Span::new(binary_expr.span.end, expect_call_expr.span.end));
            let arg_span_end = fixer
                .source_range(Span::new(matcher_arg_value.span.end, matcher_call_expr.span.end));
            let content = Self::build_code(
                binary_expr,
                call_span_end,
                arg_span_end,
                &jest_fn_call.local,
                &modifiers,
                eq_matcher,
                add_not_modifier,
                fixer,
            );
            fixer
                .replace(matcher_call_expr.span, content)
                .with_message(format!("Use `{eq_matcher}`"))
        });

        ctx.diagnostic_with_suggestions(use_equality_matcher_diagnostic(matcher.span), suggestions);
    }

    fn build_code<'a>(
        binary_expr: &BinaryExpression<'a>,
        call_span_end: &str,
        arg_span_end: &str,
        local_name: &str,
        modifiers: &[&KnownMemberExpressionProperty<'a>],
        equality_matcher: &str,
        add_not_modifier: bool,
        fixer: RuleFixer<'_, 'a>,
    ) -> String {
        let mut content = fixer.codegen();
        content.print_str(local_name);
        content.print_ascii_byte(b'(');
        content.print_expression(&binary_expr.left);
        content.print_str(call_span_end);
        content.print_ascii_byte(b'.');
        for modifier in modifiers {
            let Some(modifier_name) = modifier.name() else {
                continue;
            };
            if modifier_name != "not" {
                content.print_str(&modifier_name);
                content.print_ascii_byte(b'.');
            }
        }
        if add_not_modifier {
            content.print_str("not.");
        }
        content.print_str(equality_matcher);
        content.print_ascii_byte(b'(');
        content.print_expression(&binary_expr.right);
        content.print_str(arg_span_end);
        content.into_source_text()
    }
}

#[test]
fn test() {
    use crate::tester::{ExpectFixTestCase, Tester};

    let mut pass = vec![
        ("expect.hasAssertions", None),
        ("expect.hasAssertions()", None),
        ("expect.assertions(1)", None),
        ("expect(true).toBe(...true)", None),
        ("expect(a == 1).toBe(true)", None),
        ("expect(1 == a).toBe(true)", None),
        ("expect(a == b).toBe(true)", None),
    ];

    let mut fail = vec![
        ("expect(a !== b).toBe(true)", None),
        ("expect(a !== b).toBe(false)", None),
        ("expect(a !== b).resolves.toBe(true)", None),
        ("expect(a !== b).resolves.toBe(false)", None),
        ("expect(a !== b).not.toBe(true)", None),
        ("expect(a !== b).not.toBe(false)", None),
        ("expect(a !== b).resolves.not.toBe(true)", None),
        ("expect(a !== b).resolves.not.toBe(false)", None),
    ];

    let pass_vitest = vec![
        ("expect.hasAssertions", None),
        ("expect.hasAssertions()", None),
        ("expect.assertions(1)", None),
        ("expect(true).toBe(...true)", None),
        ("expect(a == 1).toBe(true)", None),
        ("expect(1 == a).toBe(true)", None),
        ("expect(a == b).toBe(true)", None),
        ("expect.hasAssertions", None),
        ("expect.hasAssertions()", None),
        ("expect.assertions(1)", None),
        ("expect(true).toBe(...true)", None),
        ("expect(a != 1).toBe(true)", None),
        ("expect(1 != a).toBe(true)", None),
        ("expect(a != b).toBe(true)", None),
    ];

    let fail_vitest = vec![
        ("expect(a === b).toBe(true);", None),
        ("expect(a === b,).toBe(true,);", None), // { "parserOptions": { "ecmaVersion": 2017 } },
        ("expect(a === b).toBe(false);", None),
        ("expect(a === b).resolves.toBe(true);", None),
        ("expect(a === b).resolves.toBe(false);", None),
        ("expect(a === b).not.toBe(true);", None),
        ("expect(a === b).not.toBe(false);", None),
        ("expect(a === b).resolves.not.toBe(true);", None),
        ("expect(a === b).resolves.not.toBe(false);", None),
        (r#"expect(a === b)["resolves"].not.toBe(false);"#, None),
        (r#"expect(a === b)["resolves"]["not"]["toBe"](false);"#, None),
        ("expect(a !== b).toBe(true);", None),
        ("expect(a !== b).toBe(false);", None),
        ("expect(a !== b).resolves.toBe(true);", None),
        ("expect(a !== b).resolves.toBe(false);", None),
        ("expect(a !== b).not.toBe(true);", None),
        ("expect(a !== b).not.toBe(false);", None),
        ("expect(a !== b).resolves.not.toBe(true);", None),
        ("expect(a !== b).resolves.not.toBe(false);", None),
    ];

    let fix: Vec<ExpectFixTestCase> = vec![
        (
            "expect(a === b).toBe(true);",
            ("expect(a).toBe(b);", "expect(a).toEqual(b);", "expect(a).toStrictEqual(b);"),
        )
            .into(),
        (
            "expect(a === b,).toBe(true,);",
            ("expect(a,).toBe(b,);", "expect(a,).toEqual(b,);", "expect(a,).toStrictEqual(b,);"),
        )
            .into(),
        (
            "expect(a === b).toBe(false);",
            (
                "expect(a).not.toBe(b);",
                "expect(a).not.toEqual(b);",
                "expect(a).not.toStrictEqual(b);",
            ),
        )
            .into(),
        (
            "expect(a === b).resolves.toBe(true);",
            (
                "expect(a).resolves.toBe(b);",
                "expect(a).resolves.toEqual(b);",
                "expect(a).resolves.toStrictEqual(b);",
            ),
        )
            .into(),
        (
            "expect(a === b).resolves.toBe(false);",
            (
                "expect(a).resolves.not.toBe(b);",
                "expect(a).resolves.not.toEqual(b);",
                "expect(a).resolves.not.toStrictEqual(b);",
            ),
        )
            .into(),
        (
            "expect(a === b).not.toBe(true);",
            (
                "expect(a).not.toBe(b);",
                "expect(a).not.toEqual(b);",
                "expect(a).not.toStrictEqual(b);",
            ),
        )
            .into(),
        (
            "expect(a === b).not.toBe(false);",
            ("expect(a).toBe(b);", "expect(a).toEqual(b);", "expect(a).toStrictEqual(b);"),
        )
            .into(),
        (
            "expect(a === b).resolves.not.toBe(true);",
            (
                "expect(a).resolves.not.toBe(b);",
                "expect(a).resolves.not.toEqual(b);",
                "expect(a).resolves.not.toStrictEqual(b);",
            ),
        )
            .into(),
        (
            "expect(a === b).resolves.not.toBe(false);",
            (
                "expect(a).resolves.toBe(b);",
                "expect(a).resolves.toEqual(b);",
                "expect(a).resolves.toStrictEqual(b);",
            ),
        )
            .into(),
        (
            r#"expect(a === b)["resolves"].not.toBe(false);"#,
            (
                "expect(a).resolves.toBe(b);",
                "expect(a).resolves.toEqual(b);",
                "expect(a).resolves.toStrictEqual(b);",
            ),
        )
            .into(),
        (
            r#"expect(a === b)["resolves"]["not"]["toBe"](false);"#,
            (
                "expect(a).resolves.toBe(b);",
                "expect(a).resolves.toEqual(b);",
                "expect(a).resolves.toStrictEqual(b);",
            ),
        )
            .into(),
        (
            "expect(a !== b).toBe(true);",
            (
                "expect(a).not.toBe(b);",
                "expect(a).not.toEqual(b);",
                "expect(a).not.toStrictEqual(b);",
            ),
        )
            .into(),
        (
            "expect(a !== b).toBe(false);",
            ("expect(a).toBe(b);", "expect(a).toEqual(b);", "expect(a).toStrictEqual(b);"),
        )
            .into(),
        (
            "expect(a !== b).resolves.toBe(true);",
            (
                "expect(a).resolves.not.toBe(b);",
                "expect(a).resolves.not.toEqual(b);",
                "expect(a).resolves.not.toStrictEqual(b);",
            ),
        )
            .into(),
        (
            "expect(a !== b).resolves.toBe(false);",
            (
                "expect(a).resolves.toBe(b);",
                "expect(a).resolves.toEqual(b);",
                "expect(a).resolves.toStrictEqual(b);",
            ),
        )
            .into(),
        (
            "expect(a !== b).not.toBe(true);",
            ("expect(a).toBe(b);", "expect(a).toEqual(b);", "expect(a).toStrictEqual(b);"),
        )
            .into(),
        (
            "expect(a !== b).not.toBe(false);",
            (
                "expect(a).not.toBe(b);",
                "expect(a).not.toEqual(b);",
                "expect(a).not.toStrictEqual(b);",
            ),
        )
            .into(),
        (
            "expect(a !== b).resolves.not.toBe(true);",
            (
                "expect(a).resolves.toBe(b);",
                "expect(a).resolves.toEqual(b);",
                "expect(a).resolves.toStrictEqual(b);",
            ),
        )
            .into(),
        (
            "expect(a !== b).resolves.not.toBe(false);",
            (
                "expect(a).resolves.not.toBe(b);",
                "expect(a).resolves.not.toEqual(b);",
                "expect(a).resolves.not.toStrictEqual(b);",
            ),
        )
            .into(),
    ];

    pass.extend(pass_vitest);
    fail.extend(fail_vitest);

    Tester::new(PreferEqualityMatcher::NAME, PreferEqualityMatcher::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
