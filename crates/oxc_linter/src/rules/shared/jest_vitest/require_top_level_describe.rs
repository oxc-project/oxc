use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::ScopeId;
use oxc_span::Span;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::DefaultRuleConfig,
    utils::{
        JestFnKind, JestGeneralFnKind, ParsedGeneralJestFnCall, ParsedJestFnCallNew,
        PossibleJestNode, collect_possible_jest_call_node, parse_jest_fn_call,
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

pub const DOCUMENTATION: &str = r"### What it does

Requires test cases and hooks to be inside a top-level `describe` block.

### Why is this bad?

Having tests and hooks organized within `describe` blocks provides better
structure and grouping for test suites. It makes test output more readable
and helps with test organization, especially in larger codebases.

This rule triggers a warning if a test case (`test` and `it`) or a hook
(`beforeAll`, `beforeEach`, `afterEach`, `afterAll`) is not located in a
top-level `describe` block.

### Examples

Examples of **incorrect** code for this rule:
```javascript
// Above a describe block
test('my test', () => {});
describe('test suite', () => {
    it('test', () => {});
});

// Below a describe block
describe('test suite', () => {});
test('my test', () => {});

// Same for hooks
beforeAll('my beforeAll', () => {});
describe('test suite', () => {});
afterEach('my afterEach', () => {});
```

Examples of **correct** code for this rule:
```javascript
// Above a describe block
// In a describe block
describe('test suite', () => {
    test('my test', () => {});
});

// In a nested describe block
describe('test suite', () => {
    test('my test', () => {});
    describe('another test suite', () => {
        test('my other test', () => {});
    });
});
```
";

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct RequireTopLevelDescribeConfig {
    /// The maximum number of top-level `describe` blocks allowed in a test file.
    pub max_number_of_top_level_describes: usize,
}

impl Default for RequireTopLevelDescribeConfig {
    fn default() -> Self {
        Self { max_number_of_top_level_describes: usize::MAX }
    }
}

impl RequireTopLevelDescribeConfig {
    pub fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    pub fn run_once(&self, ctx: &LintContext) {
        let mut describe_contexts: FxHashMap<ScopeId, usize> = FxHashMap::default();
        let mut possibles_jest_nodes = collect_possible_jest_call_node(ctx);
        possibles_jest_nodes.sort_unstable_by_key(|n| n.node.id());

        for possible_jest_node in &possibles_jest_nodes {
            self.run(possible_jest_node, &mut describe_contexts, ctx);
        }
    }

    fn run<'a>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, '_>,
        describe_contexts: &mut FxHashMap<ScopeId, usize>,
        ctx: &LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        let scopes = ctx.scoping();
        let is_top = scopes.scope_flags(node.scope_id()).is_top();

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(ParsedJestFnCallNew::GeneralJest(ParsedGeneralJestFnCall { kind, .. })) =
            parse_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        match kind {
            JestFnKind::General(JestGeneralFnKind::Test) if is_top => {
                ctx.diagnostic(unexpected_test_case(call_expr.span));
            }
            JestFnKind::General(JestGeneralFnKind::Hook) if is_top => {
                ctx.diagnostic(unexpected_hook(call_expr.span));
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
