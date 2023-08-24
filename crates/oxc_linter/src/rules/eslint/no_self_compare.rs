use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{ast_util::calculate_hash, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-self-compare): Disallow comparisons where both sides are exactly the same")]
#[diagnostic(
    severity(warning),
    help("If you are testing for NaN, you can use Number.isNaN function.")
)]
struct NoSelfCompareDiagnostic(#[label] pub Span, #[label] pub Span);

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
    /// ### Example
    /// ```javascript
    /// var x = 10;
    /// if (x === x) {
    ///   x = 20;
    /// }
    /// ```
    NoSelfCompare,
    pedantic // The code is not wrong if it is intended to check for NaNs, which is the majority of
             // the case.
);

impl Rule for NoSelfCompare {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(binary_expr) = node.kind() else { return };
        if !binary_expr.operator.is_compare() && !binary_expr.operator.is_equality() {
            return;
        }
        let left = calculate_hash(&binary_expr.left);
        let right = calculate_hash(&binary_expr.right);

        if left == right {
            ctx.diagnostic(NoSelfCompareDiagnostic(
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
        ("foo.bar().baz.qux >= foo.bar ().baz .qux", None),
        ("class C { #field; foo() { this.#field === this.#field; } }", None),
    ];

    Tester::new(NoSelfCompare::NAME, pass, fail).test_and_snapshot();
}
