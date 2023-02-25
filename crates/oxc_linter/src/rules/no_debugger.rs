use oxc_ast::{AstKind, Span};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-debugger): `debugger` statement is not allowed")]
#[diagnostic()]
struct NoDebuggerDiagnostic(#[label] pub Span);

#[derive(Debug, Default)]
pub struct NoDebugger;

impl Rule for NoDebugger {
    fn run<'a>(&self, kind: AstKind<'a>, ctx: &LintContext<'a>) {
        if let AstKind::DebuggerStatement(stmt) = kind {
            ctx.diagnostic(NoDebuggerDiagnostic(stmt.span));
        }
    }
}
