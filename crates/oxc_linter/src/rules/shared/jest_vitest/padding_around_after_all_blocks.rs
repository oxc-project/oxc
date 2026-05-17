use oxc_ast::AstKind;

use crate::{
    context::LintContext,
    utils::{
        JestGeneralFnKind, ParsedGeneralJestFnCall, PossibleJestNode, parse_general_jest_fn_call,
        report_missing_padding_before_jest_block,
    },
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
```

Examples of **correct** code for this rule:
```js
const thing = 123;

afterAll(() => {});
";

pub fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };
    let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx) else {
        return;
    };
    let ParsedGeneralJestFnCall { kind, name, .. } = &jest_fn_call;
    let Some(kind) = kind.to_general() else {
        return;
    };
    if kind != JestGeneralFnKind::Hook {
        return;
    }
    if name != "afterAll" {
        return;
    }
    report_missing_padding_before_jest_block(node, ctx, name);
}
