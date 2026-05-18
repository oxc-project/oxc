use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::require_top_level_describe::{DOCUMENTATION, RequireTopLevelDescribeConfig},
};

#[derive(Debug, Default, Clone)]
pub struct RequireTopLevelDescribe(Box<RequireTopLevelDescribeConfig>);

declare_oxc_lint!(
    RequireTopLevelDescribe,
    jest,
    style,
    config = RequireTopLevelDescribeConfig,
    docs = DOCUMENTATION,
    version = "0.4.2",
);

impl Rule for RequireTopLevelDescribe {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        RequireTopLevelDescribeConfig::from_configuration(value)
            .map(|config| Self(Box::new(config)))
    }

    fn run_once(&self, ctx: &LintContext) {
        self.0.run_once(ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("it.each()", None),
        ("describe(\"test suite\", () => { test(\"my test\") });", None),
        ("describe(\"test suite\", () => { it(\"my test\") });", None),
        (
            "
                describe(\"test suite\", () => {
                    beforeEach(\"a\", () => {});
                    describe(\"b\", () => {});
                    test(\"c\", () => {})
                });
            ",
            None,
        ),
        ("describe(\"test suite\", () => { beforeAll(\"my beforeAll\") });", None),
        ("describe(\"test suite\", () => { afterEach(\"my afterEach\") });", None),
        ("describe(\"test suite\", () => { afterAll(\"my afterAll\") });", None),
        (
            "
                describe(\"test suite\", () => {
                    it(\"my test\", () => {})
                    describe(\"another test suite\", () => {});
                    test(\"my other test\", () => {})
                });
            ",
            None,
        ),
        ("foo()", None),
        (
            "describe.each([1, true])(\"trues\", value => { it(\"an it\", () => expect(value).toBe(true) ); });",
            None,
        ),
        (
            "
                describe('%s', () => {
                    it('is fine', () => {
                        //
                    });
                });

                describe.each('world')('%s', () => {
                    it.each([1, 2, 3])('%n', () => {
                        //
                    });
                });
            ",
            None,
        ),
        (
            "
                describe.each('hello')('%s', () => {
                    it('is fine', () => {
                        //
                    });
                });

                describe.each('world')('%s', () => {
                    it.each([1, 2, 3])('%n', () => {
                        //
                    });
                });
        ",
            None,
        ),
        (
            "
                import { jest } from '@jest/globals';

                jest.doMock('my-module');
            ",
            None,
        ),
        ("jest.doMock(\"my-module\")", None),
        ("describe(\"test suite\", () => { test(\"my test\") });", None),
        ("foo()", None),
        (
            "describe.each([1, true])(\"trues\", value => { it(\"an it\", () => expect(value).toBe(true) ); });",
            None,
        ),
        (
            "
                describe('one', () => {});
                describe('two', () => {});
                describe('three', () => {});
            ",
            None,
        ),
        (
            "
                describe('one', () => {
                    describe('two', () => {});
                    describe('three', () => {});
                });
            ",
            Some(serde_json::json!([{ "maxNumberOfTopLevelDescribes": 1 }])),
        ),
    ];

    let fail = vec![
        ("beforeEach(\"my test\", () => {})", None),
        (
            "
                test(\"my test\", () => {})
                describe(\"test suite\", () => {});
            ",
            None,
        ),
        (
            "
                test(\"my test\", () => {})
                describe(\"test suite\", () => {
                    it(\"test\", () => {})
                });
            ",
            None,
        ),
        (
            "
                describe(\"test suite\", () => {});
                afterAll(\"my test\", () => {})
            ",
            None,
        ),
        (
            "
                import { describe, afterAll as onceEverythingIsDone } from '@jest/globals';

                describe(\"test suite\", () => {});
                onceEverythingIsDone(\"my test\", () => {})
            ",
            None,
        ),
        ("it.skip('test', () => {});", None),
        ("it.each([1, 2, 3])('%n', () => {});", None),
        ("it.skip.each([1, 2, 3])('%n', () => {});", None),
        ("it.skip.each``('%n', () => {});", None),
        ("it.each``('%n', () => {});", None),
        (
            "
                describe(\"one\", () => {});
                describe(\"two\", () => {});
                describe(\"three\", () => {});
            ",
            Some(serde_json::json!([{ "maxNumberOfTopLevelDescribes": 2 }])),
        ),
        (
            "
                describe('one', () => {
                    describe('one (nested)', () => {});
                    describe('two (nested)', () => {});
                });
                describe('two', () => {
                    describe('one (nested)', () => {});
                    describe('two (nested)', () => {});
                    describe('three (nested)', () => {});
                });
                describe('three', () => {
                    describe('one (nested)', () => {});
                    describe('two (nested)', () => {});
                    describe('three (nested)', () => {});
                });
            ",
            Some(serde_json::json!([{ "maxNumberOfTopLevelDescribes": 2 }])),
        ),
        (
            "
                import {
                    describe as describe1,
                    describe as describe2,
                    describe as describe3,
                } from '@jest/globals';

                describe1('one', () => {
                    describe('one (nested)', () => {});
                    describe('two (nested)', () => {});
                });
                describe2('two', () => {
                    describe('one (nested)', () => {});
                    describe('two (nested)', () => {});
                    describe('three (nested)', () => {});
                });
                describe3('three', () => {
                    describe('one (nested)', () => {});
                    describe('two (nested)', () => {});
                    describe('three (nested)', () => {});
                });
            ",
            Some(serde_json::json!([{ "maxNumberOfTopLevelDescribes": 2 }])),
        ),
        (
            "
                describe('one', () => {});
                describe('two', () => {});
                describe('three', () => {});
            ",
            Some(serde_json::json!([{ "maxNumberOfTopLevelDescribes": 1 }])),
        ),
    ];

    Tester::new(RequireTopLevelDescribe::NAME, RequireTopLevelDescribe::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
