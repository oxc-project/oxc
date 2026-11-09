use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    rules::shared::no_restricted_jest_methods as SharedNoRestrictedJestMethods,
};

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedJestMethods(
    Box<SharedNoRestrictedJestMethods::NoRestrictedTestMethodsConfig>,
);

declare_oxc_lint!(
    NoRestrictedJestMethods,
    jest,
    style,
    config = SharedNoRestrictedJestMethods::NoRestrictedTestMethodsConfig,
    docs = SharedNoRestrictedJestMethods::DOCUMENTATION,
    version = "0.2.3",
);

impl Rule for NoRestrictedJestMethods {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(Self(Box::new(
            SharedNoRestrictedJestMethods::NoRestrictedTestMethodsConfig::from_configuration(
                &value,
            )?,
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
