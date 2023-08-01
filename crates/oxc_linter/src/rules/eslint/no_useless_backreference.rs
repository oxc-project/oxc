use oxc_ast::AstKind;

use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-useless-backreference): Unnecessary backreference {0:?}")]
#[diagnostic(severity(warning))]
struct NoUselessBackreferenceDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoUselessBackreference;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoUselessBackreference,
    correctness
);

impl Rule for NoUselessBackreference {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::RegExpLiteral(reg_exp) = node.kind() else { return };
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["/^(?:(a)|(b)\\2)$/;"];

    let fail = vec!["/^(?:(a)|\\1b)$/;"];

    Tester::new_without_config(NoUselessBackreference::NAME, pass, fail).test_and_snapshot();
}
