use oxc_ast::{AstKind, ast::ArrayExpressionElement};
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;

use crate::{AstNode, context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct NoSparseArrays;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow sparse arrays.
    ///
    /// ### Why is this bad?
    ///
    /// Take the following example:
    ///
    /// ```javascript
    /// const items = [,,];
    /// ```
    ///
    /// While the items array in this example has a length of 2, there are actually
    /// no values in items[0] or items[1]. The fact that the array literal is
    /// valid with only commas inside, coupled with the length being set and
    /// actual item values not being set, make sparse arrays confusing for many
    /// developers.
    ///
    /// The confusion around sparse arrays is enough that it’s recommended to
    /// avoid using them unless you are certain that they are useful in your
    /// code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var items = [,,];
    /// ```
    ///
    /// ```javascript
    /// var colors = [ "red",, "blue" ];
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// var items = [];
    /// ```
    ///
    /// // trailing comma (after the last element) is not a problem
    /// ```javascript
    /// var colors = [ "red", "blue", ];
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

    let pass = vec!["var a = [ 1, 2, ]", "var a = [];"];

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
