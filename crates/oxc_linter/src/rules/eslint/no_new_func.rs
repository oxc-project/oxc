use oxc_ast::{AstKind, ast::IdentifierReference};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_new_func(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The Function constructor is eval.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNewFunc;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// The rule disallow `new` operators with the `Function` object.
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
    style
);

impl Rule for NoNewFunc {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NewExpression(new_expr) => {
                let Some(id) = new_expr.callee.get_identifier_reference() else {
                    return;
                };

                check(id, new_expr.span, ctx);
            }
            AstKind::CallExpression(call_expr) => {
                let Some(obj_id) = call_expr.callee.get_identifier_reference() else {
                    return;
                };

                check(obj_id, call_expr.span, ctx);
            }
            member_expr if member_expr.is_member_expression_kind() => {
                let Some(member_expr) = member_expr.as_member_expression_kind() else {
                    return;
                };

                let parent = ctx.nodes().ancestor_kinds(node.id()).find(|node| {
                    !matches!(
                        node,
                        AstKind::ChainExpression(_) | AstKind::ParenthesizedExpression(_)
                    )
                });

                let Some(AstKind::CallExpression(parent_call_expr)) = parent else {
                    return;
                };

                let Some(static_property_name) =
                    &member_expr.static_property_name().map(|s| s.as_str())
                else {
                    return;
                };

                if !["apply", "bind", "call"].contains(static_property_name) {
                    return;
                }

                let Some(obj_id) = member_expr.object().get_identifier_reference() else {
                    return;
                };

                check(obj_id, parent_call_expr.span, ctx);
            }
            _ => {}
        }
    }
}

fn check(ident: &IdentifierReference, span: Span, ctx: &LintContext) {
    if ident.is_global_reference_name("Function", ctx.scoping()) {
        ctx.diagnostic(no_new_func(span));
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
