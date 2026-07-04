use oxc_ast::{AstKind, ast::IdentifierReference, ast::MemberExpression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::Span;
use oxc_str::static_ident;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_new_func(function_call_span: Span, arguments_span: Option<Span>) -> OxcDiagnostic {
    let mut diagnostic = OxcDiagnostic::warn("Using `new Function` or `Function` is not allowed.")
        .with_help(
            "Avoid the `Function` constructor. Define the function directly with a function declaration/expression or an arrow function.",
        )
        .with_note(
            "The `Function` constructor compiles code from strings at runtime, which can introduce injection risks, hurts performance, and makes code harder to analyze and maintain.",
        )
        .with_label(function_call_span.primary_label("Dynamic function construction is used here."));

    if let Some(arguments_span) = arguments_span {
        diagnostic = diagnostic.and_label(
            arguments_span.label("`Function` evaluates source text at runtime, similar to `eval`."),
        );
    }
    diagnostic
}

#[derive(Debug, Default, Clone)]
pub struct NoNewFunc;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `new` operators with the `Function` object.
    ///
    /// ### Why is this bad?
    ///
    /// Using `new Function` or `Function` can lead to code that is difficult to understand and maintain. It can introduce security risks similar to those associated with `eval` because it generates a new function from a string of code, which can be a vector for injection attacks. Additionally, it impacts performance negatively as these functions are not optimized by the JavaScript engine.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var x = new Function("a", "b", "return a + b");
    /// var x = Function("a", "b", "return a + b");
    /// var x = Function.call(null, "a", "b", "return a + b");
    /// var x = Function.apply(null, ["a", "b", "return a + b"]);
    /// var x = Function.bind(null, "a", "b", "return a + b")();
    /// var f = Function.bind(null, "a", "b", "return a + b");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// let x = function (a, b) {
    ///  return a + b;
    /// };
    /// ```
    NoNewFunc,
    eslint,
    style,
    version = "0.9.2",
    short_description = "Disallow `new` operators with the `Function` object.",
);

impl Rule for NoNewFunc {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NewExpression(new_expr) => {
                let Some(id) = new_expr.callee.get_identifier_reference() else {
                    return;
                };

                check(id, new_expr.arguments_span(), ctx);
            }
            AstKind::CallExpression(call_expr) => {
                let Some(obj_id) = get_function_constructor_reference(&call_expr.callee) else {
                    return;
                };

                check(obj_id, call_expr.arguments_span(), ctx);
            }
            _ => {}
        }
    }
}

fn get_function_constructor_reference<'a>(
    callee: &'a oxc_ast::ast::Expression<'a>,
) -> Option<&'a IdentifierReference<'a>> {
    callee.get_identifier_reference().or_else(|| {
        let member_expr = member_expression_through_chain(callee)?;
        let property_name = member_expr.static_property_name()?;
        matches!(property_name, "apply" | "bind" | "call")
            .then(|| member_expr.object().get_identifier_reference())?
    })
}

fn member_expression_through_chain<'a>(
    expr: &'a oxc_ast::ast::Expression<'a>,
) -> Option<&'a MemberExpression<'a>> {
    match expr.get_inner_expression() {
        expr if expr.is_member_expression() => expr.as_member_expression(),
        oxc_ast::ast::Expression::ChainExpression(chain) => chain.expression.member_expression(),
        _ => None,
    }
}

fn check(ident: &IdentifierReference, arguments_span: Option<Span>, ctx: &LintContext) {
    if ident.is_global_reference_name(static_ident!("Function"), ctx.scoping()) {
        ctx.diagnostic(no_new_func(ident.span, arguments_span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"var a = new _function("b", "c", "return b+c");"#,
        r#"var a = _function("b", "c", "return b+c");"#,
        "class Function {}; new Function()",
        "const fn = () => { class Function {}; new Function() }",
        "function Function() {}; Function()",
        "var fn = function () { function Function() {}; Function() }",
        "var x = function Function() { Function(); }",
        "call(Function)",
        "new Class(Function)",
        "foo[Function]()",
        "foo(Function.bind)",
        "Function.toString()",
        "Function[call]()",
    ];

    let fail = vec![
        r#"var a = new Function("b", "c", "return b+c");"#,
        r#"var a = Function("b", "c", "return b+c");"#,
        r#"var a = Function.call(null, "b", "c", "return b+c");"#,
        r#"var a = Function.apply(null, ["b", "c", "return b+c"]);"#,
        r#"var a = Function.bind(null, "b", "c", "return b+c")();"#,
        r#"var a = Function.bind(null, "b", "c", "return b+c");"#,
        r#"var a = Function["call"](null, "b", "c", "return b+c");"#,
        r#"var a = (Function?.call)(null, "b", "c", "return b+c");"#,
        "const fn = () => { class Function {} }; new Function('', '')",
        "var fn = function () { function Function() {} }; Function('', '')",
    ];

    Tester::new(NoNewFunc::NAME, NoNewFunc::PLUGIN, pass, fail).test_and_snapshot();
}
