use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::ScopeId;
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::{
    context::LintContext,
    utils::{
        JestFnKind, JestGeneralFnKind, PossibleJestNode, collect_possible_jest_call_node,
        is_type_of_jest_fn_call,
    },
};

fn no_hook_on_top(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest having hooks before any test cases.")
        .with_help("Hooks should come before test cases")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

While hooks can be setup anywhere in a test file, they are always called in a
specific order, which means it can be confusing if they're intermixed with test
cases.

### Why is this bad?

When hooks are mixed with test cases, it becomes harder to understand
the test setup and execution order. This can lead to confusion about
which hooks apply to which tests and when they run. Grouping hooks at
the top of each `describe` block makes the test structure clearer and
more maintainable.

### Examples

Examples of **incorrect** code for this rule:
```javascript
describe('foo', () => {
    beforeEach(() => {
        seedMyDatabase();
    });

    it('accepts this input', () => {
        // ...
    });

    beforeAll(() => {
        createMyDatabase();
    });

    it('returns that value', () => {
        // ...
    });

    describe('when the database has specific values', () => {
        const specificValue = '...';
        beforeEach(() => {
            seedMyDatabase(specificValue);
        });

        it('accepts that input', () => {
            // ...
        });

        it('throws an error', () => {
            // ...
        });

        afterEach(() => {
            clearLogger();
        });

        beforeEach(() => {
            mockLogger();
        });

        it('logs a message', () => {
            // ...
        });
    });

    afterAll(() => {
        removeMyDatabase();
    });
});
```

Examples of **correct** code for this rule:
```javascript
describe('foo', () => {
    beforeAll(() => {
        createMyDatabase();
    });

    beforeEach(() => {
        seedMyDatabase();
    });

    afterAll(() => {
        clearMyDatabase();
    });

    it('accepts this input', () => {
        // ...
    });

    it('returns that value', () => {
        // ...
    });

    describe('when the database has specific values', () => {
        const specificValue = '...';

        beforeEach(() => {
            seedMyDatabase(specificValue);
        });

        beforeEach(() => {
            mockLogger();
        });

        afterEach(() => {
            clearLogger();
        });

        it('accepts that input', () => {
            // ...
        });

        it('throws an error', () => {
            // ...
        });

        it('logs a message', () => {
            // ...
        });
    });
});
```
";

pub fn run_once(ctx: &LintContext) {
    let mut hooks_contexts: FxHashMap<ScopeId, bool> = FxHashMap::default();
    let mut possibles_jest_nodes = collect_possible_jest_call_node(ctx);
    possibles_jest_nodes.sort_unstable_by_key(|n| n.node.id());

    for possible_jest_node in &possibles_jest_nodes {
        run(possible_jest_node, &mut hooks_contexts, ctx);
    }
}

fn run<'a>(
    possible_jest_node: &PossibleJestNode<'a, '_>,
    hooks_context: &mut FxHashMap<ScopeId, bool>,
    ctx: &LintContext<'a>,
) {
    let node = possible_jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };

    if is_type_of_jest_fn_call(
        call_expr,
        possible_jest_node,
        ctx,
        &[JestFnKind::General(JestGeneralFnKind::Test)],
    ) {
        hooks_context.insert(node.scope_id(), true);
    }

    let Some((_, has_hook)) = hooks_context.get_key_value(&node.scope_id()) else {
        return;
    };

    if *has_hook
        && is_type_of_jest_fn_call(
            call_expr,
            possible_jest_node,
            ctx,
            &[JestFnKind::General(JestGeneralFnKind::Hook)],
        )
    {
        ctx.diagnostic(no_hook_on_top(call_expr.span));
    }
}
