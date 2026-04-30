use oxc_ast::AstKind;

use crate::{
    context::LintContext,
    utils::{
        JestGeneralFnKind, ParsedGeneralJestFnCall, PossibleJestNode, parse_general_jest_fn_call,
        report_missing_padding_after_jest_block, report_missing_padding_before_jest_block,
    },
};

pub const DOCUMENTATION: &str = r"### What it does
    
This rule enforces a line of padding before and after 1 or more
`afterEach` statements.

### Why is this bad?

Inconsistent formatting of code can make the code more difficult to read
and follow. This rule helps ensure that `afterEach` blocks are visually
separated from the rest of the code, making them easier to identify while
looking through test files.

### Examples

Examples of **incorrect** code for this rule:
```js
const thing = 123;
afterEach(() => {});
const other = 456;
```

Examples of **correct** code for this rule:
```js
const thing = 123;

afterEach(() => {});

const other = 456;
";

pub fn run<'a, 'c>(jest_node: &PossibleJestNode<'a, 'c>, ctx: &'c LintContext<'a>) {
    let node = jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };
    let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, jest_node, ctx) else {
        return;
    };
    let ParsedGeneralJestFnCall { kind, name, .. } = &jest_fn_call;
    let Some(kind) = kind.to_general() else {
        return;
    };
    if kind != JestGeneralFnKind::Hook {
        return;
    }
    if name != "afterEach" {
        return;
    }
    report_missing_padding_before_jest_block(node, ctx, name);
    report_missing_padding_after_jest_block(node, ctx, name);
}
