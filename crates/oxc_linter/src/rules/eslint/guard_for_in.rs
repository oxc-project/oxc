use oxc_ast::{ast::Statement, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn guard_for_in_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Require `for-in` loops to include an `if` statement")
        .with_help("The body of a for-in should be wrapped in an if statement to filter unwanted properties from the prototype.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct GuardForIn;

declare_oxc_lint!(
    /// ### What it does
    /// This rule is aimed at preventing unexpected behavior that could arise from using a for in loop without filtering the results in the loop. As such, it will warn when for in loops do not filter their results with an if statement.
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// for (key in foo) {
    ///     doSomething(key);
    /// }
    /// ```
    GuardForIn,
    eslint,
    style
);

impl Rule for GuardForIn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ForInStatement(for_in_statement) = node.kind() {
            match &for_in_statement.body {
                Statement::EmptyStatement(_) | Statement::IfStatement(_) => return,
                Statement::BlockStatement(block_body) if block_body.body.is_empty() => return,
                Statement::BlockStatement(block_body)
                    if block_body.body.len() == 1
                        && matches!(block_body.body[0], Statement::IfStatement(_)) =>
                {
                    return;
                }
                Statement::BlockStatement(block_body) if block_body.body.len() >= 1 => {
                    let block_statement = &block_body.body[0];
                    if let Statement::IfStatement(i) = block_statement {
                        if let Statement::ContinueStatement(_) = &i.consequent {
                            return;
                        }
                        if let Statement::BlockStatement(consequent_block) = &i.consequent {
                            if consequent_block.body.len() == 1
                                && matches!(
                                    &consequent_block.body[0],
                                    Statement::ContinueStatement(_)
                                )
                            {
                                return;
                            }
                        }
                    }
                }
                _ => {}
            }
            ctx.diagnostic(guard_for_in_diagnostic(Span::new(
                for_in_statement.span.start,
                for_in_statement.right.span().end + 1,
            )));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "for (var x in o);",
        "for (var x in o) {}",
        "for (var x in o) if (x) f();",
        "for (var x in o) { if (x) { f(); } }",
        "for (var x in o) { if (x) continue; f(); }",
        "for (var x in o) { if (x) { continue; } f(); }",
    ];

    let fail = vec![
        "for (var x in o) { if (x) { f(); continue; } g(); }",
        "for (var x in o) { if (x) { continue; f(); } g(); }",
        "for (var x in o) { if (x) { f(); } g(); }",
        "for (var x in o) { if (x) f(); g(); }",
        "for (var x in o) { foo() }",
        "for (var x in o) foo();",
    ];

    Tester::new(GuardForIn::NAME, GuardForIn::PLUGIN, pass, fail).test_and_snapshot();
}
