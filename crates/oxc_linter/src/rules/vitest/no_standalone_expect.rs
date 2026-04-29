use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext, rule::Rule,
    rules::shared::no_standalone_expect as SharedNoStandaloneExpect,
};

#[derive(Debug, Default, Clone)]
pub struct NoStandaloneExpect(Box<SharedNoStandaloneExpect::NoStandaloneExpectConfig>);

declare_oxc_lint!(
    NoStandaloneExpect,
    vitest,
    correctness,
    config = SharedNoStandaloneExpect::NoStandaloneExpectConfig,
    docs = SharedNoStandaloneExpect::DOCUMENTATION,
    version = "0.0.13",
);

impl Rule for NoStandaloneExpect {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(Self(Box::new(SharedNoStandaloneExpect::NoStandaloneExpectConfig::from_configuration(
            value,
        )?)))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        self.0.run_once(ctx);
    }
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, tester::Tester};

    let pass = vec![
        ("expect.any(String)", None),
        ("expect.extend({})", None),
        ("describe('a test', () => { it('an it', () => {expect(1).toBe(1); }); });", None),
        (
            "describe('a test', () => { it('an it', () => { const func = () => { expect(1).toBe(1); }; }); });",
            None,
        ),
        ("describe('a test', () => { const func = () => { expect(1).toBe(1); }; });", None),
        ("describe('a test', () => { function func() { expect(1).toBe(1); }; });", None),
        ("describe('a test', () => { const func = function(){ expect(1).toBe(1); }; });", None),
        ("it('an it', () => expect(1).toBe(1))", None),
        ("const func = function(){ expect(1).toBe(1); };", None),
        ("const func = () => expect(1).toBe(1);", None),
        ("{}", None),
        ("it.each([1, true])('trues', value => { expect(value).toBe(true); });", None),
        (
            "it.each([1, true])('trues', value => { expect(value).toBe(true); }); it('an it', () => { expect(1).toBe(1) });",
            None,
        ),
        (
            "
                it.each`
                    num   | value
                    ${1} | ${true}
                `('trues', ({ value }) => {
                    expect(value).toBe(true);
                });
            ",
            None,
        ),
        ("it.only('an only', value => { expect(value).toBe(true); });", None),
        ("it.concurrent('an concurrent', value => { expect(value).toBe(true); });", None),
        (
            "describe.each([1, true])('trues', value => { it('an it', () => expect(value).toBe(true) ); });",
            None,
        ),
        (
            "
            describe('scenario', () => {
                const t = Math.random() ? it.only : it;
                t('testing', () => expect(true));
            });
        ",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ['t'] }])),
        ),
        (
            r"
                each([
                [1, 1, 2],
                [1, 2, 3],
                [2, 1, 3],
                ]).test('returns the result of adding %d to %d', (a, b, expected) => {
                    expect(a + b).toBe(expected);
                });
            ",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["each.test"] }])),
        ),
        (
            r"function funcWithCallback(callback) { callback(5); }
            describe('testWithCallback', () => {
              it('should call the callback', (done) => {
                funcWithCallback((result) => {
                  expect(result).toBe(5);
                  done();
                });
              });
            });",
            None,
        ),
        (
            r"it('should do something', async () => {
              render();

              await waitFor(() => {
                expect(screen.getByText('Option 2')).toBeInTheDocument();
              });
            });",
            None,
        ),
        (
            r"it('should do something', () => {
              waitFor(() => {
                expect(screen.getByText('Option 2')).toBeInTheDocument();
              });
            });",
            None,
        ),
        (
            r"describe('test suite', () => {
              it('should work with nested callbacks', () => {
                someFunction(() => {
                  anotherFunction(() => {
                    expect(true).toBe(true);
                  });
                });
              });
            });",
            None,
        ),
        (
            r"import {fakeAsync} from '@angular/core/testing';
            describe('App', () => { it('should create the app', fakeAsync(() => { expect(true).toBeTruthy(); })); });",
            None,
        ),
        (
            r"describe('App', () => { it('should work with wrapper function', wrapperFn(() => { expect(true).toBeTruthy(); })); });",
            None,
        ),
        ("beforeEach(() => { doSomething(); });", None),
        (r#"bench("a bench", () => {})"#, None),
        (r#"describe("a test", () => { it("an it", () => {expect(1).toBe(1); }); });"#, None),
        (
            r#"describe("a test", () => { it("an it", () => { const func = () => { expect(1).toBe(1); }; }); });"#,
            None,
        ),
        (r#"describe("a test", () => { const func = () => { expect(1).toBe(1); }; });"#, None),
        (r#"describe("a test", () => { function func() { expect(1).toBe(1); }; });"#, None),
        (r#"describe("a test", () => { const func = function(){ expect(1).toBe(1); }; });"#, None),
        (
            r#"describe.only.concurrent.todo("a test", () => { const func = function(){ expect(1).toBe(1); }; });"#,
            None,
        ),
        (r#"it("an it", () => expect(1).toBe(1))"#, None),
        (r#"it.only("an it", () => expect(1).toBe(1))"#, None),
        (r#"it.concurrent("an it", () => expect(1).toBe(1))"#, None),
        (r#"it.fails("a failing test", () => expect(1).toBe(1))"#, None),
        (r#"it.only.fails("a failing test", () => expect(1).toBe(1))"#, None),
        (r#"it.skip.fails("a failing test", () => expect(1).toBe(1))"#, None),
        (r#"test("a test", () => expect(1).toBe(1))"#, None),
        (r#"test.skip("a skipped test", () => expect(1).toBe(1))"#, None),
        (r#"test.fails("a failing test", () => expect(1).toBe(1))"#, None),
        (r#"test.only.fails("a failing test", () => expect(1).toBe(1))"#, None),
        (r#"test.skip.fails("a failing test", () => expect(1).toBe(1))"#, None),
        (r#"it.each([1, true])("trues", value => { expect(value).toBe(true); });"#, None),
        (
            r#"it.each([1, true])("trues", value => { expect(value).toBe(true); }); it("an it", () => { expect(1).toBe(1) });"#,
            None,
        ),
        (
            r"describe('App', () => {
              it('should work with wrapper function', wrapperFn(() => { expect(true).toBeTruthy(); }));
            });",
            None,
        ),
        (
            r#"describe('workers/repository/update/pr/code-owners', () => {
              describe('codeOwnersForPr', () => {
                it.fails('does not parse Gitea regex as Gitlab sections', () => {
                  expect("foo").toEqual("bar");
                });
              });
            });"#,
            None,
        ),
        (
            "import {describe, expect, test} from 'vitest';

                    describe('example', () => {
                      const it = test.extend<{ result: number }>({
                        result: async ({}, use) => {
                          await use(42);
                        },
                      });
                    });

                    ",
            None,
        ),
    ];

    let fail = vec![
        ("(() => {})('testing', () => expect(true).toBe(false))", None),
        ("expect.hasAssertions()", None),
        ("expect().hasAssertions()", None),
        (
            "
                describe('scenario', () => {
                    const t = Math.random() ? it.only : it;
                    t('testing', () => expect(true).toBe(false));
                });
            ",
            None,
        ),
        (
            "
                describe('scenario', () => {
                    const t = Math.random() ? it.only : it;
                    t('testing', () => expect(true).toBe(false));
                });
            ",
            None,
        ),
        (
            "
                each([
                    [1, 1, 2],
                    [1, 2, 3],
                    [2, 1, 3],
                ]).test('returns the result of adding %d to %d', (a, b, expected) => {
                    expect(a + b).toBe(expected);
                });
            ",
            None,
        ),
        (
            "
                each([
                    [1, 1, 2],
                    [1, 2, 3],
                    [2, 1, 3],
                ]).test('returns the result of adding %d to %d', (a, b, expected) => {
                    expect(a + b).toBe(expected);
                });
            ",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["each"] }])),
        ),
        (
            "
                each([
                    [1, 1, 2],
                    [1, 2, 3],
                    [2, 1, 3],
                ]).test('returns the result of adding %d to %d', (a, b, expected) => {
                    expect(a + b).toBe(expected);
                });
            ",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["test"] }])),
        ),
        ("describe('a test', () => { expect(1).toBe(1); });", None),
        ("describe('a test', () => expect(1).toBe(1));", None),
        (
            "describe('a test', () => { const func = () => { expect(1).toBe(1); }; expect(1).toBe(1); });",
            None,
        ),
        (
            "describe('a test', () => {  it(() => { expect(1).toBe(1); }); expect(1).toBe(1); });",
            None,
        ),
        ("expect(1).toBe(1);", None),
        ("{expect(1).toBe(1)}", None),
        (
            "it.each([1, true])('trues', value => { expect(value).toBe(true); }); expect(1).toBe(1);",
            None,
        ),
        ("describe.each([1, true])('trues', value => { expect(value).toBe(true); });", None),
        (r"describe('App', () => { wrapperFn(() => { expect(true).toBeTruthy(); }); });", None),
        (
            "
                import { expect as pleaseExpect } from '@jest/globals';
                describe('a test', () => { pleaseExpect(1).toBe(1); });
            ",
            None,
        ),
        (
            "
                import { expect as pleaseExpect } from '@jest/globals';
                beforeEach(() => pleaseExpect.hasAssertions());
            ",
            None,
        ),
        (
            "
                   describe('scenario', () => {
                  const t = Math.random() ? it.only : it;
                  t('testing', () => expect(true).toBe(false));
                   });
                 ",
            None,
        ),
        (
            "describe('scenario', () => {
                  const t = Math.random() ? it.only : it;
                  t('testing', () => expect(true).toBe(false));
                   });",
            None,
        ),
        (r#"describe("a test", () => { expect(1).toBe(1); });"#, None),
        (r#"describe("a test", () => expect(1).toBe(1));"#, None),
        (
            r#"describe("a test", () => { const func = () => { expect(1).toBe(1); }; expect(1).toBe(1); });"#,
            None,
        ),
        (
            r#"describe("a test", () => {  it(() => { expect(1).toBe(1); }); expect(1).toBe(1); });"#,
            None,
        ),
        (
            "
                 each([
                   [1, 1, 2],
                   [1, 2, 3],
                   [2, 1, 3],
                 ]).test('returns the result of adding %d to %d', (a, b, expected) => {
                   expect(a + b).toBe(expected);
                 });",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["test"] }])),
        ),
    ];

    Tester::new(NoStandaloneExpect::NAME, NoStandaloneExpect::PLUGIN, pass, fail)
        .test_and_snapshot();
}
