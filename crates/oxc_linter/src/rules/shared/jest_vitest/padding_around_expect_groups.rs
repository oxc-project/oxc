use oxc_ast::AstKind;
use oxc_semantic::AstNode;
use oxc_span::GetSpan;

use crate::{
    context::LintContext,
    utils::{
        PaddingDirection, ParsedJestFnCallNew, PossibleJestNode, check_padding_between,
        enclosing_statement_index, enclosing_statement_list, leading_token_of_statement,
        parse_jest_fn_call,
    },
};

pub const DOCUMENTATION: &str = r"### What it does

This rule enforces a line of padding before and after 1 or more `expect`
statements.

Note that it doesn't add/enforce a padding line if it's the last statement
in its scope and it doesn't add/enforce padding between two or more adjacent
`expect` statements.

### Why is this bad?

Inconsistent formatting of code can make the code more difficult to read
and follow. This rule helps ensure that groups of `expect` statements are
visually separated from the rest of the code, making them easier to identify
while looking through test files.

### Examples

Examples of **incorrect** code for this rule:
```js
test('thing one', () => {
  let abc = 123;
  expect(abc).toEqual(123);
  expect(123).toEqual(abc);
  abc = 456;
  expect(abc).toEqual(456);
});
```

Examples of **correct** code for this rule:
```js
test('thing one', () => {
  let abc = 123;

  expect(abc).toEqual(123);
  expect(123).toEqual(abc);

  abc = 456;

  expect(abc).toEqual(456);
});
```
";

fn is_expect_token(name: &str) -> bool {
    matches!(name, "expect" | "expectTypeOf")
}

pub fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };
    let Some(jest_fn_call) = parse_jest_fn_call(call_expr, possible_jest_node, ctx) else {
        return;
    };
    let (ParsedJestFnCallNew::Expect(expect_fn_call)
    | ParsedJestFnCallNew::ExpectTypeOf(expect_fn_call)) = jest_fn_call
    else {
        return;
    };
    check_expect_statement_padding(node, ctx, &expect_fn_call.name, is_expect_token);
}

/// Consecutive statements with the same leading token form a group: padding
/// is required before the first and after the last statement, not in between.
/// The after-check is skipped when the next statement's token is covered by
/// `next_token_skip`, as its own before-check reports the same gap.
pub(super) fn check_expect_statement_padding<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    name: &str,
    next_token_skip: fn(&str) -> bool,
) {
    let Some(statements) = enclosing_statement_list(node, ctx) else {
        return;
    };
    let Some(index) = enclosing_statement_index(node.span(), statements) else {
        return;
    };
    let statement = &statements[index];
    // The call must lead its statement; this also drops nested parses such as
    // `expect.anything()` inside matcher arguments, which would double-report.
    let Some(token) = leading_token_of_statement(statement) else {
        return;
    };
    if token.expr_start != node.span().start {
        return;
    }

    if let Some(prev) = index.checked_sub(1).map(|i| &statements[i]) {
        let same_group = leading_token_of_statement(prev).is_some_and(|t| t.name == name);
        if !same_group {
            check_padding_between(
                ctx,
                prev.span().end,
                statement.span().start,
                PaddingDirection::Before,
                name,
            );
        }
    }

    if let Some(next) = statements.get(index + 1) {
        let skip = leading_token_of_statement(next).is_some_and(|t| next_token_skip(t.name));
        if !skip {
            check_padding_between(
                ctx,
                statement.span().end,
                next.span().start,
                PaddingDirection::After,
                name,
            );
        }
    }
}
