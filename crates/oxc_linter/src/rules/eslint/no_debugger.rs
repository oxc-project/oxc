use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_debugger_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`debugger` statement is not allowed").with_label(span)
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
    ///
    /// ```javascript
    /// async function main() {
    ///     const data = await getData();
    ///     const result = complexCalculation(data);
    ///     debugger;
    /// }
    /// ```
    NoDebugger,
    eslint,
    correctness,
    fix
);

impl Rule for NoDebugger {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::DebuggerStatement(stmt) = node.kind() {
            ctx.diagnostic_with_fix(no_debugger_diagnostic(stmt.span), |fixer| {
                let Some(parent) = ctx
                    .nodes()
                    .ancestors(node.id())
                    .skip(1)
                    .find(|p| !matches!(p.kind(), AstKind::ParenthesizedExpression(_)))
                else {
                    return fixer.delete(&stmt.span);
                };

                // For statements like `if (foo) debugger;`, we can't just
                // delete the `debugger` statement; we need to replace it with an empty block.
                match parent.kind() {
                    AstKind::IfStatement(_)
                    | AstKind::WhileStatement(_)
                    | AstKind::ForStatement(_)
                    | AstKind::ForInStatement(_)
                    | AstKind::ForOfStatement(_) => fixer.replace(stmt.span, "{}"),
                    // NOTE: no need to check for
                    // AstKind::ArrowFunctionExpression because
                    // `const x = () => debugger` is a parse error
                    _ => fixer.delete(&stmt.span),
                }
            });
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![("var test = { debugger: 1 }; test.debugger;", None)];

    let fail = vec![("if (foo) debugger", None)];
    let fix = vec![
        ("let x; debugger; let y;", "let x;  let y;", None),
        ("if (foo) debugger", "if (foo) {}", None),
        ("for (;;) debugger", "for (;;) {}", None),
        ("while (i > 0) debugger", "while (i > 0) {}", None),
        ("if (foo) { debugger; }", "if (foo) {  }", None),
        ("if (foo) { debugger }", "if (foo) {  }", None),
    ];

    Tester::new(NoDebugger::NAME, NoDebugger::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
