use oxc_ast::{ast::ArrayExpressionElement, AstKind};
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule, AstNode};

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
    eslint,
    correctness
);

impl Rule for NoSparseArrays {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ArrayExpression(array_expr) = node.kind() {
            let violations = array_expr
                .elements
                .iter()
                .filter_map(|el| match el {
                    ArrayExpressionElement::Elision(elision) => Some(elision),
                    _ => None,
                })
                .map(|elision| {
                    LabeledSpan::at(
                        (elision.span.start as usize)..(elision.span.start as usize),
                        "unexpected comma",
                    )
                })
                .collect::<Vec<_>>();

            if !violations.is_empty() {
                if violations.len() < 10 {
                    ctx.diagnostic(
                        OxcDiagnostic::warn("Unexpected comma in middle of array")
                            .with_help("remove the comma or insert `undefined`")
                            .with_labels(violations),
                    );
                } else {
                    let span = if (array_expr.span.end - array_expr.span.start) < 50 {
                        LabeledSpan::at(array_expr.span, "the array here")
                    } else {
                        LabeledSpan::at(
                            (array_expr.span.start as usize)..(array_expr.span.start as usize),
                            "the array starting here",
                        )
                    };

                    ctx.diagnostic(
                        OxcDiagnostic::warn(format!(
                            "{} unexpected commas in middle of array",
                            violations.len()
                        ))
                        .with_help("remove the comma or insert `undefined`")
                        .with_label(span),
                    );
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

    Tester::new(NoSparseArrays::NAME, NoSparseArrays::PLUGIN, pass, fail).test_and_snapshot();
}
