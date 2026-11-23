use oxc_semantic::AstNode;

use crate::{
    context::LintContext,
    utils::{PaddingConfig, PaddingStatement, check_padding},
};

pub const DOCUMENTATION: &str = r"### What it does

This rule enforces a line of padding before and after 1 or more
`test`/`it` statements.

### Why is this bad?

Inconsistent formatting of code can make the code more difficult to read
and follow. This rule helps ensure that test blocks are visually
separated from the rest of the code, making them easier to identify while
looking through test files.

### Examples

Examples of **incorrect** code for this rule:
```js
const thing = 123;
test('foo', () => {});
test('bar', () => {});
```

```js
const thing = 123;
it('foo', () => {});
it('bar', () => {});
```

Examples of **correct** code for this rule:
```js
const thing = 123;

test('foo', () => {});

test('bar', () => {});
```

```js
const thing = 123;

it('foo', () => {});

it('bar', () => {});
```
";

const TEST_BLOCKS: PaddingStatement =
    PaddingStatement::Named(&["test", "it", "fit", "xit", "xtest"]);

pub const CONFIGS: &[PaddingConfig] = &[
    PaddingConfig::always(PaddingStatement::Any, TEST_BLOCKS),
    PaddingConfig::always(TEST_BLOCKS, PaddingStatement::Any),
];

pub fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
    check_padding(node, ctx, &[CONFIGS]);
}
