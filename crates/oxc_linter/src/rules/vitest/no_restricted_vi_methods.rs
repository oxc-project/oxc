use oxc_macros::declare_oxc_lint;
use rustc_hash::FxHashMap;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, NoRestrictedTestMethods, NoRestrictedTestMethodsConfig,
    },
};

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedViMethods(Box<NoRestrictedTestMethodsConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Restrict the use of specific `vi` methods.
    ///
    /// ### Example
    /// ```javascript
    /// vi.useFakeTimers();
    /// it('calls the callback after 1 second via advanceTimersByTime', () => {
    ///   // ...
    ///
    ///   vi.advanceTimersByTime(1000);
    ///
    ///   // ...
    /// });
    ///
    /// test('plays video', () => {
    ///   const spy = vi.spyOn(video, 'play');
    ///
    ///   // ...
    /// });
    ///
    NoRestrictedViMethods,
    style,
);

impl NoRestrictedTestMethods for NoRestrictedViMethods {
    fn restricted_test_methods(&self) -> &FxHashMap<String, String> {
        &self.0.restricted_test_methods
    }
}

impl Rule for NoRestrictedViMethods {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(Self::get_configuration(&value)))
    }

    fn run_once(&self, ctx: &LintContext) {
        for possible_vitest_node in &collect_possible_jest_call_node(ctx) {
            self.run_test(possible_vitest_node, ctx, false);
        }
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

    Tester::new(NoRestrictedViMethods::NAME, pass, fail).test_and_snapshot();
}
