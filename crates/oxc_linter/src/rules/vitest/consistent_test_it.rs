use oxc_macros::declare_oxc_lint;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::consistent_test_it::{ConsistentTestItConfig, DOCUMENTATION},
};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ConsistentTestIt(Box<ConsistentTestItConfig>);

declare_oxc_lint!(
    ConsistentTestIt,
    vitest,
    style,
    fix,
    config = ConsistentTestItConfig,
    docs = DOCUMENTATION,
);

impl Rule for ConsistentTestIt {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        ConsistentTestItConfig::from_configuration(&value).map(|config| Self(Box::new(config)))
    }

    fn run_once(&self, ctx: &LintContext) {
        self.0.run_once(ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
                it(\"shows error\", () => {
                    expect(true).toBe(false);
                });
            ",
            Some(serde_json::json!([{ "fn": "it" }])),
        ),
        (
            "
                it(\"foo\", function () {
                    expect(true).toBe(false);
                })
            ",
            Some(serde_json::json!([{ "fn": "it" }])),
        ),
        (
            "
                it('foo', () => {
                        expect(true).toBe(false);
                    });
                function myTest() { if ('bar') {} }
            ",
            Some(serde_json::json!([{ "fn": "it" }])),
        ),
        (
            "
                test(\"shows error\", () => {
                    expect(true).toBe(false);
                });
            ",
            Some(serde_json::json!([{ "fn": "test" }])),
        ),
        ("test.skip(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("test.concurrent(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("xtest(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("test.each([])(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("test.each``(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test" }])),
        ),
        (
            "describe(\"suite\", () => { it(\"foo\") })",
            Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }])),
        ),
        ("it(\"foo\")", Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }]))),
        ("test(\"shows error\", () => {});", None),
        ("test(\"foo\")", Some(serde_json::json!([{ "withinDescribe": "it" }]))),
        (
            "describe(\"suite\", () => { it(\"foo\") })",
            Some(serde_json::json!([{ "withinDescribe": "it" }])),
        ),
        ("test(\"foo\")", Some(serde_json::json!([{ "withinDescribe": "test" }]))),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "withinDescribe": "test" }])),
        ),
    ];

    let fail = vec![
        ("test(\"shows error\", () => {});", Some(serde_json::json!([{ "fn": "it" }]))),
        ("test.skip(\"shows error\");", Some(serde_json::json!([{ "fn": "it" }]))),
        ("test.only('shows error');", Some(serde_json::json!([{ "fn": "it" }]))),
        (
            "describe('foo', () => { it('bar', () => {}); });",
            Some(serde_json::json!([{ "fn": "it", "withinDescribe": "test" }])),
        ),
        (
            "import { test } from \"vitest\"\ntest(\"shows error\", () => {});",
            Some(serde_json::json!([{ "fn": "it" }])),
        ),
        ("it(\"shows error\", () => {});", Some(serde_json::json!([{ "fn": "test" }]))),
        ("describe(\"suite\", () => { it(\"foo\") })", Some(serde_json::json!([{ "fn": "test" }]))),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }])),
        ),
        ("test(\"foo\")", Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }]))),
        ("describe(\"suite\", () => { test(\"foo\") })", None),
        ("it(\"foo\")", Some(serde_json::json!([{ "withinDescribe": "it" }]))),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "withinDescribe": "it" }])),
        ),
        ("it(\"foo\")", Some(serde_json::json!([{ "withinDescribe": "test" }]))),
        (
            "import { it } from \"vitest\"\nit(\"foo\")",
            Some(serde_json::json!([{ "withinDescribe": "test" }])),
        ),
        (
            "describe(\"suite\", () => { it(\"foo\") })",
            Some(serde_json::json!([{ "withinDescribe": "test" }])),
        ),
    ];

    let fix = vec![
        // Note: couldn't  fixed, because the fixer doesn't support to set the options for the fix cases.
        // Todo: this need to fixer support option configuration.
        // ("test(\"shows error\", () => {});", "it(\"shows error\", () => {});"),
        // ("test.skip(\"shows error\");", "it.skip(\"shows error\");"),
        // ("test.only('shows error');", "it.only('shows error');"),
        // (
        //     "describe('foo', () => { it('bar', () => {}); });",
        //     "describe('foo', () => { test('bar', () => {}); });"
        // ),
        // (
        //     "import { test } from \"vitest\"\ntest(\"shows error\", () => {});",
        //     "import { it } from \"vitest\"\nit(\"shows error\", () => {});",
        // ),
        // ("describe(\"suite\", () => { it(\"foo\") })", "describe(\"suite\", () => { test(\"foo\") })"),
        // ("test(\"foo\")", "it(\"foo\")"),
        // ("describe(\"suite\", () => { it(\"foo\") })", "describe(\"suite\", () => { test(\"foo\") })"),
        //
        ("it(\"shows error\", () => {});", "test(\"shows error\", () => {});"),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        ("it(\"foo\")", "test(\"foo\")"),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        ("it(\"foo\")", "test(\"foo\")"),
        // Todo: need to be fixed
        // (
        //     "import { it } from \"vitest\"\nit(\"foo\")",
        //     "import { test } from \"vitest\"\ntest(\"foo\")"
        // ),
    ];

    Tester::new(ConsistentTestIt::NAME, ConsistentTestIt::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
