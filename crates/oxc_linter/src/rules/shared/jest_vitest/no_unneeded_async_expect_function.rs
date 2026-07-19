use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, ExpressionKind, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::{
    context::LintContext,
    utils::{ParsedJestFnCallNew, PossibleJestNode, parse_jest_fn_call},
};

fn no_unneeded_async_expect_function_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unnecessary async function wrapper")
        .with_help("Remove the async wrapper and pass the promise directly to expect")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

Disallows unnecessary async function wrapper for expected promises.

### Why is this bad?

When the only statement inside an async wrapper is `await someCall()`,
the call should be passed directly to `expect` instead. This makes the
test code more concise and easier to read.

### Examples

Examples of **incorrect** code for this rule:
```js
await expect(async () => {
  await doSomethingAsync();
}).rejects.toThrow();

await expect(async () => await doSomethingAsync()).rejects.toThrow();
```

Examples of **correct** code for this rule:
```js
await expect(doSomethingAsync()).rejects.toThrow();
```
";

pub fn run_on_jest_node<'a, 'c>(jest_node: &PossibleJestNode<'a, 'c>, ctx: &'c LintContext<'a>) {
    let node = jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };

    let Some(ParsedJestFnCallNew::Expect(parsed_expect_call)) =
        parse_jest_fn_call(call_expr, jest_node, ctx)
    else {
        return;
    };

    // Ensure we have a valid matcher (this ensures we're processing the full chain)
    if parsed_expect_call.matcher().is_none() {
        return;
    }

    // Get the expect() CallExpression from head.parent
    let Some(expect_call_expr) =
        parsed_expect_call.head.parent.and_then(Expression::as_call_expression)
    else {
        return;
    };

    // Get the first argument of expect()
    let Some(first_arg) = expect_call_expr.arguments.first() else {
        return;
    };

    // Check if it's an async function expression and get the inner call span
    let (func_span, inner_call_span) = match first_arg {
        Argument::Expression(expr) => match expr.kind() {
            ExpressionKind::ArrowFunctionExpression(arrow) => {
                if !arrow.r#async {
                    return;
                }
                let Some(inner_span) = get_awaited_call_span_from_arrow(arrow) else {
                    return;
                };
                (arrow.span, inner_span)
            }
            ExpressionKind::FunctionExpression(func) => {
                if !func.r#async {
                    return;
                }
                let Some(body) = &func.body else {
                    return;
                };
                let Some(inner_span) = get_awaited_call_span_from_block(body) else {
                    return;
                };
                (func.span, inner_span)
            }
            _ => return,
        },
        Argument::SpreadElement(_) => return,
    };

    ctx.diagnostic_with_fix(no_unneeded_async_expect_function_diagnostic(func_span), |fixer| {
        fixer.replace(func_span, fixer.source_range(inner_call_span).to_string())
    });
}

/// Gets the span of the awaited call expression from an async arrow function.
/// Returns `None` if the function body doesn't contain exactly one await of a call expression.
fn get_awaited_call_span_from_arrow(arrow: &oxc_ast::ast::ArrowFunctionExpression) -> Option<Span> {
    // Case 1: Arrow function with expression body (async () => await doSomething())
    if arrow.expression {
        if let Some(first) = arrow.body.statements.first()
            && let Statement::ExpressionStatement(expr_stmt) = first
            && let ExpressionKind::AwaitExpression(await_expr) = expr_stmt.expression.kind()
            && let ExpressionKind::CallExpression(call) = await_expr.argument.kind()
        {
            return Some(call.span);
        }
        return None;
    }

    // Case 2: Arrow function with block body
    get_awaited_call_span_from_block(&arrow.body)
}

fn get_awaited_call_span_from_block(body: &oxc_ast::ast::FunctionBody) -> Option<Span> {
    if body.statements.len() == 1
        && let Some(stmt) = body.statements.first()
        && let Statement::ExpressionStatement(expr_stmt) = stmt
        && let ExpressionKind::AwaitExpression(await_expr) = expr_stmt.expression.kind()
        && let ExpressionKind::CallExpression(call) = await_expr.argument.kind()
    {
        return Some(call.span);
    }

    None
}
