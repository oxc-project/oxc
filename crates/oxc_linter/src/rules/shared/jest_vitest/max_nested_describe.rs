use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::ScopeId;
use oxc_span::Span;

use crate::{
    context::LintContext,
    utils::{
        JestFnKind, JestGeneralFnKind, PossibleJestNode, collect_possible_jest_call_node,
        is_type_of_jest_fn_call,
    },
};

fn exceeded_max_depth(current: usize, max: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enforces a maximum depth to nested describe calls.")
        .with_help(format!("Too many nested describe calls ({current}) - maximum allowed is {max}"))
        .with_label(span)
}

pub const DOCUMENTATION: &str = r#"### What it does

This rule enforces a maximum depth to nested `describe()` calls.

### Why is this bad?

Nesting `describe()` blocks too deeply can make the test suite hard to read and understand.

### Examples

The following patterns are considered warnings (with the default option of
`{ "max": 5 } `):

Examples of **incorrect** code for this rule:
```javascript
describe('foo', () => {
    describe('bar', () => {
        describe('baz', () => {
            describe('qux', () => {
                describe('quxx', () => {
                    describe('too many', () => {
                        it('should get something', () => {
                            expect(getSomething()).toBe('Something');
                        });
                    });
                });
            });
        });
    });
});

describe('foo', function () {
    describe('bar', function () {
        describe('baz', function () {
            describe('qux', function () {
                describe('quxx', function () {
                    describe('too many', function () {
                        it('should get something', () => {
                            expect(getSomething()).toBe('Something');
                        });
                    });
                });
            });
        });
    });
});
```

Examples of **correct** code for this rule:
```ts
describe('foo', () => {
    describe('bar', () => {
        it('should get something', () => {
            expect(getSomething()).toBe('Something');
        });
    });
    describe('qux', () => {
        it('should get something', () => {
            expect(getSomething()).toBe('Something');
        });
    });
});

describe('foo2', function () {
    it('should get something', () => {
        expect(getSomething()).toBe('Something');
    });
});

describe('foo', function () {
    describe('bar', function () {
        describe('baz', function () {
            describe('qux', function () {
                describe('this is the limit', function () {
                    it('should get something', () => {
                        expect(getSomething()).toBe('Something');
                    });
                });
            });
        });
    });
});
```
"#;

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
#[schemars(default)]
pub struct MaxNestedDescribeConfig {
    /// Maximum allowed depth of nested describe calls.
    pub max: usize,
}

impl Default for MaxNestedDescribeConfig {
    fn default() -> Self {
        Self { max: 5 }
    }
}

impl MaxNestedDescribeConfig {
    pub fn run_once(&self, ctx: &LintContext) {
        let mut describes_hooks_depth: Vec<ScopeId> = vec![];
        let mut possibles_jest_nodes = collect_possible_jest_call_node(ctx);
        possibles_jest_nodes.sort_unstable_by_key(|n| n.node.id());

        for possible_jest_node in &possibles_jest_nodes {
            self.run(possible_jest_node, &mut describes_hooks_depth, ctx);
        }
    }

    fn run<'a>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, '_>,
        describes_hooks_depth: &mut Vec<ScopeId>,
        ctx: &LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        let scope_id = node.scope_id();
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let is_describe_call = is_type_of_jest_fn_call(
            call_expr,
            possible_jest_node,
            ctx,
            &[JestFnKind::General(JestGeneralFnKind::Describe)],
        );

        if is_describe_call && !describes_hooks_depth.contains(&scope_id) {
            describes_hooks_depth.push(scope_id);
        }

        if is_describe_call && describes_hooks_depth.len() > self.max {
            ctx.diagnostic(exceeded_max_depth(
                describes_hooks_depth.len(),
                self.max,
                call_expr.span,
            ));
        }
    }
}
