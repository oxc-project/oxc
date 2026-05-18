use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression, IdentifierReference, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::Span;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{AstNode, config::GlobalValue, context::LintContext, rule::Rule};

fn implied_eval_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Implied eval. Consider passing a function instead of a string.")
        .with_help("Pass a function callback instead of source text.")
        .with_label(span)
}

fn exec_script_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Implied eval. Do not use execScript().")
        .with_help("Avoid executing source text at runtime.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoImpliedEval;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows passing strings to `setTimeout()`, `setInterval()`, and
    /// `execScript()`.
    ///
    /// ### Why is this bad?
    ///
    /// Passing a string to these APIs evaluates the string as JavaScript source
    /// text at runtime. This has many of the same security, readability, and
    /// performance problems as `eval()`. Pass a function instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// setTimeout("alert('Hi!')", 100);
    /// setInterval("doWork()", 1000);
    /// window.setTimeout("doWork()", 100);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// setTimeout(() => alert("Hi!"), 100);
    /// setInterval(doWork, 1000);
    /// window.setTimeout(doWork, 100);
    /// ```
    NoImpliedEval,
    eslint,
    suspicious,
    version = "next",
);

impl Rule for NoImpliedEval {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call) = node.kind() else {
            return;
        };

        let Some(target) = classify_callee(&call.callee, ctx) else {
            return;
        };

        let Some(first_argument) = call.arguments.first().and_then(Argument::as_expression) else {
            return;
        };

        if !is_string_like_argument(first_argument, ctx) {
            return;
        }

        ctx.diagnostic(match target {
            EvalLikeTarget::ExecScript => exec_script_diagnostic(call.span),
            EvalLikeTarget::Timer => implied_eval_diagnostic(call.span),
        });
    }
}

#[derive(Clone, Copy)]
enum EvalLikeTarget {
    Timer,
    ExecScript,
}

impl EvalLikeTarget {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "setTimeout" | "setInterval" => Some(Self::Timer),
            "execScript" => Some(Self::ExecScript),
            _ => None,
        }
    }
}

fn classify_callee<'a>(
    callee: &'a Expression<'a>,
    ctx: &LintContext<'a>,
) -> Option<EvalLikeTarget> {
    classify_direct_callee(callee, ctx).or_else(|| classify_member_callee(callee, ctx))
}

fn classify_direct_callee<'a>(
    callee: &'a Expression<'a>,
    ctx: &LintContext<'a>,
) -> Option<EvalLikeTarget> {
    let Expression::Identifier(ident) = callee.get_inner_expression() else {
        return None;
    };
    let target = EvalLikeTarget::from_name(ident.name.as_str())?;
    is_enabled_global_reference(ident, ctx).then_some(target)
}

fn classify_member_callee<'a>(
    callee: &'a Expression<'a>,
    ctx: &LintContext<'a>,
) -> Option<EvalLikeTarget> {
    let member = member_expression_through_chain(callee)?;
    let target = EvalLikeTarget::from_name(member.static_property_name()?)?;
    let root = global_root_after_same_name_chain(member.object())?;

    is_enabled_global_reference(root, ctx).then_some(target)
}

fn member_expression_through_chain<'a>(
    expr: &'a Expression<'a>,
) -> Option<&'a MemberExpression<'a>> {
    match expr.get_inner_expression() {
        expr if expr.is_member_expression() => expr.as_member_expression(),
        Expression::ChainExpression(chain) => chain.expression.member_expression(),
        _ => None,
    }
}

fn global_root_after_same_name_chain<'a>(
    expr: &'a Expression<'a>,
) -> Option<&'a IdentifierReference<'a>> {
    match expr.get_inner_expression() {
        Expression::Identifier(ident) => {
            is_global_candidate_name(ident.name.as_str()).then_some(ident)
        }
        expr => {
            let member = member_expression_through_chain(expr)?;
            let property_name = member.static_property_name()?;
            let root = global_root_after_same_name_chain(member.object())?;

            (property_name == root.name.as_str()).then_some(root)
        }
    }
}

