use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("deepscan(missing-throw): Missing throw")]
#[diagnostic(
    severity(warning),
    help("The `throw` keyword seems to be missing in front of this 'new' expression")
)]
struct MissingThrowDiagnostic(#[label] pub Span);

/// `https://deepscan.io/docs/rules/missing-throw`
#[derive(Debug, Default, Clone)]
pub struct MissingThrow;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks whether the `throw` keyword is missing in front of a `new` expression.
    ///
    /// ### Example
    /// ```javascript
    /// function foo() { throw Error() }
    /// const foo = () => { new Error() }
    /// ```
    MissingThrow,
    correctness
);

impl Rule for MissingThrow {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else { return };
        if new_expr.callee.is_specific_id("Error") && Self::has_missing_throw(node, ctx) {
            ctx.diagnostic(MissingThrowDiagnostic(new_expr.span));
        }
    }
}

impl MissingThrow {
    fn has_missing_throw<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
        let mut node_ancestors = ctx.nodes().ancestors(node.id()).skip(1);

        let Some(node_id) = node_ancestors.next() else { return false };

        if matches!(ctx.nodes().kind(node_id), AstKind::ExpressionStatement(_)) {
            for node_id in node_ancestors {
                match ctx.nodes().kind(node_id) {
                    // ignore arrow `const foo = () => new Error()`
                    AstKind::ArrowExpression(arrow_expr) if arrow_expr.expression => return false,
                    AstKind::ArrayExpression(_) | AstKind::Function(_) => break,
                    _ => {}
                }
            }
            return true;
        }

        false
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // Note: lone `Error()` should be caught by no-effect-call
    let pass = vec![
        ("function foo() { throw new Error() }", None),
        ("const foo = () => new Error()", None),
        ("[new Error()]", None),
    ];

    let fail =
        vec![("function foo() { new Error() }", None), ("const foo = () => { new Error() }", None)];

    Tester::new(MissingThrow::NAME, pass, fail).test_and_snapshot();
}
