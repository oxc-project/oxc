use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule, rules::shared::prefer_each as SharedPreferEach};

#[derive(Debug, Default, Clone)]
pub struct PreferEach;

declare_oxc_lint!(
    PreferEach,
    vitest,
    style,
    docs = SharedPreferEach::DOCUMENTATION,
    version = "0.9.0",
);

impl Rule for PreferEach {
    fn run_once(&self, ctx: &LintContext<'_>) {
        SharedPreferEach::PreferEachConfig::run_once(ctx);
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        SharedPreferEach::PreferEachConfig::should_run(ctx)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass_jest = vec![
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

    let fail_jest = vec![
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

    let pass_vitest = vec![
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

    let fail_vitest = vec![
        "  for (const [input, expected] of data) {
                  it(`results in ${expected}`, () => {
                    expect(fn(input)).toBe(expected)
                  });
                   }",
        " for (const [input, expected] of data) {
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
        " for (const [input, expected] of data) {
                  it.skip(`results in ${expected}`, () => {
                    expect(fn(input)).toBe(expected)
                  });
                   }

                   it('is true', () => {
                  expect(true).toBe(false);
                   });",
        " it('is true', () => {
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
                  beforeEach(() => setupSomething(input));

                  test(`results in ${expected}`, () => {
                    expect(doSomething()).toBe(expected)
                  });
                   }",
        r#"
                   for (const [input, expected] of data) {
                  it("only returns numbers that are greater than seven", function () {
                    const numbers = getNumbers(input);

                    for (let i = 0; i < numbers.length; i++) {
                   expect(numbers[i]).toBeGreaterThan(7);
                    }
                  });
                   }
                 "#,
        r#"
                   for (const [input, expected] of data) {
                  beforeEach(() => setupSomething(input));

                  it("only returns numbers that are greater than seven", function () {
                    const numbers = getNumbers();

                    for (let i = 0; i < numbers.length; i++) {
                   expect(numbers[i]).toBeGreaterThan(7);
                    }
                  });
                   }
                 "#,
    ];

    let mut pass = vec![];
    pass.extend(pass_jest);
    pass.extend(pass_vitest);
    pass.extend([
        r"import { test } from 'vitest';
            const customTest = test.extend({});
            customTest('outer', () => {
                for (const value of values) {
                    customTest('inner', () => {});
                }
            });",
        r"import { test as baseTest } from 'vitest';
            const extendedTest = baseTest.extend({});
            const customTest = extendedTest.extend({});
            customTest.only('outer', () => {
                for (const value of values) {
                    customTest('inner', () => {});
                }
            });",
        r"import { test } from 'vitest';
            const customTest = test.extend({});
            customTest.each([1])('outer', () => {
                for (const value of values) {
                    customTest('inner', () => {});
                }
            });",
        r"import { test as baseTest } from 'other';
            const customTest = baseTest.extend({});
            for (const value of values) {
                customTest('not a test', () => {});
            }",
        r"import { test } from 'vitest';
            const customTest = test.extend({});
            function helper(customTest) {
                for (const value of values) {
                    customTest('not a test', () => {});
                }
            }",
    ]);
    let mut fail = vec![];
    fail.extend(fail_jest);
    fail.extend(fail_vitest);
    fail.extend([
        r"import { test } from 'vitest';
            const customTest = test.extend({});
            for (const value of values) {
                customTest('foo', () => {});
            }",
        r"import { test as baseTest } from 'vitest';
            const extendedTest = baseTest.extend({});
            const customTest = extendedTest.extend({});
            for (const value in values) {
                customTest.only('foo', () => {});
            }",
        r"import { test as customTest } from 'vitest';
            for (let i = 0; i < 2; i++) {
                customTest('foo', () => {});
            }",
        r"import { test as customTest } from 'vitest';
            function test() {}
            test.only('not a test', () => {
                for (const value of values) {
                    customTest('foo', () => {});
                }
            });",
    ]);

    Tester::new(PreferEach::NAME, PreferEach::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
