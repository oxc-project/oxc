use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{
    context::LintContext,
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call},
};

fn no_conditional_in_test(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid having conditionals in tests.")
        .with_help("Replace conditionals with separate test cases for each branch to keep tests deterministic and easy to understand.")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

Disallow conditional statements in tests.

### Why is this bad?

Conditional statements in tests can make the test harder to read and understand.
It is better to have a single test case per test function.

### Examples

Examples of **incorrect** code for this rule:
```js
it('foo', () => {
  if (true) {
	doTheThing();
  }
});

it('bar', () => {
  switch (mode) {
    case 'none':
      generateNone();
    case 'single':
      generateOne();
    case 'multiple':
      generateMany();
  }

  expect(fixtures.length).toBeGreaterThan(-1);
});

it('baz', async () => {
  const promiseValue = () => {
    return something instanceof Promise
      ? something
      : Promise.resolve(something);
  };

  await expect(promiseValue()).resolves.toBe(1);
});
```

Examples of **correct** code for this rule:
```js
describe('my tests', () => {
  if (true) {
    it('foo', () => {
      doTheThing();
    });
  }
});

beforeEach(() => {
  switch (mode) {
    case 'none':
      generateNone();
    case 'single':
      generateOne();
    case 'multiple':
      generateMany();
  }
});

it('bar', () => {
  expect(fixtures.length).toBeGreaterThan(-1);
});

const promiseValue = something => {
  return something instanceof Promise ? something : Promise.resolve(something);
};

it('baz', async () => {
  await expect(promiseValue()).resolves.toBe(1);
});
```
";

pub fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
    match node.kind() {
        AstKind::IfStatement(_)
        | AstKind::SwitchStatement(_)
        | AstKind::ConditionalExpression(_)
        | AstKind::LogicalExpression(_) => {}
        _ => return,
    }

    let is_if_statement_in_test = ctx.nodes().ancestors(node.id()).any(|node| {
        let AstKind::CallExpression(call_expr) = node.kind() else { return false };
        let vitest_node = PossibleJestNode { node, original: None };

        is_type_of_jest_fn_call(
            call_expr,
            &vitest_node,
            ctx,
            &[JestFnKind::General(JestGeneralFnKind::Test)],
        )
    });

    if is_if_statement_in_test {
        let span = match node.kind() {
            AstKind::IfStatement(stmt) => stmt.span,
            AstKind::SwitchStatement(stmt) => stmt.span,
            AstKind::ConditionalExpression(expr) => expr.span,
            AstKind::LogicalExpression(expr) => expr.span,
            _ => unreachable!(),
        };

        ctx.diagnostic(no_conditional_in_test(span));
    }
}
