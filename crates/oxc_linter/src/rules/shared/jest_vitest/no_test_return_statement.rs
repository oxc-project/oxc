use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, ReturnStatement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::{AstNode, NodeId};
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call},
};

fn no_test_return_statement_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Jest tests should not return a value")
        .with_help("Use `await` for async assertions or remove the return statement.")
        .with_note("Jest ignores returned values from tests.")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

Disallow explicitly returning from tests.

### Why is this bad?

Tests in Jest should be void and not return values.
If you are returning Promises then you should update the test to use
`async/await`.

### Examples

Examples of **incorrect** code for this rule:
```javascript
test('one', () => {
   return expect(1).toBe(1);
});
```

Examples of **correct** code for this rule:
```javascript
test('one', () => {
   expect(1).toBe(1);
});
```
";

pub fn run<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
    let AstKind::ReturnStatement(return_stmt) = node.kind() else { return };
    let Some(call_expr) = returned_expect_call(return_stmt) else { return };

    if !is_in_test_context(node, ctx) {
        return;
    }

    ctx.diagnostic(no_test_return_statement_diagnostic(Span::new(
        return_stmt.span.start,
        call_expr.span.start - 1,
    )));
}

fn returned_expect_call<'a>(
    return_stmt: &'a ReturnStatement<'a>,
) -> Option<&'a CallExpression<'a>> {
    let Expression::CallExpression(call_expr) = return_stmt.argument.as_ref()? else {
        return None;
    };
    let mem_expr = call_expr.callee.as_member_expression()?;
    let Expression::CallExpression(mem_call_expr) = mem_expr.object() else {
        return None;
    };
    let Expression::Identifier(ident) = &mem_call_expr.callee else {
        return None;
    };

    if ident.name != "expect" {
        return None;
    }

    Some(call_expr)
}

fn is_in_test_context<'a>(return_node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let Some(function_node) = containing_function_node(return_node, ctx) else { return false };

    is_direct_test_callback(function_node, ctx) || is_referenced_test_callback(function_node, ctx)
}

fn containing_function_node<'a, 'c>(
    return_node: &AstNode<'a>,
    ctx: &'c LintContext<'a>,
) -> Option<&'c AstNode<'a>> {
    ctx.nodes().ancestors(return_node.id()).find(|ancestor| {
        matches!(ancestor.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_))
    })
}

fn is_direct_test_callback<'a>(function_node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let call_node = ctx.nodes().parent_node(function_node.id());
    let AstKind::CallExpression(call_expr) = call_node.kind() else { return false };

    is_function_callback_argument(call_expr, function_node)
        && is_test_call(call_expr, call_node, ctx)
}

fn is_referenced_test_callback<'a>(function_node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let AstKind::Function(function) = function_node.kind() else { return false };
    if !function.is_function_declaration() {
        return false;
    }

    let Some(symbol_id) = function.id.as_ref().map(oxc_ast::ast::BindingIdentifier::symbol_id)
    else {
        return false;
    };

    ctx.semantic()
        .symbol_references(symbol_id)
        .any(|reference| is_reference_test_callback(reference.node_id(), ctx))
}

fn is_reference_test_callback(reference_id: NodeId, ctx: &LintContext) -> bool {
    let call_node = ctx.nodes().parent_node(reference_id);
    let AstKind::CallExpression(call_expr) = call_node.kind() else { return false };

    is_reference_callback_argument(call_expr, ctx.nodes().kind(reference_id).span())
        && is_test_call(call_expr, call_node, ctx)
}

fn is_function_callback_argument(call_expr: &CallExpression, function_node: &AstNode) -> bool {
    call_expr.arguments.iter().filter_map(|arg| arg.as_expression()).any(|expr| {
        match (expr.get_inner_expression(), function_node.kind()) {
            (
                Expression::ArrowFunctionExpression(arrow_expr),
                AstKind::ArrowFunctionExpression(node),
            ) => arrow_expr.span == node.span,
            (Expression::FunctionExpression(func_expr), AstKind::Function(node)) => {
                func_expr.span == node.span
            }
            _ => false,
        }
    })
}

fn is_reference_callback_argument(call_expr: &CallExpression, reference_span: Span) -> bool {
    call_expr.arguments.iter().filter_map(|arg| arg.as_expression()).any(|expr| {
        matches!(expr.get_inner_expression(), Expression::Identifier(ident) if ident.span == reference_span)
    })
}

fn is_test_call<'a>(
    call_expr: &'a CallExpression<'a>,
    call_node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    is_type_of_jest_fn_call(
        call_expr,
        &PossibleJestNode { node: call_node, original: None },
        ctx,
        &[JestFnKind::General(JestGeneralFnKind::Test)],
    )
}
