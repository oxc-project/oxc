use oxc_ast::AstKind;
use oxc_ast::ast::Statement;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_useless_continue_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unnecessary continue statement.")
        .with_help("Remove this continue statement since it is the last statement in the loop body and has no effect.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessContinue;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows unnecessary `continue` statements at the end of loop bodies.
    ///
    /// ### Why is this bad?
    ///
    /// A `continue` statement at the end of a loop body is redundant because
    /// the loop will continue to the next iteration automatically.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// for (let i = 0; i < 10; i++) {
    ///   doSomething(i);
    ///   continue;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// for (let i = 0; i < 10; i++) {
    ///   doSomething(i);
    /// }
    /// ```
    NoUselessContinue,
    eslint,
    suspicious,
    pending
);

impl Rule for NoUselessContinue {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ContinueStatement(cont) = node.kind() else {
            return;
        };

        // Only flag unlabeled continues
        if cont.label.is_some() {
            return;
        }

        let parent = ctx.nodes().parent_node(node.id());

        // The continue must be the last statement in a block that is directly
        // the body of a loop
        let AstKind::BlockStatement(block) = parent.kind() else {
            return;
        };

        // Check that continue is the last statement
        let Some(last) = block.body.last() else {
            return;
        };
        let Statement::ContinueStatement(last_cont) = last else {
            return;
        };
        if last_cont.span != cont.span {
            return;
        }

        // Verify the block is directly the body of a loop
        let grandparent = ctx.nodes().parent_node(parent.id());
        match grandparent.kind() {
            AstKind::ForStatement(_)
            | AstKind::ForInStatement(_)
            | AstKind::ForOfStatement(_)
            | AstKind::WhileStatement(_)
            | AstKind::DoWhileStatement(_) => {
                ctx.diagnostic(no_useless_continue_diagnostic(cont.span));
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "for (let i = 0; i < 10; i++) { if (i > 5) { continue; } doSomething(i); }",
        "for (let i = 0; i < 10; i++) { doSomething(i); }",
        "while (true) { if (cond) continue; doSomething(); }",
        "for (let i = 0; i < 10; i++) { if (i > 5) continue; }",
        // labeled continue is allowed
        "outer: for (let i = 0; i < 10; i++) { for (let j = 0; j < 10; j++) { continue outer; } }",
    ];

    let fail = vec![
        "for (let i = 0; i < 10; i++) { doSomething(i); continue; }",
        "while (true) { doSomething(); continue; }",
        "for (const x of arr) { process(x); continue; }",
        "for (const k in obj) { process(k); continue; }",
        "do { doSomething(); continue; } while (true);",
    ];

    Tester::new(NoUselessContinue::NAME, NoUselessContinue::PLUGIN, pass, fail).test_and_snapshot();
}
