use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_continue_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected use of `continue` statement.")
        .with_help("Do not use the `continue` statement.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoContinue;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow `continue` statements
    ///
    /// ### Why is this bad?
    /// The continue statement terminates execution of the statements in the current iteration of the current or labeled loop, and continues execution of the loop with the next iteration. When used incorrectly it makes code less testable, less readable and less maintainable. Structured control flow statements such as if should be used instead.
    ///
    /// ### Example
    /// ```javascript
    /// var sum = 0,
    ///     i;
    ///
    /// for(i = 0; i < 10; i++) {
    ///     if(i >= 5) {
    ///         continue;
    ///     }
    ///
    ///     sum += i;
    /// }
    /// ```
    NoContinue,
    eslint,
    style
);

impl Rule for NoContinue {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ContinueStatement(continue_statement) = node.kind() {
            ctx.diagnostic(no_continue_diagnostic(Span::new(
                continue_statement.span.start,
                continue_statement.span.start + 8,
            )));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var sum = 0, i; for(i = 0; i < 10; i++){ if(i > 5) { sum += i; } }",
        "var sum = 0, i = 0; while(i < 10) { if(i > 5) { sum += i; } i++; }",
    ];

    let fail = vec![
        "var sum = 0, i; for(i = 0; i < 10; i++){ if(i <= 5) { continue; } sum += i; }",
        "var sum = 0, i; myLabel: for(i = 0; i < 10; i++){ if(i <= 5) { continue myLabel; } sum += i; }",
        "var sum = 0, i = 0; while(i < 10) { if(i <= 5) { i++; continue; } sum += i; i++; }",
        "var sum = 0, i = 0; myLabel: while(i < 10) { if(i <= 5) { i++; continue myLabel; } sum += i; i++; }",
    ];

    Tester::new(NoContinue::NAME, NoContinue::PLUGIN, pass, fail).test_and_snapshot();
}
