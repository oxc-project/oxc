use oxc_semantic::AstNode;

use crate::{
    context::LintContext,
    utils::{PaddingConfig, PaddingStatement, check_padding},
};

pub const DOCUMENTATION: &str = r"### What it does

This rule enforces a line of padding before and after 1 or more `afterEach`
statements.

It doesn't add or enforce a padding line when the statement is the last one
in its scope.

### Why is this bad?

Inconsistent formatting of code can make the code more difficult to read
and follow. This rule helps ensure that `afterEach` blocks are visually
separated from the rest of the code, making them easier to identify while
looking through test files.

### Examples

Examples of **incorrect** code for this rule:
```js
const something = 123;
afterEach(() => {
  // more stuff
});
describe('foo', () => {});
```

Examples of **correct** code for this rule:
```js
const something = 123;

afterEach(() => {
  // more stuff
});

describe('foo', () => {});
```
";

const AFTER_EACH: PaddingStatement = PaddingStatement::Named(&["afterEach"]);

pub const CONFIGS: &[PaddingConfig] = &[
    PaddingConfig::always(PaddingStatement::Any, AFTER_EACH),
    PaddingConfig::always(AFTER_EACH, PaddingStatement::Any),
];

pub fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
    check_padding(node, ctx, &[CONFIGS]);
}
