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
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
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
    pending,
    config = NoPromiseExecutorReturnConfig,
);

impl Rule for NoPromiseExecutorReturn {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };

        // Check if the callee is `Promise` and it's a reference to the global `Promise`
        let Some(ident) = new_expr.callee.get_identifier_reference() else {
            return;
        };

        if ident.name != "Promise" || !ctx.is_reference_to_global_variable(ident) {
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
        ("function foo(resolve, reject) { return 1; }", None),
        ("function Promise(resolve, reject) { return 1; }", None),
        ("(function (resolve, reject) { return 1; })", None),
        ("(function foo(resolve, reject) { return 1; })", None),
        ("(function Promise(resolve, reject) { return 1; })", None),
        ("var foo = function (resolve, reject) { return 1; }", None),
        ("var foo = function Promise(resolve, reject) { return 1; }", None),
        ("var Promise = function (resolve, reject) { return 1; }", None),
        ("(resolve, reject) => { return 1; }", None),
        ("(resolve, reject) => 1", None),
        ("var foo = (resolve, reject) => { return 1; }", None),
        ("var Promise = (resolve, reject) => { return 1; }", None),
        ("var foo = (resolve, reject) => 1", None),
        ("var Promise = (resolve, reject) => 1", None),
        ("var foo = { bar(resolve, reject) { return 1; } }", None),
        ("var foo = { Promise(resolve, reject) { return 1; } }", None),
        ("new foo(function (resolve, reject) { return 1; });", None),
        ("new foo(function bar(resolve, reject) { return 1; });", None),
        ("new foo(function Promise(resolve, reject) { return 1; });", None),
        ("new foo((resolve, reject) => { return 1; });", None),
        ("new foo((resolve, reject) => 1);", None),
        ("new promise(function foo(resolve, reject) { return 1; });", None),
        ("new Promise.foo(function foo(resolve, reject) { return 1; });", None),
        ("new foo.Promise(function foo(resolve, reject) { return 1; });", None),
        ("new Promise.Promise(function foo(resolve, reject) { return 1; });", None),
        ("new Promise()(function foo(resolve, reject) { return 1; });", None),
        ("Promise(function (resolve, reject) { return 1; });", None),
        ("Promise((resolve, reject) => { return 1; });", None),
        ("Promise((resolve, reject) => 1);", None),
        ("new Promise(foo, function (resolve, reject) { return 1; });", None),
        ("new Promise(foo, (resolve, reject) => { return 1; });", None),
        ("new Promise(foo, (resolve, reject) => 1);", None),
        // globals are not supported in tests.
        // ("/* globals Promise:off */ new Promise(function (resolve, reject) { return 1; });", None)
        // ("new Promise((resolve, reject) => { return 1; });", None), // { "globals": { "Promise": "off" } }
        ("let Promise; new Promise(function (resolve, reject) { return 1; });", None),
        ("function f() { new Promise((resolve, reject) => { return 1; }); var Promise; }", None),
        ("function f(Promise) { new Promise((resolve, reject) => 1); }", None),
        (
            "if (x) { const Promise = foo(); new Promise(function (resolve, reject) { return 1; }); }",
            None,
        ),
        ("x = function Promise() { new Promise((resolve, reject) => { return 1; }); }", None),
        ("new Promise(function (resolve, reject) { return; });", None),
        ("new Promise(function (resolve, reject) { reject(new Error()); return; });", None),
        ("new Promise(function (resolve, reject) { if (foo) { return; } });", None),
        ("new Promise((resolve, reject) => { return; });", None),
        (
            "new Promise((resolve, reject) => { if (foo) { resolve(1); return; } reject(new Error()); });",
            None,
        ),
        ("new Promise(function (resolve, reject) { throw new Error(); });", None),
        ("new Promise((resolve, reject) => { throw new Error(); });", None),
        ("new Promise(function (resolve, reject) { function foo() { return 1; } });", None),
        ("new Promise((resolve, reject) => { (function foo() { return 1; })(); });", None),
        ("new Promise(function (resolve, reject) { () => { return 1; } });", None),
        ("new Promise((resolve, reject) => { () => 1 });", None),
        (
            "function foo() { return new Promise(function (resolve, reject) { resolve(bar); }) };",
            None,
        ),
        (
            "foo => new Promise((resolve, reject) => { bar(foo, (err, data) => { if (err) { reject(err); return; } resolve(data); })});",
            None,
        ),
        ("new Promise(function (resolve, reject) {}); function foo() { return 1; }", None),
        ("new Promise((resolve, reject) => {}); (function () { return 1; });", None),
        ("new Promise(function (resolve, reject) {}); () => { return 1; };", None),
        ("new Promise((resolve, reject) => {}); () => 1;", None),
        ("return 1;", None), // { "sourceType": "commonjs" }
        ("return 1;", None), // { "sourceType": "script", "parserOptions": { "ecmaFeatures": { "globalReturn": true } } }
        ("return 1; function foo(){ return 1; } return 1;", None), // { "sourceType": "commonjs" }
        (
            "function foo(){} return 1; var bar = function*(){ return 1; }; return 1; var baz = () => {}; return 1;",
            None,
        ), // { "sourceType": "commonjs" }
        ("new Promise(function (resolve, reject) {}); return 1;", None), // { "sourceType": "commonjs" }
        ("new Promise((r) => void cbf(r));", Some(serde_json::json!([ { "allowVoid": true, }, ]))),
        ("new Promise(r => void 0)", Some(serde_json::json!([ { "allowVoid": true, }, ]))),
        (
            "new Promise(r => { return void 0 })",
            Some(serde_json::json!([ { "allowVoid": true, }, ])),
        ),
        (
            "new Promise(r => { if (foo) { return void 0 } return void 0 })",
            Some(serde_json::json!([ { "allowVoid": true, }, ])),
        ),
        ("new Promise(r => {0})", None),
    ];

    let fail = vec![
        ("new Promise(function (resolve, reject) { return 1; })", None),
        (
            "new Promise((resolve, reject) => resolve(1))",
            Some(serde_json::json!([ { "allowVoid": true, }, ])),
        ),
        (
            "new Promise((resolve, reject) => { return 1 })",
            Some(serde_json::json!([ { "allowVoid": true, }, ])),
        ),
        ("new Promise(r => 1)", Some(serde_json::json!([ { "allowVoid": true, }, ]))),
        ("new Promise(r => 1 ? 2 : 3)", Some(serde_json::json!([ { "allowVoid": true, }, ]))),
        ("new Promise(r => (1 ? 2 : 3))", Some(serde_json::json!([ { "allowVoid": true, }, ]))),
        ("new Promise(r => (1))", Some(serde_json::json!([ { "allowVoid": true, }, ]))),
        ("new Promise(r => () => {})", Some(serde_json::json!([ { "allowVoid": true, }, ]))),
        ("new Promise(r => null)", Some(serde_json::json!([ { "allowVoid": true, }, ]))),
        ("new Promise(r => null)", Some(serde_json::json!([ { "allowVoid": false, }, ]))),
        ("new Promise(r => /*hi*/ ~0)", Some(serde_json::json!([ { "allowVoid": true, }, ]))),
        ("new Promise(r => /*hi*/ ~0)", Some(serde_json::json!([ { "allowVoid": false, }, ]))),
        ("new Promise(r => { return 0 })", Some(serde_json::json!([ { "allowVoid": true, }, ]))),
        ("new Promise(r => { return 0 })", Some(serde_json::json!([ { "allowVoid": false, }, ]))),
        (
            "new Promise(r => { if (foo) { return void 0 } return 0 })",
            Some(serde_json::json!([ { "allowVoid": true, }, ])),
        ),
        (
            "new Promise(resolve => { return (foo = resolve(1)); })",
            Some(serde_json::json!([ { "allowVoid": true, }, ])),
        ),
        (
            "new Promise(resolve => r = resolve)",
            Some(serde_json::json!([ { "allowVoid": true, }, ])),
        ),
        ("new Promise(r => { return(1) })", Some(serde_json::json!([ { "allowVoid": true, }, ]))),
        ("new Promise(r =>1)", Some(serde_json::json!([ { "allowVoid": true, }, ]))),
        ("new Promise(r => ((1)))", Some(serde_json::json!([ { "allowVoid": true, }, ]))),
        ("new Promise(function foo(resolve, reject) { return 1; })", None),
        ("new Promise((resolve, reject) => { return 1; })", None),
        ("new Promise(function (resolve, reject) { return undefined; })", None),
        ("new Promise((resolve, reject) => { return null; })", None),
        ("new Promise(function (resolve, reject) { return false; })", None),
        ("new Promise((resolve, reject) => resolve)", None),
        ("new Promise((resolve, reject) => null)", None),
        ("new Promise(function (resolve, reject) { return resolve(foo); })", None),
        ("new Promise((resolve, reject) => { return reject(foo); })", None),
        ("new Promise((resolve, reject) => x + y)", None),
        ("new Promise((resolve, reject) => { return Promise.resolve(42); })", None),
        ("new Promise(function (resolve, reject) { if (foo) { return 1; } })", None),
        ("new Promise((resolve, reject) => { try { return 1; } catch(e) {} })", None),
        (
            "new Promise(function (resolve, reject) { while (foo){ if (bar) break; else return 1; } })",
            None,
        ),
        ("new Promise(() => { return void 1; })", None),
        ("new Promise(() => (1))", None),
        ("() => new Promise(() => ({}));", None),
        ("new Promise(function () { return 1; })", None),
        ("new Promise(() => { return 1; })", None),
        ("new Promise(() => 1)", None),
        ("function foo() {} new Promise(function () { return 1; });", None),
        ("function foo() { return; } new Promise(() => { return 1; });", None),
        ("function foo() { return 1; } new Promise(() => { return 2; });", None),
        ("function foo () { return new Promise(function () { return 1; }); }", None),
        (
            "function foo() { return new Promise(() => { bar(() => { return 1; }); return false; }); }",
            None,
        ),
        (
            "() => new Promise(() => { if (foo) { return 0; } else bar(() => { return 1; }); })",
            None,
        ),
        (
            "function foo () { return 1; return new Promise(function () { return 2; }); return 3;}",
            None,
        ),
        ("() => 1; new Promise(() => { return 1; })", None),
        ("new Promise(function () { return 1; }); function foo() { return 1; } ", None),
        ("() => new Promise(() => { return 1; });", None),
        ("() => new Promise(() => 1);", None),
        ("() => new Promise(() => () => 1);", None),
        ("() => new Promise(() => async () => 1);", None), // { "ecmaVersion": 2017 }
        ("() => new Promise(() => function () {});", None),
        ("() => new Promise(() => function foo() {});", None),
        ("() => new Promise(() => []);", None),
        ("new Promise((Promise) => { return 1; })", None),
        ("new Promise(function Promise(resolve, reject) { return 1; })", None),
    ];

    Tester::new(NoPromiseExecutorReturn::NAME, NoPromiseExecutorReturn::PLUGIN, pass, fail)
        .test_and_snapshot();
}
