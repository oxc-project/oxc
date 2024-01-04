use miette::{miette, LabeledSpan};
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
        if let AstKind::ArrayExpression(array_expr) = node.kind() {
            let violations = array_expr
                .elements
                .iter()
                .filter_map(|el| match el {
                    ArrayExpressionElement::Elision(span) => Some(span),
                    _ => None,
                })
                .map(|span| {
                    LabeledSpan::at(
                        (span.start as usize)..(span.start as usize),
                        "unexpected comma",
                    )
                })
                .collect::<Vec<_>>();

            if !violations.is_empty() {
                if violations.len() < 10 {
                    ctx.diagnostic(miette!(
                        labels = violations,
                        help = "remove the comma or insert `undefined`",
                        "eslint(no-sparse-arrays): Unexpected comma in middle of array"
                    ));
                } else {
                    let span = if (array_expr.span.end - array_expr.span.start) < 50 {
                        LabeledSpan::at(array_expr.span, "the array here")
                    } else {
                        LabeledSpan::at(
                            (array_expr.span.start as usize)..(array_expr.span.start as usize),
                            "the array starting here",
                        )
                    };

                    ctx.diagnostic(miette!(
                        labels = vec![span],
                        help = "remove the comma or insert `undefined`",
                        "eslint(no-sparse-arrays): {} unexpected commas in middle of array",
                        violations.len()
                    ));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["var a = [ 1, 2, ]"];

    let fail = vec![
        "var a = [,];",
        "var a = [ 1,, 2];",
        "var a = [ 1,,,, 2];",
        "var a = [ 1,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,, 2];",
        "var a = [ 1, , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , ,  2];",
        "var a = [ 1, , , , , , , , , , , , , , , , , , , , , , , , , , hello, , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , ,  2];",
        "var a = [ 1, , , , , , , , , , , , , , , , , , , , , , , , , ,
        
        
        hello, , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , , 
        
        
        , , , , , , , , , , , , , , , , , , ,  2];",
    ];

    Tester::new_without_config(NoSparseArrays::NAME, pass, fail).test_and_snapshot();
}
