use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_to_have_length::{DOCUMENTATION, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PreferToHaveLength;

declare_oxc_lint!(PreferToHaveLength, jest, style, fix, docs = DOCUMENTATION, version = "0.2.13",);

impl Rule for PreferToHaveLength {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(jest_node, ctx);
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    let pass = vec![
        ("expect.hasAssertions", None),
        ("expect.hasAssertions()", None),
        ("expect(files).toHaveLength(1);", None),
        ("expect(files.name).toBe('file');", None),
        ("expect(files[`name`]).toBe('file');", None),
        ("expect(users[0]?.permissions?.length).toBe(1);", None),
        ("expect(result).toBe(true);", None),
        ("expect(user.getUserName(5)).resolves.toEqual('Paul')", None),
        ("expect(user.getUserName(5)).rejects.toEqual('Paul')", None),
        ("expect(a);", None),
        ("expect().toBe();", None),
    ];

    let fail = vec![
        ("expect(files[\"length\"]).toBe(1);", None),
        ("expect(files[\"length\"]).toBe(1,);", None),
        ("expect(files[\"length\"])[\"not\"].toBe(1);", None),
        ("expect(files[\"length\"])[\"toBe\"](1);", None),
        ("expect(files[\"length\"]).not[\"toBe\"](1);", None),
        ("expect(files[\"length\"])[\"not\"][\"toBe\"](1);", None),
        ("expect(files.length).toBe(1);", None),
        ("expect(files.length).toEqual(1);", None),
        ("expect(files.length).toStrictEqual(1);", None),
        ("expect(files.length).not.toStrictEqual(1);", None),
        (
            "expect((meta.get('pages') as YArray<unknown>).length).toBe((originalMeta.get('pages') as YArray<unknown>).length);",
            None,
        ),
        (
            "expect(assetTypeContainer.getElementsByTagName('time').length).toEqual(
          0,
        );",
            None,
        ),
    ];

    let fix = vec![
        ("expect(files[\"length\"]).not.toBe(1);", "expect(files).not.toHaveLength(1);", None),
        (r#"expect(files["length"]).toBe(1,);"#, "expect(files).toHaveLength(1,);", None),
        (
            "expect(files[\"length\"])[\"resolves\"].toBe(1,);",
            "expect(files)[\"resolves\"].toHaveLength(1,);",
            None,
        ),
        (
            "expect(files[\"length\"])[\"not\"].toBe(1);",
            "expect(files)[\"not\"].toHaveLength(1);",
            None,
        ),
        ("expect(files[\"length\"])[\"toBe\"](1);", "expect(files).toHaveLength(1);", None),
        ("expect(files[\"length\"]).not[\"toBe\"](1);", "expect(files).not.toHaveLength(1);", None),
        (
            "expect(files[\"length\"])[\"not\"][\"toBe\"](1);",
            "expect(files)[\"not\"].toHaveLength(1);",
            None,
        ),
        ("expect(files.length).toBe(1);", "expect(files).toHaveLength(1);", None),
        ("expect(files.length).toEqual(1);", "expect(files).toHaveLength(1);", None),
        ("expect(files.length).toStrictEqual(1);", "expect(files).toHaveLength(1);", None),
        ("expect(files.length).not.toStrictEqual(1);", "expect(files).not.toHaveLength(1);", None),
        (
            "expect((meta.get('pages') as YArray<unknown>).length).toBe((originalMeta.get('pages') as YArray<unknown>).length);",
            "expect((meta.get('pages') as YArray<unknown>)).toHaveLength((originalMeta.get('pages') as YArray<unknown>).length);",
            None,
        ),
        (
            "expect(assetTypeContainer.getElementsByTagName('time').length).toEqual(
          0,
        );",
            "expect(assetTypeContainer.getElementsByTagName('time')).toHaveLength(
          0,
        );",
            None,
        ),
    ];

    Tester::new(PreferToHaveLength::NAME, PreferToHaveLength::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
