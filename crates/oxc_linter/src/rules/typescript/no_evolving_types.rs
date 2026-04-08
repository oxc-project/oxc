use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_evolving_types_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Variable is initialized to a value that will cause its type to evolve.")
        .with_help("Add an explicit type annotation to prevent the type from evolving implicitly.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEvolvingTypes;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows variables that are initialized with values that cause TypeScript
    /// to infer an evolving type (e.g., `any[]`).
    ///
    /// ### Why is this bad?
    ///
    /// When a variable is initialized to an empty array `[]` or `null` without
    /// a type annotation, TypeScript infers an evolving `any[]` or widening type.
    /// This defeats the purpose of type checking.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// let arr = [];
    /// let val = null;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// let arr: string[] = [];
    /// let val: string | null = null;
    /// const arr = [1, 2, 3];
    /// ```
    NoEvolvingTypes,
    typescript,
    suspicious,
    pending
);

impl Rule for NoEvolvingTypes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !ctx.source_type().is_typescript() {
            return;
        }

        let AstKind::VariableDeclarator(declarator) = node.kind() else {
            return;
        };

        // Skip if there's a type annotation
        if declarator.type_annotation.is_some() {
            return;
        }

        // Only check `let` declarations (const/var have different semantics)
        let parent = ctx.nodes().parent_node(node.id());
        if let AstKind::VariableDeclaration(decl) = parent.kind() {
            if decl.kind != oxc_ast::ast::VariableDeclarationKind::Let {
                return;
            }
        }

        let Some(init) = &declarator.init else {
            return;
        };

        match init {
            // Empty array literal: `let x = []`
            Expression::ArrayExpression(arr) if arr.elements.is_empty() => {
                ctx.diagnostic(no_evolving_types_diagnostic(declarator.span));
            }
            // null literal: `let x = null`
            Expression::NullLiteral(_) => {
                ctx.diagnostic(no_evolving_types_diagnostic(declarator.span));
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("let arr: string[] = [];", None),
        ("let val: string | null = null;", None),
        ("const arr = [];", None),      // const is fine
        ("let arr = [1, 2, 3];", None), // non-empty array
        ("let x = 'hello';", None),     // not evolving
        ("let x;", None),               // no init
    ];

    let fail = vec![("let arr = [];", None), ("let val = null;", None)];

    Tester::new(NoEvolvingTypes::NAME, NoEvolvingTypes::PLUGIN, pass, fail).test_and_snapshot();
}
