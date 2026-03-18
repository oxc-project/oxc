use oxc_ast::{AstKind, ast::Statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn guard_for_in_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Require `for-in` loops to include an `if` statement")
        .with_help("The body of a for-in should be wrapped in an if statement to filter unwanted properties from the prototype.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct GuardForIn;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require for-in loops to include an if statement.
    ///
    /// ### Why is this bad?
    ///
    /// Looping over objects with a `for in` loop will include properties that are inherited through
    /// the prototype chain. Using a `for in` loop without filtering the results in the loop can
    /// lead to unexpected items in your for loop which can then lead to unexpected behaviour.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```javascript
    /// for (key in foo) {
    ///   doSomething(key);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// for (key in foo) {
    ///   if (Object.hasOwn(foo, key)) {
    ///     doSomething(key);
    ///   }
    /// }
    /// ```
    ///
    /// ```javascript
    /// for (key in foo) {
    ///   if (Object.prototype.hasOwnProperty.call(foo, key)) {
    ///     doSomething(key);
    ///   }
    /// }
    /// ```
    ///
    /// ```javascript
    /// for (key in foo) {
    ///    if ({}.hasOwnProperty.call(foo, key)) {
    ///        doSomething(key);
    ///    }
    /// }
    /// ```
    GuardForIn,
    eslint,
    style
);

impl Rule for GuardForIn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ForInStatement(for_in_statement) = node.kind() {
            match for_in_statement.body.kind() {
                StatementKind::EmptyStatement(_) | StatementKind::IfStatement(_) => return,
                StatementKind::BlockStatement(block_body) if block_body.body.is_empty() => return,
                StatementKind::BlockStatement(block_body)
                    if block_body.body.len() == 1
                        && block_body.body[0].is_if_statement() =>
                {
                    return;
                }
                StatementKind::BlockStatement(block_body) if !block_body.body.is_empty() => {
                    let block_statement = &block_body.body[0];
                    if let Some(i) = block_statement.as_if_statement() {
                        if let Some(_) = i.consequent.as_continue_statement() {
                            return;
                        }
                        if let Some(consequent_block) = i.consequent.as_block_statement()
                            && consequent_block.body.len() == 1
                            && &consequent_block.body[0].is_continue_statement()
                        {
                            return;
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
