use oxc_ast::{AstKind, Span};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-debugger): `debugger` statement is not allowed")]
#[diagnostic()]
struct NoDebuggerDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoDebugger;

const RULE_NAME: &str = "no-debugger";

impl Rule for NoDebugger {
    const NAME: &'static str = RULE_NAME;

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::DebuggerStatement(stmt) = node.get().kind() {
            ctx.diagnostic(NoDebuggerDiagnostic(stmt.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["var test = { debugger: 1 }; test.debugger;"];

    let fail = vec!["if (foo) debugger"];

    Tester::new(RULE_NAME, pass, fail).test_and_snapshot();
}
