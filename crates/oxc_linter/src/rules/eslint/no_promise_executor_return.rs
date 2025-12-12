use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, FunctionBody},
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::UnaryOperator;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_promise_executor_return_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Return statement should not be used in Promise executor.")
        .with_help("Use `resolve()` or `reject()` instead of returning a value.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoPromiseExecutorReturn(Box<NoPromiseExecutorReturnConfig>);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoPromiseExecutorReturnConfig {
    /// If `true`, allows returning `void` expressions (e.g., `return void resolve()`).
    allow_void: bool,
}

impl std::ops::Deref for NoPromiseExecutorReturn {
    type Target = NoPromiseExecutorReturnConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow returning values from Promise executor functions.
    ///
    /// ### Why is this bad?
    ///
    /// The `new Promise` constructor accepts an executor function as an argument,
    /// which has `resolve` and `reject` parameters that can be used to control the
    /// state of the created Promise.
    ///
    /// The return value of the executor is ignored. Returning a value from an executor
    /// function is a possible error because the returned value cannot be used and it
    /// doesn't affect the promise in any way.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// new Promise((resolve, reject) => {
    ///     if (someCondition) {
    ///         return defaultResult;
    ///     }
    ///     getSomething((err, result) => {
    ///         if (err) {
    ///             reject(err);
    ///         } else {
    ///             resolve(result);
    ///         }
    ///     });
    /// });
    ///
    /// new Promise((resolve, reject) => getSomething((err, data) => {
    ///     if (err) {
    ///         reject(err);
    ///     } else {
    ///         resolve(data);
    ///     }
    /// }));
    ///
    /// new Promise(() => {
    ///     return 1;
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// new Promise((resolve, reject) => {
    ///     if (someCondition) {
    ///         resolve(defaultResult);
    ///         return;
    ///     }
    ///     getSomething((err, result) => {
    ///         if (err) {
    ///             reject(err);
    ///         } else {
    ///             resolve(result);
    ///         }
    ///     });
    /// });
    ///
    /// new Promise((resolve, reject) => {
    ///     getSomething((err, data) => {
    ///         if (err) {
    ///             reject(err);
    ///         } else {
    ///             resolve(data);
    ///         }
    ///     });
    /// });
    ///
    /// new Promise(r => { r(1) });
    /// ```
    NoPromiseExecutorReturn,
    eslint,
    pedantic,
    config = NoPromiseExecutorReturnConfig,
);

impl Rule for NoPromiseExecutorReturn {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoPromiseExecutorReturn>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };

        if !new_expr.callee.is_specific_id("Promise") {
            return;
        }

        let Some(first_arg) = new_expr.arguments.first().and_then(Argument::as_expression) else {
            return;
        };

        let inner_expr = first_arg.get_inner_expression();

        match inner_expr {
            Expression::ArrowFunctionExpression(arrow) => {
                // Check for implicit return (expression body without braces)
                if let Some(expr) = arrow.get_expression() {
                    // Arrow function with expression body: `new Promise(r => r(1))`
                    // This is an implicit return, report it unless allowVoid and it's a void expression
                    if self.allow_void
                        && let Expression::UnaryExpression(unary) = expr.get_inner_expression()
                        && unary.operator == UnaryOperator::Void
                    {
                        return;
                    }
                    ctx.diagnostic(no_promise_executor_return_diagnostic(arrow.body.span));
                } else {
                    // Arrow function with block body: check for return statements
                    self.check_function_body(&arrow.body, ctx);
                }
            }
            Expression::FunctionExpression(func) => {
                if let Some(body) = &func.body {
                    self.check_function_body(body, ctx);
                }
            }
            _ => {}
        }
    }
}

impl NoPromiseExecutorReturn {
    fn check_function_body(&self, body: &FunctionBody, ctx: &LintContext) {
        let mut finder = ReturnStatementFinder::new(self.allow_void);
        finder.visit_function_body(body);

        for span in finder.return_spans {
            ctx.diagnostic(no_promise_executor_return_diagnostic(span));
        }
    }
}

