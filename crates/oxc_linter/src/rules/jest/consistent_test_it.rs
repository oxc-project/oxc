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
    jest,
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
        // consistent-test-it with fn=test
        ("test(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("test.only(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("test.skip(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("test.concurrent(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("xtest(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("test.each([])(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("test.each``(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test" }])),
        ),
        // consistent-test-it with fn=it
        ("it(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("fit(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("xit(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("it.only(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("it.skip(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("it.concurrent(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("it.each([])(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("it.each``(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("describe(\"suite\", () => { it(\"foo\") })", Some(serde_json::json!([{ "fn": "it" }]))),
        // consistent-test-it with fn=test and withinDescribe=it
        ("test(\"foo\")", Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }]))),
        ("test.only(\"foo\")", Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }]))),
        ("test.skip(\"foo\")", Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }]))),
        (
            "test.concurrent(\"foo\")",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        ("xtest(\"foo\")", Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }]))),
        (
            "[1,2,3].forEach(() => { test(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        // consistent-test-it with fn=it and withinDescribe=test
        ("it(\"foo\")", Some(serde_json::json!([{ "fn": "it", "withinDescribe": "test" }]))),
        ("it.only(\"foo\")", Some(serde_json::json!([{ "fn": "it", "withinDescribe": "test" }]))),
        ("it.skip(\"foo\")", Some(serde_json::json!([{ "fn": "it", "withinDescribe": "test" }]))),
        (
            "it.concurrent(\"foo\")",
            Some(serde_json::json!([{ "fn": "it", "withinDescribe": "test" }])),
        ),
        ("xit(\"foo\")", Some(serde_json::json!([{ "fn": "it", "withinDescribe": "test" }]))),
        (
            "[1,2,3].forEach(() => { it(\"foo\") })",
            Some(serde_json::json!([{ "fn": "it", "withinDescribe": "test" }])),
        ),
        // consistent-test-it with fn=test and withinDescribe=test
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "test" }])),
        ),
        ("test(\"foo\");", Some(serde_json::json!([{ "fn": "test", "withinDescribe": "test" }]))),
        // consistent-test-it with fn=it and withinDescribe=it
        (
            "describe(\"suite\", () => { it(\"foo\") })",
            Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }])),
        ),
        ("it(\"foo\")", Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }]))),
        // consistent-test-it defaults without config object
        ("test(\"foo\")", None),
        // consistent-test-it with withinDescribe=it
        ("test(\"foo\")", Some(serde_json::json!([{ "withinDescribe": "it" }]))),
        (
            "describe(\"suite\", () => { it(\"foo\") })",
            Some(serde_json::json!([{ "withinDescribe": "it" }])),
        ),
        // consistent-test-it with withinDescribe=test
        ("test(\"foo\")", Some(serde_json::json!([{ "withinDescribe": "test" }]))),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "withinDescribe": "test" }])),
        ),
    ];

    let fail = vec![
        // consistent-test-it with fn=test
        ("it(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        (
            "
                import { it } from '@jest/globals';

                it(\"foo\")
            ",
            Some(serde_json::json!([{ "fn": "test" }])),
        ),
        (
            "
                import { it as testThisThing } from '@jest/globals';

                testThisThing(\"foo\")
            ",
            Some(serde_json::json!([{ "fn": "test" }])),
        ),
        ("xit(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("fit(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("it.skip(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("it.concurrent(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("it.only(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("it.each([])(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        ("it.each``(\"foo\")", Some(serde_json::json!([{ "fn": "test" }]))),
        (
            "describe.each``(\"foo\", () => { it.each``(\"bar\") })",
            Some(serde_json::json!([{ "fn": "test" }])),
        ),
        (
            "describe.each``(\"foo\", () => { test.each``(\"bar\") })",
            Some(serde_json::json!([{ "fn": "it" }])),
        ),
        (
            "
                describe.each()(\"%s\", () => {
                    test(\"is valid, but should not be\", () => {});

                    it(\"is not valid, but should be\", () => {});
                });
            ",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "
                describe.only.each()(\"%s\", () => {
                    test(\"is valid, but should not be\", () => {});

                    it(\"is not valid, but should be\", () => {});
                });
            ",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        ("describe(\"suite\", () => { it(\"foo\") })", Some(serde_json::json!([{ "fn": "test" }]))),
        // consistent-test-it with fn=it
        ("test(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("xtest(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("test.skip(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("test.concurrent(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("test.only(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("test.each([])(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        (
            "describe.each``(\"foo\", () => { test.each``(\"bar\") })",
            Some(serde_json::json!([{ "fn": "it" }])),
        ),
        ("test.each``(\"foo\")", Some(serde_json::json!([{ "fn": "it" }]))),
        ("describe(\"suite\", () => { test(\"foo\") })", Some(serde_json::json!([{ "fn": "it" }]))),
        // consistent-test-it with fn=test and withinDescribe=it
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "describe(\"suite\", () => { test.only(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "describe(\"suite\", () => { xtest(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "
                import { xtest as dontTestThis } from '@jest/globals';

                describe(\"suite\", () => { dontTestThis(\"foo\") });
            ",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "
                import { describe as context, xtest as dontTestThis } from '@jest/globals';

                context(\"suite\", () => { dontTestThis(\"foo\") });
            ",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "describe(\"suite\", () => { test.skip(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "describe(\"suite\", () => { test.concurrent(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        // consistent-test-it with fn=it and withinDescribe=test
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "describe(\"suite\", () => { test.only(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "describe(\"suite\", () => { xtest(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "
                import { xtest as dontTestThis } from '@jest/globals';

                describe(\"suite\", () => { dontTestThis(\"foo\") });
            ",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "
            import { describe as context, xtest as dontTestThis } from '@jest/globals';

            context(\"suite\", () => { dontTestThis(\"foo\") });
        ",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "describe(\"suite\", () => { test.skip(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        (
            "describe(\"suite\", () => { test.concurrent(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "it" }])),
        ),
        // consistent-test-it with fn=test and withinDescribe=test
        (
            "describe(\"suite\", () => { it(\"foo\") })",
            Some(serde_json::json!([{ "fn": "test", "withinDescribe": "test" }])),
        ),
        ("it(\"foo\")", Some(serde_json::json!([{ "fn": "test", "withinDescribe": "test" }]))),
        // consistent-test-it with fn=it and withinDescribe=it
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }])),
        ),
        ("test(\"foo\")", Some(serde_json::json!([{ "fn": "it", "withinDescribe": "it" }]))),
        // consistent-test-it defaults without config object
        ("describe(\"suite\", () => { test(\"foo\") })", None),
        // consistent-test-it with withinDescribe=it
        ("it(\"foo\")", Some(serde_json::json!([{ "withinDescribe": "it" }]))),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            Some(serde_json::json!([{ "withinDescribe": "it" }])),
        ),
        // consistent-test-it with withinDescribe=test
        ("it(\"foo\")", Some(serde_json::json!([{ "withinDescribe": "test" }]))),
        (
            "describe(\"suite\", () => { it(\"foo\") })",
            Some(serde_json::json!([{ "withinDescribe": "test" }])),
        ),
    ];

    let fix = vec![
        // consistent-test-it with fn=test
        ("it(\"foo\")", "test(\"foo\")"),
        (
            "
                import { it } from '@jest/globals';
                it(\"foo\")
            ",
            "
                import { it } from '@jest/globals';
                test(\"foo\")
            ",
        ),
        (
            "
                import { it as testThisThing } from '@jest/globals';
                testThisThing(\"foo\")
            ",
            "
                import { it as testThisThing } from '@jest/globals';
                test(\"foo\")
            ",
        ),
        ("xit(\"foo\")", "xtest(\"foo\")"),
        ("fit(\"foo\")", "test.only(\"foo\")"),
        ("it.skip(\"foo\")", "test.skip(\"foo\")"),
        ("it.concurrent(\"foo\")", "test.concurrent(\"foo\")"),
        ("it.only(\"foo\")", "test.only(\"foo\")"),
        ("it.each([])(\"foo\")", "test.each([])(\"foo\")"),
        ("it.each``(\"foo\")", "test.each``(\"foo\")"),
        // Note: couldn't fix
        // Todo: this need to fixer support option configuration.
        // (
        //     "describe.each``(\"foo\", () => { it.each``(\"bar\") })",
        //     "describe.each``(\"foo\", () => { test.each``(\"bar\") })",
        // ),
        (
            "describe.each``(\"foo\", () => { test.each``(\"bar\") })",
            "describe.each``(\"foo\", () => { it.each``(\"bar\") })",
        ),
        (
            "
                describe.each()(\"%s\", () => {
                    test(\"is valid, but should not be\", () => {});
                    it(\"is not valid, but should be\", () => {});
                });
            ",
            "
                describe.each()(\"%s\", () => {
                    it(\"is valid, but should not be\", () => {});
                    it(\"is not valid, but should be\", () => {});
                });
            ",
        ),
        (
            "
                describe.only.each()(\"%s\", () => {
                    test(\"is valid, but should not be\", () => {});
                    it(\"is not valid, but should be\", () => {});
                });
            ",
            "
                describe.only.each()(\"%s\", () => {
                    it(\"is valid, but should not be\", () => {});
                    it(\"is not valid, but should be\", () => {});
                });
            ",
        ),
        // Note: couldn't fix, because the fixer couldn't be set option `fn=it`
        // (
        //     "describe(\"suite\", () => { it(\"foo\") })",
        //     "describe(\"suite\", () => { test(\"foo\") })",
        // ),
        // consistent-test-it with fn=it
        // ("test(\"foo\")", "it(\"foo\")"),
        // ("xtest(\"foo\")", "xit(\"foo\")"),
        // ("test.skip(\"foo\")", "it.skip(\"foo\")"),
        // ("test.concurrent(\"foo\")", "it.concurrent(\"foo\")"),
        // ("test.only(\"foo\")", "it.only(\"foo\")"),
        // ("test.each([])(\"foo\")", "it.each([])(\"foo\")"),
        // ("test.each``(\"foo\")", "it.each``(\"foo\")"),
        (
            "describe.each``(\"foo\", () => { test.each``(\"bar\") })",
            "describe.each``(\"foo\", () => { it.each``(\"bar\") })",
        ),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        //
        // consistent-test-it with fn=test and withinDescribe=it
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        (
            "describe(\"suite\", () => { test.only(\"foo\") })",
            "describe(\"suite\", () => { it.only(\"foo\") })",
        ),
        (
            "describe(\"suite\", () => { xtest(\"foo\") })",
            "describe(\"suite\", () => { xit(\"foo\") })",
        ),
        (
            "
                import { xtest as dontTestThis } from '@jest/globals';
                describe(\"suite\", () => { dontTestThis(\"foo\") });
            ",
            "
                import { xtest as dontTestThis } from '@jest/globals';
                describe(\"suite\", () => { xit(\"foo\") });
            ",
        ),
        (
            "
                import { describe as context, xtest as dontTestThis } from '@jest/globals';
                context(\"suite\", () => { dontTestThis(\"foo\") });
            ",
            "
                import { describe as context, xtest as dontTestThis } from '@jest/globals';
                context(\"suite\", () => { xit(\"foo\") });
            ",
        ),
        (
            "describe(\"suite\", () => { test.skip(\"foo\") })",
            "describe(\"suite\", () => { it.skip(\"foo\") })",
        ),
        (
            "describe(\"suite\", () => { test.concurrent(\"foo\") })",
            "describe(\"suite\", () => { it.concurrent(\"foo\") })",
        ),
        // consistent-test-it with fn=it and withinDescribe=test
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        (
            "describe(\"suite\", () => { test.only(\"foo\") })",
            "describe(\"suite\", () => { it.only(\"foo\") })",
        ),
        (
            "describe(\"suite\", () => { xtest(\"foo\") })",
            "describe(\"suite\", () => { xit(\"foo\") })",
        ),
        (
            "
                import { xtest as dontTestThis } from '@jest/globals';
                describe(\"suite\", () => { dontTestThis(\"foo\") });
            ",
            "
                import { xtest as dontTestThis } from '@jest/globals';
                describe(\"suite\", () => { xit(\"foo\") });
            ",
        ),
        (
            "
                import { describe as context, xtest as dontTestThis } from '@jest/globals';
                context(\"suite\", () => { dontTestThis(\"foo\") });
            ",
            "
                import { describe as context, xtest as dontTestThis } from '@jest/globals';
                context(\"suite\", () => { xit(\"foo\") });
            ",
        ),
        (
            "describe(\"suite\", () => { test.skip(\"foo\") })",
            "describe(\"suite\", () => { it.skip(\"foo\") })",
        ),
        (
            "describe(\"suite\", () => { test.concurrent(\"foo\") })",
            "describe(\"suite\", () => { it.concurrent(\"foo\") })",
        ),
        // Note: couldn't fix
        // Todo: this need to fixer support option configuration.
        // consistent-test-it with fn=test and withinDescribe=test
        // (
        //     "describe(\"suite\", () => { it(\"foo\") })",
        //     "describe(\"suite\", () => { test(\"foo\") })",
        // ),
        // ("it(\"foo\")", "test(\"foo\")"),
        //
        // consistent-test-it with fn=it and withinDescribe=it
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        // ("test(\"foo\")", "it(\"foo\")"),
        // consistent-test-it defaults without config object
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        // consistent-test-it with withinDescribe=it
        ("it(\"foo\")", "test(\"foo\")"),
        (
            "describe(\"suite\", () => { test(\"foo\") })",
            "describe(\"suite\", () => { it(\"foo\") })",
        ),
        // consistent-test-it with withinDescribe=test
        ("it(\"foo\")", "test(\"foo\")"),
        // Note: couldn't fixed
        // Todo: this need to fixer support option configuration.
        // (
        //     "describe(\"suite\", () => { it(\"foo\") })",
        //     "describe(\"suite\", () => { test(\"foo\") })",
        // ),
    ];

    Tester::new(ConsistentTestIt::NAME, ConsistentTestIt::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
