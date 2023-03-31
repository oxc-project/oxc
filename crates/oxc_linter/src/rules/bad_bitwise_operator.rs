use oxc_ast::{AstKind, Span};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, fixer::Fix, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("deepscan(BAD_BITWISE_OPERATOR): Bitwise operator '&' seems unintended.")]
#[diagnostic(severity(warning, help("Did you mean logical operator '&&'?")))]
struct BadBitwiseOperatorDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct BadBitwiseOperator;

declare_oxc_lint!(
    /// ### What it does
    /// This rule applies when bitwise operators are used where logical operators are expected.
    ///
    /// ### Why is this bad?
    /// The use of bitwise operators in JavaScript is very rare and often `&` or `|` is simply a mistyped `&&` or `||`,
    /// it is obvious that logical operators are expected in the following code patterns:
    /// ```javascript
    /// e && e.x
    /// e || {}
    /// e || ''
    /// ```
    ///
    /// ### Example
    /// ```javascript
    /// if (obj & obj.prop) { // BAD_BITWISE_OPERATOR alarm
    ///  console.log(obj.prop);
    /// }
    /// ```
    BadBitwiseOperator,
    correctness
);

impl Rule for BadBitwiseOperator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    // let pass = vec![("var test = { debugger: 1 }; test.debugger;", None)];

    // let fail = vec![("if (foo) debugger", None)];

    Tester::new(BadBitwiseOperator::NAME, pass, fail).test_and_snapshot();
}
