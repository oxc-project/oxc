use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::no_interpolation_in_snapshots::{DOCUMENTATION, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct NoInterpolationInSnapshots;

declare_oxc_lint!(
    NoInterpolationInSnapshots,
    vitest,
    style,
    docs = DOCUMENTATION,
    version = "0.0.13",
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

    Tester::new(NoInterpolationInSnapshots::NAME, NoInterpolationInSnapshots::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