fn is_global_candidate_name(name: &str) -> bool {
    matches!(name, "global" | "window" | "globalThis" | "self")
}

fn is_enabled_global_reference(ident: &IdentifierReference<'_>, ctx: &LintContext<'_>) -> bool {
    ident.is_global_reference(ctx.scoping())
        && ctx
            .get_global_variable_value(ident.name.as_str())
            .is_some_and(|value| value != GlobalValue::Off)
}

fn is_string_like_argument<'a>(expr: &'a Expression<'a>, ctx: &LintContext<'a>) -> bool {
    is_evaluated_string(expr) || static_value_is_string(expr, ctx, 0)
}

fn is_evaluated_string(expr: &Expression<'_>) -> bool {
    match expr.get_inner_expression() {
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => true,
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            is_evaluated_string(&binary.left) || is_evaluated_string(&binary.right)
        }
        _ => false,
    }
}

const MAX_STATIC_IDENTIFIER_DEPTH: u8 = 16;

fn static_value_is_string<'a>(expr: &'a Expression<'a>, ctx: &LintContext<'a>, depth: u8) -> bool {
    match expr.get_inner_expression() {
        expr if is_evaluated_string(expr) => true,
        Expression::Identifier(ident) => initialized_symbol_init(ident, ctx).is_some_and(|init| {
            depth < MAX_STATIC_IDENTIFIER_DEPTH && static_value_is_string(init, ctx, depth + 1)
        }),
        Expression::SequenceExpression(sequence) => {
            sequence.expressions.last().is_some_and(|expr| static_value_is_string(expr, ctx, depth))
        }
        // `typeof` always produces a string if evaluation completes.
        // Reporting it is consistent with reporting other string-producing expressions.
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::Typeof => true,
        Expression::CallExpression(call) => is_global_string_producing_call(call, ctx),
        _ => false,
    }
}

fn initialized_symbol_init<'a>(
    ident: &IdentifierReference<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'a Expression<'a>> {
    let symbol_id = ctx.scoping().get_reference(ident.reference_id()).symbol_id()?;
    let declaration = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id));
    let AstKind::VariableDeclarator(declarator) = declaration.kind() else {
        return None;
    };
    declarator.init.as_ref()
}

