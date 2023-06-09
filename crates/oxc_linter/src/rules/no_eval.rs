use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-eval): eval can be harmful.")]
#[diagnostic(severity(warning))]
struct NoEvalDiagnostic(#[label("eval can be harmful")] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoEval;

declare_oxc_lint!(
    /// ### What it does
    /// Disallows referencing the 'eval' function.
    ///
    /// ### Why is this bad?
    /// Calling 'eval' is not supported in some secure contexts and can lead to
    /// vulnerabilities.
    ///
    /// ### Example
    /// ```javascript
    /// const someString = "console.log('pwned')"
    /// eval(someString);
    /// ```
    NoEval,
    nursery
);

impl Rule for NoEval {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::IdentifierReference(ident) = node.get().kind()
				&& ident.name == "eval"
			{
                ctx.diagnostic(NoEvalDiagnostic(ident.span));
			}
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("this.eval();", None),
        ("globalThis.eval();", None),
        ("asdf.eval();", None),
        ("const asdf = { eval: true };", None),
    ];

    let fail = vec![
        ("eval();", None),
        ("eval('...');", None),
        ("eval('...');", None),
        ("let a = eval;", None),
        ("const foo = { asdf: eval };", None),
    ];

    Tester::new(NoEval::NAME, pass, fail).test_and_snapshot();
}
