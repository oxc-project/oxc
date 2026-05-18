use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::ScopeId;
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::{
    context::LintContext,
    utils::{
        JestFnKind, JestGeneralFnKind, ParsedJestFnCallNew, PossibleJestNode, parse_jest_fn_call,
    },
};

fn reorder_hooks(hook: (&str, Span), previous_hook: (&str, Span)) -> OxcDiagnostic {
    OxcDiagnostic::warn("Test hooks are not in a consistent order.")
        .with_help(format!("{:?} hooks should be before any {:?} hooks", hook.0, previous_hook.0))
        .with_label(
            hook.1.label(format!("this should be moved to before the {:?} hook", previous_hook.0)),
        )
        .and_label(previous_hook.1.label(format!("{:?} hook should be called before this", hook.0)))
}

pub const DOCUMENTATION: &str = r"### What it does

Ensures that hooks are in the order that they are called in.

### Why is this bad?

While hooks can be setup in any order, they're always called by `jest` in this
specific order:
1. `beforeAll`
2. `beforeEach`
3. `afterEach`
4. `afterAll`

This rule aims to make that more obvious by enforcing grouped hooks be setup in
that order within tests.

### Examples

Examples of **incorrect** code for this rule:
```javascript
describe('foo', () => {
  beforeEach(() => {
    seedMyDatabase();
  });
  beforeAll(() => {
    createMyDatabase();
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
    it('accepts that input', () => {
      // ...
    });
    it('throws an error', () => {
      // ...
    });
    beforeEach(() => {
      mockLogger();
    });
    afterEach(() => {
      clearLogger();
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
";

pub fn run_once(ctx: &LintContext) {
    let mut previous_hook_orders: FxHashMap<ScopeId, (u8, Span)> = FxHashMap::default();

    for node in ctx.nodes() {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            continue;
        };

        let possible_jest_node = &PossibleJestNode { node, original: None };
        let Some(ParsedJestFnCallNew::GeneralJest(jest_fn_call)) =
            parse_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            previous_hook_orders.remove(&node.scope_id());
            continue;
        };

        if !matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Hook)) {
            previous_hook_orders.remove(&node.scope_id());
            continue;
        }

        let previous_hook_order = previous_hook_orders.get(&node.scope_id());

        let hook_name = jest_fn_call.name.as_ref();
        let Some(hook_order) = get_hook_order(hook_name) else {
            continue;
        };

        if let Some((previous_hook_order, previous_hook_span)) = previous_hook_order
            && hook_order < *previous_hook_order
        {
            let Some(previous_hook_name) = get_hook_name(*previous_hook_order) else {
                continue;
            };

            ctx.diagnostic(reorder_hooks(
                (hook_name, call_expr.span),
                (previous_hook_name, *previous_hook_span),
            ));
            continue;
        }
        previous_hook_orders.insert(node.scope_id(), (hook_order, call_expr.span));
    }
}

fn get_hook_order(hook_name: &str) -> Option<u8> {
    match hook_name {
        "beforeAll" => Some(0),
        "beforeEach" => Some(1),
        "afterEach" => Some(2),
        "afterAll" => Some(3),
        _ => None,
    }
}

fn get_hook_name(hook_order: u8) -> Option<&'static str> {
    match hook_order {
        0 => Some("beforeAll"),
        1 => Some("beforeEach"),
        2 => Some("afterEach"),
        3 => Some("afterAll"),
        _ => None,
    }
}
