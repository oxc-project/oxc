use oxc_allocator::Box as OBox;
use oxc_ast::{
    ast::{CallExpression, Expression, FunctionBody, Statement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{is_type_of_jest_fn_call, JestFnKind, JestGeneralFnKind, PossibleJestNode},
};

fn no_test_return_statement_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Jest tests should not return a value").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoTestReturnStatement;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow explicitly returning from tests.
    ///
    /// ### Why is this bad?
    ///
    /// Tests in Jest should be void and not return values.
    /// If you are returning Promises then you should update the test to use
    /// `async/await`.
    ///
    /// ### Example
    /// ```javascript
    /// test('one', () => {
    ///    return expect(1).toBe(1);
    /// });
    /// ```
    NoTestReturnStatement,
    jest,
    style,
);

impl Rule for NoTestReturnStatement {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
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
        match arg_expr {
            Expression::ArrowFunctionExpression(arrow_expr) => {
                check_test_return_statement(&arrow_expr.body, ctx);
            }
            Expression::FunctionExpression(func_expr) => {
                let Some(func_body) = &func_expr.body else {
                    continue;
                };
                check_test_return_statement(func_body, ctx);
            }
            _ => continue,
        }
    }
}

fn check_test_return_statement<'a>(func_body: &OBox<'_, FunctionBody<'a>>, ctx: &LintContext<'a>) {
    let Some(return_stmt) =
        func_body.statements.iter().find(|stmt| matches!(stmt, Statement::ReturnStatement(_)))
    else {
        return;
    };

    let Statement::ReturnStatement(stmt) = return_stmt else {
        return;
    };
    let Some(Expression::CallExpression(call_expr)) = &stmt.argument else {
        return;
    };
    let Some(mem_expr) = call_expr.callee.as_member_expression() else {
        return;
    };
    let Expression::CallExpression(mem_call_expr) = mem_expr.object() else {
        return;
    };
    let Expression::Identifier(ident) = &mem_call_expr.callee else {
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

#[test]
fn test() {
    use crate::tester::Tester;

    // Note: Both Jest and Vitest share the same unit tests

    let pass = vec![
        ("it('noop', () => {});", None),
        ("test('noop', () => {});", None),
        ("test('one', () => expect(1).toBe(1));", None),
        ("test('empty')", None),
        (
            "
                test('one', () => {
                    expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it('one', function () {
                    expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it('one', myTest);
                function myTest() {
                    expect(1).toBe(1);
                }
            ",
            None,
        ),
        (
            "
                it('one', () => expect(1).toBe(1));
                function myHelper() {}
            ",
            None,
        ),
    ];

    let fail = vec![
        (
            "
                test('one', () => {
                   return expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it('one', function () {
                    return expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it.skip('one', function () {
                    return expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it.each``('one', function () {
                    return expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it.each()('one', function () {
                    return expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it.only.each``('one', function () {
                    return expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it.only.each()('one', function () {
                    return expect(1).toBe(1);
                });
            ",
            None,
        ),
        (
            "
                it('one', myTest);
                function myTest () {
                    return expect(1).toBe(1);
                }
            ",
            None,
        ),
    ];

    Tester::new(NoTestReturnStatement::NAME, NoTestReturnStatement::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
