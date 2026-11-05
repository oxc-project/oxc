use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{
    context::LintContext,
    utils::{PossibleJestNode, parse_expect_jest_fn_call},
};

fn require_to_throw_message_diagnostic(matcher_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Require a message for {matcher_name:?}."))
        .with_help(format!("Add an error message to {matcher_name:?}"))
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

This rule triggers a warning if `toThrow()` or `toThrowError()` is used without an error message.

### Why is this bad?

Using `toThrow()` or `toThrowError()` without specifying an expected error message
makes tests less specific and harder to debug. When a test only checks that an
error was thrown but not what kind of error, it can pass even when the wrong
error is thrown, potentially hiding bugs. Providing an expected error message
or error type makes tests more precise and helps catch regressions more effectively.

### Examples

Examples of **incorrect** code for this rule:
```javascript
test('all the things', async () => {
    expect(() => a()).toThrow();
    expect(() => a()).toThrowError();
    await expect(a()).rejects.toThrow();
    await expect(a()).rejects.toThrowError();
});
```

Examples of **correct** code for this rule:
```javascript
test('all the things', async () => {
  expect(() => a()).toThrow('a');
  expect(() => a()).toThrowError('a');
  await expect(a()).rejects.toThrow('a');
  await expect(a()).rejects.toThrowError('a');
});
```
";

pub fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };

    let Some(possible_jest_node) = PossibleJestNode::from_call_expression(node, call_expr, ctx)
    else {
        return;
    };
    let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, &possible_jest_node, ctx) else {
        return;
    };

    let Some(matcher) = jest_fn_call.matcher() else {
        return;
    };

    let Some(matcher_name) = matcher.name() else {
        return;
    };

    let has_not = jest_fn_call.modifiers().any(|modifier| modifier.is_name_equal("not"));

    if jest_fn_call.args.is_empty()
        && (matcher_name == "toThrow" || matcher_name == "toThrowError")
        && !has_not
    {
        ctx.diagnostic(require_to_throw_message_diagnostic(&matcher_name, matcher.span));
    }
}
