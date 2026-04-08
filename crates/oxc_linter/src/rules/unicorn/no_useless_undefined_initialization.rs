use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_useless_undefined_initialization_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unnecessary initialization to `undefined`.")
        .with_help("Remove `= undefined` since `let` and `var` declarations are already `undefined` by default.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessUndefinedInitialization;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows initializing variables to `undefined`.
    ///
    /// ### Why is this bad?
    ///
    /// A `let` or `var` variable declaration without an initializer is already
    /// `undefined`. Explicitly initializing to `undefined` is redundant.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// let x = undefined;
    /// var y = undefined;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// let x;
    /// var y;
    /// const z = undefined; // const requires an initializer
    /// ```
    NoUselessUndefinedInitialization,
    unicorn,
    style,
    pending
);

impl Rule for NoUselessUndefinedInitialization {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::VariableDeclaration(decl) = node.kind() else {
            return;
        };

        // const requires an initializer, so skip it
        if decl.kind == oxc_ast::ast::VariableDeclarationKind::Const {
            return;
        }

        for declarator in &decl.declarations {
            let Some(init) = &declarator.init else {
                continue;
            };

            if let Expression::Identifier(ident) = init {
                if ident.name == "undefined" {
                    ctx.diagnostic(no_useless_undefined_initialization_diagnostic(ident.span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["let x;", "var y;", "const z = undefined;", "let a = 1;", "let b = null;"];

    let fail = vec!["let x = undefined;", "var y = undefined;", "let a = 1, b = undefined;"];

    Tester::new(
        NoUselessUndefinedInitialization::NAME,
        NoUselessUndefinedInitialization::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
