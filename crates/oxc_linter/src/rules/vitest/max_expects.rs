use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    rules::shared::max_expects::{DOCUMENTATION, MaxExpectsConfig},
};

#[derive(Debug, Default, Clone)]
pub struct MaxExpects(Box<MaxExpectsConfig>);

declare_oxc_lint!(
    MaxExpects,
    vitest,
    style,
    config = MaxExpectsConfig,
    docs = DOCUMENTATION,
    version = "0.0.18",
);

impl Rule for MaxExpects {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let config = serde_json::from_value::<DefaultRuleConfig<MaxExpectsConfig>>(value)?;
        Ok(Self(Box::new(config.into_inner())))
    }

    fn run_once(&self, ctx: &LintContext) {
        self.0.run_once(ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        ("test('should pass')", None),
        ("test('should pass', () => {})", None),
        ("test.skip('should pass', () => {})", None),
        (
            "
                test('should pass', function () {
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    // expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                it('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should pass', async () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should pass', async () => {
                    expect.hasAssertions();

                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should pass', async () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toEqual(expect.any(Boolean));
                });
            ",
            None,
        ),
        (
            "
                test('should pass', async () => {
                    expect.hasAssertions();

                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toEqual(expect.any(Boolean));
                });
            ",
            None,
        ),
        (
            "
                describe('test', () => {
                    test('should pass', () => {
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                    });
                });
            ",
            None,
        ),
        (
            "
                test.each(['should', 'pass'], () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
                test('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                function myHelper() {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                };

                test('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                function myHelper1() {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                };

                test('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });

                function myHelper2() {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                };
            ",
            None,
        ),
        (
            "
                test('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });

                function myHelper() {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                };
            ",
            None,
        ),
        (
            "
                const myHelper1 = () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                };

                test('should pass', function() {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });

                const myHelper2 = function() {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                };
            ",
            None,
        ),
        (
            "
                test('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            Some(serde_json::json!([{ "max": 10 }])),
        ),
    ];

    let pass_vitest = vec![
        (
            "test('should pass', () => {
			      expect(true).toBeDefined();
			      expect(true).toBeDefined();
			      expect(true).toBeDefined();
			      expect(true).toBeDefined();
			      expect(true).toBeDefined();
			    });",
            None,
        ),
        (
            "test('should pass', () => {
			     expect(true).toBeDefined();
			     expect(true).toBeDefined();
			     expect(true).toBeDefined();
			     expect(true).toBeDefined();
			     expect(true).toBeDefined();
			      });",
            None,
        ),
        (
            " test('should pass', async () => {
			     expect.hasAssertions();

			     expect(true).toBeDefined();
			     expect(true).toBeDefined();
			     expect(true).toBeDefined();
			     expect(true).toBeDefined();
			     expect(true).toEqual(expect.any(Boolean));
			      });",
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

                  it('works', ({ result }) => {
                    expect(result).toBe(42);
                  });
                });

                ",
            None,
        ),
    ];

    pass.extend(pass_vitest);

    let mut fail = vec![
        (
            "
                test('should not pass', function () {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should not pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                it('should not pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                it('should not pass', async () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should not pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
                test('should not pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                describe('test', () => {
                    test('should not pass', () => {
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                    });
                });
            ",
            None,
        ),
        (
            "
                test.each(['should', 'not', 'pass'], () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should not pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
    ];

    let fail_vitest = vec![
        (
            "test('should not pass', function () {
			       expect(true).toBeDefined();
			       expect(true).toBeDefined();
			       expect(true).toBeDefined();
			       expect(true).toBeDefined();
			       expect(true).toBeDefined();
			       expect(true).toBeDefined();
			     });
			      ",
            None,
        ),
        (
            "test('should not pass', () => {
			      expect(true).toBeDefined();
			      expect(true).toBeDefined();
			      expect(true).toBeDefined();
			      expect(true).toBeDefined();
			      expect(true).toBeDefined();
			      expect(true).toBeDefined();
			    });
			    test('should not pass', () => {
			      expect(true).toBeDefined();
			      expect(true).toBeDefined();
			      expect(true).toBeDefined();
			      expect(true).toBeDefined();
			      expect(true).toBeDefined();
			      expect(true).toBeDefined();
			    });",
            None,
        ),
        (
            "test('should not pass', () => {
			      expect(true).toBeDefined();
			      expect(true).toBeDefined();
			       });",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
    ];

    fail.extend(fail_vitest);

    Tester::new(MaxExpects::NAME, MaxExpects::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
