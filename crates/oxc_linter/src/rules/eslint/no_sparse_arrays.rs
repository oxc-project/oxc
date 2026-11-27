use oxc_ast::{AstKind, ast::ArrayExpressionElement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_sparse_arrays_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected comma in middle of array")
        .with_help("remove the comma or insert `undefined`")
        .with_label(span.label("unexpected comma"))
}

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
    correctness,
    recommended,
    version = "0.0.4",
    short_description = "Disallow sparse arrays.",
);

impl Rule for NoSparseArrays {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ArrayExpression(array_expr) = node.kind() else {
            return;
        };
        for element in &array_expr.elements {
            if let ArrayExpressionElement::Elision(elision) = element {
                ctx.diagnostic(no_sparse_arrays_diagnostic(elision.span));
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
        "var a = [ 1, , hello, , 2];",
        "const a = [\n  ,\n  ,\n];",
        "const a = [\n  // eslint-disable-next-line no-sparse-arrays\n  ,\n  ,\n];",
    ];

    Tester::new(NoSparseArrays::NAME, NoSparseArrays::PLUGIN, pass, fail).test_and_snapshot();
}
