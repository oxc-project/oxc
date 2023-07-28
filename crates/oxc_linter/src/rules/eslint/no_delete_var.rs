use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::UnaryOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-delete-var): variables should not be deleted")]
#[diagnostic(severity(warning))]
struct NoDeleteVarDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoDeleteVar;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// The purpose of the delete operator is to remove a property from an object.
    ///
    /// ### Why is this bad?
    ///
    /// Using the delete operator on a variable might lead to unexpected behavior.
    ///
    /// ### Example
    /// ```javascript
    /// var x;
    /// delete x;
    /// ```
    NoDeleteVar,
    correctness
);

impl Rule for NoDeleteVar {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::UnaryExpression(expr) = node.kind() else { return };
        if expr.operator == UnaryOperator::Delete && expr.argument.is_identifier_reference() {
            ctx.diagnostic(NoDeleteVarDiagnostic(expr.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![("delete x.prop;", None)];

    let fail = vec![("delete x", None)];

    Tester::new(NoDeleteVar::NAME, pass, fail).test_and_snapshot();
}
