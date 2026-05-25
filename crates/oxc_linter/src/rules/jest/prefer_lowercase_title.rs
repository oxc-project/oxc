use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_lowercase_title::{DOCUMENTATION, PreferLowercaseTitleConfig},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PreferLowercaseTitle(Box<PreferLowercaseTitleConfig>);

declare_oxc_lint!(
    PreferLowercaseTitle,
    jest,
    style,
    fix,
    config = PreferLowercaseTitleConfig,
    docs = DOCUMENTATION,
    version = "0.15.9",
);

impl Rule for PreferLowercaseTitle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        PreferLowercaseTitleConfig::from_configuration(value).map(|config| Self(Box::new(config)))
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        self.0.run_on_jest_node(possible_jest_node, ctx);
    }
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, tester::Tester};

    let pass = vec![
        ("it.each()", None),
        ("it.each()(1)", None),
        ("randomFunction()", None),
        ("foo.bar()", None),
        ("it()", None),
        ("it(' ', function () {})", None),
        ("it(true, function () {})", None),
        ("it(MY_CONSTANT, function () {})", None),
        ("it(\" \", function () {})", None),
        ("it(` `, function () {})", None),
        ("it('foo', function () {})", None),
        ("it(\"foo\", function () {})", None),
        ("it(`foo`, function () {})", None),
        ("it(\"<Foo/>\", function () {})", None),
        ("it(\"123 foo\", function () {})", None),
        ("it(42, function () {})", None),
        ("it(``)", None),
        ("it(\"\")", None),
        ("it(42)", None),
        ("test()", None),
        ("test('foo', function () {})", None),
        ("test(\"foo\", function () {})", None),
        ("test(`foo`, function () {})", None),
        ("test(\"<Foo/>\", function () {})", None),
        ("test(\"123 foo\", function () {})", None),
        ("test(\"42\", function () {})", None),
        ("test(``)", None),
        ("test(\"\")", None),
        ("test(42)", None),
        ("describe()", None),
        ("describe('foo', function () {})", None),
        ("describe(\"foo\", function () {})", None),
        ("describe(`foo`, function () {})", None),
        ("describe(\"<Foo/>\", function () {})", None),
        ("describe(\"123 foo\", function () {})", None),
        ("describe(\"42\", function () {})", None),
        ("describe(function () {})", None),
        ("describe(``)", None),
        ("describe(\"\")", None),
        (
            "
                describe.each()(1);
                describe.each()(2);
            ",
            None,
        ),
        ("jest.doMock(\"my-module\")", None),
        (
            "
                import { jest } from '@jest/globals';
                jest.doMock('my-module');
            ",
            None,
        ),
        ("describe(42)", None),
        ("describe(42)", Some(serde_json::json!([{ "ignore": [], "allowedPrefixes": [] }]))),
        ("describe('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["describe"] }]))),
        (
            "describe(\"Foo\", function () {})",
            Some(serde_json::json!([{ "ignore": ["describe"] }])),
        ),
        ("describe(`Foo`, function () {})", Some(serde_json::json!([{ "ignore": ["describe"] }]))),
        ("fdescribe(`Foo`, function () {})", Some(serde_json::json!([{ "ignore": ["describe"] }]))),
        (
            "describe.skip(`Foo`, function () {})",
            Some(serde_json::json!([{ "ignore": ["describe"] }])),
        ),
        ("test('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["test"] }]))),
        ("test(\"Foo\", function () {})", Some(serde_json::json!([{ "ignore": ["test"] }]))),
        ("test(`Foo`, function () {})", Some(serde_json::json!([{ "ignore": ["test"] }]))),
        ("xtest(`Foo`, function () {})", Some(serde_json::json!([{ "ignore": ["test"] }]))),
        ("test.only(`Foo`, function () {})", Some(serde_json::json!([{ "ignore": ["test"] }]))),
        ("it('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["it"] }]))),
        ("it(\"Foo\", function () {})", Some(serde_json::json!([{ "ignore": ["it"] }]))),
        ("it(`Foo`, function () {})", Some(serde_json::json!([{ "ignore": ["it"] }]))),
        ("fit(`Foo`, function () {})", Some(serde_json::json!([{ "ignore": ["it"] }]))),
        ("it.skip(`Foo`, function () {})", Some(serde_json::json!([{ "ignore": ["it"] }]))),
        (
            "it('GET /live', function () {})",
            Some(serde_json::json!([{ "allowedPrefixes": ["GET"] }])),
        ),
        (
            "it(\"POST /live\", function () {})",
            Some(serde_json::json!([{ "allowedPrefixes": ["GET", "POST"] }])),
        ),
        (
            "it(`PATCH /live`, function () {})",
            Some(serde_json::json!([{ "allowedPrefixes": ["GET", "PATCH"] }])),
        ),
        (
            "describe(\"MyClass\", () => {});",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
        (
            "
                describe('MyClass', () => {
                    describe('#myMethod', () => {
                        it('does things', () => {
                            //
                        });
                    });
                });
            ",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
        (
            "
                describe('Strings', () => {
                    it('are strings', () => {
                        expect('abc').toBe('abc');
                    });
                });

                describe('Booleans', () => {
                    it('are booleans', () => {
                        expect(true).toBe(true);
                    });
                });
            ",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
    ];

    let fail = vec![
        ("it('Foo', function () {})", None),
        ("xit('Foo', function () {})", None),
        ("it(\"Foo\", function () {})", None),
        ("it(`Foo`, function () {})", None),
        ("test('Foo', function () {})", None),
        ("xtest('Foo', function () {})", None),
        ("test(\"Foo\", function () {})", None),
        ("test(`Foo`, function () {})", None),
        ("describe('Foo', function () {})", None),
        ("describe(\"Foo\", function () {})", None),
        ("describe(`Foo`, function () {})", None),
        (
            "
                import { describe as context } from '@jest/globals';
                context(`Foo`, () => {});
            ",
            None,
        ),
        ("describe(`Some longer description`, function () {})", None),
        ("fdescribe(`Some longer description`, function () {})", None),
        ("it.each(['green', 'black'])('Should return %', () => {})", None),
        ("describe.each(['green', 'black'])('Should return %', () => {})", None),
        ("test('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["describe"] }]))),
        ("xit('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["describe"] }]))),
        ("describe('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["test"] }]))),
        ("it('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["test"] }]))),
        ("xit('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["test"] }]))),
        ("describe('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["it"] }]))),
        ("test('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["it"] }]))),
        ("xtest('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["it"] }]))),
        (
            "it(\"Works!\", () => {});",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
        (
            "
                describe('MyClass', () => {
                    describe('MyMethod', () => {
                        it('Does things', () => {
                            //
                        });
                    });
                });
            ",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
        (
            "
                import { describe, describe as context } from '@jest/globals';
                describe('MyClass', () => {
                    context('MyMethod', () => {
                        it('Does things', () => {
                            //
                        });
                    });
                });
            ",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
        (
            "
                describe('MyClass', () => {
                    describe('MyMethod', () => {
                        it('Does things', () => {
                            //
                        });
                    });
                });
            ",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": false }])),
        ),
    ];

    let fix = vec![
        ("it('Foo', function () {})", "it('foo', function () {})", None),
        ("xit('Foo', function () {})", "xit('foo', function () {})", None),
        ("it(\"Foo\", function () {})", "it(\"foo\", function () {})", None),
        ("it(`Foo`, function () {})", "it(`foo`, function () {})", None),
        ("test('Foo', function () {})", "test('foo', function () {})", None),
        ("xtest('Foo', function () {})", "xtest('foo', function () {})", None),
        ("test(\"Foo\", function () {})", "test(\"foo\", function () {})", None),
        ("test(`Foo`, function () {})", "test(`foo`, function () {})", None),
        ("describe('Foo', function () {})", "describe('foo', function () {})", None),
        ("describe(\"Foo\", function () {})", "describe(\"foo\", function () {})", None),
        ("describe(`Foo`, function () {})", "describe(`foo`, function () {})", None),
        (
            "
                import { describe as context } from '@jest/globals';
                context(`Foo`, () => {});
            ",
            "
                import { describe as context } from '@jest/globals';
                context(`foo`, () => {});
            ",
            None,
        ),
        (
            "describe(`Some longer description`, function () {})",
            "describe(`some longer description`, function () {})",
            None,
        ),
        (
            "fdescribe(`Some longer description`, function () {})",
            "fdescribe(`some longer description`, function () {})",
            None,
        ),
        (
            "it.each(['green', 'black'])('Should return %', () => {})",
            "it.each(['green', 'black'])('should return %', () => {})",
            None,
        ),
        (
            "describe.each(['green', 'black'])('Should return %', () => {})",
            "describe.each(['green', 'black'])('should return %', () => {})",
            None,
        ),
        (
            "test('Foo', function () {})",
            "test('foo', function () {})",
            Some(serde_json::json!([{ "ignore": ["describe"] }])),
        ),
        (
            "xit('Foo', function () {})",
            "xit('foo', function () {})",
            Some(serde_json::json!([{ "ignore": ["describe"] }])),
        ),
        (
            "describe('Foo', function () {})",
            "describe('foo', function () {})",
            Some(serde_json::json!([{ "ignore": ["test"] }])),
        ),
        (
            "it('Foo', function () {})",
            "it('foo', function () {})",
            Some(serde_json::json!([{ "ignore": ["test"] }])),
        ),
        (
            "xit('Foo', function () {})",
            "xit('foo', function () {})",
            Some(serde_json::json!([{ "ignore": ["test"] }])),
        ),
        (
            "describe('Foo', function () {})",
            "describe('foo', function () {})",
            Some(serde_json::json!([{ "ignore": ["it"] }])),
        ),
        (
            "test('Foo', function () {})",
            "test('foo', function () {})",
            Some(serde_json::json!([{ "ignore": ["it"] }])),
        ),
        (
            "xtest('Foo', function () {})",
            "xtest('foo', function () {})",
            Some(serde_json::json!([{ "ignore": ["it"] }])),
        ),
        (
            "it(\"Works!\", () => {});",
            "it(\"works!\", () => {});",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
        (
            "
                describe('MyClass', () => {
                    describe('MyMethod', () => {
                        it('Does things', () => {
                            //
                        });
                    });
                });
            ",
            "
                describe('MyClass', () => {
                    describe('myMethod', () => {
                        it('does things', () => {
                            //
                        });
                    });
                });
            ",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
        (
            "
                import { describe, describe as context } from '@jest/globals';
                describe('MyClass', () => {
                    context('MyMethod', () => {
                        it('Does things', () => {
                            //
                        });
                    });
                });
            ",
            "
                import { describe, describe as context } from '@jest/globals';
                describe('MyClass', () => {
                    context('myMethod', () => {
                        it('does things', () => {
                            //
                        });
                    });
                });
            ",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
        (
            "
                describe('MyClass', () => {
                    describe('MyMethod', () => {
                        it('Does things', () => {
                            //
                        });
                    });
                });
            ",
            "
                describe('myClass', () => {
                    describe('myMethod', () => {
                        it('does things', () => {
                            //
                        });
                    });
                });
            ",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": false }])),
        ),
    ];

    Tester::new(PreferLowercaseTitle::NAME, PreferLowercaseTitle::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
