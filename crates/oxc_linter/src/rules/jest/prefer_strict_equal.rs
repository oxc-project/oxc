use crate::{
    context::LintContext,
    fixer::Fix,
    rule::Rule,
    utils::{collect_possible_jest_call_node, parse_expect_jest_fn_call, PossibleJestNode},
};

use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(prefer-strict-equal): Suggest using `toStrictEqual()`.")]
#[diagnostic(severity(warning), help("Use `toStrictEqual()` instead"))]
struct UseToStrictEqual(#[label] Span);

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
            ctx.diagnostic_with_fix(UseToStrictEqual(matcher.span), || {
                let mut formatter = ctx.codegen();
                formatter.print_str(
                    matcher
                        .span
                        .source_text(ctx.source_text())
                        .replace(matcher_name.to_string().as_str(), "toStrictEqual")
                        .as_bytes(),
                );
                Fix::new(formatter.into_source_text(), matcher.span)
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
