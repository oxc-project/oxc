use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::{
    context::LintContext,
    utils::{
        JestFnKind, JestGeneralFnKind, collect_possible_jest_call_node, is_type_of_jest_fn_call,
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
        let mut describe_call_spans = collect_possible_jest_call_node(ctx)
            .into_iter()
            .filter_map(|possible_jest_node| {
                let AstKind::CallExpression(call_expr) = possible_jest_node.node.kind() else {
                    return None;
                };

                is_type_of_jest_fn_call(
                    call_expr,
                    &possible_jest_node,
                    ctx,
                    &[JestFnKind::General(JestGeneralFnKind::Describe)],
                )
                .then_some(call_expr.span)
            })
            .collect::<Vec<_>>();

        describe_call_spans
            .sort_unstable_by(|a, b| a.start.cmp(&b.start).then_with(|| b.end.cmp(&a.end)));

        let mut active_describes: Vec<Span> = Vec::new();
        for span in describe_call_spans {
            while active_describes.last().is_some_and(|parent| !parent.contains_inclusive(span)) {
                active_describes.pop();
            }
            active_describes.push(span);

            let current_depth = active_describes.len();
            if current_depth > self.max {
                ctx.diagnostic(exceeded_max_depth(current_depth, self.max, span));
            }
        }
    }
}
