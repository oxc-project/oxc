use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::{
    context::LintContext,
    utils::{PossibleJestNode, parse_expect_jest_fn_call},
};

fn use_to_strict_equal(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using `toStrictEqual()`.")
        .with_help("Use `toStrictEqual()` instead")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

This rule triggers a warning if `toEqual()` is used to assert equality.

### Why is this bad?

The `toEqual()` matcher performs a deep equality check but ignores
`undefined` values in objects and arrays. This can lead to false
positives where tests pass when they should fail. `toStrictEqual()`
provides more accurate comparison by checking for `undefined` values.

### Examples

Examples of **incorrect** code for this rule:
```javascript
expect({ a: 'a', b: undefined }).toEqual({ a: 'a' });
```

Examples of **correct** code for this rule:
```javascript
expect({ a: 'a', b: undefined }).toStrictEqual({ a: 'a' });
```
";

pub fn run_on_jest_node<'a, 'c>(jest_node: &PossibleJestNode<'a, 'c>, ctx: &'c LintContext<'a>) {
    run(jest_node, ctx);
}

fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) -> Option<()> {
    let call_expr = possible_jest_node.node.kind().as_call_expression()?;
    let parse_jest_expect_fn_call = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)?;
    let matcher = parse_jest_expect_fn_call.matcher()?;
    let matcher_name = matcher.name()?;

    if matcher_name.eq("toEqual") {
        ctx.diagnostic_with_fix(use_to_strict_equal(matcher.span), |fixer| {
            let replacement = match fixer.source_range(matcher.span).chars().next().unwrap() {
                '\'' => "'toStrictEqual'",
                '"' => "\"toStrictEqual\"",
                '`' => "`toStrictEqual`",
                _ => "toStrictEqual",
            };
            fixer.replace(matcher.span, replacement)
        });
    }
    None
}
