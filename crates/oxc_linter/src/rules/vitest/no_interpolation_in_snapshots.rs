use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    rules::shared::no_interpolation_in_snapshots::{DOCUMENTATION, run},
};

#[derive(Debug, Default, Clone)]
pub struct NoInterpolationInSnapshots;

declare_oxc_lint!(
    NoInterpolationInSnapshots,
    vitest,
    style,
    recommended,
    docs = DOCUMENTATION,
    version = "0.0.13",
);

impl Rule for NoInterpolationInSnapshots {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TemplateLiteral(template) = node.kind() else { return };
        run(node, template, ctx);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.frameworks().is_test()
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
