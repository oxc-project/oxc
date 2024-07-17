use oxc_allocator::Box as OBox;
use oxc_ast::{
    ast::{Expression, FunctionBody, Statement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{ast_util::is_promise, context::LintContext, rule::Rule, AstNode};

fn no_return_in_finally_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "eslint-plugin-promise(no-return-in-finally): Disallow return statements in finally",
    )
    .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct NoReturnInFinally;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow return statements in finally().
    ///
    /// ### Why is this bad?
    ///
    /// Disallow return statements inside a callback passed to finally(), since nothing would
    /// consume what's returned.
    ///
    /// ### Example
    /// ```javascript
    /// myPromise.finally(function (val) {
    ///   return val
    /// })
    /// ```
    NoReturnInFinally,
    correctness,
);

impl Rule for NoReturnInFinally {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_promise(call_expr) {
            return;
        }

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        let Some(prop_name) = member_expr.static_property_name() else {
            return;
        };

        if prop_name != "finally" {
            return;
        }

        for argument in &call_expr.arguments {
            let Some(arg_expr) = argument.as_expression() else {
                continue;
            };
            match arg_expr {
                Expression::ArrowFunctionExpression(arrow_expr) => {
                    find_return_statement(&arrow_expr.body, ctx);
                }
                Expression::FunctionExpression(func_expr) => {
                    let Some(func_body) = &func_expr.body else {
                        continue;
                    };
                    find_return_statement(func_body, ctx);
                }
                _ => continue,
            }
        }
    }
}

fn find_return_statement<'a>(func_body: &OBox<'_, FunctionBody<'a>>, ctx: &LintContext<'a>) {
    let Some(return_stmt) =
        func_body.statements.iter().find(|stmt| matches!(stmt, Statement::ReturnStatement(_)))
    else {
        return;
    };

    let Statement::ReturnStatement(stmt) = return_stmt else {
        return;
    };

    ctx.diagnostic(no_return_in_finally_diagnostic(stmt.span));
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Promise.resolve(1).finally(() => { console.log(2) })",
        "Promise.reject(4).finally(() => { console.log(2) })",
        "Promise.reject(4).finally(() => {})",
        "myPromise.finally(() => {});",
        "Promise.resolve(1).finally(function () { })",
    ];

    let fail = vec![
        "Promise.resolve(1).finally(() => { return 2 })",
        "Promise.reject(0).finally(() => { return 2 })",
        "myPromise.finally(() => { return 2 });",
        "Promise.resolve(1).finally(function () { return 2 })",
    ];

    Tester::new(NoReturnInFinally::NAME, pass, fail).test_and_snapshot();
}
