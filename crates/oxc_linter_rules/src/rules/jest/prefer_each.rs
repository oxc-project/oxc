use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule, rules::shared::prefer_each as SharedPreferEach};

#[derive(Debug, Default, Clone)]
pub struct PreferEach;

declare_oxc_lint!(
    PreferEach,
    jest,
    style,
    docs = SharedPreferEach::DOCUMENTATION,
    version = "0.9.0",
);

impl Rule for PreferEach {
    fn run_once(&self, ctx: &LintContext<'_>) {
        SharedPreferEach::PreferEachConfig::run_once(ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"it("is true", () => { expect(true).toBe(false) });"#,
        r#"it.each(getNumbers())("only returns numbers that are greater than seven", number => {
              expect(number).toBeGreaterThan(7);
            });"#,
        r#"it("returns numbers that are greater than five", function () {
              for (const number of getNumbers()) {
                expect(number).toBeGreaterThan(5);
              }
            });"#,
        r#"it("returns things that are less than ten", function () {
              for (const thing in things) {
                expect(thing).toBeLessThan(10);
              }
            });"#,
        r#"it("only returns numbers that are greater than seven", function () {
              const numbers = getNumbers();
              for (let i = 0; i < numbers.length; i++) {
                expect(numbers[i]).toBeGreaterThan(7);
              }
            });"#,
    ];

    let fail = vec![
        "for (const [input, expected] of data) {
              it(`results in ${expected}`, () => {
                expect(fn(input)).toBe(expected)
              });
            }",
        "for (const [input, expected] of data) {
              describe(`when the input is ${input}`, () => {
                it(`results in ${expected}`, () => {
                  expect(fn(input)).toBe(expected)
                });
              });
            }",
        "for (const [input, expected] of data) {
              describe(`when the input is ${input}`, () => {
                it(`results in ${expected}`, () => {
                  expect(fn(input)).toBe(expected)
                });
              });
            }
            for (const [input, expected] of data) {
              it.skip(`results in ${expected}`, () => {
                expect(fn(input)).toBe(expected)
              });
            }",
        "for (const [input, expected] of data) {
              it.skip(`results in ${expected}`, () => {
                expect(fn(input)).toBe(expected)
              });
            }",
        "it('is true', () => {
              expect(true).toBe(false);
            });
            for (const [input, expected] of data) {
              it.skip(`results in ${expected}`, () => {
                expect(fn(input)).toBe(expected)
              });
            }",
        "for (const [input, expected] of data) {
              it.skip(`results in ${expected}`, () => {
                expect(fn(input)).toBe(expected)
              });
            }
            it('is true', () => {
              expect(true).toBe(false);
            });",
        "it('is true', () => {
              expect(true).toBe(false);
            });
            for (const [input, expected] of data) {
              it.skip(`results in ${expected}`, () => {
                expect(fn(input)).toBe(expected)
              });
            }
            it('is true', () => {
              expect(true).toBe(false);
            });",
        "for (const [input, expected] of data) {
              it(`results in ${expected}`, () => {
                expect(fn(input)).toBe(expected)
              });
              it(`results in ${expected}`, () => {
                expect(fn(input)).toBe(expected)
              });
            }",
        "for (const [input, expected] of data) {
              it(`results in ${expected}`, () => {
                expect(fn(input)).toBe(expected)
              });
            }
            for (const [input, expected] of data) {
              it(`results in ${expected}`, () => {
                expect(fn(input)).toBe(expected)
              });
            }",
        "for (const [input, expected] of data) {
              it(`results in ${expected}`, () => {
                expect(fn(input)).toBe(expected)
              });
            }
            for (const [input, expected] of data) {
              test(`results in ${expected}`, () => {
                expect(fn(input)).toBe(expected)
              });
            }",
        "for (const [input, expected] of data) {
              beforeEach(() => setupSomething(input));
              test(`results in ${expected}`, () => {
                expect(doSomething()).toBe(expected)
              });
            }",
        r#"for (const [input, expected] of data) {
              it("only returns numbers that are greater than seven", function () {
                const numbers = getNumbers(input);
                for (let i = 0; i < numbers.length; i++) {
                  expect(numbers[i]).toBeGreaterThan(7);
                }
              });
            }"#,
        r#"for (const [input, expected] of data) {
              beforeEach(() => setupSomething(input));
              it("only returns numbers that are greater than seven", function () {
                const numbers = getNumbers();
                for (let i = 0; i < numbers.length; i++) {
                  expect(numbers[i]).toBeGreaterThan(7);
                }
              });
            }"#,
    ];

    Tester::new(PreferEach::NAME, PreferEach::PLUGIN, pass, fail).test_and_snapshot();
}
