use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{collect_possible_jest_call_node, parse_expect_jest_fn_call, PossibleJestNode},
};

fn use_to_strict_equal(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using `toStrictEqual()`.")
        .with_help("Use `toStrictEqual()` instead")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct PreferStrictEqual;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule triggers a warning if `toEqual()` is used to assert equality.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // invalid
    /// expect({ a: 'a', b: undefined }).toEqual({ a: 'a' });
    ///
    /// // valid
    /// expect({ a: 'a', b: undefined }).toStrictEqual({ a: 'a' });
    /// ```
    ///
    PreferStrictEqual,
    style,
    fix
);

impl Rule for PreferStrictEqual {
    fn run_once(&self, ctx: &LintContext) {
        for possible_jest_node in &collect_possible_jest_call_node(ctx) {
            Self::run(possible_jest_node, ctx);
        }
    }
}

impl PreferStrictEqual {
    fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(parse_jest_expect_fn_call) =
            parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };
        let Some(matcher) = parse_jest_expect_fn_call.matcher() else {
            return;
        };
        let Some(matcher_name) = matcher.name() else {
            return;
        };

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
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

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

    Tester::new(PreferStrictEqual::NAME, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
