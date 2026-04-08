use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_confusing_labels_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Label '{name}' is confusing because it shares a name with a variable in scope."))
        .with_help("Use a different name for this label to avoid confusion with the variable of the same name.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoConfusingLabels;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows labels that share a name with a variable in scope.
    ///
    /// ### Why is this bad?
    ///
    /// Using the same name for both a label and a variable can be confusing
    /// and makes code harder to understand. It may lead developers to
    /// mistake references to one for the other.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// let x = 1;
    /// x: while (true) { break x; }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// let x = 1;
    /// loop1: while (true) { break loop1; }
    /// ```
    NoConfusingLabels,
    eslint,
    suspicious
);

impl Rule for NoConfusingLabels {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::LabeledStatement(labeled) = node.kind() else {
            return;
        };

        if ctx.scoping().find_binding(node.scope_id(), labeled.label.name).is_some() {
            ctx.diagnostic(no_confusing_labels_diagnostic(
                labeled.label.span,
                labeled.label.name.as_str(),
            ));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "loop1: while (true) { break loop1; }",
        "let x = 1; loop1: while (true) { break loop1; }",
        "myLabel: for (;;) { break myLabel; }",
    ];

    let fail = vec![
        "let x = 1; x: while (true) { break x; }",
        "var foo = 1; foo: while (true) { break foo; }",
        "const bar = 1; bar: for (;;) { break bar; }",
    ];

    Tester::new(NoConfusingLabels::NAME, NoConfusingLabels::PLUGIN, pass, fail).test_and_snapshot();
}
