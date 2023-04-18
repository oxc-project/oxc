use oxc_ast::{ AstKind, Span};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("Bad bitwise operator")]
#[diagnostic(
    severity(warning),
    help("Bitwise operator '{0}' seems unintended. Did you mean logical operator '{1}'?")
)]
struct BadArrayMethodOnArgumentsDiagnostic(&'static str, &'static str, #[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Bad array method on arguments")]
#[diagnostic(
    severity(warning),
    help(
        "The 'arguments' object does not have '{0}()' method. If an array method was intended, consider converting the 'arguments' object to an array or using ES6 rest parameter instead."
    )
)]
struct BadBitwiseOrOperatorDiagnostic(&'static str, #[label] pub Span);

/// `https://deepscan.io/docs/rules/bad-array-method-on-arguments`
#[derive(Debug, Default, Clone)]
pub struct BadArrayMethodOnArguments;

declare_oxc_lint!(
    /// ### What it does
    /// This rule applies when an array method is called on the arguments object itself.
    ///
    /// ### Why is this bad?
    /// The arguments object is not an array, but an array-like object. It should be converted to a real array before calling an array method.
    /// Otherwise, a TypeError exception will be thrown because of the non-existent method.
    ///
    /// ### Example
    /// ```javascript
    /// function add(x, y) {
    ///   return x + y;
    /// }
    /// function sum() {
    ///   return arguments.reduce(add, 0);
    /// }
    /// ```
    BadArrayMethodOnArguments,
    correctness,
);

impl Rule for BadArrayMethodOnArguments {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.get().kind() {}
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![];

    let fail = vec![];

    Tester::new(BadArrayMethodOnArguments::NAME, pass, fail).test_and_snapshot();
}
