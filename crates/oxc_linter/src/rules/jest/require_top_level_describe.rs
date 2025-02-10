use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeId;
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, parse_jest_fn_call, JestFnKind, JestGeneralFnKind,
        ParsedGeneralJestFnCall, ParsedJestFnCallNew, PossibleJestNode,
    },
};

fn too_many_describes(max: usize, repeat: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Require test cases and hooks to be inside a `describe` block")
        .with_help(format!(
            "There should not be more than {max:?} describe{repeat} at the top level."
        ))
        .with_label(span)
}

fn unexpected_test_case(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Require test cases and hooks to be inside a `describe` block")
        .with_help("All test cases must be wrapped in a describe block.")
        .with_label(span)
}

fn unexpected_hook(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Require test cases and hooks to be inside a `describe` block")
        .with_help("All hooks must be wrapped in a describe block.")
        .with_label(span)
}

#[derive(Debug, Clone)]
pub struct RequireTopLevelDescribe {
    pub max_number_of_top_level_describes: usize,
}

impl Default for RequireTopLevelDescribe {
    fn default() -> Self {
        Self { max_number_of_top_level_describes: usize::MAX }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule triggers a warning if a test case (`test` and `it`) or a hook
    /// (`beforeAll`, `beforeEach`, `afterEach`, `afterAll`) is not located in a
    /// top-level `describe` block.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // invalid
    ///
    /// // Above a describe block
    /// test('my test', () => {});
    /// describe('test suite', () => {
    ///     it('test', () => {});
    /// });
    ///
    /// // Below a describe block
    /// describe('test suite', () => {});
    /// test('my test', () => {});
    ///
    /// // Same for hooks
    /// beforeAll('my beforeAll', () => {});
    /// describe('test suite', () => {});
    /// afterEach('my afterEach', () => {});
    ///
    /// //valid
    ///
    /// // Above a describe block
    /// // In a describe block
    /// describe('test suite', () => {
    ///     test('my test', () => {});
    /// });
    ///
    /// // In a nested describe block
    /// describe('test suite', () => {
    ///     test('my test', () => {});
    ///     describe('another test suite', () => {
    ///         test('my other test', () => {});
    ///     });
    /// });
    /// ```
    ///
    /// ### Options
    ///
    /// You can also enforce a limit on the number of describes allowed at the top-level
    /// using the `maxNumberOfTopLevelDescribes` option:
    ///
    /// ```json
    /// {
    ///   "jest/require-top-level-describe": [
    ///     "error",
    ///     {
    ///       "maxNumberOfTopLevelDescribes": 2
    ///     }
    ///   ]
    /// }
    /// ```
    ///
    RequireTopLevelDescribe,
    jest,
    style,
);

impl Rule for RequireTopLevelDescribe {
    fn from_configuration(value: serde_json::Value) -> Self {
        let max_number_of_top_level_describes = value
            .get(0)
            .and_then(|config| config.get("maxNumberOfTopLevelDescribes"))
            .and_then(serde_json::Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .map_or(usize::MAX, |v| usize::try_from(v).unwrap_or(usize::MAX));

        Self { max_number_of_top_level_describes }
    }

    fn run_once(&self, ctx: &LintContext) {
        let mut describe_contexts: FxHashMap<ScopeId, usize> = FxHashMap::default();
        let mut possibles_jest_nodes = collect_possible_jest_call_node(ctx);
        possibles_jest_nodes.sort_by_key(|n| n.node.id());

        for possible_jest_node in &possibles_jest_nodes {
            self.run(possible_jest_node, &mut describe_contexts, ctx);
        }
    }
}

impl RequireTopLevelDescribe {
    fn run<'a>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, '_>,
        describe_contexts: &mut FxHashMap<ScopeId, usize>,
        ctx: &LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        let scopes = ctx.scopes();
        let is_top = scopes.get_flags(node.scope_id()).is_top();

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(ParsedJestFnCallNew::GeneralJest(ParsedGeneralJestFnCall { kind, .. })) =
            parse_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        match kind {
            JestFnKind::General(JestGeneralFnKind::Test) => {
                if is_top {
                    ctx.diagnostic(unexpected_test_case(call_expr.span));
                }
            }
            JestFnKind::General(JestGeneralFnKind::Hook) => {
                if is_top {
                    ctx.diagnostic(unexpected_hook(call_expr.span));
                }
            }
            JestFnKind::General(JestGeneralFnKind::Describe) => {
                if !is_top {
                    return;
                }

                let Some((_, count)) = describe_contexts.get_key_value(&node.scope_id()) else {
                    describe_contexts.insert(node.scope_id(), 1);
                    return;
                };

                if count >= &self.max_number_of_top_level_describes {
                    ctx.diagnostic(too_many_describes(
                        self.max_number_of_top_level_describes,
                        if *count == 1 { "" } else { "s" },
                        call_expr.span,
                    ));
                } else {
                    describe_contexts.insert(node.scope_id(), count + 1);
                }
            }
            _ => (),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // Note: Both Jest and Vitest share the same unit tests

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
            Some(serde_json::json!({ "maxNumberOfTopLevelDescribes": 1 })),
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
