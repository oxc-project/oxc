use oxc_macros::declare_oxc_shared_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::{PossibleJestNode, shared::valid_title as SharedValidTitle},
};

#[derive(Debug, Default, Clone)]
pub struct ValidTitle(Box<SharedValidTitle::ValidTitleConfig>);

declare_oxc_shared_lint!(
    ValidTitle,
    vitest,
    correctness,
    conditional_fix,
    shared_docs = crate::rules::shared::valid_title
);

impl Rule for ValidTitle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        SharedValidTitle::ValidTitleConfig::from_configuration(&value)
            .map(|config| Self(Box::new(config)))
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        SharedValidTitle::ValidTitleConfig::run_rule(&self.0, jest_node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("describe('the correct way to properly handle all the things', () => {});", None),
        ("test('that all is as it should be', () => {});", None),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([
              { "ignoreTypeOfDescribeName": false, "disallowedWords": ["correct"] },
            ])),
        ),
        ("it('correctly sets the value', () => {});", Some(serde_json::json!([]))),
        ("describe('the correct way to properly handle all the things', () => {});", None),
        ("test('that all is as it should be', () => {});", None),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([{ "mustMatch": {} }])),
        ),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([{ "mustMatch": " " }])),
        ),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([{ "mustMatch": [" "] }])),
        ),
        (
            "it('correctly sets the value #unit', () => {});",
            Some(serde_json::json!([{ "mustMatch": "#(?:unit|integration|e2e)" }])),
        ),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([{ "mustMatch": "^[^#]+$|(?:#(?:unit|e2e))" }])),
        ),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([{ "mustMatch": { "test": "#(?:unit|integration|e2e)" } }])),
        ),
        (
            "
            describe('things to test', () => {
                describe('unit tests #unit', () => {
                it('is true', () => {
                    expect(true).toBe(true);
                });
                });

                describe('e2e tests #e2e', () => {
                it('is another test #jest4life', () => {});
                });
            });
            ",
            Some(serde_json::json!([{ "mustMatch": { "test": "^[^#]+$|(?:#(?:unit|e2e))" } }])),
        ),
        ("it('is a string', () => {});", None),
        ("it('is' + ' a ' + ' string', () => {});", None),
        ("it(1 + ' + ' + 1, () => {});", None),
        ("test('is a string', () => {});", None),
        ("xtest('is a string', () => {});", None),
        ("xtest(`${myFunc} is a string`, () => {});", None),
        ("describe('is a string', () => {});", None),
        ("describe.skip('is a string', () => {});", None),
        ("describe.skip(`${myFunc} is a string`, () => {});", None),
        ("fdescribe('is a string', () => {});", None),
        (
            "describe(String(/.+/), () => {});",
            Some(serde_json::json!([{ "ignoreTypeOfDescribeName": true }])),
        ),
        (
            "describe(myFunction, () => {});",
            Some(serde_json::json!([{ "ignoreTypeOfDescribeName": true }])),
        ),
        (
            "xdescribe(skipFunction, () => {});",
            Some(serde_json::json!([{ "ignoreTypeOfDescribeName": true, "disallowedWords": [] }])),
        ),
        ("describe()", None),
        ("someFn('', function () {})", None),
        ("describe('foo', function () {})", None),
        ("describe('foo', function () { it('bar', function () {}) })", None),
        ("test('foo', function () {})", None),
        ("test.concurrent('foo', function () {})", None),
        ("test(`foo`, function () {})", None),
        ("test.concurrent(`foo`, function () {})", None),
        ("test(`${foo}`, function () {})", None),
        ("test.concurrent(`${foo}`, function () {})", None),
        ("it('foo', function () {})", None),
        ("it.each([])()", None),
        ("it.concurrent('foo', function () {})", None),
        ("xdescribe('foo', function () {})", None),
        ("xit('foo', function () {})", None),
        ("xtest('foo', function () {})", None),
        ("it()", None),
        ("it.concurrent()", None),
        ("describe()", None),
        ("it.each()()", None),
        ("describe('foo', function () {})", None),
        ("fdescribe('foo', function () {})", None),
        ("xdescribe('foo', function () {})", None),
        ("it('foo', function () {})", None),
        ("it.concurrent('foo', function () {})", None),
        ("fit('foo', function () {})", None),
        ("fit.concurrent('foo', function () {})", None),
        ("xit('foo', function () {})", None),
        ("test('foo', function () {})", None),
        ("test.concurrent('foo', function () {})", None),
        ("xtest('foo', function () {})", None),
        ("xtest(`foo`, function () {})", None),
        ("someFn('foo', function () {})", None),
        (
            "
                describe('foo', () => {
                it('bar', () => {})
                })
            ",
            None,
        ),
        (
            "it(`GIVEN...
            `, () => {});",
            Some(serde_json::json!([{ "ignoreSpaces": true }])),
        ),
        ("describe('foo', function () {})", None),
        ("fdescribe('foo', function () {})", None),
        ("xdescribe('foo', function () {})", None),
        ("xdescribe(`foo`, function () {})", None),
        ("test('foo', function () {})", None),
        ("test('foo', function () {})", None),
        ("xtest('foo', function () {})", None),
        ("xtest(`foo`, function () {})", None),
        ("test('foo test', function () {})", None),
        ("xtest('foo test', function () {})", None),
        ("it('foo', function () {})", None),
        ("fit('foo', function () {})", None),
        ("xit('foo', function () {})", None),
        ("xit(`foo`, function () {})", None),
        ("it('foos it correctly', function () {})", None),
        (
            "
                describe('foo', () => {
                it('bar', () => {})
                })
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                it('describes things correctly', () => {})
                })
            ",
            None,
        ),
        ("it(abc, function () {})", Some(serde_json::json!([{ "ignoreTypeOfTestName": true }]))),
        // Vitest-specific tests with allowArguments option
        ("it(foo, () => {});", Some(serde_json::json!([{ "allowArguments": true }]))),
        ("describe(bar, () => {});", Some(serde_json::json!([{ "allowArguments": true }]))),
        ("test(baz, () => {});", Some(serde_json::json!([{ "allowArguments": true }]))),
        // Vitest-specific tests with .extend()
        (
            "export const myTest = test.extend({
                archive: []
            })",
            None,
        ),
        ("const localTest = test.extend({})", None),
        (
            "import { it } from 'vitest'

            const test = it.extend({
                fixture: [
                    async ({}, use) => {
                        setup()
                        await use()
                        teardown()
                    },
                    { auto: true }
                ],
            })

            test('', () => {})",
            None,
        ),
    ];

    let fail = vec![
        (
            "test('the correct way to properly handle all things', () => {});",
            Some(serde_json::json!([{ "disallowedWords": ["correct", "properly", "all"] }])),
        ),
        (
            "describe('the correct way to do things', function () {})",
            Some(serde_json::json!([{ "disallowedWords": ["correct"] }])),
        ),
        (
            "it('has ALL the things', () => {})",
            Some(serde_json::json!([{ "disallowedWords": ["all"] }])),
        ),
        (
            "xdescribe('every single one of them', function () {})",
            Some(serde_json::json!([{ "disallowedWords": ["every"] }])),
        ),
        (
            "describe('Very Descriptive Title Goes Here', function () {})",
            Some(serde_json::json!([{ "disallowedWords": ["descriptive"] }])),
        ),
        (
            "test(`that the value is set properly`, function () {})",
            Some(serde_json::json!([{ "disallowedWords": ["properly"] }])),
        ),
        // TODO: The regex `(?:#(?!unit|e2e))\w+` in those test cases is not valid in Rust
        // (
        //     "
        //         describe('things to test', () => {
        //             describe('unit tests #unit', () => {
        //                 it('is true', () => {
        //                     expect(true).toBe(true);
        //                 });
        //             });

        //             describe('e2e tests #e4e', () => {
        //                 it('is another test #e2e #jest4life', () => {});
        //             });
        //         });
        //     ",
        //     Some(serde_json::json!([
        //         {
        //             "mustNotMatch": r#"(?:#(?!unit|e2e))\w+"#,
        //             "mustMatch": "^[^#]+$|(?:#(?:unit|e2e))",
        //         },
        //     ])),
        // ),
        // (
        //     "
        //         import { describe, describe as context, it as thisTest } from '@jest/globals';

        //         describe('things to test', () => {
        //             context('unit tests #unit', () => {
        //             thisTest('is true', () => {
        //                 expect(true).toBe(true);
        //             });
        //             });

        //             context('e2e tests #e4e', () => {
        //                 thisTest('is another test #e2e #jest4life', () => {});
        //             });
        //         });
        //         ",
        //     Some(
        //         serde_json::json!([ { "mustNotMatch": r#"(?:#(?!unit|e2e))\w+"#, "mustMatch": "^[^#]+$|(?:#(?:unit|e2e))", }, ]),
        //     ),
        // ),
        // (
        //     "
        //         describe('things to test', () => {
        //             describe('unit tests #unit', () => {
        //                 it('is true', () => {
        //                     expect(true).toBe(true);
        //                 });
        //             });

        //             describe('e2e tests #e4e', () => {
        //                 it('is another test #e2e #jest4life', () => {});
        //             });
        //         });
        //     ",
        //     Some(serde_json::json!([
        //       {
        //         "mustNotMatch": [
        //           r#"(?:#(?!unit|e2e))\w+"#,
        //           "Please include '#unit' or '#e2e' in titles",
        //         ],
        //         "mustMatch": [
        //           "^[^#]+$|(?:#(?:unit|e2e))",
        //           "Please include '#unit' or '#e2e' in titles",
        //         ],
        //       },
        //     ])),
        // ),
        // (
        //     "
        //         describe('things to test', () => {
        //             describe('unit tests #unit', () => {
        //                 it('is true', () => {
        //                     expect(true).toBe(true);
        //                 });
        //             });

        //             describe('e2e tests #e4e', () => {
        //                 it('is another test #e2e #jest4life', () => {});
        //             });
        //         });
        //     ",
        //     Some(serde_json::json!([
        //       {
        //         "mustNotMatch": { "describe": [r#"(?:#(?!unit|e2e))\w+"#] },
        //         "mustMatch": { "describe": "^[^#]+$|(?:#(?:unit|e2e))" },
        //       },
        //     ])),
        // ),
        // (
        //     "
        //         describe('things to test', () => {
        //             describe('unit tests #unit', () => {
        //                 it('is true', () => {
        //                     expect(true).toBe(true);
        //                 });
        //             });

        //             describe('e2e tests #e4e', () => {
        //                 it('is another test #e2e #jest4life', () => {});
        //             });
        //         });
        //     ",
        //     Some(serde_json::json!([
        //       {
        //         "mustNotMatch": {
        //           "describe": [
        //             r#"(?:#(?!unit|e2e))\w+"#,
        //             "Please include '#unit' or '#e2e' in describe titles",
        //           ],
        //         },
        //         "mustMatch": { "describe": "^[^#]+$|(?:#(?:unit|e2e))" },
        //       },
        //     ])),
        // ),
        // (
        //     "
        //         describe('things to test', () => {
        //             describe('unit tests #unit', () => {
        //             it('is true', () => {
        //                 expect(true).toBe(true);
        //             });
        //             });

        //             describe('e2e tests #e4e', () => {
        //                 it('is another test #e2e #jest4life', () => {});
        //             });
        //         });
        //     ",
        //     Some(serde_json::json!([
        //       {
        //         "mustNotMatch": { "describe": r#"(?:#(?!unit|e2e))\w+"# },
        //         "mustMatch": { "it": "^[^#]+$|(?:#(?:unit|e2e))" },
        //       },
        //     ])),
        // ),
        // (
        //     "
        //         describe('things to test', () => {
        //             describe('unit tests #unit', () => {
        //             it('is true #jest4life', () => {
        //                 expect(true).toBe(true);
        //             });
        //             });

        //             describe('e2e tests #e4e', () => {
        //             it('is another test #e2e #jest4life', () => {});
        //             });
        //         });
        //     ",
        //     Some(serde_json::json!([
        //       {
        //         "mustNotMatch": {
        //           "describe": [
        //             r#"(?:#(?!unit|e2e))\w+"#,
        //             "Please include '#unit' or '#e2e' in describe titles",
        //           ],
        //         },
        //         "mustMatch": {
        //           "it": [
        //             "^[^#]+$|(?:#(?:unit|e2e))",
        //             "Please include '#unit' or '#e2e' in it titles",
        //           ],
        //         },
        //       },
        //     ])),
        // ),
        (
            "test('the correct way to properly handle all things', () => {});",
            Some(serde_json::json!([{ "mustMatch": "#(?:unit|integration|e2e)" }])),
        ),
        (
            "describe('the test', () => {});",
            Some(serde_json::json!([
              { "mustMatch": { "describe": "#(?:unit|integration|e2e)" } },
            ])),
        ),
        (
            "xdescribe('the test', () => {});",
            Some(serde_json::json!([
              { "mustMatch": { "describe": "#(?:unit|integration|e2e)" } },
            ])),
        ),
        (
            "describe.skip('the test', () => {});",
            Some(serde_json::json!([
              { "mustMatch": { "describe": "#(?:unit|integration|e2e)" } },
            ])),
        ),
        ("it.each([])(1, () => {});", None),
        ("it.skip.each([])(1, () => {});", None),
        ("it.skip.each``(1, () => {});", None),
        ("it(123, () => {});", None),
        ("it.concurrent(123, () => {});", None),
        ("it(1 + 2 + 3, () => {});", None),
        ("it.concurrent(1 + 2 + 3, () => {});", None),
        (
            "test.skip(123, () => {});",
            Some(serde_json::json!([{ "ignoreTypeOfDescribeName": true }])),
        ),
        ("describe(String(/.+/), () => {});", None),
        (
            "describe(myFunction, () => 1);",
            Some(serde_json::json!([{ "ignoreTypeOfDescribeName": false }])),
        ),
        ("describe(myFunction, () => {});", None),
        ("xdescribe(myFunction, () => {});", None),
        ("describe(6, function () {})", None),
        ("describe.skip(123, () => {});", None),
        ("describe('', function () {})", None),
        (
            "
                describe('foo', () => {
                    it('', () => {});
                });
            ",
            None,
        ),
        ("it('', function () {})", None),
        ("it.concurrent('', function () {})", None),
        ("test('', function () {})", None),
        ("test.concurrent('', function () {})", None),
        ("test(``, function () {})", None),
        ("test.concurrent(``, function () {})", None),
        ("xdescribe('', () => {})", None),
        ("xit('', () => {})", None),
        ("xtest('', () => {})", None),
        ("describe(' foo', function () {})", None),
        ("describe.each()(' foo', function () {})", None),
        ("describe.only.each()(' foo', function () {})", None),
        ("describe(' foo foe fum', function () {})", None),
        ("describe('foo foe fum ', function () {})", None),
        ("fdescribe(' foo', function () {})", None),
        ("fdescribe(' foo', function () {})", None),
        ("xdescribe(' foo', function () {})", None),
        ("it(' foo', function () {})", None),
        ("it.concurrent(' foo', function () {})", None),
        ("fit(' foo', function () {})", None),
        ("it.skip(' foo', function () {})", None),
        ("fit('foo ', function () {})", None),
        ("it.skip('foo ', function () {})", None),
        (
            "
                import { test as testThat } from '@jest/globals';

                testThat('foo works ', () => {});
            ",
            None,
        ),
        ("xit(' foo', function () {})", None),
        ("test(' foo', function () {})", None),
        ("test.concurrent(' foo', function () {})", None),
        ("test(` foo`, function () {})", None),
        ("test.concurrent(` foo`, function () {})", None),
        ("test(` foo bar bang`, function () {})", None),
        ("test.concurrent(` foo bar bang`, function () {})", None),
        ("test(` foo bar bang  `, function () {})", None),
        ("test.concurrent(` foo bar bang  `, function () {})", None),
        ("xtest(' foo', function () {})", None),
        ("xtest(' foo  ', function () {})", None),
        (
            "
                describe(' foo', () => {
                    it('bar', () => {})
                })
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    it(' bar', () => {})
                })
            ",
            None,
        ),
        ("describe('describe foo', function () {})", None),
        ("fdescribe('describe foo', function () {})", None),
        ("xdescribe('describe foo', function () {})", None),
        ("describe('describe foo', function () {})", None),
        ("fdescribe(`describe foo`, function () {})", None),
        ("test('test foo', function () {})", None),
        ("xtest('test foo', function () {})", None),
        ("test(`test foo`, function () {})", None),
        ("test(`test foo test`, function () {})", None),
        ("it('it foo', function () {})", None),
        ("fit('it foo', function () {})", None),
        ("xit('it foo', function () {})", None),
        ("it('it foos it correctly', function () {})", None),
        (
            "
                describe('describe foo', () => {
                    it('bar', () => {})
                })
            ",
            None,
        ),
        (
            "
                describe('describe foo', () => {
                    it('describes things correctly', () => {})
                })
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    it('it bar', () => {})
                })
            ",
            None,
        ),
        ("it(abc, function () {})", None),
        // Vitest-specific fail test with allowArguments: false
        ("test(bar, () => {});", Some(serde_json::json!([{ "allowArguments": false }]))),
    ];

    let fix = vec![
        ("describe(' foo', function () {})", "describe('foo', function () {})"),
        ("describe.each()(' foo', function () {})", "describe.each()('foo', function () {})"),
        (
            "describe.only.each()(' foo', function () {})",
            "describe.only.each()('foo', function () {})",
        ),
        ("describe(' foo foe fum', function () {})", "describe('foo foe fum', function () {})"),
        ("describe('foo foe fum ', function () {})", "describe('foo foe fum', function () {})"),
        ("fdescribe(' foo', function () {})", "fdescribe('foo', function () {})"),
        ("fdescribe(' foo', function () {})", "fdescribe('foo', function () {})"),
        ("xdescribe(' foo', function () {})", "xdescribe('foo', function () {})"),
        ("it(' foo', function () {})", "it('foo', function () {})"),
        ("it.concurrent(' foo', function () {})", "it.concurrent('foo', function () {})"),
        ("fit(' foo', function () {})", "fit('foo', function () {})"),
        ("it.skip(' foo', function () {})", "it.skip('foo', function () {})"),
        ("fit('foo ', function () {})", "fit('foo', function () {})"),
        ("it.skip('foo ', function () {})", "it.skip('foo', function () {})"),
        (
            "
                import { test as testThat } from '@jest/globals';

                testThat('foo works ', () => {});
            ",
            "
                import { test as testThat } from '@jest/globals';

                testThat('foo works', () => {});
            ",
        ),
        ("xit(' foo', function () {})", "xit('foo', function () {})"),
        ("test(' foo', function () {})", "test('foo', function () {})"),
        ("test.concurrent(' foo', function () {})", "test.concurrent('foo', function () {})"),
        ("test(` foo`, function () {})", "test(`foo`, function () {})"),
        ("test.concurrent(` foo`, function () {})", "test.concurrent(`foo`, function () {})"),
        ("test(` foo bar bang`, function () {})", "test(`foo bar bang`, function () {})"),
        (
            "test.concurrent(` foo bar bang`, function () {})",
            "test.concurrent(`foo bar bang`, function () {})",
        ),
        ("test(` foo bar bang  `, function () {})", "test(`foo bar bang`, function () {})"),
        (
            "test.concurrent(` foo bar bang  `, function () {})",
            "test.concurrent(`foo bar bang`, function () {})",
        ),
        ("xtest(' foo', function () {})", "xtest('foo', function () {})"),
        ("xtest(' foo  ', function () {})", "xtest('foo', function () {})"),
        (
            "
                describe(' foo', () => {
                    it('bar', () => {})
                })
            ",
            "
                describe('foo', () => {
                    it('bar', () => {})
                })
            ",
        ),
        (
            "
                describe('foo', () => {
                    it(' bar', () => {})
                })
            ",
            "
                describe('foo', () => {
                    it('bar', () => {})
                })
            ",
        ),
        ("describe('describe foo', function () {})", "describe('foo', function () {})"),
        ("fdescribe('describe foo', function () {})", "fdescribe('foo', function () {})"),
        ("xdescribe('describe foo', function () {})", "xdescribe('foo', function () {})"),
        ("describe('describe foo', function () {})", "describe('foo', function () {})"),
        ("fdescribe(`describe foo`, function () {})", "fdescribe(`foo`, function () {})"),
        ("test('test foo', function () {})", "test('foo', function () {})"),
        ("xtest('test foo', function () {})", "xtest('foo', function () {})"),
        ("test(`test foo`, function () {})", "test(`foo`, function () {})"),
        ("test(`test foo test`, function () {})", "test(`foo test`, function () {})"),
        ("it('it foo', function () {})", "it('foo', function () {})"),
        ("fit('it foo', function () {})", "fit('foo', function () {})"),
        ("xit('it foo', function () {})", "xit('foo', function () {})"),
        ("it('it foos it correctly', function () {})", "it('foos it correctly', function () {})"),
        (
            "
                describe('describe foo', () => {
                    it('bar', () => {})
                })
            ",
            "
                describe('foo', () => {
                    it('bar', () => {})
                })
            ",
        ),
        (
            "
                describe('describe foo', () => {
                    it('describes things correctly', () => {})
                })
            ",
            "
                describe('foo', () => {
                    it('describes things correctly', () => {})
                })
            ",
        ),
        (
            "
                describe('foo', () => {
                    it('it bar', () => {})
                })
            ",
            "
                describe('foo', () => {
                    it('bar', () => {})
                })
            ",
        ),
        // AccidentalSpace: preserve escape sequences when trimming spaces
        (
            "test('issue #225513: Cmd-Click doesn\\'t work on JSDoc {@link URL|LinkText} format ', () => { assert(true); });",
            "test('issue #225513: Cmd-Click doesn\\'t work on JSDoc {@link URL|LinkText} format', () => { assert(true); });",
        ),
        // DuplicatePrefix: preserve escape sequences when removing prefix
        (
            "test('test that it doesn\\'t break', () => {});",
            "test('that it doesn\\'t break', () => {});",
        ),
    ];

    Tester::new(ValidTitle::NAME, ValidTitle::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
