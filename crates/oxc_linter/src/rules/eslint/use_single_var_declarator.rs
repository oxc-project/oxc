use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn use_single_var_declarator_diagnostic(span: Span, kind: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Declare variables separately instead of using a single `{kind}` declaration."
    ))
    .with_help("Split this declaration into separate statements, one per variable.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct UseSingleVarDeclarator;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires each variable to be declared in its own statement.
    ///
    /// ### Why is this bad?
    ///
    /// Declaring multiple variables in a single statement can make code harder
    /// to read and maintain. Each variable declaration in its own statement
    /// is clearer and makes diffs more readable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// let a, b;
    /// const x = 1, y = 2;
    /// var foo = 1, bar = 2;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// let a;
    /// let b;
    /// const x = 1;
    /// const y = 2;
    /// ```
    UseSingleVarDeclarator,
    eslint,
    style,
    pending
);

impl Rule for UseSingleVarDeclarator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::VariableDeclaration(decl) = node.kind() else {
            return;
        };

        if decl.declarations.len() <= 1 {
            return;
        }

        // Skip for-loop initializers: `for (let i = 0, j = 0; ...)`
        let parent = ctx.nodes().parent_node(node.id());
        if matches!(parent.kind(), AstKind::ForStatement(_)) {
            return;
        }

        let kind = match decl.kind {
            oxc_ast::ast::VariableDeclarationKind::Var => "var",
            oxc_ast::ast::VariableDeclarationKind::Let => "let",
            oxc_ast::ast::VariableDeclarationKind::Const => "const",
            oxc_ast::ast::VariableDeclarationKind::Using => "using",
            oxc_ast::ast::VariableDeclarationKind::AwaitUsing => "await using",
        };

        ctx.diagnostic(use_single_var_declarator_diagnostic(decl.span, kind));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "let a;",
        "let a = 1;",
        "const a = 1;",
        "var a = 1;",
        "for (let i = 0, j = 0; i < 10; i++) {}",
    ];

    let fail = vec![
        "let a, b;",
        "const x = 1, y = 2;",
        "var foo = 1, bar = 2;",
        "let a = 1, b = 2, c = 3;",
    ];

    Tester::new(UseSingleVarDeclarator::NAME, UseSingleVarDeclarator::PLUGIN, pass, fail)
        .test_and_snapshot();
}
