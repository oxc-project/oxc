use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    rules::shared::no_restricted_jest_methods as SharedNoRestrictedJestMethods,
};

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedViMethods(Box<SharedNoRestrictedJestMethods::NoRestrictedTestMethodsConfig>);

declare_oxc_lint!(
    NoRestrictedViMethods,
    vitest,
    style,
    config = SharedNoRestrictedJestMethods::NoRestrictedTestMethodsConfig,
    docs = SharedNoRestrictedJestMethods::DOCUMENTATION,
    version = "0.2.3",
);

impl Rule for NoRestrictedViMethods {
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
        ("vi", None),
        ("vi()", None),
        ("vi.mock()", None),
        ("expect(a).rejects;", None),
        ("expect(a);", None),
        (
            "
			     import { vi } from 'vitest';
			     vi;
			    ",
            None,
        ), // { "parserOptions": { "sourceType": "module" } }
    ];

    let fail = vec![
        ("vi.fn()", Some(serde_json::json!([{ "fn": null }]))),
        ("vi.mock()", Some(serde_json::json!([{ "mock": "Do not use mocks" }]))),
        (
            "
			     import { vi } from 'vitest';
			     vi.advanceTimersByTime();
			    ",
            Some(serde_json::json!([{ "advanceTimersByTime": null }])),
        ), // { "parserOptions": { "sourceType": "module" } },
        (r#"vi["fn"]()"#, Some(serde_json::json!([{ "fn": null }]))),
    ];

    Tester::new(NoRestrictedViMethods::NAME, NoRestrictedViMethods::PLUGIN, pass, fail)
        .test_and_snapshot();
}
