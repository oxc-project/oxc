use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{ContentEq, GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_self_compare_diagnostic(left_span: Span, right_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Both sides of this comparison are exactly the same")
        .with_help("If you are testing for NaN, you can use the `Number.isNaN()` function.")
        .with_labels([left_span, right_span])
}

#[derive(Debug, Default, Clone)]
pub struct NoSelfCompare;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow comparisons where both sides are exactly the same
    ///
    /// ### Why is this bad?
    ///
    /// Comparing a variable against itself is usually an error, either a typo or refactoring error.
    /// It is confusing to the reader and may potentially introduce a runtime error.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var x = 10;
    /// if (x === x) {
    ///   x = 20;
    /// }
    /// ```
    NoSelfCompare,
    eslint,
    pedantic // The code is not wrong if it is intended to check for NaNs, which is the majority of
             // the case.
);

impl Rule for NoSelfCompare {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(binary_expr) = node.kind() else {
            return;
        };
        if !binary_expr.operator.is_compare() && !binary_expr.operator.is_equality() {
            return;
        }

        if binary_expr
            .left
            .without_parentheses()
            .content_eq(binary_expr.right.without_parentheses())
        {
            ctx.diagnostic(no_self_compare_diagnostic(
                binary_expr.left.span(),
                binary_expr.right.span(),
            ));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("if (x === y) { }", None),
        ("if (1 === 2) { }", None),
        ("y=x*x", None),
        ("foo.bar.baz === foo.bar.qux", None),
        ("class C { #field; foo() { this.#field === this['#field']; } }", None),
        ("class C { #field; foo() { this['#field'] === this.#field; } }", None),
    ];

    let fail = vec![
        ("if (x === x) { }", None),
        ("if (x !== x) { }", None),
        ("if (x > x) { }", None),
        ("if ('x' > 'x') { }", None),
        ("do {} while (x === x)", None),
        ("x === x", None),
        ("x !== x", None),
        ("x == x", None),
        ("x != x", None),
        ("x > x", None),
        ("x < x", None),
        ("x >= x", None),
        ("x <= x", None),
        ("x > (x)", None),
        ("(x) == x", None),
        ("(x) >= ((x))", None),
        ("foo.bar().baz.qux >= foo.bar ().baz .qux", None),
        ("class C { #field; foo() { this.#field === this.#field; } }", None),
    ];

    Tester::new(NoSelfCompare::NAME, NoSelfCompare::PLUGIN, pass, fail).test_and_snapshot();
}
