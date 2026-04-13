use std::ops::Deref;

use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    rules::PossibleJestNode,
    utils::{
        JestFnKind, JestGeneralFnKind, collect_possible_jest_call_node, parse_expect_jest_fn_call,
    },
};

fn snapshot_matcher_too_many_arguments_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`toMatchSnapshot` takes at most two arguments.")
        .with_help("Pass a hint string, or a property matcher object followed by a hint string.")
        .with_label(span)
}

fn snapshot_missing_hint_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Snapshot is missing a hint.")
        .with_help("Include a hint string to identify this snapshot in the snapshot file.")
        .with_label(span)
}

fn snapshot_hint_must_be_string_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Snapshot hint must be a string literal.")
        .with_help(
            "Provide a string literal as the hint, or pass a property matcher object as the first argument and the hint string as the second.",
        )
        .with_label(span)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotHintMode {
    /// Require a hint to always be provided when using external snapshot matchers.
    Always,
    /// Require a hint to be provided when there are multiple external snapshot matchers within the scope (meaning it includes nested calls).
    #[default]
    Multi,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PreferSnapshotHint(Box<SnapshotHintMode>);

impl Deref for PreferSnapshotHint {
    type Target = SnapshotHintMode;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces including a hint string with snapshot matchers (toMatchSnapshot and toThrowErrorMatchingSnapshot).
    ///
    /// ### Why is this bad?
    ///
    /// Auto-numbered snapshot names are fragile — adding or reordering assertions shifts all subsequent numbers, causing unrelated snapshots to appear changed and obscuring real differences in code review.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule configured as `always`:
    /// ```js
    /// const snapshotOutput = ({ stdout, stderr }) => {
    ///   expect(stdout).toMatchSnapshot();
    ///   expect(stderr).toMatchSnapshot();
    /// };
    ///
    /// describe('cli', () => {
    ///   describe('--version flag', () => {
    ///     it('prints the version', async () => {
    ///       snapshotOutput(await runCli(['--version']));
    ///     });
    ///   });
    ///
    ///   describe('--config flag', () => {
    ///     it('reads the config', async () => {
    ///       const { stdout, parsedConfig } = await runCli([
    ///         '--config',
    ///         'jest.config.js',
    ///       ]);
    ///
    ///       expect(stdout).toMatchSnapshot();
    ///       expect(parsedConfig).toMatchSnapshot();
    ///     });
    ///
    ///     it('prints nothing to stderr', async () => {
    ///       const { stderr } = await runCli(['--config', 'jest.config.js']);
    ///
    ///       expect(stderr).toMatchSnapshot();
    ///     });
    ///
    ///     describe('when the file does not exist', () => {
    ///       it('throws an error', async () => {
    ///         await expect(
    ///           runCli(['--config', 'does-not-exist.js']),
    ///         ).rejects.toThrowErrorMatchingSnapshot();
    ///       });
    ///     });
    ///   });
    /// });
    /// ```
    ///
    /// Examples of **incorrect** code for this rule configured as `multi`:
    ///
    /// ```js
    /// const snapshotOutput = ({ stdout, stderr }) => {
    ///   expect(stdout).toMatchSnapshot();
    ///   expect(stderr).toMatchSnapshot();
    /// };
    ///
    /// describe('cli', () => {
    ///   describe('--version flag', () => {
    ///     it('prints the version', async () => {
    ///       snapshotOutput(await runCli(['--version']));
    ///     });
    ///   });
    ///
    ///   describe('--config flag', () => {
    ///     it('reads the config', async () => {
    ///       const { stdout, parsedConfig } = await runCli([
    ///         '--config',
    ///         'jest.config.js',
    ///       ]);
    ///
    ///       expect(stdout).toMatchSnapshot();
    ///       expect(parsedConfig).toMatchSnapshot();
    ///     });
    ///
    ///     it('prints nothing to stderr', async () => {
    ///       const { stderr } = await runCli(['--config', 'jest.config.js']);
    ///
    ///       expect(stderr).toMatchSnapshot();
    ///     });
    ///   });
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule configured as `always`:
    /// ```js
    /// const snapshotOutput = ({ stdout, stderr }, hints) => {
    ///   expect(stdout).toMatchSnapshot({}, `stdout: ${hints.stdout}`);
    ///   expect(stderr).toMatchSnapshot({}, `stderr: ${hints.stderr}`);
    /// };
    ///
    /// describe('cli', () => {
    ///   describe('--version flag', () => {
    ///     it('prints the version', async () => {
    ///       snapshotOutput(await runCli(['--version']), {
    ///         stdout: 'version string',
    ///         stderr: 'empty',
    ///       });
    ///     });
    ///   });
    ///
    ///   describe('--config flag', () => {
    ///     it('reads the config', async () => {
    ///       const { stdout } = await runCli(['--config', 'jest.config.js']);
    ///
    ///       expect(stdout).toMatchSnapshot({}, 'stdout: config settings');
    ///     });
    ///
    ///     it('prints nothing to stderr', async () => {
    ///       const { stderr } = await runCli(['--config', 'jest.config.js']);
    ///
    ///       expect(stderr).toMatchInlineSnapshot();
    ///     });
    ///
    ///     describe('when the file does not exist', () => {
    ///       it('throws an error', async () => {
    ///         await expect(
    ///           runCli(['--config', 'does-not-exist.js']),
    ///         ).rejects.toThrowErrorMatchingSnapshot('stderr: config error');
    ///       });
    ///     });
    ///   });
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule configured as `multi`:
    /// ```js
    /// const snapshotOutput = ({ stdout, stderr }, hints) => {
    ///   expect(stdout).toMatchSnapshot({}, `stdout: ${hints.stdout}`);
    ///   expect(stderr).toMatchSnapshot({}, `stderr: ${hints.stderr}`);
    /// };
    ///
    /// describe('cli', () => {
    ///   describe('--version flag', () => {
    ///     it('prints the version', async () => {
    ///       snapshotOutput(await runCli(['--version']), {
    ///         stdout: 'version string',
    ///         stderr: 'empty',
    ///       });
    ///     });
    ///   });
    ///
    ///   describe('--config flag', () => {
    ///     it('reads the config', async () => {
    ///       const { stdout } = await runCli(['--config', 'jest.config.js']);
    ///
    ///       expect(stdout).toMatchSnapshot();
    ///     });
    ///
    ///     it('prints nothing to stderr', async () => {
    ///       const { stderr } = await runCli(['--config', 'jest.config.js']);
    ///
    ///       expect(stderr).toMatchInlineSnapshot();
    ///     });
    ///
    ///     describe('when the file does not exist', () => {
    ///       it('throws an error', async () => {
    ///         await expect(
    ///           runCli(['--config', 'does-not-exist.js']),
    ///         ).rejects.toThrowErrorMatchingSnapshot();
    ///       });
    ///     });
    ///   });
    /// });
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/docs/rules/prefer-snapshot-hint.md),
    /// to use it, add the following configuration to your `.oxlintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/prefer-snapshot-hint": "error"
    ///   }
    /// }
    /// ```
    PreferSnapshotHint,
    jest,
    correctness,
    config = SnapshotHintMode,
    version = "1.59.0",
);

