use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_label_var_diagnostic(name: &str, id_span: Span, label_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Found identifier '{name}' with the same name as a label."))
        .with_labels([
            id_span.label(format!("Identifier '{name}' found here.")),
            label_span.label("Label with the same name."),
        ])
}

#[derive(Debug, Default, Clone)]
pub struct NoLabelVar;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow labels that share a name with a variable.
    ///
    /// ### Why is this bad?
    ///
    /// This rule aims to create clearer code by disallowing the bad practice of creating a label
    /// that shares a name with a variable that is in scope.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var x = foo;
    /// function bar() {
    /// x:
    ///   for (;;) {
    ///     break x;
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// // The variable that has the same name as the label is not in scope.
    ///
    /// function foo() {
    ///     var q = t;
    /// }
    ///
    /// function bar() {
    /// q:
    ///     for(;;) {
    ///         break q;
    ///     }
    /// }
    /// ```
    NoLabelVar,
    eslint,
    style,
);

impl Rule for NoLabelVar {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::LabeledStatement(labeled_stmt) = node.kind() else { return };

        if let Some(symbol_id) =
            ctx.scopes().find_binding(node.scope_id(), &labeled_stmt.label.name)
        {
            let decl_span = ctx.symbols().get_span(symbol_id);
            let label_decl = labeled_stmt.span.start;
            ctx.diagnostic(no_label_var_diagnostic(
                &labeled_stmt.label.name,
                decl_span,
                Span::new(label_decl, label_decl + 1),
            ));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function bar() { q: for(;;) { break q; } } function foo () { var q = t; }",
        "function bar() { var x = foo; q: for(;;) { break q; } }",
    ];

    let fail = vec![
        "var x = foo; function bar() { x: for(;;) { break x; } }",
        "function bar() { var x = foo; x: for(;;) { break x; } }",
        "function bar(x) { x: for(;;) { break x; } }",
    ];

    Tester::new(NoLabelVar::NAME, NoLabelVar::PLUGIN, pass, fail).test_and_snapshot();
}
