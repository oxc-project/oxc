use oxc_semantic::AstNode;

use crate::{
    context::LintContext,
    utils::{PaddingConfig, PaddingStatement, check_padding},
};

pub const DOCUMENTATION: &str = r"### What it does

This rule enforces a line of padding before and after 1 or more
`afterAll` statements.

### Why is this bad?

Inconsistent formatting of code can make the code more difficult to read
and follow. This rule helps ensure that `afterAll` blocks are visually
separated from the rest of the code, making them easier to identify while
looking through test files.

### Examples

Examples of **incorrect** code for this rule:
```js
const thing = 123;
afterAll(() => {});
describe('foo', () => {});
```

Examples of **correct** code for this rule:
```js
const thing = 123;

afterAll(() => {});

describe('foo', () => {});
```
";

const AFTER_ALL: PaddingStatement = PaddingStatement::Named(&["afterAll"]);

pub const CONFIGS: &[PaddingConfig] = &[
    PaddingConfig::always(PaddingStatement::Any, AFTER_ALL),
    PaddingConfig::always(AFTER_ALL, PaddingStatement::Any),
];

pub fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
    check_padding(node, ctx, &[CONFIGS]);
}
