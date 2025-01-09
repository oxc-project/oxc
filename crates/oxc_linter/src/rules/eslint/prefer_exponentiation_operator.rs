use oxc_ast::{ast::Expression, match_member_expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    ast_util::is_method_call, context::LintContext, globals::GLOBAL_OBJECT_NAMES, rule::Rule,
    AstNode,
};

#[derive(Debug, Default, Clone)]
pub struct PreferExponentiationOperator;

fn prefer_exponentian_operator_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `**` over `Math.pow`.").with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the use of Math.pow in favor of the ** operator
    ///
    /// ### Why is this bad?
    ///
    /// Introduced in ES2016, the infix exponentiation operator ** is an alternative for the
    /// standard Math.pow function. Infix notation is considered to be more readable and thus more
    /// preferable than the function notation.
    ///
    /// ### Example
    /// ```javascript
    /// Math.pow(a, b)
    /// ```
    PreferExponentiationOperator,
    eslint,
    style,
);

impl Rule for PreferExponentiationOperator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_method_call(call_expr, None, Some(&["pow"]), Some(2), Some(2)) {
            return;
        };

        let Some(member_expr) = call_expr.callee.as_member_expression() else {
            return;
        };

        let member_expor_obj = member_expr.object();

        match member_expor_obj {
            Expression::Identifier(ident) => {
                if ident.name.as_str() == "Math"
                    && ctx.semantic().is_reference_to_global_variable(ident)
                {
                    ctx.diagnostic(prefer_exponentian_operator_diagnostic(call_expr.span));
                }
            }
            match_member_expression!(Expression) => {
                let member_expr = member_expor_obj.to_member_expression();
                let Some(static_prop_name) = member_expr.static_property_name() else {
                    return;
                };
                if static_prop_name != "Math" {
                    return;
                }

                if let Expression::Identifier(ident) = member_expr.object().without_parentheses() {
                    if GLOBAL_OBJECT_NAMES.contains(ident.name.as_str())
                        && ctx.semantic().is_reference_to_global_variable(ident)
                    {
                        ctx.diagnostic(prefer_exponentian_operator_diagnostic(call_expr.span));
                    }
                }
            }
            _ => {}
        };
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Object.pow(a, b)",
        "Math.max(a, b)",
        "Math",
        "Math(a, b)",
        "pow",
        "pow(a, b)",
        "Math.pow",
        "Math.Pow(a, b)",
        "math.pow(a, b)",
        "foo.Math.pow(a, b)",
        "new Math.pow(a, b)",
        "Math[pow](a, b)",
        "globalThis.Object.pow(a, b)",
        "globalThis.Math.max(a, b)",
        // "/* globals Math:off*/ Math.pow(a, b)",
        "let Math; Math.pow(a, b);",
        "if (foo) { const Math = 1; Math.pow(a, b); }",
        "var x = function Math() { Math.pow(a, b); }",
        "function foo(Math) { Math.pow(a, b); }",
        "function foo() { Math.pow(a, b); var Math; }",
        "
			                var globalThis = bar;
			                globalThis.Math.pow(a, b)
			            ",
        "class C { #pow; foo() { Math.#pow(a, b); } }",
    ];

    let fail = vec![
        "globalThis.Math.pow(a, b)",
        "globalThis.Math['pow'](a, b)",
        "Math.pow(a, b) + Math.pow(c,
			 d)",
        "Math.pow(Math.pow(a, b), Math.pow(c, d))",
        "Math.pow(a, b)**Math.pow(c, d)",
        "Math.pow(a, b as any)",
        "Math.pow(a as any, b)",
        "Math.pow(a, b) as any",
    ];

    Tester::new(
        PreferExponentiationOperator::NAME,
        PreferExponentiationOperator::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
