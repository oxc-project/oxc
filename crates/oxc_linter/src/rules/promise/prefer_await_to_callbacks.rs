use oxc_ast::{
    ast::{Argument, Expression, FormalParameters, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn prefer_await_to_callbacks(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `async`/`await` to the callback pattern").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferAwaitToCallbacks;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// The rule encourages the use of `async/await` for handling asynchronous code
    /// instead of traditional callback functions. `async/await`, introduced in ES2017,
    /// provides a clearer and more concise syntax for writing asynchronous code,
    /// making it easier to read and maintain.
    ///
    /// ### Why is this bad?
    ///
    /// Using callbacks can lead to complex, nested structures known as "callback hell,"
    /// which make code difficult to read and maintain. Additionally, error handling can
    /// become cumbersome with callbacks, whereas `async/await` allows for more straightforward
    /// try/catch blocks for managing errors.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// cb()
    /// callback()
    /// doSomething(arg, (err) => {})
    /// function doSomethingElse(cb) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// await doSomething(arg)
    /// async function doSomethingElse() {}
    /// function* generator() {
    ///     yield yieldValue(err => {})
    /// }
    /// eventEmitter.on('error', err => {})
    /// ```
    PreferAwaitToCallbacks,
    promise,
    style,
);

impl Rule for PreferAwaitToCallbacks {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(expr) => {
                let callee_name = expr.callee.get_identifier_reference().map(|id| id.name.as_str());
                if matches!(callee_name, Some("callback" | "cb")) {
                    ctx.diagnostic(prefer_await_to_callbacks(expr.span));
                    return;
                }

                if let Some(last_arg) = expr.arguments.last() {
                    let args = match last_arg {
                        Argument::FunctionExpression(expr) => &expr.params,
                        Argument::ArrowFunctionExpression(expr) => &expr.params,
                        _ => return,
                    };

                    let callee_property_name = expr
                        .callee
                        .as_member_expression()
                        .and_then(MemberExpression::static_property_name);

                    if matches!(callee_property_name, Some("on" | "once")) {
                        return;
                    }

                    let is_lodash = expr.callee.as_member_expression().is_some_and( |mem_expr| {
                        matches!(mem_expr.object(), Expression::Identifier(id) if matches!(id.name.as_str(), "_" | "lodash" | "underscore"))
                    });

                    let calls_array_method = callee_property_name
                        .is_some_and(Self::is_array_method)
                        && (expr.arguments.len() == 1 || (expr.arguments.len() == 2 && is_lodash));

                    let is_array_method =
                        callee_name.is_some_and(Self::is_array_method) && expr.arguments.len() == 2;

                    if calls_array_method || is_array_method {
                        return;
                    }

                    let Some(param) = args.items.first() else {
                        return;
                    };

                    if matches!(
                        param.pattern.get_identifier_name().as_deref(),
                        Some("err" | "error")
                    ) && !Self::is_inside_yield_or_await(node.id(), ctx)
                    {
                        ctx.diagnostic(prefer_await_to_callbacks(last_arg.span()));
                    }
                }
            }
            AstKind::Function(func) => {
                Self::check_last_params_for_callback(&func.params, ctx);
            }
            AstKind::ArrowFunctionExpression(expr) => {
                Self::check_last_params_for_callback(&expr.params, ctx);
            }
            _ => {}
        }
    }
}

impl PreferAwaitToCallbacks {
    fn check_last_params_for_callback(params: &FormalParameters, ctx: &LintContext) {
        let Some(param) = params.items.last() else {
            return;
        };

        let id = param.pattern.get_identifier_name();
        if matches!(id.as_deref(), Some("callback" | "cb")) {
            ctx.diagnostic(prefer_await_to_callbacks(param.span));
        }
    }

    fn is_inside_yield_or_await(id: NodeId, ctx: &LintContext) -> bool {
        ctx.nodes().ancestors(id).skip(1).any(|parent| {
            matches!(parent.kind(), AstKind::AwaitExpression(_) | AstKind::YieldExpression(_))
        })
    }

    fn is_array_method(name: &str) -> bool {
        ["map", "every", "forEach", "some", "find", "filter"].contains(&name)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "async function hi() { await thing().catch(err => console.log(err)) }",
        "async function hi() { await thing().then() }",
        "async function hi() { await thing().catch() }",
        r#"dbConn.on("error", err => { console.error(err) })"#,
        r#"dbConn.once("error", err => { console.error(err) })"#,
        "heart(something => {})",
        "getErrors().map(error => responseTo(error))",
        "errors.filter(err => err.status === 402)",
        r#"errors.some(err => err.message.includes("Yo"))"#,
        "errors.every(err => err.status === 402)",
        "errors.filter(err => console.log(err))",
        r#"const error = errors.find(err => err.stack.includes("file.js"))"#,
        "this.myErrors.forEach(function(error) { log(error); })",
        r#"find(errors, function(err) { return  err.type === "CoolError" })"#,
        r#"map(errors, function(error) { return  err.type === "CoolError" })"#,
        r#"_.find(errors, function(error) { return  err.type === "CoolError" })"#,
        r#"_.map(errors, function(err) { return  err.type === "CoolError" })"#,
    ];

    let fail = vec![
        "heart(function(err) {})",
        "heart(err => {})",
        r#"heart("ball", function(err) {})"#,
        "function getData(id, callback) {}",
        "const getData = (cb) => {}",
        "var x = function (x, cb) {}",
        "cb()",
        "callback()",
        "heart(error => {})",
        "async.map(files, fs.stat, function(err, results) { if (err) throw err; });",
        "_.map(files, fs.stat, function(err, results) { if (err) throw err; });",
        "map(files, fs.stat, function(err, results) { if (err) throw err; });",
        "map(function(err, results) { if (err) throw err; });",
        "customMap(errors, (err) => err.message)",
    ];

    Tester::new(PreferAwaitToCallbacks::NAME, PreferAwaitToCallbacks::PLUGIN, pass, fail)
        .test_and_snapshot();
}