impl Rule for PreferSnapshotHint {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once<'a>(&self, ctx: &LintContext<'a>) {
        let mut scoped_expects: FxHashMap<NodeId, Vec<&'a CallExpression<'a>>> =
            FxHashMap::default();
        let mut possible_jest_nodes = collect_possible_jest_call_node(ctx);
        possible_jest_nodes.sort_unstable_by_key(|n| n.node.id());

        for possible_jest_node in possible_jest_nodes {
            Self::check_node(&possible_jest_node, &mut scoped_expects, ctx);
        }

        for call_expects in scoped_expects.values() {
            self.check_expects(call_expects, ctx);
        }
    }
}

fn is_test_node(ancestor_call_expr: &CallExpression<'_>) -> bool {
    let Some(id) = ancestor_call_expr.callee_name() else {
        return false;
    };

    matches!(JestFnKind::from(id), JestFnKind::General(JestGeneralFnKind::Test))
}

fn get_scope_owner(node_id: NodeId, ctx: &LintContext<'_>) -> NodeId {
    let mut last_function_id = None;

    for ancestor in ctx.nodes().ancestors(node_id) {
        match ancestor.kind() {
            AstKind::ArrowFunctionExpression(_) | AstKind::Function(_) => {
                last_function_id = Some(ancestor.id());

                let parent = ctx.nodes().parent_node(ancestor.id());
                if let AstKind::CallExpression(call) = parent.kind()
                    && is_test_node(call)
                {
                    return ancestor.id();
                }
            }
            AstKind::Program(_) => {
                return last_function_id.unwrap_or(ancestor.id());
            }
            _ => {}
        }
    }

    last_function_id.unwrap_or(NodeId::ROOT)
}

