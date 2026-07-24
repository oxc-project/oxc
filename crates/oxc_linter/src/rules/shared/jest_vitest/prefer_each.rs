use oxc_ast::{AstKind, AstType};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::{AstNode, AstTypesBitset, NodeId};
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{
    context::{ContextHost, LintContext},
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, parse_jest_fn_call},
};

fn use_prefer_each(span: Span, fn_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enforce using `each` rather than manual loops")
        .with_help(format!("Prefer using `{fn_name}.each` rather than a manual loop."))
        .with_label(span)
}

#[inline]
fn is_in_test(ctx: &LintContext<'_>, id: NodeId) -> bool {
    ctx.nodes().ancestors(id).any(|node| {
        let AstKind::CallExpression(ancestor_call_expr) = node.kind() else { return false };
        let Some(ancestor_member_expr) = ancestor_call_expr.callee.as_member_expression() else {
            return false;
        };
        let Some(id) = ancestor_member_expr.object().get_identifier_reference() else {
            return false;
        };

        matches!(JestFnKind::from(id.name.as_str()), JestFnKind::General(JestGeneralFnKind::Test))
    })
}

pub const DOCUMENTATION: &str = r"### What it does

This rule enforces using `each` rather than manual loops.

### Why is this bad?

Manual loops for tests can be less readable and more error-prone. Using
`each` provides a clearer and more concise way to run parameterized tests,
improving readability and maintainability.

### Examples

Examples of **incorrect** code for this rule:
```js
for (const item of items) {
	describe(item, () => {
		expect(item).toBe('foo')
	})
}
```

Examples of **correct** code for this rule:
```js
describe.each(items)('item', (item) => {
	expect(item).toBe('foo')
})
```
";

/// A diagnostic requires a test call directly inside a `for`/`for-in`/`for-of` loop,
/// so files without any loop statement (or without any call) can be skipped entirely.
const LOOP_NODE_TYPES: &AstTypesBitset = &AstTypesBitset::from_types(&[
    AstType::ForStatement,
    AstType::ForInStatement,
    AstType::ForOfStatement,
]);

#[derive(Debug, Default, Clone)]
pub struct PreferEachConfig;

impl PreferEachConfig {
    pub fn should_run(ctx: &ContextHost) -> bool {
        let nodes = ctx.semantic().nodes();
        nodes.contains_any(LOOP_NODE_TYPES) && nodes.contains(AstType::CallExpression)
    }

    pub fn run_once(ctx: &LintContext<'_>) {
        let mut skip = FxHashSet::<NodeId>::default();
        ctx.nodes().iter().for_each(|node| {
            Self::run(node, ctx, &mut skip);
        });
    }

    fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>, skip: &mut FxHashSet<NodeId>) {
        let kind = node.kind();

        let AstKind::CallExpression(call_expr) = kind else { return };

        // Walking up to the nearest enclosing loop is much cheaper than
        // `parse_jest_fn_call`, so find it first: only calls sitting directly in a
        // loop (not nested in another call) can be reported.
        let Some(loop_node) = ctx
            .nodes()
            .ancestors(node.id())
            .find_map(|parent_node| match parent_node.kind() {
                AstKind::CallExpression(_) => Some(None),
                AstKind::ForStatement(_)
                | AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_) => Some(Some(parent_node)),
                _ => None,
            })
            .flatten()
        else {
            return;
        };

        if skip.contains(&loop_node.id()) {
            return;
        }

        let Some(vitest_fn_call) =
            parse_jest_fn_call(call_expr, &PossibleJestNode { node, original: None }, ctx)
        else {
            return;
        };

        let fn_name = match vitest_fn_call.kind() {
            JestFnKind::General(JestGeneralFnKind::Test) => "it",
            JestFnKind::General(JestGeneralFnKind::Describe | JestGeneralFnKind::Hook) => {
                "describe"
            }
            _ => return,
        };

        if is_in_test(ctx, loop_node.id()) {
            return;
        }

        let span = match loop_node.kind() {
            AstKind::ForStatement(statement) => {
                Span::new(statement.span.start, statement.body.span().start)
            }
            AstKind::ForInStatement(statement) => {
                Span::new(statement.span.start, statement.body.span().start)
            }
            AstKind::ForOfStatement(statement) => {
                Span::new(statement.span.start, statement.body.span().start)
            }
            _ => unreachable!(),
        };

        skip.insert(loop_node.id());
        ctx.diagnostic(use_prefer_each(span, fn_name));
    }
}