// `String(x)` always returns a string; `Date()` (without `new`) also returns a string.
fn is_global_string_producing_call(call: &CallExpression<'_>, ctx: &LintContext<'_>) -> bool {
    let Expression::Identifier(callee) = call.callee.get_inner_expression() else {
        return false;
    };

    matches!(callee.name.as_str(), "String" | "Date") && is_enabled_global_reference(callee, ctx)
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    #[expect(clippy::unnecessary_wraps)]
    fn browser_env() -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "env": {
                "browser": true
            }
        }))
    }

    #[expect(clippy::unnecessary_wraps)]
    fn node_env() -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "env": {
                "node": true
            }
        }))
    }

    #[expect(clippy::unnecessary_wraps)]
    fn exec_script_global() -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "globals": {
                "execScript": false
            }
        }))
    }

    #[expect(clippy::unnecessary_wraps)]
    fn no_globals() -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "globals": {}
        }))
    }

    #[expect(clippy::unnecessary_wraps)]
    fn window_global() -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "globals": {
                "window": "readonly"
            }
        }))
    }

    #[expect(clippy::unnecessary_wraps)]
    fn self_global() -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "globals": {
                "self": "readonly"
            }
        }))
    }

    #[expect(clippy::unnecessary_wraps)]
    fn off_globals() -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "globals": {
                "setTimeout": "off",
                "window": "off"
            }
        }))
    }

    #[expect(clippy::unnecessary_wraps)]
    fn global_this_off() -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "globals": {
                "globalThis": "off"
            }
        }))
    }

    #[expect(clippy::unnecessary_wraps)]
    fn window_on_exec_script_off() -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "globals": {
                "window": "readonly",
                "execScript": "off"
            }
        }))
    }

    #[expect(clippy::unnecessary_wraps)]
    fn cjs_path() -> Option<PathBuf> {
        Some(PathBuf::from("no_implied_eval.cjs"))
    }

    #[expect(clippy::unnecessary_wraps)]
    fn mjs_path() -> Option<PathBuf> {
        Some(PathBuf::from("no_implied_eval.mjs"))
    }

    let pass = vec![
        ("setTimeout();", None, browser_env(), None),
        ("setTimeout;", None, browser_env(), None),
        ("setTimeout = foo;", None, browser_env(), None),
        ("window.setTimeout;", None, browser_env(), None),
        ("window.setTimeout = foo;", None, browser_env(), None),
        ("window['setTimeout'];", None, browser_env(), None),
        ("window['setTimeout'] = foo;", None, browser_env(), None),
        ("global.setTimeout;", None, node_env(), cjs_path()),
        ("global.setTimeout = foo;", None, node_env(), cjs_path()),
        ("global['setTimeout'];", None, node_env(), cjs_path()),
        ("global['setTimeout'] = foo;", None, node_env(), cjs_path()),
        ("globalThis['setTimeout'] = foo;", None, None, None),
        ("window.setTimeout('foo')", None, None, None),
        ("window.setInterval('foo')", None, None, None),
        ("window['setTimeout']('foo')", None, None, None),
        ("window['setInterval']('foo')", None, None, None),
        ("window.setTimeout('foo')", None, None, cjs_path()),
        ("window.setInterval('foo')", None, None, cjs_path()),
        ("window['setTimeout']('foo')", None, None, cjs_path()),
        ("window['setInterval']('foo')", None, None, cjs_path()),
        ("global.setTimeout('foo')", None, browser_env(), None),
        ("global.setInterval('foo')", None, browser_env(), None),
        ("global['setTimeout']('foo')", None, browser_env(), None),
        ("global['setInterval']('foo')", None, browser_env(), None),
        ("window[`SetTimeOut`]('foo', 100);", None, browser_env(), None),
        ("global[`SetTimeOut`]('foo', 100);", None, node_env(), cjs_path()),
        ("global[`setTimeout${foo}`]('foo', 100);", None, browser_env(), None),
        ("global[`setTimeout${foo}`]('foo', 100);", None, node_env(), cjs_path()),
        ("globalThis[`setTimeout${foo}`]('foo', 100);", None, None, None),
        ("setTimeout(function() { x = 1; }, 100);", None, browser_env(), None),
        ("setTimeout(() => {}, 100);", None, browser_env(), None),
        ("setInterval(function() { x = 1; }, 100);", None, browser_env(), None),
        ("execScript(function() { x = 1; }, 100);", None, exec_script_global(), None),
        ("window.setTimeout(function() { x = 1; }, 100);", None, browser_env(), None),
        ("window.setInterval(function() { x = 1; }, 100);", None, browser_env(), None),
        ("window.execScript(function() { x = 1; }, 100);", None, browser_env(), None),
        ("window.setTimeout(foo, 100);", None, browser_env(), None),
        ("window.setInterval(foo, 100);", None, browser_env(), None),
        ("window.execScript(foo, 100);", None, browser_env(), None),
        ("global.setTimeout(function() { x = 1; }, 100);", None, node_env(), cjs_path()),
        ("global.setInterval(function() { x = 1; }, 100);", None, node_env(), cjs_path()),
        ("global.execScript(function() { x = 1; }, 100);", None, node_env(), cjs_path()),
        ("global.setTimeout(foo, 100);", None, node_env(), cjs_path()),
        ("global.setInterval(foo, 100);", None, node_env(), cjs_path()),
        ("global.execScript(foo, 100);", None, node_env(), cjs_path()),
        ("globalThis.setTimeout(foo, 100);", None, None, None),
        ("foo.setTimeout('hi')", None, browser_env(), None),
        ("setTimeout(foo, 10)", None, browser_env(), None),
        ("setTimeout?.(foo, 10)", None, browser_env(), None),
        ("setInterval(1, 10)", None, browser_env(), None),
        ("execScript(2)", None, exec_script_global(), None),
        ("setTimeout(function() {}, 10)", None, browser_env(), None),
        ("foo.setInterval('hi')", None, browser_env(), None),
        ("setInterval(foo, 10)", None, browser_env(), None),
        ("setInterval(function() {}, 10)", None, browser_env(), None),
        ("foo.execScript('hi')", None, browser_env(), None),
        ("execScript(foo)", None, exec_script_global(), None),
        ("execScript(function() {})", None, exec_script_global(), None),
        ("setTimeout(foo + bar, 10)", None, browser_env(), None),
        ("setTimeout(foobar, 'buzz')", None, browser_env(), None),
        ("setTimeout(foobar, foo + 'bar')", None, browser_env(), None),
        ("setTimeout(function() { return 'foobar'; }, 10)", None, browser_env(), None),
        ("setTimeoutFooBar('Foo Bar')", None, browser_env(), None),
        ("foo.window.setTimeout('foo', 100);", None, browser_env(), None),
        ("foo.global.setTimeout('foo', 100);", None, browser_env(), cjs_path()),
        ("var window; window.setTimeout('foo', 100);", None, browser_env(), None),
        ("var global; global.setTimeout('foo', 100);", None, node_env(), cjs_path()),
        ("function foo(window) { window.setTimeout('foo', 100); }", None, browser_env(), None),
        ("function foo(global) { global.setTimeout('foo', 100); }", None, node_env(), cjs_path()),
        ("foo('', window.setTimeout);", None, browser_env(), None),
        ("foo('', global.setTimeout);", None, node_env(), cjs_path()),
        (
            r#"
            function execScript(string) {
                console.log("This is not your grandparent's execScript().");
            }

            execScript('wibble');
            "#,
            None,
            exec_script_global(),
            None,
        ),
        (
            r#"
            function setTimeout(string) {
                console.log("This is not your grandparent's setTimeout().");
            }

            setTimeout('wibble');
            "#,
            None,
            browser_env(),
            None,
        ),
        (
            r#"
            function setInterval(string) {
                console.log("This is not your grandparent's setInterval().");
            }

            setInterval('wibble');
            "#,
            None,
            browser_env(),
            None,
        ),
        (
            r#"
            function outer() {
                function setTimeout(string) {
                    console.log("Shadowed setTimeout");
                }
                setTimeout('code');
            }
            "#,
            None,
            browser_env(),
            None,
        ),
        (
            r#"
            function outer() {
                function setInterval(string) {
                    console.log("Shadowed setInterval");
                }
                setInterval('code');
            }
            "#,
            None,
            browser_env(),
            None,
        ),
        (
            r#"
            function outer() {
                function execScript(string) {
                    console.log("Shadowed execScript");
                }
                execScript('code');
            }
            "#,
            None,
            exec_script_global(),
            None,
        ),
        (
            r#"
            {
                const setTimeout = function(string) {
                    console.log("Block-scoped setTimeout");
                };
                setTimeout('code');
            }
            "#,
            None,
            browser_env(),
            None,
        ),
        (
            r#"
            {
                const setInterval = function(string) {
                    console.log("Block-scoped setInterval");
                };
                setInterval('code');
            }
            "#,
            None,
            browser_env(),
            None,
        ),
        ("setTimeout('code');", None, no_globals(), None),
        ("setInterval('code');", None, no_globals(), None),
        ("execScript('code');", None, no_globals(), None),
        ("window.setTimeout('code');", None, no_globals(), None),
        ("setTimeout('code');", None, off_globals(), None),
        ("window.setTimeout('code');", None, off_globals(), None),
        ("self.setTimeout;", None, browser_env(), None),
        ("self.setTimeout = foo;", None, browser_env(), None),
        ("self['setTimeout'];", None, browser_env(), None),
        ("self['setTimeout'] = foo;", None, browser_env(), None),
        ("self[`SetTimeOut`]('foo', 100);", None, browser_env(), None),
        ("self[`setTimeout${foo}`]('foo', 100);", None, browser_env(), None),
        ("self.setTimeout(function() { x = 1; }, 100);", None, browser_env(), None),
        ("self.setInterval(function() { x = 1; }, 100);", None, browser_env(), None),
        ("self.execScript(function() { x = 1; }, 100);", None, browser_env(), None),
        ("self.setTimeout(foo, 100);", None, browser_env(), None),
        ("foo.self.setTimeout('foo', 100);", None, browser_env(), None),
        ("var self; self.setTimeout('foo', 100);", None, browser_env(), None),
        ("function foo(self) { self.setTimeout('foo', 100); }", None, browser_env(), None),
        ("foo('', self.setTimeout);", None, browser_env(), None),
        (
            r#"
            function outer() {
                function self() {
                    console.log("Shadowed self");
                }
                self.setTimeout('code');
            }
            "#,
            None,
            browser_env(),
            None,
        ),
        ("self.setTimeout('code');", None, no_globals(), None),
        ("eval('x')", None, browser_env(), None),
        ("Function('x')", None, browser_env(), None),
        ("new Function('x')", None, browser_env(), None),
        ("setImmediate('x')", None, browser_env(), None),
        ("requestAnimationFrame('x')", None, browser_env(), None),
        ("setTimeout.call(window, 'x')", None, browser_env(), None),
        ("window.setTimeout.call(window, 'x')", None, browser_env(), None),
        ("(0, setTimeout)('x')", None, browser_env(), None),
        ("const f = setTimeout; f('x')", None, browser_env(), None),
        ("const f = window.setTimeout; f('x')", None, browser_env(), None),
        ("window['set' + 'Timeout']('x')", None, browser_env(), None),
        ("window[name]('x')", None, browser_env(), None),
        ("window['global'].setTimeout('x')", None, browser_env(), None),
        ("global['window'].setTimeout('x')", None, node_env(), cjs_path()),
        ("self['window'].setTimeout('x')", None, browser_env(), None),
        ("foo?.setTimeout('code', 0)", None, window_global(), None),
        ("foo?.['setTimeout']('code', 0)", None, window_global(), None),
        ("window?.setTimeout(foo, 0)", None, window_global(), None),
        ("function f(String) { setTimeout(String('x')); }", None, browser_env(), None),
        ("const String = () => 'x'; setTimeout(String('x'));", None, browser_env(), None),
        ("let String = globalThis.String; setTimeout(String('x'));", None, browser_env(), None),
        ("const S = globalThis.String; setTimeout(S('x'));", None, browser_env(), None),
        ("setTimeout(globalThis.String('x'))", None, browser_env(), None),
        ("setTimeout(({s: 'x', s: foo}).s);", None, browser_env(), None),
        ("setTimeout(({s: 'x', ...foo}).s);", None, browser_env(), None),
        ("const f = 'x'.toUpperCase; setTimeout(f());", None, browser_env(), None),
        ("const f = ['x'].join; setTimeout(f());", None, browser_env(), None),
        (
            "const f = String.prototype.toUpperCase; setTimeout(f.call('x'));",
            None,
            browser_env(),
            None,
        ),
        ("setTimeout(String.raw('x'));", None, browser_env(), None),
        ("const f = String.raw; setTimeout(f('x'));", None, browser_env(), None),
        ("setTimeout(String.raw`x`);", None, browser_env(), None),
        ("const R = String.raw; setTimeout(R`x`);", None, browser_env(), None),
        ("const S = String; setTimeout(S('x'));", None, browser_env(), None),
        ("const D = Date; setTimeout(D());", None, browser_env(), None),
        ("setTimeout(new String('x'))", None, browser_env(), None),
        ("setTimeout(new Date())", None, browser_env(), None),
        ("setTimeout('x'.at(99));", None, browser_env(), None),
        ("const s = +'abc' ? 'x' : foo; setTimeout(s);", None, browser_env(), None),
        ("const s = +{} ? 'x' : foo; setTimeout(s);", None, browser_env(), None),
        ("globalThis.setTimeout('code');", None, global_this_off(), None),
        ("import { setTimeout } from 'timers'; setTimeout('x')", None, browser_env(), mjs_path()),
        ("import { setInterval } from 'timers'; setInterval('x')", None, browser_env(), mjs_path()),
    ];

    let fail = vec![
        (r#"setTimeout("x = 1;");"#, None, browser_env(), None),
        (r#"setTimeout("x = 1;", 100);"#, None, browser_env(), None),
        (r#"setInterval("x = 1;");"#, None, browser_env(), None),
        (r#"execScript("x = 1;");"#, None, exec_script_global(), None),
        ("const s = 'x=1'; setTimeout(s, 100);", None, browser_env(), None),
        ("const s = `x=1`; setTimeout(s, 100);", None, browser_env(), None),
        ("const s = 1 + ';' + 1; setTimeout(s, 100);", None, browser_env(), None),
        ("setTimeout(s); const s = 'x';", None, browser_env(), None),
        ("const a = 'x'; const b = a; setTimeout(b);", None, browser_env(), None),
        (
            "const a0 = 'x'; const a1 = a0; const a2 = a1; const a3 = a2; const a4 = a3; const a5 = a4; const a6 = a5; const a7 = a6; const a8 = a7; const a9 = a8; setTimeout(a9);",
            None,
            browser_env(),
            None,
        ),
        ("let s = 'x'; setTimeout(s);", None, browser_env(), None),
        ("var s = 'x'; setTimeout(s);", None, browser_env(), None),
        ("let s = 'x'; s = foo; setTimeout(s);", None, browser_env(), None),
        ("let s = 'x'; setTimeout(s); s = foo;", None, browser_env(), None),
        ("let s = 'x'; function mutate(){ s = foo; } setTimeout(s);", None, browser_env(), None),
        ("setTimeout(typeof foo);", None, browser_env(), None),
        ("const s = typeof foo; setTimeout(s);", None, browser_env(), None),
        ("setTimeout(typeof foo.bar);", None, browser_env(), None),
        ("setTimeout(typeof 1);", None, browser_env(), None),
        ("const s = typeof 1; setTimeout(s);", None, browser_env(), None),
        ("setTimeout(String('x=1'), 100);", None, browser_env(), None),
        ("setTimeout(String(1), 100);", None, browser_env(), None),
        ("setTimeout(String(), 100);", None, browser_env(), None),
        ("setTimeout(String(undefined), 100);", None, browser_env(), None),
        ("setTimeout(String(null), 100);", None, browser_env(), None),
        ("setTimeout(String(true), 100);", None, browser_env(), None),
        ("setTimeout(String(1n), 100);", None, browser_env(), None),
        ("setTimeout(String({}));", None, browser_env(), None),
        ("setTimeout(String([]));", None, browser_env(), None),
        ("setTimeout(String(/x/));", None, browser_env(), None),
        ("setTimeout(String(Symbol()));", None, browser_env(), None),
        ("setTimeout(String(foo));", None, browser_env(), None),
        ("const code = String(foo); setTimeout(code);", None, browser_env(), None),
        ("const code = `x${foo}`; setTimeout(code);", None, browser_env(), None),
        ("const o = {}; o.x = 1; setTimeout(String(o));", None, browser_env(), None),
        ("const a = []; a.push(1); setTimeout(String(a));", None, browser_env(), None),
        ("setTimeout(Date());", None, browser_env(), None),
        ("window.setTimeout('foo')", None, browser_env(), None),
        ("window.setInterval('foo')", None, browser_env(), None),
        ("window.execScript('foo')", None, browser_env(), None),
        ("window['setTimeout']('foo')", None, browser_env(), None),
        ("window['setInterval']('foo')", None, browser_env(), None),
        ("window[`setInterval`]('foo')", None, browser_env(), None),
        ("window['execScript']('foo')", None, browser_env(), None),
        ("window[`execScript`]('foo')", None, browser_env(), None),
        ("window.window['setInterval']('foo')", None, browser_env(), None),
        ("window.window['execScript']('foo')", None, browser_env(), None),
        ("window['window'].setTimeout('foo')", None, browser_env(), None),
        ("global.setTimeout('foo')", None, node_env(), cjs_path()),
        ("global.setInterval('foo')", None, node_env(), cjs_path()),
        ("global.execScript('foo')", None, node_env(), cjs_path()),
        ("global['setTimeout']('foo')", None, node_env(), cjs_path()),
        ("global['setInterval']('foo')", None, node_env(), cjs_path()),
        ("global[`setInterval`]('foo')", None, node_env(), cjs_path()),
        ("global['execScript']('foo')", None, node_env(), cjs_path()),
        ("global[`execScript`]('foo')", None, node_env(), cjs_path()),
        ("global.global['setInterval']('foo')", None, node_env(), cjs_path()),
        ("global.global['execScript']('foo')", None, node_env(), cjs_path()),
        ("global['global'].setTimeout('foo')", None, node_env(), cjs_path()),
        ("globalThis.setTimeout('foo')", None, None, None),
        ("globalThis.setInterval('foo')", None, None, None),
        ("globalThis.execScript('foo')", None, None, None),
        ("globalThis.globalThis.setTimeout('foo')", None, None, None),
        ("globalThis['globalThis'].setInterval('foo')", None, None, None),
        ("setTimeout(`foo${bar}`)", None, browser_env(), None),
        ("window.setTimeout(`foo${bar}`)", None, browser_env(), None),
        ("window.window.setTimeout(`foo${bar}`)", None, browser_env(), None),
        ("global.global.setTimeout(`foo${bar}`)", None, node_env(), cjs_path()),
        ("setTimeout('foo' + bar)", None, browser_env(), None),
        ("setTimeout(foo + 'bar')", None, browser_env(), None),
        ("setTimeout(`foo` + bar)", None, browser_env(), None),
        ("setTimeout(1 + ';' + 1)", None, browser_env(), None),
        ("window.setTimeout('foo' + bar)", None, browser_env(), None),
        ("window.setTimeout(foo + 'bar')", None, browser_env(), None),
        ("window.setTimeout(`foo` + bar)", None, browser_env(), None),
        ("window.setTimeout(1 + ';' + 1)", None, browser_env(), None),
        ("window.window.setTimeout(1 + ';' + 1)", None, browser_env(), None),
        ("global.setTimeout('foo' + bar)", None, node_env(), cjs_path()),
        ("global.setTimeout(foo + 'bar')", None, node_env(), cjs_path()),
        ("global.setTimeout(`foo` + bar)", None, node_env(), cjs_path()),
        ("global.setTimeout(1 + ';' + 1)", None, node_env(), cjs_path()),
        ("global.global.setTimeout(1 + ';' + 1)", None, node_env(), cjs_path()),
        ("globalThis.setTimeout('foo' + bar)", None, None, None),
        (
            "setTimeout('foo' + (function() {\n   setTimeout(helper);\n   execScript('str');\n   return 'bar';\n})())",
            None,
            Some(serde_json::json!({
                "env": {
                    "browser": true
                },
                "globals": {
                    "execScript": false
                }
            })),
            None,
        ),
        (
            "window.setTimeout('foo' + (function() {\n   setTimeout(helper);\n   window.execScript('str');\n   return 'bar';\n})())",
            None,
            browser_env(),
            None,
        ),
        (
            "global.setTimeout('foo' + (function() {\n   setTimeout(helper);\n   global.execScript('str');\n   return 'bar';\n})())",
            None,
            Some(serde_json::json!({
                "env": {
                    "browser": true,
                    "node": true
                }
            })),
            cjs_path(),
        ),
        ("window?.setTimeout('code', 0)", None, window_global(), None),
        ("(window?.setTimeout)('code', 0)", None, window_global(), None),
        ("window?.execScript('code')", None, window_global(), None),
        ("(window?.execScript)('code')", None, window_global(), None),
        ("window?.window.setTimeout('code')", None, window_global(), None),
        ("window.window?.setTimeout('code')", None, window_global(), None),
        ("window.setTimeout?.('code')", None, window_global(), None),
        ("window['setTimeout']?.('code')", None, window_global(), None),
        ("window?.['setTimeout']('code')", None, window_global(), None),
        ("self.setTimeout('foo')", None, browser_env(), None),
        ("self.setInterval('foo')", None, browser_env(), None),
        ("self.execScript('foo')", None, browser_env(), None),
        ("self['setTimeout']('foo')", None, browser_env(), None),
        ("self['setInterval']('foo')", None, browser_env(), None),
        ("self[`setInterval`]('foo')", None, browser_env(), None),
        ("self['execScript']('foo')", None, browser_env(), None),
        ("self[`execScript`]('foo')", None, browser_env(), None),
        ("self.self['setInterval']('foo')", None, browser_env(), None),
        ("self.self['execScript']('foo')", None, browser_env(), None),
        ("self['self'].execScript('foo')", None, browser_env(), None),
        ("self.setTimeout(`foo${bar}`)", None, browser_env(), None),
        ("self.self.setTimeout(`foo${bar}`)", None, browser_env(), None),
        ("self.setTimeout('foo' + bar)", None, browser_env(), None),
        ("self.setTimeout(foo + 'bar')", None, browser_env(), None),
        ("self.setTimeout(`foo` + bar)", None, browser_env(), None),
        ("self.setTimeout(1 + ';' + 1)", None, browser_env(), None),
        ("self.self.setTimeout(1 + ';' + 1)", None, browser_env(), None),
        ("self?.setTimeout('code', 0)", None, self_global(), None),
        ("(self?.setTimeout)('code', 0)", None, self_global(), None),
        ("self?.execScript('code')", None, self_global(), None),
        ("(self?.execScript)('code')", None, self_global(), None),
        ("self.execScript?.('code')", None, self_global(), None),
        ("(setTimeout)('x')", None, browser_env(), None),
        ("((setTimeout))('x')", None, browser_env(), None),
        ("setTimeout?.('x')", None, browser_env(), None),
        ("setInterval?.('x')", None, browser_env(), None),
        ("execScript?.('x')", None, exec_script_global(), None),
        ("window.execScript('x')", None, window_on_exec_script_off(), None),
    ];

    Tester::new(NoImpliedEval::NAME, NoImpliedEval::PLUGIN, pass, fail)
        .change_rule_path_extension("js")
        .test_and_snapshot();
}

#[test]
fn test_typescript_wrappers() {
    use crate::tester::Tester;

    #[expect(clippy::unnecessary_wraps)]
    fn browser_env() -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "env": {
                "browser": true
            }
        }))
    }

    let pass = vec![
        (
            "function f(setTimeout: (handler: string) => void) { setTimeout('x' as string); }",
            None,
            browser_env(),
        ),
        (
            "function f(window: { setTimeout(handler: string): void }) { window.setTimeout('x' as string); }",
            None,
            browser_env(),
        ),
        (
            "const String = (value: unknown): string => 'x'; setTimeout(String('x' as string));",
            None,
            browser_env(),
        ),
    ];

    let fail = vec![
        ("setTimeout('x' as string);", None, browser_env()),
        ("setTimeout(<string>'x');", None, browser_env()),
        ("setTimeout('x' satisfies string);", None, browser_env()),
        ("setTimeout(('x')!);", None, browser_env()),
        ("setTimeout<string>('x');", None, browser_env()),
        ("(setTimeout as typeof setTimeout)('x');", None, browser_env()),
        ("window.setTimeout('x' as string);", None, browser_env()),
        ("(window as Window).setTimeout('x');", None, browser_env()),
        ("(window.setTimeout as Window['setTimeout'])('x');", None, browser_env()),
        ("const code = 'x' as string; setTimeout(code);", None, browser_env()),
        ("setTimeout(String(foo as string));", None, browser_env()),
        ("setTimeout(typeof (foo as string));", None, browser_env()),
    ];

    Tester::new(NoImpliedEval::NAME, NoImpliedEval::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .with_snapshot_suffix("ts")
        .test_and_snapshot();
}
