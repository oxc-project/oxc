use crate::{AstNode, context::LintContext, rule::Rule};
use oxc_ast::AstKind;
use oxc_ast::ast::{Statement, Statement::BlockStatement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

fn no_lonely_if_diagnostic(span: Span, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected `if` as the only statement in an `else` block")
        .with_help("Consider using `else if` instead")
        .with_labels([span, span1])
}

#[derive(Debug, Default, Clone)]
pub struct NoLonelyIf;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow if statements as the only statement in else blocks
    ///
    /// ### Why is this bad?
    ///
    /// If an if statement is the only statement in the else block, it is often clearer to use an
    /// else if form.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    ///
    /// ```js
    /// if (condition) {
    ///   // ...
    /// } else {
    ///   if (anotherCondition) {
    ///     // ...
    ///   }
    /// }
    /// ```
    ///
    /// ```js
    /// if (condition) {
    ///   // ...
    /// } else {
    ///   if (anotherCondition) {
    ///     // ...
    ///   } else {
    ///       // ...
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// if (condition) {
    ///   // ...
    /// } else if (anotherCondition) {
    ///   // ...
    /// }
    /// ```
    ///
    /// ```js
    /// if (condition) {
    ///   // ...
    /// } else if (anotherCondition) {
    ///   // ...
    /// } else {
    ///   // ...
    /// }
    /// ```
    ///
    /// ```js
    /// if (condition) {
    ///   // ...
    /// } else {
    ///   if (anotherCondition) {
    ///       // ...
    ///   }
    ///   doSomething();
    /// }
    /// ```
    NoLonelyIf,
    eslint,
    pedantic,
    pending
);

impl Rule for NoLonelyIf {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IfStatement(if_stmt) = node.kind() else {
            return;
        };

        let Some(ref alt) = if_stmt.alternate else {
            return;
        };

        let BlockStatement(b) = alt else {
            return;
        };

        if let Some(AstKind::IfStatement(_)) = ctx.nodes().parent_node(node.id()).map(AstNode::kind)
        {
            return;
        };

        if b.body.len() == 1 {
            let Some(only_stmt) = b.body.first() else {
                return;
            };

            if let Statement::BlockStatement(b) = only_stmt {
                if b.body.len() == 1 {
                    if let Some(Statement::IfStatement(lone_if)) = b.body.first() {
                        ctx.diagnostic(no_lonely_if_diagnostic(
                            Span::new(lone_if.span.start, lone_if.span.start + 2),
                            Span::new(if_stmt.span.start, if_stmt.span.start + 2),
                        ));
                    };
                }
            };

            if let Statement::IfStatement(lonely_if) = only_stmt {
                ctx.diagnostic(no_lonely_if_diagnostic(
                    Span::new(lonely_if.span.start, lonely_if.span.start + 2),
                    Span::new(if_stmt.span.start, if_stmt.span.start + 2),
                ));
            };
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "if (a) {;} else if (b) {;}",
        "if (a) {;} else { if (b) {;} ; }",
        "if (a) if (a) {} else { if (b) {} } else {}",
    ];

    let fail = vec![
        "if (a) {;} else { if (b) {;} }",
        "if (foo) {} else { if (bar) baz(); }",
        "if (foo) {} else { if (bar) baz() } qux();",
        "if (foo) {} else { if (bar) baz(); } qux();",
    ];

    /* Pending
    let fix = vec![
        ("if (a) {;} else { if (b) {;} }", "if (a) {;} else if (b) {;}", None),
        ("if (foo) {} else { if (bar) baz(); }", "if (foo) {} else if (bar) baz();", None),
        (
            "if (foo) {} else { if (bar) baz(); } qux();",
            "if (foo) {} else if (bar) baz(); qux();",
            None,
        ),
    ];
    */

    Tester::new(NoLonelyIf::NAME, NoLonelyIf::PLUGIN, pass, fail)
        //.expect_fix(fix)
        .test_and_snapshot();
}
