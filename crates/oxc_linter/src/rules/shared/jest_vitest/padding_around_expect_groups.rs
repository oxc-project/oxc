use oxc_semantic::AstNode;

use crate::{
    context::LintContext,
    utils::{PaddingConfig, PaddingStatement, check_padding},
};

pub const DOCUMENTATION: &str = r"### What it does

This rule enforces a line of padding before and after 1 or more `expect`
statements.

It doesn't add or enforce a padding line when the statement is the last one
in its scope, or between two or more adjacent `expect` statements.

The Vitest version applies the same grouping to `expectTypeOf` statements.
Adjacent `expect` and `expectTypeOf` statements still need a padding line.

### Why is this bad?

Inconsistent formatting of code can make the code more difficult to read
and follow. This rule helps ensure that groups of assertions are visually
separated from the rest of the code, making them easier to identify while
looking through test files.

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

const EXPECT: PaddingStatement = PaddingStatement::Named(&["expect"]);
const EXPECT_TYPE_OF: PaddingStatement = PaddingStatement::Named(&["expectTypeOf"]);

pub const EXPECT_CONFIGS: &[PaddingConfig] = &[
    PaddingConfig::always(PaddingStatement::Any, EXPECT),
    PaddingConfig::always(EXPECT, PaddingStatement::Any),
    PaddingConfig::any(EXPECT, EXPECT),
];

pub const EXPECT_TYPE_OF_CONFIGS: &[PaddingConfig] = &[
    PaddingConfig::always(PaddingStatement::Any, EXPECT_TYPE_OF),
    PaddingConfig::always(EXPECT_TYPE_OF, PaddingStatement::Any),
    PaddingConfig::any(EXPECT_TYPE_OF, EXPECT_TYPE_OF),
];

pub const JEST_CONFIGS: &[&[PaddingConfig]] = &[EXPECT_CONFIGS];

pub const VITEST_CONFIGS: &[&[PaddingConfig]] = &[EXPECT_CONFIGS, EXPECT_TYPE_OF_CONFIGS];

pub fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>, configs: &[&[PaddingConfig]]) {
    check_padding(node, ctx, configs);
}
