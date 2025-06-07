use oxc_ast::{
    AstKind,
    ast::{Statement, VariableDeclarationKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_case_declarations_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected lexical declaration in case block.").with_label(span)
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
    pedantic
);

impl Rule for NoCaseDeclarations {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::SwitchCase(switch_case) = node.kind() {
            let consequent = &switch_case.consequent;

            for stmt in consequent {
                match stmt {
                    Statement::FunctionDeclaration(d) => {
                        let start = d.span.start;
                        let end = start + 8;
                        ctx.diagnostic(no_case_declarations_diagnostic(Span::new(start, end)));
                    }
                    Statement::ClassDeclaration(d) => {
                        let start = d.span.start;
                        let end = start + 5;
                        ctx.diagnostic(no_case_declarations_diagnostic(Span::new(start, end)));
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
                        let end = start + len;
                        ctx.diagnostic(no_case_declarations_diagnostic(Span::new(start, end)));
                    }
                    _ => {}
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

    Tester::new(NoCaseDeclarations::NAME, NoCaseDeclarations::PLUGIN, pass, fail)
        .test_and_snapshot();
}
