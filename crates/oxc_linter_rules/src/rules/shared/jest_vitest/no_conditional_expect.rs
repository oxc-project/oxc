use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::{AstNode, NodeId};
use oxc_span::Span;
use rustc_hash::FxHashSet;

use crate::{
    context::LintContext,
    utils::{
        JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call,
        parse_expect_jest_fn_call,
    },
};

fn no_conditional_expect_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected conditional expect")
        .with_help("Avoid calling `expect` conditionally")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

This rule prevents the use of `expect` in conditional blocks, such as `if` and `catch`.
This includes using `expect` in callbacks to functions named `catch`, which are assumed to be promises.

### Why is this bad?

Jest only considers a test to have failed if it throws an error, meaning if calls to
assertion functions like `expect` occur in conditional code such as a `catch` statement,
tests can end up passing but not actually test anything. Additionally, conditionals
tend to make tests more brittle and complex, as they increase the amount of mental
thinking needed to understand what is actually being tested.

### Examples

Examples of **incorrect** code for this rule:
```js
it('foo', () => {
  doTest && expect(1).toBe(2);
});

it('bar', () => {
  if (!skipTest) {
    expect(1).toEqual(2);
  }
});

it('baz', async () => {
  try {
    await foo();
  } catch (err) {
    expect(err).toMatchObject({ code: 'MODULE_NOT_FOUND' });
  }
});

it('throws an error', async () => {
  await foo().catch(error => expect(error).toBeInstanceOf(error));
});
```

Examples of **correct** code for this rule:
```js
it('foo', () => {
  expect(!value).toBe(false);
});

function getValue() {
  if (process.env.FAIL) {
    return 1;
  }
  return 2;
}

it('foo', () => {
  expect(getValue()).toBe(2);
});

it('validates the request', () => {
  try {
    processRequest(request);
  } catch { } finally {
    expect(validRequest).toHaveBeenCalledWith(request);
  }
});

it('throws an error', async () => {
  await expect(foo).rejects.toThrow(Error);
});
```
";

// To flag we encountered a conditional block/catch block when traversing the parents.
#[derive(Debug, Clone, Copy)]
struct InConditional(bool);

pub fn run_on_jest_node<'a, 'c>(
    possible_jest_node: &PossibleJestNode<'a, 'c>,
    ctx: &'c LintContext<'a>,
) {
    let node = possible_jest_node.node;
    if let AstKind::CallExpression(call_expr) = node.kind() {
        let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        // Record visited nodes for avoid infinite loop.
        let mut visited = FxHashSet::default();

        // When first visiting the node, we assume it's not in a conditional block.
        let has_condition_or_catch = check_parents(node, &mut visited, InConditional(false), ctx);
        if matches!(has_condition_or_catch, InConditional(true)) {
            ctx.diagnostic(no_conditional_expect_diagnostic(jest_fn_call.head.span));
        }
    }
}

fn is_in_test_context<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let mut current = node;
    loop {
        current = ctx.nodes().parent_node(current.id());

        if let AstKind::CallExpression(call_expr) = current.kind() {
            let jest_node = PossibleJestNode { node: current, original: None };
            if is_type_of_jest_fn_call(
                call_expr,
                &jest_node,
                ctx,
                &[JestFnKind::General(JestGeneralFnKind::Test)],
            ) {
                return true;
            }
        }

        if matches!(current.kind(), AstKind::Program(_)) {
            return false;
        }
    }
}

fn check_parents<'a>(
    node: &AstNode<'a>,
    visited: &mut FxHashSet<NodeId>,
    in_conditional: InConditional,
    ctx: &LintContext<'a>,
) -> InConditional {
    // if the node is already visited, we should return `false` to avoid infinite loop.
    if !visited.insert(node.id()) {
        return InConditional(false);
    }

    let parent_node = ctx.nodes().parent_node(node.id());

    match parent_node.kind() {
        AstKind::CallExpression(call_expr) => {
            let jest_node = PossibleJestNode { node: parent_node, original: None };

            if is_type_of_jest_fn_call(
                call_expr,
                &jest_node,
                ctx,
                &[JestFnKind::General(JestGeneralFnKind::Test)],
            ) {
                return in_conditional;
            }

            if let Some(member_expr) = call_expr.callee.as_member_expression()
                && member_expr.static_property_name() == Some("catch")
            {
                return check_parents(parent_node, visited, InConditional(true), ctx);
            }
        }
        AstKind::CatchClause(_)
        | AstKind::SwitchStatement(_)
        | AstKind::IfStatement(_)
        | AstKind::ConditionalExpression(_)
        | AstKind::LogicalExpression(_) => {
            return check_parents(parent_node, visited, InConditional(true), ctx);
        }
        AstKind::Function(function) => {
            let Some(ident) = &function.id else {
                return InConditional(false);
            };
            let symbol_table = ctx.scoping();
            let symbol_id = ident.symbol_id();

            // Check if this function is used in a test context
            let is_used_in_test =
                symbol_table.get_resolved_references(symbol_id).any(|reference| {
                    let parent = ctx.nodes().parent_node(reference.node_id());

                    // Check if directly used as test callback
                    if let AstKind::CallExpression(call_expr) = parent.kind() {
                        let jest_node = PossibleJestNode { node: parent, original: None };
                        if is_type_of_jest_fn_call(
                            call_expr,
                            &jest_node,
                            ctx,
                            &[JestFnKind::General(JestGeneralFnKind::Test)],
                        ) {
                            return true;
                        }
                    }

                    // Check if called within a test context by traversing from the call site
                    is_in_test_context(parent, ctx)
                });

            return if is_used_in_test { in_conditional } else { InConditional(false) };
        }
        AstKind::Program(_) => return InConditional(false),
        _ => {}
    }

    check_parents(parent_node, visited, in_conditional, ctx)
}
