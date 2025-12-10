use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{PossibleJestNode, parse_expect_jest_fn_call},
};

fn use_to_strict_equal(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using `toStrictEqual()`.")
        .with_help("Use `toStrictEqual()` instead")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferStrictEqual;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule triggers a warning if `toEqual()` is used to assert equality.
    ///
    /// ### Why is this bad?
    ///
    /// The `toEqual()` matcher performs a deep equality check but ignores
    /// `undefined` values in objects and arrays. This can lead to false
    /// positives where tests pass when they should fail. `toStrictEqual()`
    /// provides more accurate comparison by checking for `undefined` values.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// expect({ a: 'a', b: undefined }).toEqual({ a: 'a' });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// expect({ a: 'a', b: undefined }).toStrictEqual({ a: 'a' });
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/docs/rules/prefer-strict-equal.md),
    /// to use it, add the following configuration to your `.oxlintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/prefer-strict-equal": "error"
    ///   }
    /// }
    /// ```
    PreferStrictEqual,
    jest,
    style,
    fix
);

impl Rule for PreferStrictEqual {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }
}

impl PreferStrictEqual {
    fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) -> Option<()> {
        let call_expr = possible_jest_node.node.kind().as_call_expression()?;
        let parse_jest_expect_fn_call =
            parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)?;
        let matcher = parse_jest_expect_fn_call.matcher()?;
        let matcher_name = matcher.name()?;

        if matcher_name.eq("toEqual") {
            ctx.diagnostic_with_fix(use_to_strict_equal(matcher.span), |fixer| {
                let replacement = match fixer.source_range(matcher.span).chars().next().unwrap() {
                    '\'' => "'toStrictEqual'",
                    '"' => "\"toStrictEqual\"",
                    '`' => "`toStrictEqual`",
                    _ => "toStrictEqual",
                };
                fixer.replace(matcher.span, replacement)
            });
        }
        None
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // Note: Both Jest and Vitest share the same unit tests

    let pass = vec![
        ("expect(something).toStrictEqual(somethingElse);", None),
        ("a().toEqual('b')", None),
        ("expect(a);", None),
    ];

    let fail = vec![
        ("expect(something).toEqual(somethingElse);", None),
        ("expect(something).toEqual(somethingElse,);", None),
        ("expect(something)[\"toEqual\"](somethingElse);", None),
    ];

    let fix = vec![
        (
            "expect(something).toEqual(somethingElse);",
            "expect(something).toStrictEqual(somethingElse);",
            None,
        ),
        (
            "expect(something).toEqual(somethingElse,);",
            "expect(something).toStrictEqual(somethingElse,);",
            None,
        ),
        (
            "expect(something)[\"toEqual\"](somethingElse);",
            "expect(something)[\"toStrictEqual\"](somethingElse);",
            None,
        ),
    ];

    Tester::new(PreferStrictEqual::NAME, PreferStrictEqual::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
