use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_strict_equal::{DOCUMENTATION, run_on_jest_node},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PreferStrictEqual;

declare_oxc_lint!(PreferStrictEqual, vitest, style, fix, docs = DOCUMENTATION, version = "0.2.13",);

impl Rule for PreferStrictEqual {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run_on_jest_node(jest_node, ctx);
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

    Tester::new(PreferStrictEqual::NAME, PreferStrictEqual::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
