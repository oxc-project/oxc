use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn use_while_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `while` instead of `for` with no init and update.")
        .with_help("Replace `for(;condition;)` with `while(condition)` for clarity.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct UseWhile;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of `while` loops instead of `for` loops when there
    /// is no initializer and no update expression.
    ///
    /// ### Why is this bad?
    ///
    /// A `for` loop without an initializer and update expression is
    /// functionally identical to a `while` loop but less readable.
    /// Using `while` makes the intent clearer.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// for (; x < 10;) { x++; }
    /// for (;;) { break; }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// while (x < 10) { x++; }
    /// while (true) { break; }
    /// for (let i = 0; i < 10; i++) {}
    /// ```
    UseWhile,
    eslint,
    style,
    pending
);

impl Rule for UseWhile {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ForStatement(for_stmt) = node.kind() else {
            return;
        };

        if for_stmt.init.is_none() && for_stmt.update.is_none() {
            ctx.diagnostic(use_while_diagnostic(Span::new(
                for_stmt.span.start,
                for_stmt.span.start + 3, // "for"
            )));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "while (true) {}",
        "while (x < 10) { x++; }",
        "for (let i = 0; i < 10; i++) {}",
        "for (;; i++) {}",      // has update
        "for (let i = 0;;) {}", // has init
    ];

    let fail =
        vec!["for (; x < 10;) { x++; }", "for (;;) { break; }", "for (; true;) { doSomething(); }"];

    Tester::new(UseWhile::NAME, UseWhile::PLUGIN, pass, fail).test_and_snapshot();
}
