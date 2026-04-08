use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn use_collapsed_if_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("This `if` statement can be collapsed into the outer `if` statement.")
        .with_help("Combine conditions using `&&` to reduce nesting.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct UseCollapsedIf;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces collapsing nested `if` statements into a single one with `&&`.
    ///
    /// ### Why is this bad?
    ///
    /// Unnecessary nesting makes code harder to read. When an `if` statement's
    /// body contains only another `if` statement (with no `else`), the two
    /// conditions can be combined with `&&`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if (a) {
    ///   if (b) {
    ///     doSomething();
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// if (a && b) {
    ///   doSomething();
    /// }
    /// ```
    UseCollapsedIf,
    unicorn,
    style,
    pending
);

impl Rule for UseCollapsedIf {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IfStatement(if_stmt) = node.kind() else {
            return;
        };

        // The outer if must not have an else clause
        if if_stmt.alternate.is_some() {
            return;
        }

        let Statement::BlockStatement(block) = &if_stmt.consequent else {
            return;
        };

        // The block must contain exactly one statement
        if block.body.len() != 1 {
            return;
        }

        let Statement::IfStatement(inner_if) = &block.body[0] else {
            return;
        };

        // The inner if must also not have an else clause
        if inner_if.alternate.is_some() {
            return;
        }

        ctx.diagnostic(use_collapsed_if_diagnostic(inner_if.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "if (a && b) { doSomething(); }",
        "if (a) { doSomething(); } else { doOther(); }",
        "if (a) { if (b) { doSomething(); } else { doOther(); } }",
        "if (a) { doSomething(); doOtherThing(); }",
        "if (a) { if (b) { doSomething(); } doOther(); }",
    ];

    let fail = vec![
        "if (a) { if (b) { doSomething(); } }",
        "if (x > 0) { if (y > 0) { console.log('both positive'); } }",
    ];

    Tester::new(UseCollapsedIf::NAME, UseCollapsedIf::PLUGIN, pass, fail).test_and_snapshot();
}
