use oxc_semantic::AstNode;

use super::{
    padding_around_after_all_blocks, padding_around_after_each_blocks,
    padding_around_before_all_blocks, padding_around_before_each_blocks,
    padding_around_describe_blocks, padding_around_expect_groups, padding_around_test_blocks,
};
use crate::{
    context::LintContext,
    utils::{PaddingConfig, check_padding},
};

pub const DOCUMENTATION: &str = r"### What it does

This rule enforces padding around test framework functions. It combines
the behavior of these rules:

- `padding-around-after-all-blocks`
- `padding-around-after-each-blocks`
- `padding-around-before-all-blocks`
- `padding-around-before-each-blocks`
- `padding-around-describe-blocks`
- `padding-around-expect-groups`
- `padding-around-test-blocks`

### Why is this bad?

Inconsistent formatting of code can make the code more difficult to read
and follow. This rule helps ensure that hooks, test blocks, and assertion
groups are visually separated from the rest of the code, making them
easier to identify while looking through test files.

### Examples

Examples of **incorrect** code for this rule:
```js
const thing = 123;
beforeEach(() => {});
test('foo', () => {
  const value = 1;
  expect(value).toBe(1);
});
```

Examples of **correct** code for this rule:
```js
const thing = 123;

beforeEach(() => {});

test('foo', () => {
  const value = 1;

  expect(value).toBe(1);
});
```
";

pub const JEST_CONFIGS: &[&[PaddingConfig]] = &[
    padding_around_after_all_blocks::CONFIGS,
    padding_around_after_each_blocks::CONFIGS,
    padding_around_before_all_blocks::CONFIGS,
    padding_around_before_each_blocks::CONFIGS,
    padding_around_describe_blocks::CONFIGS,
    padding_around_expect_groups::EXPECT_CONFIGS,
    padding_around_test_blocks::CONFIGS,
];

pub const VITEST_CONFIGS: &[&[PaddingConfig]] = &[
    padding_around_after_all_blocks::CONFIGS,
    padding_around_after_each_blocks::CONFIGS,
    padding_around_before_all_blocks::CONFIGS,
    padding_around_before_each_blocks::CONFIGS,
    padding_around_describe_blocks::CONFIGS,
    padding_around_expect_groups::EXPECT_CONFIGS,
    padding_around_expect_groups::EXPECT_TYPE_OF_CONFIGS,
    padding_around_test_blocks::CONFIGS,
];

pub fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>, configs: &[&[PaddingConfig]]) {
    check_padding(node, ctx, configs);
}
