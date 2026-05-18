use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_equality_matcher::{DOCUMENTATION, run_on_jest_node},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PreferEqualityMatcher;

declare_oxc_lint!(
    PreferEqualityMatcher,
    vitest,
    style,
    suggestion,
    docs = DOCUMENTATION,
    version = "0.2.9",
);

impl Rule for PreferEqualityMatcher {
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
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
