use oxc_ast::{AstKind, ast::VariableDeclarationKind};
// use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
// use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

// fn block_scoped_var_diagnostic(span: Span) -> OxcDiagnostic {
//     // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
//     OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
//         .with_help("Should be a command-like statement that tells the user how to fix the issue")
//         .with_label(span)
// }

#[derive(Debug, Default, Clone)]
pub struct BlockScopedVar;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Generates warnings when variables are used outside of the block in which they were defined.
    /// This emulates C-style block scope.
    ///
    /// ### Why is this bad?
    ///
    /// This rule aims to reduce the usage of variables outside of their binding context
    /// and emulate traditional block scope from other languages.
    /// This is to help newcomers to the language avoid difficult bugs with variable hoisting.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// function doIf() {
    ///     if (true) {
    ///         var build = true;
    ///     }
    ///     console.log(build);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// function doIf() {
    ///     var build;
    ///     if (true) {
    ///         build = true;
    ///    }
    ///     console.log(build);
    /// }
    ///
    /// ```
    BlockScopedVar,
    eslint,
    suspicious,
);

impl Rule for BlockScopedVar {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::VariableDeclaration(decl) = node.kind() else {
            return;
        };
        if decl.kind != VariableDeclarationKind::Var {
            return;
        }
        let declarations = &decl.declarations;
        for item in declarations {
            let id = &item.id;
            for ident in id.get_binding_identifiers() {
                let Some(symbol_id) =
                    ctx.scoping().find_binding(node.scope_id(), &ident.name.as_str())
                else {
                    continue;
                };
                let bind_scope_id = ctx.symbol_scope(symbol_id);
                // for _scope in ctx.scoping().scope_ancestors(node.scope_id()) {
                // }

                // for _reference in ctx.scoping().get_resolved_references(ident.symbol_id()) {
                // }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![];

    let fail = vec![
        r"
            if (true) {
                var build = true;
            }
            console.log(build);
            let t = build;
            
        ",
    ];

    Tester::new(BlockScopedVar::NAME, BlockScopedVar::PLUGIN, pass, fail).test_and_snapshot();
}
