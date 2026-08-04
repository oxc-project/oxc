use oxc_ast::{
    AstKind,
    ast::{Expression, MemberExpression, VariableDeclarationKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_async_promise_finally_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not pass an async function to `Promise#finally()`.")
        .with_help(
            "Use a synchronous callback so the promise is not delayed by unrelated async work.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAsyncPromiseFinally;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows async functions as `Promise#finally()` callbacks.
    ///
    /// ### Why is this bad?
    ///
    /// An async `finally` callback delays settlement of the original promise and
    /// can replace its result with a rejection from the callback. Cleanup that
    /// must be awaited should usually be expressed explicitly.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// promise.finally(async () => cleanup());
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// promise.finally(() => cleanup());
    /// ```
    NoAsyncPromiseFinally,
    unicorn,
    suspicious,
    none,
    version = "next",
    short_description = "Disallow async functions as `Promise#finally()` callbacks.",
);

impl Rule for NoAsyncPromiseFinally {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call) = node.kind() else { return };
        let Some(member) = call.callee.get_member_expr() else { return };
        if !is_finally_member(member, ctx) {
            return;
        }

        let Some(callback) = call.arguments.first().and_then(|argument| argument.as_expression())
        else {
            return;
        };
        if is_async_non_generator_function(callback.get_inner_expression(), ctx) {
            ctx.diagnostic(no_async_promise_finally_diagnostic(callback.span()));
        }
    }
}

fn is_finally_member(member: &MemberExpression<'_>, ctx: &LintContext<'_>) -> bool {
    if member.static_property_name() == Some("finally") {
        return true;
    }

    let MemberExpression::ComputedMemberExpression(computed) = member else { return false };
    let Expression::Identifier(identifier) = computed.expression.get_inner_expression() else {
        return false;
    };
    let Some(symbol_id) = ctx.scoping().get_reference(identifier.reference_id()).symbol_id() else {
        return false;
    };
    let AstKind::VariableDeclarator(declarator) = ctx.symbol_declaration(symbol_id).kind() else {
        return false;
    };
    declarator.kind == VariableDeclarationKind::Const
        && declarator.init.as_ref().is_some_and(|init| {
            matches!(init.get_inner_expression(), Expression::StringLiteral(literal) if literal.value == "finally")
        })
}

fn is_async_non_generator_function(expression: &Expression<'_>, ctx: &LintContext<'_>) -> bool {
    match expression {
        Expression::ArrowFunctionExpression(function) => function.r#async,
        Expression::FunctionExpression(function) => function.r#async && !function.generator,
        Expression::Identifier(identifier) => {
            let Some(symbol_id) =
                ctx.scoping().get_reference(identifier.reference_id()).symbol_id()
            else {
                return false;
            };
            match ctx.symbol_declaration(symbol_id).kind() {
                AstKind::Function(function) => function.r#async && !function.generator,
                AstKind::VariableDeclarator(declarator)
                    if declarator.kind == VariableDeclarationKind::Const =>
                {
                    declarator.init.as_ref().is_some_and(|initializer| {
                        is_async_non_generator_function(initializer.get_inner_expression(), ctx)
                    })
                }
                _ => false,
            }
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

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
        "type Callback = () => void; promise.finally((async () => {}) as Callback);", // {"parser": parsers.typescript},
        "type Callback = () => void; const cleanup = (async () => {}) as Callback; promise.finally(cleanup);", // {"parser": parsers.typescript}
    ];

    Tester::new(NoAsyncPromiseFinally::NAME, NoAsyncPromiseFinally::PLUGIN, pass, fail)
        .test_and_snapshot();
}
