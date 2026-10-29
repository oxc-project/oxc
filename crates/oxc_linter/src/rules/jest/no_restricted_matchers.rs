use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    rules::shared::no_restricted_matchers as SharedNoRestrictedMatchers,
};

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedMatchers(Box<SharedNoRestrictedMatchers::NoRestrictedMatchersConfig>);

declare_oxc_lint!(
    NoRestrictedMatchers,
    jest,
    style,
    config = SharedNoRestrictedMatchers::NoRestrictedMatchersConfig,
    docs = SharedNoRestrictedMatchers::DOCUMENTATION,
    version = "0.2.3",
);

impl Rule for NoRestrictedMatchers {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(Self(Box::new(
            SharedNoRestrictedMatchers::NoRestrictedMatchersConfig::from_configuration(&value)?,
        )))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(_) = node.kind() else { return };
        self.0.run(node, ctx);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.frameworks().is_test()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // Note: Both Jest and Vitest share the same unit tests

    let pass = vec![
        ("expect(a).toHaveBeenCalled()", None),
        ("expect(a).not.toHaveBeenCalled()", None),
        ("expect(a).toHaveBeenCalledTimes()", None),
        ("expect(a).toHaveBeenCalledWith()", None),
        ("expect(a).toHaveBeenLastCalledWith()", None),
        ("expect(a).toHaveBeenNthCalledWith()", None),
        ("expect(a).toHaveReturned()", None),
        ("expect(a).toHaveReturnedTimes()", None),
        ("expect(a).toHaveReturnedWith()", None),
        ("expect(a).toHaveLastReturnedWith()", None),
        ("expect(a).toHaveNthReturnedWith()", None),
        ("expect(a).toThrow()", None),
        ("expect(a).rejects;", None),
        ("expect(a);", None),
        ("expect(a).resolves", Some(serde_json::json!([{ "not": null }]))),
        ("expect(a).toBe(b)", Some(serde_json::json!([{ "not.toBe": null }]))),
        ("expect(a).toBeUndefined(b)", Some(serde_json::json!([{ "toBe": null }]))),
        ("expect(a)[\"toBe\"](b)", Some(serde_json::json!([{ "not.toBe": null }]))),
        ("expect(a).resolves.not.toBe(b)", Some(serde_json::json!([{ "not": null }]))),
        ("expect(a).resolves.not.toBe(b)", Some(serde_json::json!([{ "not.toBe": null }]))),
        (
            "expect(uploadFileMock).resolves.toHaveBeenCalledWith('file.name')",
            Some(
                serde_json::json!([{ "not.toHaveBeenCalledWith": "Use not.toHaveBeenCalled instead" }]),
            ),
        ),
        (
            "expect(uploadFileMock).resolves.not.toHaveBeenCalledWith('file.name')",
            Some(
                serde_json::json!([{ "not.toHaveBeenCalledWith": "Use not.toHaveBeenCalled instead" }]),
            ),
        ),
    ];

    let fail = vec![
        ("expect(a).toBe(b)", Some(serde_json::json!([{ "toBe": null }]))),
        ("expect(a)[\"toBe\"](b)", Some(serde_json::json!([{ "toBe": null }]))),
        ("expect(a).not[x]()", Some(serde_json::json!([{ "not": null }]))),
        ("expect(a).not.toBe(b)", Some(serde_json::json!([{ "not": null }]))),
        ("expect(a).resolves.toBe(b)", Some(serde_json::json!([{ "resolves": null }]))),
        ("expect(a).resolves.not.toBe(b)", Some(serde_json::json!([{ "resolves": null }]))),
        ("expect(a).resolves.not.toBe(b)", Some(serde_json::json!([{ "resolves.not": null }]))),
        ("expect(a).not.toBe(b)", Some(serde_json::json!([{ "not.toBe": null }]))),
        (
            "expect(a).resolves.not.toBe(b)",
            Some(serde_json::json!([{ "resolves.not.toBe": null }])),
        ),
        (
            "expect(a).toBe(b)",
            Some(serde_json::json!([{ "toBe": "Prefer `toStrictEqual` instead" }])),
        ),
        (
            "
                test('some test', async () => {
                    await expect(Promise.resolve(1)).resolves.toBe(1);
                });
            ",
            Some(serde_json::json!([{ "resolves": "Use `expect(await promise)` instead." }])),
        ),
        (
            "expect(Promise.resolve({})).rejects.toBeFalsy()",
            Some(serde_json::json!([{ "rejects.toBeFalsy": null }])),
        ),
        (
            "expect(uploadFileMock).not.toHaveBeenCalledWith('file.name')",
            Some(serde_json::json!([
                { "not.toHaveBeenCalledWith": "Use not.toHaveBeenCalled instead" },
            ])),
        ),
    ];

    Tester::new(NoRestrictedMatchers::NAME, NoRestrictedMatchers::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