struct ReturnStatementFinder {
    return_spans: Vec<Span>,
    allow_void: bool,
}

impl ReturnStatementFinder {
    fn new(allow_void: bool) -> Self {
        Self { return_spans: Vec::new(), allow_void }
    }
}

impl Visit<'_> for ReturnStatementFinder {
    fn visit_return_statement(&mut self, it: &oxc_ast::ast::ReturnStatement<'_>) {
        // Empty return is allowed
        let Some(argument) = &it.argument else {
            return;
        };

        // Check for void expression if allowVoid is true
        if self.allow_void
            && let Expression::UnaryExpression(unary) = argument.get_inner_expression()
            && unary.operator == UnaryOperator::Void
        {
            return;
        }

        self.return_spans.push(it.span);
    }

    fn visit_function(
        &mut self,
        _it: &oxc_ast::ast::Function<'_>,
        _flags: oxc_semantic::ScopeFlags,
    ) {
        // Skip visiting nested functions - they have their own scope
    }

    fn visit_arrow_function_expression(&mut self, _it: &oxc_ast::ast::ArrowFunctionExpression<'_>) {
        // Skip visiting nested arrow functions - they have their own scope
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Empty return is allowed
        ("new Promise((resolve, reject) => { return; })", None),
        ("new Promise(function (resolve, reject) { return; })", None),
        // No return at all
        ("new Promise((resolve, reject) => { resolve(1); })", None),
        ("new Promise(function (resolve, reject) { resolve(1); })", None),
        // Arrow with block body, no return
        ("new Promise(r => { r(1) })", None),
        // Not a Promise
        ("new Foo((resolve, reject) => { return 1; })", None),
        ("new Foo(r => r(1))", None),
        // Nested function with return is ok
        ("new Promise((resolve) => { function inner() { return 1; } inner(); resolve(); })", None),
        ("new Promise((resolve) => { const inner = () => 1; resolve(inner()); })", None),
        // void is allowed with allowVoid option (explicit return)
        (
            "new Promise((resolve) => { return void resolve(1); })",
            Some(serde_json::json!([{ "allowVoid": true }])),
        ),
        (
            "new Promise((resolve) => { return void 0; })",
            Some(serde_json::json!([{ "allowVoid": true }])),
        ),
        // void is allowed with allowVoid option (implicit return in arrow expression body)
        ("new Promise(r => void r(1))", Some(serde_json::json!([{ "allowVoid": true }]))),
    ];

    let fail = vec![
        // Arrow with expression body (implicit return)
        ("new Promise(r => r(1))", None),
        ("new Promise((resolve, reject) => resolve(1))", None),
        // Explicit return with value
        ("new Promise((resolve, reject) => { return 1; })", None),
        ("new Promise(function (resolve, reject) { return 1; })", None),
        // Return in control flow
        ("new Promise((resolve, reject) => { if (true) { return 1; } })", None),
        ("new Promise((resolve, reject) => { if (true) return 1; })", None),
        // Return variable
        ("new Promise((resolve, reject) => { return resolve; })", None),
        // Return function call result (not resolve/reject call itself)
        ("new Promise((resolve, reject) => { return foo(); })", None),
        // Return resolve/reject call result - these should NOT be special-cased as valid
        ("new Promise((resolve, reject) => { return resolve(1); })", None),
        ("new Promise((resolve, reject) => { return reject(new Error('fail')); })", None),
        // Nested blocks
        ("new Promise((resolve, reject) => { { { return 1; } } })", None),
        // In try-catch
        ("new Promise((resolve, reject) => { try { return 1; } catch(e) {} })", None),
        // void is not allowed by default
        ("new Promise((resolve) => { return void resolve(1); })", None),
        // In switch
        ("new Promise((resolve) => { switch(x) { case 1: return 1; } })", None),
        // In loops
        ("new Promise((resolve) => { while(true) { return 1; } })", None),
        ("new Promise((resolve) => { for(;;) { return 1; } })", None),
    ];

    Tester::new(NoPromiseExecutorReturn::NAME, NoPromiseExecutorReturn::PLUGIN, pass, fail)
        .test_and_snapshot();
}
