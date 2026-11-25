use oxc_ast::{
    AstKind,
    ast::{Expression, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_catch_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `catch` to `then(a, b)` or `then(null, b)`")
        .with_help(
            "Handle promise errors in a `catch` instead of using the second argument of `then`.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferCatch;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `catch` to `then(a, b)` and `then(null, b)`. This rule disallows the passing of an
    /// argument into the second parameter of `then` calls for handling promise errors.
    ///
    /// ### Why is this bad?
    ///
    /// A `then` call with two arguments can make it more difficult to recognize that a catch error
    /// handler is present. Another issue with using the second argument in `then` calls is that
    /// the ordering of promise error handling is less obvious.
    ///
    /// For example on first glance it may appear that `prom.then(fn1, fn2)` is equivalent to
    /// `prom.then(fn1).catch(fn2)`. However they aren't equivalent. In fact
    /// `prom.catch(fn2).then(fn1)` is the equivalent. This kind of confusion is a good reason for
    /// preferring explicit `catch` calls over passing an argument to the second parameter of
    /// `then` calls.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// prom.then(fn1, fn2)
    ///
    /// prom.then(null, fn2)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// prom.catch(fn2).then(fn1)
    ///
    /// prom.catch(fn2)
    /// ```
    PreferCatch,
    promise,
    style,
    conditional_suggestion,
    version = "0.15.14",
    short_description = "Prefer `catch` to `then(a, b)` and `then(null, b)`.",
);

impl Rule for PreferCatch {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(MemberExpression::StaticMemberExpression(member_expr)) =
            call_expr.callee.get_member_expr()
        else {
            return;
        };

        if member_expr.property.name != "then" || call_expr.arguments.len() < 2 {
            return;
        }

        let diagnostic = prefer_catch_diagnostic(member_expr.property.span);

        let Some(then_arg) = call_expr.arguments[0].as_expression() else {
            ctx.diagnostic(diagnostic);
            return;
        };
        let is_null_or_undefined = match then_arg.without_parentheses() {
            Expression::NullLiteral(_) => true,
            Expression::Identifier(ident) => {
                ident.is_undefined() && ctx.is_reference_to_global_variable(ident)
            }
            _ => false,
        };

        if !is_null_or_undefined || call_expr.type_arguments.is_some() {
            ctx.diagnostic(diagnostic);
            return;
        }

        ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
            let fixer = fixer.for_multifix();
            fixer
                .new_fix_with_capacity(2)
                .extend(fixer.replace(member_expr.property.span, "catch"))
                .extend(fixer.delete_range(Span::new(
                    then_arg.span().start,
                    call_expr.arguments[1].span().start,
                )))
                .with_message("Replace with `catch`")
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "prom.then()",
        "prom.then(fn)",
        "prom.then(fn1).then(fn2)",
        "prom.then(() => {})",
        "prom.then(function () {})",
        "prom.catch()",
        "prom.catch(handleErr).then(handle)",
        "prom.catch(handleErr)",
        "prom['then'](fn1, fn2)",
    ];

    let fail = vec![
        "prom.then(fn1, fn2)",
        "prom.then(fn1, (fn2))",
        "prom.then(null, fn2)",
        "prom.then(undefined, fn2)",
        "function foo() { prom.then(x => {}, () => {}) }",
        "function foo() {
           prom.then(function a() { }, function b() {}).then(fn1, fn2)
        }",
        "const x = prom.then(fn1, fn2)",
        "prom?.then(fn1, fn2)",
        "prom?.then(null, fn2)",
        "prom.then(fn1, ...rest)",
        "prom.then(...args, fn2)",
        "prom.then(void sideEffect(), fn2)",
        "prom.then(getHandler(), fn2)",
        "prom.then((null), (fn2))",
        "function foo() { prom.then((undefined), fn2) }",
        "prom.then(null, fn2, extra())",
        "prom.then(null, ...handlers)",
        "x = prom.then(fn1, fn2)",
        "function foo() { return prom.then(fn1, fn2) }",
        "async function foo() { await prom.then(fn1, fn2) }",
        "(prom.then(fn1, fn2))",
        "prom.then(fn1, fn2).finally(fn3)",
        "foo(prom.then(fn1, fn2))",
        "cond ? prom.then(fn1, fn2) : other",
        "export default prom.then(fn1, fn2)",
        "function foo(undefined) { prom.then(undefined, fn2) }",
    ];

    let fix = vec![
        ("prom.then(fn1, fn2)", "prom.then(fn1, fn2)"),
        ("prom.then(fn1, (fn2))", "prom.then(fn1, (fn2))"),
        ("prom.then(null, fn2)", "prom.catch(fn2)"),
        ("prom.then(undefined, fn2)", "prom.catch(fn2)"),
        (
            "function foo() { prom.then(x => {}, () => {}) }",
            "function foo() { prom.then(x => {}, () => {}) }",
        ),
        ("prom?.then(null, fn2)", "prom?.catch(fn2)"),
        ("prom.then(fn1, ...rest)", "prom.then(fn1, ...rest)"),
        ("prom.then(...args, fn2)", "prom.then(...args, fn2)"),
        ("prom.then(void sideEffect(), fn2)", "prom.then(void sideEffect(), fn2)"),
        ("prom.then(getHandler(), fn2)", "prom.then(getHandler(), fn2)"),
        ("prom.then((null), (fn2))", "prom.catch((fn2))"),
        ("function foo() { prom.then((undefined), fn2) }", "function foo() { prom.catch(fn2) }"),
        ("prom.then(null, fn2, extra())", "prom.catch(fn2, extra())"),
        ("prom.then(null, ...handlers)", "prom.catch(...handlers)"),
        ("foo(prom.then(null, fn2))", "foo(prom.catch(fn2))"),
        (
            "Promise.resolve(1).then<number, void>(null, () => {})",
            "Promise.resolve(1).then<number, void>(null, () => {})",
        ),
        ("prom.then<string>(undefined, fn2)", "prom.then<string>(undefined, fn2)"),
        (
            "function foo(undefined) { prom.then(undefined, fn2) }",
            "function foo(undefined) { prom.then(undefined, fn2) }",
        ),
    ];

    Tester::new(PreferCatch::NAME, PreferCatch::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
