use oxc_ast::{ast::ArrayExpressionElement, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-sparse-arrays): Unexpected comma in middle of array")]
#[diagnostic(severity(warning), help("remove the comma or insert `undefined`"))]
struct NoSparseArraysDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoSparseArrays;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow sparse arrays.
    ///
    /// ### Why is this bad?
    ///
    /// The confusion around sparse arrays is enough that it’s recommended to avoid using them unless you are certain that they are useful in your code.
    ///
    /// ### Example
    /// ```javascript
    /// var items = [,,];
    /// var colors = [ "red",, "blue" ];
    /// ```
    NoSparseArrays,
    correctness
);

impl Rule for NoSparseArrays {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ArrayExpressionElement(ArrayExpressionElement::Elision(span)) = node.kind()
        {
            ctx.diagnostic(NoSparseArraysDiagnostic(Span::new(span.start, span.start)));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![("var a = [ 1, 2, ]", None)];

    let fail = vec![("var a = [,];", None), ("var a = [ 1,, 2];", None)];

    Tester::new(NoSparseArrays::NAME, pass, fail).test_and_snapshot();
}
