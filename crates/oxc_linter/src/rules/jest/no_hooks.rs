use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    rules::shared::no_hooks::{DOCUMENTATION, NoHooksConfig},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct NoHooks(Box<NoHooksConfig>);

declare_oxc_lint!(
    NoHooks,
    jest,
    style,
    config = NoHooksConfig,
    docs = DOCUMENTATION,
    version = "0.0.16",
);

impl Rule for NoHooks {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let config = serde_json::from_value::<DefaultRuleConfig<NoHooksConfig>>(value)?;
        Ok(Self(Box::new(config.into_inner())))
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        self.0.run_on_jest_node(jest_node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("test(\"foo\")", None),
        ("describe(\"foo\", () => { it(\"bar\") })", None),
        ("test(\"foo\", () => { expect(subject.beforeEach()).toBe(true) })", None),
        (
            "afterEach(() => {}); afterAll(() => {});",
            Some(serde_json::json!([{ "allow": ["afterEach", "afterAll"] }])),
        ),
    ];

    let fail = vec![
        ("beforeAll(() => {})", None),
        ("beforeEach(() => {})", None),
        ("afterAll(() => {})", None),
        ("afterEach(() => {})", None),
        (
            "beforeEach(() => {}); afterEach(() => { jest.resetModules() });",
            Some(serde_json::json!([{ "allow": ["afterEach"] }])),
        ),
        (
            "
                import { beforeEach as afterEach, afterEach as beforeEach } from '@jest/globals';

                afterEach(() => {});
                beforeEach(() => { jest.resetModules() });
            ",
            Some(serde_json::json!([{ "allow": ["afterEach"] }])),
        ),
    ];

    Tester::new(NoHooks::NAME, NoHooks::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}

#[test]
fn invalid_configs_error_in_from_configuration() {
    // An array with an object that has unknown keys should produce an error
    let invalid = serde_json::json!([{ "foo": "bar" }]);
    assert!(NoHooks::from_configuration(invalid).is_err());

    // Configs containing `null` or the string "undefined" should be rejected under strict validation
    let undefined_allow = serde_json::json!([{ "allow": "undefined" }]);
    assert!(NoHooks::from_configuration(undefined_allow).is_err());

    let null_allow = serde_json::json!([{ "allow": null }]);
    assert!(NoHooks::from_configuration(null_allow).is_err());
}
