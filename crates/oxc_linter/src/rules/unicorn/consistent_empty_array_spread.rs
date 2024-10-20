use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{ast_util::outermost_paren_parent, context::LintContext, rule::Rule, AstNode};

fn consistent_empty_array_spread_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer consistent types when spreading a ternary in an array literal.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentEmptyArraySpread;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When spreading a ternary in an array, we can use both [] and '' as fallbacks,
    /// but it's better to have consistent types in both branches.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const array = [
    ///    a,
    ///    ...(foo ? [b, c] : ''),
    /// ];
    ///
    /// const array = [
    /// 	a,
    /// 	...(foo ? 'bc' : []),
    /// ];
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const array = [
    ///    a,
    ///    ...(foo ? [b, c] : []),
    /// ];
    ///
    /// const array = [
    /// 	a,
    /// 	...(foo ? 'bc' : ''),
    /// ];
    /// ```
    ConsistentEmptyArraySpread,
    pedantic,
    suggestion
);

impl Rule for ConsistentEmptyArraySpread {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ConditionalExpression(conditional_expr) = node.kind() else {
            return;
        };

        let Some(parent) = outermost_paren_parent(node, ctx) else {
            return;
        };

        let AstKind::SpreadElement(_) = parent.kind() else {
            return;
        };

        let Some(parent) = outermost_paren_parent(parent, ctx) else {
            return;
        };

        let AstKind::ArrayExpressionElement(_) = parent.kind() else {
            return;
        };

        match (
            conditional_expr.consequent.get_inner_expression(),
            conditional_expr.alternate.get_inner_expression(),
        ) {
            (Expression::ArrayExpression(_), Expression::StringLiteral(right_str_lit)) => {
                if right_str_lit.value.is_empty() {
                    ctx.diagnostic_with_suggestion(
                        consistent_empty_array_spread_diagnostic(conditional_expr.span),
                        |fixer| fixer.replace(right_str_lit.span, "[]"),
                    );
                }
            }
            (Expression::StringLiteral(_), Expression::ArrayExpression(right_array_expr)) => {
                if right_array_expr.elements.is_empty() {
                    ctx.diagnostic_with_suggestion(
                        consistent_empty_array_spread_diagnostic(conditional_expr.span),
                        |fixer| fixer.replace(right_array_expr.span, "''"),
                    );
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const array = foo ? [b, c] : []",
        r"const array = foo ? 'bc' : ''",
        r"const obj = { a, ...(foo ? {d: [b, c]} : {}) }",
        r"const obj = { a, ...(foo ? {d: 'bc'} : {}) }",
        r"const array = [ a, ...(foo ? [b, c] : []) ]",
        r"const array = [ a, ...(foo ? 'bc' : '') ]",
    ];

    let fail: Vec<&str> = vec![
        r"const array = [ a, ...(foo ? [b, c] : '') ]",
        r"const array = [ a, ...(foo ? 'bc' : []) ]",
        r"const array = [ a, ...(foo ? ['str', 'str'] : '') ]",
        r"const array = [ a, ...(foo ? [1, 2, 3] : '') ]",
        r"const array = [ {}, ...(foo ? [{}, {}] : '') ]",
    ];

    let fix = vec![
        (
            r"const array = [ a, ...(foo ? [b, c] : '') ]",
            r"const array = [ a, ...(foo ? [b, c] : []) ]",
            None,
        ),
        (
            r"const array = [ a, ...(foo ? 'bc' : []) ]",
            r"const array = [ a, ...(foo ? 'bc' : '') ]",
            None,
        ),
        (
            r"const array = [ a, ...(foo ? ['str', 'str', 'str'] : '') ]",
            r"const array = [ a, ...(foo ? ['str', 'str', 'str'] : []) ]",
            None,
        ),
        (
            r"const array = [ a, ...(foo ? [1, 2, 3] : '') ]",
            r"const array = [ a, ...(foo ? [1, 2, 3] : []) ]",
            None,
        ),
        (
            r"const array = [ {}, ...(foo ? [{}, {}, {}] : '') ]",
            r"const array = [ {}, ...(foo ? [{}, {}, {}] : []) ]",
            None,
        ),
        (
            r"const array = [ a, ...(foo ? [b, c] : ''), b, ...(foo ? 'bc' : []), c, ...(foo ? [1, 2, 3] : '') ]",
            r"const array = [ a, ...(foo ? [b, c] : []), b, ...(foo ? 'bc' : ''), c, ...(foo ? [1, 2, 3] : []) ]",
            None,
        ),
    ];

    Tester::new(ConsistentEmptyArraySpread::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
