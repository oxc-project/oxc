use oxc_allocator::ArenaBox;
use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, ExpressionKind, FunctionBody, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
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
    match node.kind() {
        AstKind::CallExpression(call_expr) => {
            check_call_expression(call_expr, node, ctx);
        }
        AstKind::Function(fn_decl) => {
            let Some(func_body) = &fn_decl.body else {
                return;
            };
            check_test_return_statement(func_body, ctx);
        }
        _ => (),
    }
}

fn check_call_expression<'a>(
    call_expr: &'a CallExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) {
    if !is_type_of_jest_fn_call(
        call_expr,
        &PossibleJestNode { node, original: None },
        ctx,
        &[JestFnKind::General(JestGeneralFnKind::Test)],
    ) {
        return;
    }

    for argument in &call_expr.arguments {
        let Some(arg_expr) = argument.as_expression() else {
            continue;
        };
        match arg_expr.kind() {
            ExpressionKind::ArrowFunctionExpression(arrow_expr) => {
                check_test_return_statement(&arrow_expr.body, ctx);
            }
            ExpressionKind::FunctionExpression(func_expr) => {
                let Some(func_body) = &func_expr.body else {
                    continue;
                };
                check_test_return_statement(func_body, ctx);
            }
            _ => {}
        }
    }
}

fn check_test_return_statement<'a>(
    func_body: &ArenaBox<'_, FunctionBody<'a>>,
    ctx: &LintContext<'a>,
) {
    let Some(return_stmt) =
        func_body.statements.iter().find(|stmt| matches!(stmt, Statement::ReturnStatement(_)))
    else {
        return;
    };

    let Statement::ReturnStatement(stmt) = return_stmt else {
        return;
    };
    let Some(call_expr) = stmt.argument.as_ref().and_then(Expression::as_call_expression) else {
        return;
    };
    let Some(mem_expr) = call_expr.callee.as_member_expression() else {
        return;
    };
    let ExpressionKind::CallExpression(mem_call_expr) = mem_expr.object().kind() else {
        return;
    };
    let ExpressionKind::Identifier(ident) = mem_call_expr.callee.kind() else {
        return;
    };

    if ident.name != "expect" {
        return;
    }

    ctx.diagnostic(no_test_return_statement_diagnostic(Span::new(
        return_stmt.span().start,
        call_expr.span.start - 1,
    )));
}
