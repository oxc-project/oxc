use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_debugger_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(no-debugger): `debugger` statement is not allowed")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct NoDebugger;

declare_oxc_lint!(
    /// ### What it does
    /// Checks for usage of the `debugger` statement
    ///
    /// ### Why is this bad?
    /// `debugger` statements do not affect functionality when a debugger isn't attached.
    /// They're most commonly an accidental debugging leftover.
    ///
    /// ### Example
    /// ```javascript
    /// const data = await getData();
    /// const result = complexCalculation(data);
    /// debugger;
    /// ```
    NoDebugger,
    correctness
);

impl Rule for NoDebugger {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::DebuggerStatement(stmt) = node.kind() {
            ctx.diagnostic_with_fix(no_debugger_diagnostic(stmt.span), |fixer| {
                fixer.delete(&stmt.span)
            });
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![("var test = { debugger: 1 }; test.debugger;", None)];

    let fail = vec![("if (foo) debugger", None)];

    Tester::new(NoDebugger::NAME, pass, fail).test_and_snapshot();
}
