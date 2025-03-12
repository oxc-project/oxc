use crate::{AstNode, context::LintContext, rule::Rule};
use oxc_ast::AstKind;
use oxc_ast::ast::{Statement, Statement::BlockStatement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

fn no_lonely_if_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected `if` as the only statement in an `else` block")
        .with_help("Consider using `else if` instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoLonelyIf;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `if` statements as the only statement in `else` blocks
    ///
    /// ### Why is this bad?
    ///
    /// When an `if` statement is the only statement in an `else` block, it is often clearer to use
    /// an `else if` instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
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
    ///     // ...
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

        let Some(ref alternate) = if_stmt.alternate else {
            return;
        };

        let BlockStatement(b) = alternate else {
            return;
        };

        let Some(only_stmt) = b.body.first() else {
            return;
        };

        if let Some(AstKind::IfStatement(_)) = ctx.nodes().parent_kind(node.id()) {
            return;
        };

        if b.body.len() == 1 {
            if let Statement::BlockStatement(inner_block) = only_stmt {
                if inner_block.body.len() == 1 {
                    if let Some(Statement::IfStatement(lonely_if)) = inner_block.body.first() {
                        ctx.diagnostic(no_lonely_if_diagnostic(Span::new(
                            lonely_if.span.start,
                            lonely_if.span.start + 2,
                        )));
                    };
                }
            } else if let Statement::IfStatement(lonely_if) = only_stmt {
                ctx.diagnostic(no_lonely_if_diagnostic(Span::new(
                    lonely_if.span.start,
                    lonely_if.span.start + 2,
                )));
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
        "if (a) {
           if (b) {} else { }
         } else {
           if (c) {  }
           if (d) {  }
          }",
    ];

    let fail = vec![
        "if (a) {;} else { if (b) {;} }",
        "if (foo) {} else { if (bar) baz(); }",
        "if (foo) {} else { if (bar) baz() } qux();",
        "if (foo) {} else { if (bar) baz(); } qux();",
        "if (a) {
           foo();
         } else {
           /* otherwise, do the other thing */ if (b) {
             bar();
           }
         }",
        "if (a) {
           foo();
         } else {
           if (b) {
             bar();
           } /* this comment will prevent this test case from being autofixed. */
         }",
        // No fix; removing the braces would cause a SyntaxError.
        "if (foo) {
         } else {
           if (bar) baz();
         }
         qux();",
        // Not fixed; removing the braces would change the semantics due to ASI.
        "if (foo) {
         } else {
           if (bar) baz();
         }
         [1, 2, 3].forEach(foo);",
        // Not fixed; removing the braces would change the semantics due to ASI.
        "if (foo) { } else {
           if (bar) baz++;
         }
         foo;",
        // Not fixed; bar() would be interpreted as a template literal tag
        "if (a) {
           foo();
         } else {
           if (b) bar();
         }
         `template literal`;",
    ];

    /* Pending
    let fix = vec![
        (
            "if (a) {
               foo();
             } else {
               if (b) {
                 bar();
               }
             }",
            "if (a) {
               foo();
             } else if (b) {
               bar();
             }",
            None,
        ),
        (
        "if (a) {
           foo();
         } /* comment */
 else {
           if (b) {
             bar();
           }
         }",
        "if (a) {
           foo();
    } /* comment */
 else {
           if (b) {
             bar();
           }
         }",
            None,
        ),
        (
        "if (a) {
           foo();
         } else {
    if ( /* this comment is ok */
 b) {
             bar();
           }
         }",
        "if (a) {
           foo();
    } else if ( /* this comment is ok */
 b) {
           bar();
         }",
            None,
        ),
        (
        "if (foo) {} else { if (bar) baz(); }",
        "if (foo) {} else if (bar) baz();",
            None,
        ),
        (
        "if (foo) { } else { if (bar) baz(); } qux();",
        "if (foo) { } else if (bar) baz(); qux();",
            None,
        ),
        (
            "if (foo) { } else { if (bar) baz++; } foo;",
            "if (foo) { } else if (bar) baz++; foo;",
            None,
        ),
        (
        "if (a) {
           foo();
         } else {
           if (b) {
             bar();
           } else if (c) {
             baz();
           } else {
             qux();
           }
         }",
        "if (a) {
           foo();
         } else if (b) {
           bar();
         } else if (c) {
           baz();
         } else {
           qux();
         }",
            None,
        ),
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
        //  .expect_fix(fix)
        .test_and_snapshot();
}
