use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::{
        PossibleJestNode, shared::no_restricted_jest_methods as SharedNoRestrictedJestMethods,
    },
};

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedJestMethods(
    Box<SharedNoRestrictedJestMethods::NoRestrictedJestMethodsConfig>,
);

declare_oxc_lint!(
    NoRestrictedJestMethods,
    jest,
    style,
    config = SharedNoRestrictedJestMethods::NoRestrictedJestMethodsConfig,
    docs = SharedNoRestrictedJestMethods::DOCUMENTATION,
    version = "0.2.3",
);

impl Rule for NoRestrictedJestMethods {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(Self(Box::new(
            SharedNoRestrictedJestMethods::NoRestrictedJestMethodsConfig::from_configuration(
                &value,
            )?,
        )))
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        self.0.run(jest_node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("jest", None),
        ("jest()", None),
        ("jest.mock()", None),
        ("expect(a).rejects;", None),
        ("expect(a);", None),
        (
            "
                import { jest } from '@jest/globals';

                jest;
            ",
            None,
        ),
    ];

    let fail = vec![
        ("jest.fn()", Some(serde_json::json!([{ "fn": null }]))),
        ("jest[\"fn\"]()", Some(serde_json::json!([{ "fn": null }]))),
        ("jest.mock()", Some(serde_json::json!([{ "mock": "Do not use mocks" }]))),
        ("jest[\"mock\"]()", Some(serde_json::json!([{ "mock": "Do not use mocks" }]))),
        (
            "
                import { jest } from '@jest/globals';
                jest.advanceTimersByTime();
            ",
            Some(serde_json::json!([{ "advanceTimersByTime": null }])),
        ),
    ];

    Tester::new(NoRestrictedJestMethods::NAME, NoRestrictedJestMethods::PLUGIN, pass, fail)
        .test_and_snapshot();
}
