use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;

use crate::{
    context::LintContext,
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call},
};

fn unexpected_hook_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use setup or teardown hooks.")
        .with_help("Inline the setup or teardown logic directly in each test for better readability and isolation.")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

Disallows Jest setup and teardown hooks, such as `beforeAll`.

### Why is this bad?

Jest provides global functions for setup and teardown tasks, which are
called before/after each test case and each test suite. The use of these
hooks promotes shared state between tests.

This rule reports for the following function calls:
* `beforeAll`
* `beforeEach`
* `afterAll`
* `afterEach`

### Examples

Examples of **incorrect** code for this rule:
```javascript
function setupFoo(options) { /* ... */ }
function setupBar(options) { /* ... */ }

describe('foo', () => {
    let foo;
    beforeEach(() => {
        foo = setupFoo();
    });
    afterEach(() => {
        foo = null;
    });
    it('does something', () => {
        expect(foo.doesSomething()).toBe(true);
    });
    describe('with bar', () => {
        let bar;
        beforeEach(() => {
            bar = setupBar();
        });
        afterEach(() => {
            bar = null;
        });
        it('does something with bar', () => {
            expect(foo.doesSomething(bar)).toBe(true);
        });
    });
});
```
";

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoHooksConfig {
    /// An array of hook function names that are permitted for use.
    allow: Vec<CompactStr>,
}

impl NoHooksConfig {
    pub fn run_on_jest_node<'a>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, '_>,
        ctx: &LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        if !is_type_of_jest_fn_call(
            call_expr,
            possible_jest_node,
            ctx,
            &[JestFnKind::General(JestGeneralFnKind::Hook)],
        ) {
            return;
        }

        if let Expression::Identifier(ident) = &call_expr.callee {
            let name = CompactStr::from(ident.name.as_str());
            if !self.allow.contains(&name) {
                ctx.diagnostic(unexpected_hook_diagnostic(call_expr.callee.span()));
            }
        }
    }
}
