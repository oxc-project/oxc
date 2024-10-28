use oxc_ast::{ast::Argument, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{parse_expect_jest_fn_call, PossibleJestNode},
};

fn no_interpolation_in_snapshots_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use string interpolation inside of snapshots")
        .with_help("Remove string interpolation from snapshots")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoInterpolationInSnapshots;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents the use of string interpolations in snapshots.
    ///
    /// ### Why is this bad?
    ///
    /// Interpolation prevents snapshots from being updated. Instead, properties should
    /// be overloaded with a matcher by using
    /// [property matchers](https://jestjs.io/docs/en/snapshot-testing#property-matchers).
    ///
    /// ### Example
    ///
    /// ```javascript
    /// expect(something).toMatchInlineSnapshot(
    ///   `Object {
    ///     property: ${interpolated}
    ///   }`,
    /// );
    ///
    /// expect(something).toMatchInlineSnapshot(
    ///   { other: expect.any(Number) },
    ///   `Object {
    ///     other: Any<Number>,
    ///     property: ${interpolated}
    ///   }`,
    /// );
    ///
    /// expect(errorThrowingFunction).toThrowErrorMatchingInlineSnapshot(
    ///   `${interpolated}`,
    /// );
    /// ```
    NoInterpolationInSnapshots,
    style
);

impl Rule for NoInterpolationInSnapshots {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(jest_node, ctx);
    }
}

fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };
    let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx) else {
        return;
    };
    let Some(matcher) = jest_fn_call.matcher() else {
        return;
    };

    if matcher.is_name_unequal("toMatchInlineSnapshot")
        && matcher.is_name_unequal("toThrowErrorMatchingInlineSnapshot")
    {
        return;
    }

    // Check all since the optional 'propertyMatchers' argument might be present
    // `.toMatchInlineSnapshot(propertyMatchers?, inlineSnapshot)`
    for arg in jest_fn_call.args {
        if let Argument::TemplateLiteral(template_lit) = arg {
            if !template_lit.expressions.is_empty() {
                ctx.diagnostic(no_interpolation_in_snapshots_diagnostic(template_lit.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("expect('something').toEqual('else');", None),
        ("expect(something).toMatchInlineSnapshot();", None),
        ("expect(something).toMatchInlineSnapshot(`No interpolation`);", None),
        ("expect(something).toMatchInlineSnapshot({}, `No interpolation`);", None),
        ("expect(something);", None),
        ("expect(something).not;", None),
        ("expect.toHaveAssertions();", None),
        ("myObjectWants.toMatchInlineSnapshot({}, `${interpolated}`);", None),
        ("myObjectWants.toMatchInlineSnapshot({}, `${interpolated1} ${interpolated2}`);", None),
        ("toMatchInlineSnapshot({}, `${interpolated}`);", None),
        ("toMatchInlineSnapshot({}, `${interpolated1} ${interpolated2}`);", None),
        ("expect(something).toThrowErrorMatchingInlineSnapshot();", None),
        ("expect(something).toThrowErrorMatchingInlineSnapshot(`No interpolation`);", None),
    ];

    let fail = vec![
        ("expect(something).toMatchInlineSnapshot(`${interpolated}`);", None),
        ("expect(something).not.toMatchInlineSnapshot(`${interpolated}`);", None),
        ("expect(something).toMatchInlineSnapshot({}, `${interpolated}`);", None),
        ("expect(something).not.toMatchInlineSnapshot({}, `${interpolated}`);", None),
        ("expect(something).toThrowErrorMatchingInlineSnapshot(`${interpolated}`);", None),
        ("expect(something).not.toThrowErrorMatchingInlineSnapshot(`${interpolated}`);", None),
    ];

    Tester::new(NoInterpolationInSnapshots::NAME, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
