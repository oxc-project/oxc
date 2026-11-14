use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, IdentifierReference, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::{get_declaration_of_variable, get_symbol_id_of_variable, variable_declaration_kind},
    context::LintContext,
    rule::Rule,
};

fn no_async_promise_finally_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not pass an async function to `Promise#finally()`.")
        .with_help(
            "Use a synchronous callback, or handle the cleanup promise's errors explicitly so they do not replace the original rejection.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAsyncPromiseFinally;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows async functions as `Promise#finally()` callbacks.
    ///
    /// ### Why is this bad?
    ///
    /// Passing an async function to `Promise#finally()` can hide the original
    /// promise outcome. If the promise rejects and the async finalizer also
    /// rejects, the resulting promise rejects with the finalizer's error and
    /// the original error is lost.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// promise.finally(async () => {
    ///     await cleanup();
    /// });
    ///
    /// async function cleanup() {}
    /// promise.finally(cleanup);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// promise.finally(() => {
    ///     cleanupSynchronously();
    /// });
    ///
    /// promise.finally(() => {
    ///     return cleanup().catch((error) => {
    ///         logCleanupError(error);
    ///     });
    /// });
    /// ```
    NoAsyncPromiseFinally,
    unicorn,
    correctness,
    recommended,
    version = "next",
    short_description = "Disallow async functions as `Promise#finally()` callbacks.",
);

impl Rule for NoAsyncPromiseFinally {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        // Cheap syntactic checks first; semantic lookups only for candidates.
        // Spread arguments (`finally(...[cleanup])`) are not checked.
        let Some(callback) = call_expr.arguments.first().and_then(Argument::as_expression) else {
            return;
        };
        let callback = callback.get_inner_expression();
        if !matches!(
            callback,
            Expression::ArrowFunctionExpression(_)
                | Expression::FunctionExpression(_)
                | Expression::Identifier(_)
        ) {
            return;
        }
        let Some(member_expr) = call_expr.callee.get_inner_expression().as_member_expression()
        else {
            return;
        };
        if !is_finally_property(member_expr, ctx) {
            return;
        }
        if is_async_finally_callback(callback, ctx) {
            ctx.diagnostic(no_async_promise_finally_diagnostic(callback.span()));
        }
    }
}

/// `x.finally`, `x["finally"]`, ``x[`finally`]``, or `x[method]` where `method` is a
/// `const` bound to the string `"finally"`.
fn is_finally_property<'a>(member_expr: &MemberExpression<'a>, ctx: &LintContext<'a>) -> bool {
    if let Some(name) = member_expr.static_property_name() {
        return name == "finally";
    }
    let MemberExpression::ComputedMemberExpression(computed) = member_expr else {
        return false;
    };
    let Expression::Identifier(ident) = computed.expression.get_inner_expression() else {
        return false;
    };
    match const_initializer(ident, ctx).map(Expression::get_inner_expression) {
        Some(Expression::StringLiteral(lit)) => lit.value == "finally",
        Some(Expression::TemplateLiteral(lit)) => {
            lit.single_quasi().is_some_and(|quasi| quasi == "finally")
        }
        _ => false,
    }
}

/// `callback` must already be unwrapped with `get_inner_expression`.
fn is_async_finally_callback<'a>(callback: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    if is_async_non_generator_function(callback) {
        return true;
    }
    let Expression::Identifier(ident) = callback else {
        return false;
    };
    if let Some(init) = const_initializer(ident, ctx) {
        return is_async_non_generator_function(init.get_inner_expression());
    }
    is_async_function_declaration_reference(ident, ctx)
}

fn is_async_non_generator_function(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::ArrowFunctionExpression(arrow) => arrow.r#async,
        Expression::FunctionExpression(func) => func.r#async && !func.generator,
        _ => false,
    }
}

