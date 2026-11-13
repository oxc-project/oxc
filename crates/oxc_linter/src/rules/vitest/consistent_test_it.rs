use oxc_macros::declare_oxc_lint;
use serde::Deserialize;

use crate::{
    context::LintContext, rule::Rule, rules::shared::consistent_test_it::ConsistentTestItConfig,
};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ConsistentTestIt(Box<ConsistentTestItConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Vitest allows you to choose how you want to define your tests, using the `it`
    /// or the `test` keywords, with multiple permutations for each:
    ///
    /// - **it:** `it`, `it.only`, `it.skip`, `it.concurrent`, `it.each`.
    /// - **test:** `test`, `test.only`, `test.skip`, `test.concurrent`, `test.each`.
    ///
    /// ### Why is this bad?
    ///
    /// It's a good practice to be consistent in your test suite, so that all tests are written in the same way.
    ConsistentTestIt,
    vitest,
    style,
    conditional_fix,
    config = ConsistentTestItConfig,
    version = "0.5.3",
    short_description =
        "Enforce consistent use of either the `it` or `test` keyword for defining tests.",
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
        (
            "
                describe('suite', () => {});

                test('foo', () => {});
            ",
            Some(serde_json::json!([{ "withinDescribe": "it" }])),
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
        (
            "import {describe, expect, test, it} from 'vitest';

                describe('example', () => {
                  const testExtended = test.extend<{ result: number }>({
                    result: async ({}, use) => {
                      await use(42);
                    },
                  });

                  testExtended('works', ({ result }) => {
                    expect(result).toBe(42);
                  });

                  it('works', () => {
                    expect(42).toBe(42);
                  })
                });

                ",
            Some(serde_json::json!([{ "withinDescribe": "it" }])),
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

/// An imported `it` or `test` is renamed only when the new name is imported from the test framework
/// too; otherwise the call would be left unbound, so it is reported without a fix (#29181).
#[test]
fn test_imported_test_function_keeps_its_binding() {
    use crate::tester::Tester;

    let fn_test = Some(serde_json::json!([{ "fn": "test" }]));
    let fn_it = Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }]));
    let only_it = "import { it } from \"vitest\";\nit(\"example\", () => {});";
    let both = "import { it, test } from \"vitest\";\nit(\"example\", () => {});";
    let only_test = "import { test } from \"vitest\";\ntest(\"example\", () => {});";
    // `test` is bound, but to another function or to a local: renaming to it would call that.
    let describe_as_test =
        "import { describe as test, it } from \"vitest\";\nit(\"example\", () => {});";
    let local_test = "const test = () => {};\nit(\"example\", () => {});";
    // `test` is imported under another name: not renamed to the alias (reported without a fix).
    let test_as_t = "import { it, test as t } from \"vitest\";\nit(\"example\", () => {});";
    let effect = "import { it, test } from \"@effect/vitest\";\nit(\"example\", () => {});";
    let fail = vec![
        (only_it, fn_test.clone()),
        (both, fn_test.clone()),
        (only_test, fn_it.clone()),
        (describe_as_test, fn_test.clone()),
        (local_test, fn_test.clone()),
        (test_as_t, fn_test.clone()),
        (effect, fn_test.clone()),
    ];
    let fix = vec![
        (only_it, only_it, fn_test.clone()),
        (
            both,
            "import { it, test } from \"vitest\";\ntest(\"example\", () => {});",
            fn_test.clone(),
        ),
        (only_test, only_test, fn_it),
        (describe_as_test, describe_as_test, fn_test.clone()),
        (local_test, local_test, fn_test.clone()),
        (test_as_t, test_as_t, fn_test.clone()),
        (
            effect,
            "import { it, test } from \"@effect/vitest\";\ntest(\"example\", () => {});",
            fn_test,
        ),
    ];
    Tester::new(ConsistentTestIt::NAME, ConsistentTestIt::PLUGIN, vec![], fail)
        .expect_fix(fix)
        .test();
}
