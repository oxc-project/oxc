use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::{
    context::LintContext,
    utils::{PossibleJestNode, collect_possible_jest_call_node},
};

fn exceeded_max_assertion(count: usize, max: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enforces a maximum number assertion calls in a test body.")
        .with_help(format!("Too many assertion calls ({count}) - maximum allowed is {max}"))
        .with_label(span)
}

pub const DOCUMENTATION: &str = r#"### What it does

This rule enforces a maximum number of `expect()` calls in a single test.

### Why is this bad?

Tests with many different assertions are likely mixing multiple objectives.
It is generally better to have a single objective per test to ensure that when a test fails,
the problem is easy to identify.

### Examples

Examples of **incorrect** code for this rule:
```javascript
test('should not pass', () => {
    expect(true).toBeDefined();
    expect(true).toBeDefined();
    expect(true).toBeDefined();
    expect(true).toBeDefined();
    expect(true).toBeDefined();
    expect(true).toBeDefined();
});

it('should not pass', () => {
    expect(true).toBeDefined();
    expect(true).toBeDefined();
    expect(true).toBeDefined();
    expect(true).toBeDefined();
    expect(true).toBeDefined();
    expect(true).toBeDefined();
});
```

This rule is compatible with [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/docs/rules/max-expects.md),
to use it, add the following configuration to your `.oxlintrc.json`:

```json
{
  "rules": {
     "vitest/max-expects": "error"
  }
}
```
"#;

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct MaxExpectsConfig {
    /// Maximum number of `expect()` assertion calls allowed within a single test.
    pub max: usize,
}

impl Default for MaxExpectsConfig {
    fn default() -> Self {
        Self { max: 5 }
    }
}

impl MaxExpectsConfig {
    pub fn run_once(&self, ctx: &LintContext) {
        let mut count_map: FxHashMap<usize, usize> = FxHashMap::default();

        for possible_jest_node in &collect_possible_jest_call_node(ctx) {
            self.run(possible_jest_node, &mut count_map, ctx);
        }
    }

    fn run<'a>(
        &self,
        jest_node: &PossibleJestNode<'a, '_>,
        count_map: &mut FxHashMap<usize, usize>,
        ctx: &LintContext<'a>,
    ) {
        let node = jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Expression::Identifier(ident) = &call_expr.callee else {
            return;
        };

        if ident.name == "expect" {
            let position = node.scope_id().index();

            if let Some(count) = count_map.get(&position) {
                if count > &self.max {
                    ctx.diagnostic(exceeded_max_assertion(*count, self.max, ident.span));
                } else {
                    count_map.insert(position, count + 1);
                }
            } else {
                count_map.insert(position, 2);
            }
        }
    }
}
