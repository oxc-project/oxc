use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    utils::{ParsedExpectFnCall, PossibleJestNode, parse_expect_jest_fn_call},
};
use oxc_ast::{AstKind, ast::CallExpression};

fn prefer_to_have_been_called_times_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Prefer `toHaveBeenCalledTimes()` over `toHaveLength()` when asserting mock call counts",
    )
    .with_help(
        "Use `toHaveBeenCalledTimes()` to assert the number of times a mock function was called",
    )
    .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

In order to have a better failure message, [`toHaveBeenCalledTimes` should be used
instead of directly checking the length of `mock.calls`](https://github.com/jest-community/eslint-plugin-jest/blob/v29.5.0/docs/rules/prefer-to-have-been-called-times.md).

### Why is this bad?

This rule triggers a warning if `toHaveLength` is used to assert the number of times a mock is called.

### Examples

Examples of **incorrect** code for this rule:
```js
expect(someFunction.mock.calls).toHaveLength(1);
expect(someFunction.mock.calls).toHaveLength(0);
expect(someFunction.mock.calls).not.toHaveLength(1);
```

Examples of **correct** code for this rule:
```js
expect(someFunction).toHaveBeenCalledTimes(1);
expect(someFunction).toHaveBeenCalledTimes(0);
expect(someFunction).not.toHaveBeenCalledTimes(0);
expect(uncalledFunction).not.toBeCalled();
expect(method.mock.calls[0][0]).toStrictEqual(value);
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
    let Some(parsed_expect_call) = parse_expect_jest_fn_call(call_expr, &possible_jest_node, ctx)
    else {
        return;
    };

    check_and_fix(&parsed_expect_call, call_expr, ctx);
}

fn check_and_fix<'a>(
    parsed_expect_call: &ParsedExpectFnCall<'a>,
    call_expr: &CallExpression<'a>,
    ctx: &LintContext<'a>,
) {
    let Some(matcher) = parsed_expect_call.matcher() else {
        return;
    };

    let is_wanted_matcher = matcher.is_name_equal("toHaveLength");
    if !is_wanted_matcher {
        return;
    }

    if parsed_expect_call.matcher_arguments.is_none_or(|args| args.is_empty()) {
        return;
    }

    let Some(calls_member) = parsed_expect_call
        .expect_arguments
        .and_then(|args| args.first())
        .and_then(|arg| arg.as_expression())
        .and_then(|expr| expr.as_member_expression())
    else {
        return;
    };
    let Some(mock_member) = calls_member.object().as_member_expression() else {
        return;
    };
    if !calls_member.static_property_name().is_some_and(|name| name == "calls")
        || !mock_member.static_property_name().is_some_and(|name| name == "mock")
    {
        return;
    }

    ctx.diagnostic_with_fix(prefer_to_have_been_called_times_diagnostic(call_expr.span), |fixer| {
        let replacement = match fixer.source_range(matcher.span).chars().next() {
            Some('\'') => "'toHaveBeenCalledTimes'",
            Some('"') => "\"toHaveBeenCalledTimes\"",
            Some('`') => "`toHaveBeenCalledTimes`",
            _ => "toHaveBeenCalledTimes",
        };
        let fixer = fixer.for_multifix();
        let mut fix = fixer.new_fix_with_capacity(2);
        fix.push(
            fixer.delete_range(Span::new(mock_member.object().span().end, calls_member.span().end)),
        );
        fix.push(fixer.replace(matcher.span, replacement));
        fix.with_message("Use `toHaveBeenCalledTimes()` to check the mock call count.")
    });
}