/// Initializer of a `const` binding referenced by `ident`. Destructured bindings return `None`.
fn const_initializer<'a, 'b>(
    ident: &IdentifierReference<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b Expression<'a>> {
    let declaration = get_declaration_of_variable(ident, ctx.semantic())?;
    let AstKind::VariableDeclarator(declarator) = declaration.kind() else {
        return None;
    };
    if !variable_declaration_kind(declarator, ctx.nodes())?.is_const() {
        return None;
    }
    // Destructured bindings (`const { a } = b`) do not hold the initializer's value.
    declarator.id.get_binding_identifier()?;
    declarator.init.as_ref()
}

/// `ident` refers to exactly one `async function name() {}` declaration (not a generator)
/// that is never reassigned.
fn is_async_function_declaration_reference(
    ident: &IdentifierReference<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    let Some(symbol_id) = get_symbol_id_of_variable(ident, ctx.semantic()) else {
        return false;
    };
    let scoping = ctx.scoping();
    if !scoping.symbol_redeclarations(symbol_id).is_empty() || scoping.symbol_is_mutated(symbol_id)
    {
        return false;
    }
    let declaration = ctx.nodes().get_node(scoping.symbol_declaration(symbol_id));
    matches!(declaration.kind(), AstKind::Function(func) if func.r#async && !func.generator)
}

#[test]
fn test() {
    use crate::tester::Tester;

    // source: eslint-plugin-unicorn v78.0.0 test/no-async-promise-finally.js
    // Type-aware cases from upstream are omitted (oxlint has no type information here).
    let pass = vec![
        "promise.finally(() => {})",
        "promise.finally(() => cleanup())",
        "promise.finally(function () {})",
        "promise.finally(async function * () {})",
        "promise.finally()",
        "promise.finally(undefined)",
        "promise.finally(cleanup)",
        "promise.finally(object.cleanup)",
        "promise.then(async () => {})",
        "promise.catch(async () => {})",
        "finalizer(async () => {})",
        "promise.notFinally(async () => {})",
        "promise[method](async () => {})",
        "const cleanup = () => {}; promise.finally(cleanup);",
        "let cleanup = async () => {}; promise.finally(cleanup);",
        "async function * cleanup() {} promise.finally(cleanup);",
        "const cleanup = async function * () {}; promise.finally(cleanup);",
        "const cleanup = async () => {}; promise.finally(...[cleanup]);",
        r#"import {cleanup} from "./cleanup.js"; promise.finally(cleanup);"#,
        // Additional: destructured const bindings are not the binding's value.
        "const { cleanup } = { cleanup: async () => {} }; promise.finally(cleanup);",
        // Additional: a reassigned function declaration may no longer be async.
        "async function cleanup() {} cleanup = () => {}; promise.finally(cleanup);",
        // Additional: non-"finally" const key.
        r#"const method = "then"; promise[method](async () => {});"#,
    ];

    let fail = vec![
        "promise.finally(async () => {})",
        "promise.finally(async () => cleanup())",
        "promise.finally(async () => { await cleanup(); })",
        "promise.finally(async function () { await cleanup(); })",
        "Promise.resolve(value).finally(async () => cleanup())",
        "new Promise(resolve => resolve()).finally(async () => cleanup())",
        "promise?.finally(async () => {})",
        "promise.finally?.(async () => {})",
        r#"promise["finally"](async () => {})"#,
        "promise[`finally`](async () => {})",
        r#"const method = "finally"; promise[method](async () => {});"#,
        "async function cleanup() {} promise.finally(cleanup);",
        "const cleanup = async () => {}; promise.finally(cleanup);",
        "const cleanup = async function () {}; promise.finally(cleanup);",
        "type Callback = () => void; promise.finally((async () => {}) as Callback);",
        "type Callback = () => void; const cleanup = (async () => {}) as Callback; promise.finally(cleanup);",
        // Additional: template literal const key.
        "const method = `finally`; promise[method](async () => {});",
        // Additional: parenthesized callback.
        "promise.finally((async () => {}))",
    ];

    Tester::new(NoAsyncPromiseFinally::NAME, NoAsyncPromiseFinally::PLUGIN, pass, fail)
        .test_and_snapshot();
}