impl PreferSnapshotHint {
    fn check_node<'a>(
        possible_jest_node: &PossibleJestNode<'a, '_>,
        scoped_expects: &mut FxHashMap<NodeId, Vec<&'a CallExpression<'a>>>,
        ctx: &LintContext<'a>,
    ) {
        let node = possible_jest_node.node;

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(expect_fn) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx) else {
            return;
        };

        if !expect_fn.members.iter().any(|member| {
            member.is_name_equal("toMatchSnapshot")
                || member.is_name_equal("toThrowErrorMatchingSnapshot")
        }) {
            return;
        }

        let scope_owner = get_scope_owner(node.id(), ctx);

        if let Some(expects) = scoped_expects.get_mut(&scope_owner) {
            expects.push(call_expr);
        } else {
            scoped_expects.insert(scope_owner, vec![call_expr]);
        }
    }

    fn check_expects<'a>(
        &self,
        scoped_expects: &Vec<&'a CallExpression<'a>>,
        ctx: &LintContext<'a>,
    ) {
        if matches!(**self, SnapshotHintMode::Multi) && scoped_expects.len() < 2 {
            return;
        }

        for expect_call_expr in scoped_expects {
            if expect_call_expr.arguments.len() == 2 {
                continue;
            }

            let Some(callee_name) = expect_call_expr.callee_name() else {
                continue;
            };

            if expect_call_expr.arguments.is_empty() {
                ctx.diagnostic(snapshot_missing_hint_diagnostic(expect_call_expr.span));
                continue;
            }

            if callee_name == "toMatchSnapshot" && expect_call_expr.arguments.len() > 2 {
                ctx.diagnostic(snapshot_matcher_too_many_arguments_diagnostic(
                    expect_call_expr.span,
                ));
                continue;
            }

            let Some(first_arg) = expect_call_expr.arguments.first() else {
                continue;
            };

            if !first_arg.as_expression().is_some_and(Expression::is_string_literal) {
                ctx.diagnostic(snapshot_hint_must_be_string_diagnostic(expect_call_expr.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
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

    let mut fail = vec![
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

    let vitest_pass = vec![
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
            "
                   it('is true', () => {
                  expect(1).toMatchSnapshot();
                   });
                 ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "
                   it('is true', () => {
                  expect(1).toMatchSnapshot(undefined, 'my first snapshot');
                   });
                 ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "
                   describe('my tests', () => {
                  it('is true', () => {
                    expect(1).toMatchSnapshot('this is a hint, all by itself');
                  });

                  it('is false', () => {
                    expect(2).toMatchSnapshot('this is a hint');
                    expect(2).toMatchSnapshot('and so is this');
                  });
                   });
                 ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "
                   it('is true', () => {
                  expect(1).toMatchSnapshot();
                   });

                   it('is false', () => {
                  expect(2).toMatchSnapshot('this is a hint');
                  expect(2).toMatchSnapshot('and so is this');
                   });
                 ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "
                   it('is true', () => {
                  expect(1).toMatchSnapshot();
                   });

                   it('is false', () => {
                  expect(2).toThrowErrorMatchingSnapshot();
                   });
                 ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "
                   it('is true', () => {
                  expect(1).toStrictEqual(1);
                  expect(1).toStrictEqual(2);
                  expect(1).toMatchSnapshot();
                   });

                   it('is false', () => {
                  expect(1).toStrictEqual(1);
                  expect(1).toStrictEqual(2);
                  expect(2).toThrowErrorMatchingSnapshot();
                   });
                 ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "
                   it('is true', () => {
                  expect(1).toMatchInlineSnapshot();
                   });

                   it('is false', () => {
                  expect(1).toMatchInlineSnapshot();
                  expect(1).toMatchInlineSnapshot();
                  expect(1).toThrowErrorMatchingInlineSnapshot();
                   });
                 ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "
                   it('is true', () => {
                  expect(1).toMatchSnapshot();
                   });

                   it('is false', () => {
                  expect(1).toMatchSnapshot();
                   });
                 ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "
                   const myReusableTestBody = (value, snapshotHint) => {
                  const innerFn = anotherValue => {
                    expect(anotherValue).toMatchSnapshot();

                    expect(value).toBe(1);
                  };

                  expect(value).toBe(1);
                   };

                   it('my test', () => {
                  expect(1).toMatchSnapshot();
                   });
                 ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "
                   const myReusableTestBody = (value, snapshotHint) => {
                  const innerFn = anotherValue => {
                    expect(value).toBe(1);
                  };

                  expect(value).toBe(1);
                  expect(anotherValue).toMatchSnapshot();
                   };

                   it('my test', () => {
                  expect(1).toMatchSnapshot();
                   });
                 ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "
                   const myReusableTestBody = (value, snapshotHint) => {
                  const innerFn = anotherValue => {
                    expect(anotherValue).toMatchSnapshot();

                    expect(value).toBe(1);
                  };

                  expect(value).toBe(1);
                   };

                   expect(1).toMatchSnapshot();
                 ",
            Some(serde_json::json!(["multi"])),
        ),
    ];

    pass.extend(vitest_pass);

    let vitest_fail = vec![
        (
            "it('is true', () => {
                  expect(1).toMatchSnapshot();
                  expect(2).toMatchSnapshot();
                   });
                 ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
                    expect(1).toMatchSnapshot();
                    expect(2).toThrowErrorMatchingSnapshot();
                  });
                   ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
                    expect(1).toThrowErrorMatchingSnapshot();
                    expect(2).toMatchSnapshot();
                  });
                   ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
                    expect(1).toMatchSnapshot({});
                    expect(2).toMatchSnapshot({});
                  });
                   ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
                   expect(1).toMatchSnapshot({});
                   {
                  expect(2).toMatchSnapshot({});
                   }
                 });
                  ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
                   { expect(1).toMatchSnapshot(); }
                   { expect(2).toMatchSnapshot(); }
                 });
                  ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
                    expect(1).toMatchSnapshot();
                    expect(2).toMatchSnapshot(undefined, 'my second snapshot');
                  });
                   ",
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
                  });
                   ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
                    expect(1).toMatchSnapshot({}, 'my first snapshot');
                    expect(2).toMatchSnapshot(undefined);
                    expect(2).toMatchSnapshot();
                  });
                   ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
                    expect(2).toMatchSnapshot();
                    expect(1).toMatchSnapshot({}, 'my second snapshot');
                    expect(2).toMatchSnapshot();
                  });
                   ",
            Some(serde_json::json!(["multi"])),
        ),
        (
            "it('is true', () => {
                    expect(2).toMatchSnapshot(undefined);
                    expect(2).toMatchSnapshot();
                    expect(1).toMatchSnapshot(null, 'my third snapshot');
                  });
                   ",
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
                  });
                   ",
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
                  });
                   ",
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
                  });
                   ",
            Some(serde_json::json!(["multi"])),
        ),
    ];

    fail.extend(vitest_fail);

    Tester::new(PreferSnapshotHint::NAME, PreferSnapshotHint::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
