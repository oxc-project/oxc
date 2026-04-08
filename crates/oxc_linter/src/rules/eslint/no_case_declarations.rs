use oxc_ast::{
    AstKind,
    ast::{Statement, VariableDeclarationKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_case_declarations_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected lexical declaration in case block.")
        .with_help("Wrap the case body in braces `{}` to create an explicit block scope for the lexical declaration.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoCaseDeclarations;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow lexical declarations in case clauses.
    ///
    /// ### Why is this bad?
    ///
    /// The reason is that the lexical declaration is visible
    /// in the entire switch block but it only gets initialized when it is assigned,
    /// which will only happen if the case where it is defined is reached.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// switch (foo) {
    ///   case 1:
    ///       let x = 1;
    ///       break;
    ///   case 2:
    ///       const y = 2;
    ///       break;
    ///   case 3:
    ///       function f() {}
    ///       break;
    ///   default:
    ///       class C {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// switch (foo) {
    ///   case 1: {
    ///       let x = 1;
    ///       break;
    ///   }
    ///   case 2: {
    ///       const y = 2;
    ///       break;
    ///   }
    ///   case 3: {
    ///       function f() {}
    ///       break;
    ///   }
    ///   default: {
    ///       class C {}
    ///   }
    /// }
    /// ```
    NoCaseDeclarations,
    eslint,
    pedantic,
    suggestion
);

impl Rule for NoCaseDeclarations {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::SwitchCase(switch_case) = node.kind() {
            let consequent = &switch_case.consequent;

            // Collect all declaration spans first, then emit a single suggestion
            // wrapping the entire body in braces (avoids duplicate suggestions
            // when multiple declarations exist in the same case).
            let mut has_emitted_fix = false;

            for stmt in consequent {
                let diag_span = match stmt {
                    Statement::FunctionDeclaration(d) => {
                        let start = d.span.start;
                        Some(Span::new(start, start + 8))
                    }
                    Statement::ClassDeclaration(d) => {
                        let start = d.span.start;
                        Some(Span::new(start, start + 5))
                    }
                    Statement::VariableDeclaration(var) if var.kind.is_lexical() => {
                        let start = var.span.start;
                        let len = match var.kind {
                            VariableDeclarationKind::Const | VariableDeclarationKind::Using => 5,
                            VariableDeclarationKind::Let => 3,
                            #[expect(clippy::cast_possible_truncation)]
                            VariableDeclarationKind::AwaitUsing => {
                                ctx.source_range(Span::new(start, var.declarations[0].span.start))
                                    .trim_end()
                                    .len() as u32
                            }
                            VariableDeclarationKind::Var => unreachable!(),
                        };
                        Some(Span::new(start, start + len))
                    }
                    _ => None,
                };

                if let Some(span) = diag_span {
                    if !has_emitted_fix {
                        // Get the span covering all consequent statements
                        let first_start = consequent.first().unwrap().span().start;
                        let last_end = consequent.last().unwrap().span().end;
                        let body_span = Span::new(first_start, last_end);

                        ctx.diagnostic_with_suggestion(
                            no_case_declarations_diagnostic(span),
                            |fixer| {
                                let body_text = fixer.source_range(body_span);
                                fixer.replace(body_span, format!("{{ {body_text} }}"))
                            },
                        );
                        has_emitted_fix = true;
                    } else {
                        ctx.diagnostic(no_case_declarations_diagnostic(span));
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("switch (a) { case 1: { let x = 1; break; } default: { let x = 2; break; } }", None),
        ("switch (a) { case 1: { const x = 1; break; } default: { const x = 2; break; } }", None),
        (
            "switch (a) { case 1: { function f() {} break; } default: { function f() {} break; } }",
            None,
        ),
        ("switch (a) { case 1: { class C {} break; } default: { class C {} break; } }", None),
    ];

    let fail = vec![
        ("switch (a) { case 1: let x = 1; break; }", None),
        ("switch (a) { default: let x = 2; break; }", None),
        ("switch (a) { case 1: const x = 1; break; }", None),
        ("switch (a) { default: const x = 2; break; }", None),
        ("switch (a) { case 1: function f() {} break; }", None),
        ("switch (a) { default: function f() {} break; }", None),
        ("switch (a) { case 1: class C {} break; }", None),
        ("switch (a) { default: class C {} break; }", None),
        ("switch (a) { default: using x = {}; break; }", None),
        ("switch (a) { default: await using x = {}; break; }", None),
    ];

    let fix = vec![
        (
            "switch (a) { case 1: let x = 1; break; }",
            "switch (a) { case 1: { let x = 1; break; } }",
            None,
        ),
        (
            "switch (a) { default: const x = 2; break; }",
            "switch (a) { default: { const x = 2; break; } }",
            None,
        ),
        (
            "switch (a) { case 1: function f() {} break; }",
            "switch (a) { case 1: { function f() {} break; } }",
            None,
        ),
        (
            "switch (a) { case 1: class C {} break; }",
            "switch (a) { case 1: { class C {} break; } }",
            None,
        ),
    ];

    Tester::new(NoCaseDeclarations::NAME, NoCaseDeclarations::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
