use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{ParsedJestFnCallNew, PossibleJestNode, parse_jest_fn_call},
};

fn no_unneeded_async_expect_function_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unnecessary async function wrapper")
        .with_help("Remove the async wrapper and pass the promise directly to expect")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnneededAsyncExpectFunction;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows unnecessary async function wrapper for expected promises.
    ///
    /// ### Why is this bad?
    ///
    /// When the only statement inside an async wrapper is `await someCall()`,
    /// the call should be passed directly to `expect` instead. This makes the
    /// test code more concise and easier to read.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// await expect(async () => {
    ///   await doSomethingAsync();
    /// }).rejects.toThrow();
    ///
    /// await expect(async () => await doSomethingAsync()).rejects.toThrow();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// await expect(doSomethingAsync()).rejects.toThrow();
    /// ```
    NoUnneededAsyncExpectFunction,
    vitest,
    style,
    fix
);

impl Rule for NoUnneededAsyncExpectFunction {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
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
        let Some(Expression::CallExpression(expect_call_expr)) = parsed_expect_call.head.parent
        else {
            return;
        };

        // Get the first argument of expect()
        let Some(first_arg) = expect_call_expr.arguments.first() else {
            return;
        };

        // Check if it's an async function expression and get the inner call span
        let (func_span, inner_call_span) = match first_arg {
            Argument::ArrowFunctionExpression(arrow) => {
                if !arrow.r#async {
                    return;
                }
                let Some(inner_span) = get_awaited_call_span_from_arrow(arrow) else {
                    return;
                };
                (arrow.span, inner_span)
            }
            Argument::FunctionExpression(func) => {
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
        };

        ctx.diagnostic_with_fix(no_unneeded_async_expect_function_diagnostic(func_span), |fixer| {
            let inner_call_text = fixer.source_range(inner_call_span).to_string();
            fixer.replace(func_span, inner_call_text)
        });
    }
}

/// Get the span of the awaited call expression from an async arrow function
fn get_awaited_call_span_from_arrow(arrow: &oxc_ast::ast::ArrowFunctionExpression) -> Option<Span> {
    // Case 1: Arrow function with expression body (async () => await doSomething())
    if arrow.expression {
        // When arrow.expression is true, body has exactly one ExpressionStatement
        let stmt = arrow.body.statements.first()?;
        let Statement::ExpressionStatement(expr_stmt) = stmt else {
            return None;
        };
        let Expression::AwaitExpression(await_expr) = &expr_stmt.expression else {
            return None;
        };
        if let Expression::CallExpression(call) = &await_expr.argument {
            return Some(call.span);
        }
        return None;
    }

    // Case 2: Arrow function with block body
    get_awaited_call_span_from_block(&arrow.body)
}

fn get_awaited_call_span_from_block(body: &oxc_ast::ast::FunctionBody) -> Option<Span> {
    // Must have exactly one statement
    if body.statements.len() != 1 {
        return None;
    }

    let stmt = body.statements.first()?;

    // Must be an expression statement
    let Statement::ExpressionStatement(expr_stmt) = stmt else {
        return None;
    };

    // Must be an await expression
    let Expression::AwaitExpression(await_expr) = &expr_stmt.expression else {
        return None;
    };

    // The awaited value must be a call expression
    if let Expression::CallExpression(call) = &await_expr.argument {
        return Some(call.span);
    }

    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Simple cases that should pass
        "expect.hasAssertions()",
        // Direct promise to expect (correct usage)
        "await expect(doSomethingAsync()).rejects.toThrow();",
        "await expect(doSomethingAsync()).resolves.toBe(1);",
        // Multiple statements in async wrapper (can't simplify)
        "await expect(async () => { await a(); await b(); }).rejects.toThrow();",
        // No await in wrapper (can't simplify)
        "await expect(async () => { doSync(); }).rejects.toThrow();",
        // Non-async wrapper
        "await expect(() => { throw new Error(); }).rejects.toThrow();",
        // Variable declaration before await
        "await expect(async () => { const a = 1; await fn(a); }).rejects.toThrow();",
        // Await in for-loop (multiple awaits)
        "await expect(async () => { for (const a of b) { await a(); } }).rejects.toThrow();",
        // Await in array (not a simple call expression)
        "await expect(async () => [await a()]).rejects.toThrow();",
        // Return statement (not a simple expression statement)
        "await expect(async () => { return await a(); }).rejects.toThrow();",
    ];

    let fail = vec![
        // Block body with single await
        "await expect(async () => { await doSomethingAsync(); }).rejects.toThrow();",
        // Expression body with await
        "await expect(async () => await doSomethingAsync()).rejects.toThrow();",
        // Function expression
        "await expect(async function() { await doSomethingAsync(); }).rejects.toThrow();",
        // With function arguments
        "await expect(async () => await doSomethingAsync(1, 2)).rejects.toThrow();",
        // With resolves matcher
        "await expect(async () => { await doSomethingAsync(); }).resolves.toBe(1);",
        // Promise.all wrapped
        "await expect(async () => await Promise.all([a(), b()])).rejects.toThrow();",
    ];

    let fix = vec![
        (
            "await expect(async () => { await doSomethingAsync(); }).rejects.toThrow();",
            "await expect(doSomethingAsync()).rejects.toThrow();",
            None,
        ),
        (
            "await expect(async () => await doSomethingAsync()).rejects.toThrow();",
            "await expect(doSomethingAsync()).rejects.toThrow();",
            None,
        ),
        (
            "await expect(async function() { await doSomethingAsync(); }).rejects.toThrow();",
            "await expect(doSomethingAsync()).rejects.toThrow();",
            None,
        ),
        (
            "await expect(async () => await doSomethingAsync(1, 2)).rejects.toThrow();",
            "await expect(doSomethingAsync(1, 2)).rejects.toThrow();",
            None,
        ),
        (
            "await expect(async () => { await doSomethingAsync(); }).resolves.toBe(1);",
            "await expect(doSomethingAsync()).resolves.toBe(1);",
            None,
        ),
        (
            "await expect(async () => await Promise.all([a(), b()])).rejects.toThrow();",
            "await expect(Promise.all([a(), b()])).rejects.toThrow();",
            None,
        ),
    ];
    Tester::new(
        NoUnneededAsyncExpectFunction::NAME,
        NoUnneededAsyncExpectFunction::PLUGIN,
        pass,
        fail,
    )
    .with_vitest_plugin(true)
    .expect_fix(fix)
    .test_and_snapshot();
}
