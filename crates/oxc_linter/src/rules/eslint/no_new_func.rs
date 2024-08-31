use oxc_ast::{ast::IdentifierReference, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_new_func(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The Function constructor is eval.").with_label(span)
}

fn is_global_function_reference(ctx: &LintContext, id: &IdentifierReference) -> bool {
    if let Some(reference_id) = id.reference_id() {
        return id.name == "Function" && ctx.symbols().is_global_reference(reference_id);
    }
    false
}

#[derive(Debug, Default, Clone)]
pub struct NoNewFunc;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// The rule disallow `new` operators with the `Function` object.
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
    style
);

impl Rule for NoNewFunc {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let id_and_span = match node.kind() {
            AstKind::NewExpression(new_expr) => {
                let Some(id) = new_expr.callee.get_identifier_reference() else {
                    return;
                };

                Some((id, new_expr.span))
            }
            AstKind::CallExpression(call_expr) => {
                let Some(obj_id) = call_expr.callee.get_identifier_reference() else {
                    return;
                };

                Some((obj_id, call_expr.span))
            }
            AstKind::MemberExpression(mem_expr) => {
                let parent: Option<&AstNode<'a>> =
                    ctx.nodes().iter_parents(node.id()).skip(1).find(|node| {
                        !matches!(
                            node.kind(),
                            AstKind::ChainExpression(_) | AstKind::ParenthesizedExpression(_)
                        )
                    });

                let Some(AstKind::CallExpression(parent_call_expr)) = parent.map(AstNode::kind)
                else {
                    return;
                };

                let Some(static_property_name) = &mem_expr.static_property_name() else {
                    return;
                };

                if !["apply", "bind", "call"].contains(static_property_name) {
                    return;
                }

                let Some(obj_id) = mem_expr.object().get_identifier_reference() else {
                    return;
                };

                Some((obj_id, parent_call_expr.span))
            }
            _ => None,
        };

        if let Some((id, span)) = id_and_span {
            if is_global_function_reference(ctx, id) {
                ctx.diagnostic(no_new_func(span));
            }
        }
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

    Tester::new(NoNewFunc::NAME, pass, fail).test_and_snapshot();
}
