use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_implicit_any_let_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Variable declaration without type annotation or initializer implicitly has `any` type.",
    )
    .with_help("Add an explicit type annotation or initialize the variable.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoImplicitAnyLet;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows `let` declarations without type annotations or initializers
    /// in TypeScript files, as they implicitly have the `any` type.
    ///
    /// ### Why is this bad?
    ///
    /// A `let` declaration without a type annotation or initializer has an
    /// implicit `any` type in TypeScript. This defeats the purpose of using
    /// TypeScript for type safety.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// let x;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// let x: string;
    /// let y = 'hello';
    /// let z: number;
    /// ```
    NoImplicitAnyLet,
    typescript,
    suspicious,
    pending
);

impl Rule for NoImplicitAnyLet {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !ctx.source_type().is_typescript() {
            return;
        }

        let AstKind::VariableDeclarator(declarator) = node.kind() else {
            return;
        };

        // Only check if there's no type annotation AND no initializer
        if declarator.type_annotation.is_some() || declarator.init.is_some() {
            return;
        }

        // Only flag `let` declarations
        let parent = ctx.nodes().parent_node(node.id());
        if let AstKind::VariableDeclaration(decl) = parent.kind() {
            if decl.kind == oxc_ast::ast::VariableDeclarationKind::Let {
                ctx.diagnostic(no_implicit_any_let_diagnostic(declarator.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("let x: string;", None),
        ("let y = 'hello';", None),
        ("let z: number;", None),
        ("var x;", None),       // only let
        ("const x = 1;", None), // const requires init
    ];

    let fail = vec![("let x;", None), ("let a, b;", None)];

    Tester::new(NoImplicitAnyLet::NAME, NoImplicitAnyLet::PLUGIN, pass, fail).test_and_snapshot();
}
