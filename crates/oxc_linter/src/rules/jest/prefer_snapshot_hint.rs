use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    rules::shared::prefer_snapshot_hint::{DOCUMENTATION, SnapshotHintMode},
};

#[derive(Debug, Default, Clone)]
pub struct PreferSnapshotHint(Box<SnapshotHintMode>);

declare_oxc_lint!(
    PreferSnapshotHint,
    jest,
    correctness,
    config = SnapshotHintMode,
    docs = DOCUMENTATION,
    version = "1.59.0",
);

impl Rule for PreferSnapshotHint {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let config = serde_json::from_value::<DefaultRuleConfig<SnapshotHintMode>>(value)?;
        Ok(Self(Box::new(config.into_inner())))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        (*self.0).run_once(ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("expect(something).toStrictEqual(somethingElse);", Some(serde_json::json!(["always"]))),
        ("a().toEqual('b')", Some(serde_json::json!(["always"]))),
        ("expect(a);", Some(serde_json::json!(["always"]))),
        (r#"expect(1).toMatchSnapshot({}, "my snapshot");"#, Some(serde_json::json!(["always"]))),
        (r#"expect(1).toMatchSnapshot("my snapshot");"#, Some(serde_json::json!(["always"]))),
        ("expect(1).toMatchSnapshot(`my snapshot`);", Some(serde_json::json!(["always"]))),
        (
            r#"const x = {};
            expect(1).toMatchSnapshot(x, "my snapshot");"#,
            Some(serde_json::json!(["always"])),
        ),
        (
            r#"expect(1).toThrowErrorMatchingSnapshot("my snapshot");"#,
            Some(serde_json::json!(["always"])),
        ),
        ("expect(1).toMatchInlineSnapshot();", Some(serde_json::json!(["always"]))),
        ("expect(1).toThrowErrorMatchingInlineSnapshot();", Some(serde_json::json!(["always"]))),
        ("expect(something).toStrictEqual(somethingElse);", Some(serde_json::json!(["multi"]))),
        ("a().toEqual('b')", Some(serde_json::json!(["multi"]))),
        ("expect(a);", Some(serde_json::json!(["multi"]))),
        (r#"expect(1).toMatchSnapshot({}, "my snapshot");"#, Some(serde_json::json!(["multi"]))),
        (
            r#"expect(1).toThrowErrorMatchingSnapshot("my snapshot");"#,
            Some(serde_json::json!(["multi"])),
        ),
        ("expect(1).toMatchSnapshot({});", Some(serde_json::json!(["multi"]))),
        ("expect(1).toThrowErrorMatchingSnapshot();", Some(serde_json::json!(["multi"]))),
        (
            "it('is true', () => {
              expect(1).toMatchSnapshot();
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              expect(1).toMatchSnapshot(undefined, 'my first snapshot');
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "describe('my tests', () => {
              it('is true', () => {
                expect(1).toMatchSnapshot('this is a hint, all by itself');
              });
              it('is false', () => {
                expect(2).toMatchSnapshot('this is a hint');
                expect(2).toMatchSnapshot('and so is this');
              });
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              expect(1).toMatchSnapshot();
            });
            it('is false', () => {
              expect(2).toMatchSnapshot('this is a hint');
              expect(2).toMatchSnapshot('and so is this');
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              expect(1).toMatchSnapshot();
            });
            it('is false', () => {
              expect(2).toThrowErrorMatchingSnapshot();
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              expect(1).toStrictEqual(1);
              expect(1).toStrictEqual(2);
              expect(1).toMatchSnapshot();
            });
            it('is false', () => {
              expect(1).toStrictEqual(1);
              expect(1).toStrictEqual(2);
              expect(2).toThrowErrorMatchingSnapshot();
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              expect(1).toMatchInlineSnapshot();
            });
            it('is false', () => {
              expect(1).toMatchInlineSnapshot();
              expect(1).toMatchInlineSnapshot();
              expect(1).toThrowErrorMatchingInlineSnapshot();
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              expect(1).toMatchSnapshot();
            });
            it('is false', () => {
              expect(1).toMatchSnapshot();
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "import { it as itIs } from '@jest/globals';
            it('is true', () => {
              expect(1).toMatchSnapshot();
            });
            itIs('false', () => {
              expect(1).toMatchSnapshot();
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "const myReusableTestBody = (value, snapshotHint) => {
              const innerFn = anotherValue => {
                expect(anotherValue).toMatchSnapshot();
                expect(value).toBe(1);
              };
              expect(value).toBe(1);
            };
            it('my test', () => {
              expect(1).toMatchSnapshot();
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "const myReusableTestBody = (value, snapshotHint) => {
              const innerFn = anotherValue => {
                expect(value).toBe(1);
              };
              expect(value).toBe(1);
              expect(anotherValue).toMatchSnapshot();
            };
            it('my test', () => {
              expect(1).toMatchSnapshot();
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "const myReusableTestBody = (value, snapshotHint) => {
              const innerFn = anotherValue => {
                expect(anotherValue).toMatchSnapshot();
                expect(value).toBe(1);
              };
              expect(value).toBe(1);
            };
            expect(1).toMatchSnapshot();",
            Some(serde_json::json!(["multi"])),
        ),
    ];

    let fail = vec![
        ("expect(1).toMatchSnapshot();", Some(serde_json::json!(["always"]))),
        ("expect(1).toMatchSnapshot({});", Some(serde_json::json!(["always"]))),
        (
            r#"const x = "we can't know if this is a string or not";
            expect(1).toMatchSnapshot(x);"#,
            Some(serde_json::json!(["always"])),
        ),
        ("expect(1).toThrowErrorMatchingSnapshot();", Some(serde_json::json!(["always"]))),
        (
            "it('is true', () => {
              expect(1).toMatchSnapshot();
            });",
            Some(serde_json::json!(["always"])),
        ),
        (
            "it('is true', () => {
              expect(1).toMatchSnapshot();
              expect(2).toMatchSnapshot();
            });",
            Some(serde_json::json!(["always"])),
        ),
        (
            r#"it('is true', () => {
              expect(1).toMatchSnapshot();
              expect(2).toThrowErrorMatchingSnapshot("my error");
            });"#,
            Some(serde_json::json!(["always"])),
        ),
        (
            "const expectSnapshot = value => {
              expect(value).toMatchSnapshot();
            };",
            Some(serde_json::json!(["always"])),
        ),
        (
            "const expectSnapshot = value => {
              expect(value).toThrowErrorMatchingSnapshot();
            };",
            Some(serde_json::json!(["always"])),
        ),
        (
            "it('is true', () => {
              { expect(1).toMatchSnapshot(); }
            });",
            Some(serde_json::json!(["always"])),
        ),
        (
            r#"const x = "snapshot";
            expect(1).toMatchSnapshot(\\`my $\\{x}\\`);"#,
            Some(serde_json::json!(["always"])),
        ),
        (
            "it('is true', () => {
              expect(1).toMatchSnapshot();
              expect(2).toMatchSnapshot();
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              expect(1).toMatchSnapshot();
              expect(2).toThrowErrorMatchingSnapshot();
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              expect(1).toThrowErrorMatchingSnapshot();
              expect(2).toMatchSnapshot();
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              expect(1).toMatchSnapshot({});
              expect(2).toMatchSnapshot({});
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              expect(1).toMatchSnapshot({});
              {
                expect(2).toMatchSnapshot({});
              }
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              { expect(1).toMatchSnapshot(); }
              { expect(2).toMatchSnapshot(); }
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              expect(1).toMatchSnapshot();
              expect(2).toMatchSnapshot(undefined, 'my second snapshot');
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              expect(1).toMatchSnapshot({});
              expect(2).toMatchSnapshot(undefined, 'my second snapshot');
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              expect(1).toMatchSnapshot({}, 'my first snapshot');
              expect(2).toMatchSnapshot(undefined);
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              expect(1).toMatchSnapshot({}, 'my first snapshot');
              expect(2).toMatchSnapshot(undefined);
              expect(2).toMatchSnapshot();
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              expect(2).toMatchSnapshot();
              expect(1).toMatchSnapshot({}, 'my second snapshot');
              expect(2).toMatchSnapshot();
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
              expect(2).toMatchSnapshot(undefined);
              expect(2).toMatchSnapshot();
              expect(1).toMatchSnapshot(null, 'my third snapshot');
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "describe('my tests', () => {
              it('is true', () => {
                expect(1).toMatchSnapshot();
              });
              it('is false', () => {
                expect(2).toMatchSnapshot();
                expect(2).toMatchSnapshot();
              });
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "describe('my tests', () => {
              it('is true', () => {
                expect(1).toMatchSnapshot();
              });
              it('is false', () => {
                expect(2).toMatchSnapshot();
                expect(2).toMatchSnapshot('hello world');
              });
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "describe('my tests', () => {
              describe('more tests', () => {
                it('is true', () => {
                  expect(1).toMatchSnapshot();
                });
              });
              it('is false', () => {
                expect(2).toMatchSnapshot();
                expect(2).toMatchSnapshot('hello world');
              });
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "describe('my tests', () => {
              it('is true', () => {
                expect(1).toMatchSnapshot();
              });
              describe('more tests', () => {
                it('is false', () => {
                  expect(2).toMatchSnapshot();
                  expect(2).toMatchSnapshot('hello world');
                });
              });
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "import { describe as context, it as itIs } from '@jest/globals';
            describe('my tests', () => {
              it('is true', () => {
                expect(1).toMatchSnapshot();
              });
              context('more tests', () => {
                itIs('false', () => {
                  expect(2).toMatchSnapshot();
                  expect(2).toMatchSnapshot('hello world');
                });
              });
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "const myReusableTestBody = (value, snapshotHint) => {
              expect(value).toMatchSnapshot();
              const innerFn = anotherValue => {
                expect(anotherValue).toMatchSnapshot();
              };
              expect(value).toBe(1);
              expect(value + 1).toMatchSnapshot(null);
              expect(value + 2).toThrowErrorMatchingSnapshot(snapshotHint);
            };",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "const myReusableTestBody = (value, snapshotHint) => {
              expect(value).toMatchSnapshot();
              const innerFn = anotherValue => {
                expect(anotherValue).toMatchSnapshot();
                expect(value).toBe(1);
                expect(value + 1).toMatchSnapshot(null);
                expect(value + 2).toMatchSnapshot(null, snapshotHint);
              };
            };",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "const myReusableTestBody = (value, snapshotHint) => {
              const innerFn = anotherValue => {
                expect(anotherValue).toMatchSnapshot();
                expect(value).toBe(1);
                expect(value + 1).toMatchSnapshot(null);
                expect(value + 2).toMatchSnapshot(null, snapshotHint);
              };
              expect(value).toThrowErrorMatchingSnapshot();
            };",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "const myReusableTestBody = (value, snapshotHint) => {
              const innerFn = anotherValue => {
                expect(anotherValue).toMatchSnapshot();
                expect(value).toBe(1);
              };
              expect(value).toMatchSnapshot();
            };
            it('my test', () => {
              expect(1).toMatchSnapshot();
            });",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "const myReusableTestBody = value => {
              expect(value).toMatchSnapshot();
            };
            expect(1).toMatchSnapshot();
            expect(1).toThrowErrorMatchingSnapshot();",
            Some(serde_json::json!(["multi"])),
        ),
    ];

    Tester::new(PreferSnapshotHint::NAME, PreferSnapshotHint::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
