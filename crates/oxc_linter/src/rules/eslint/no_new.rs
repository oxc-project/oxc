use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_new_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(no-new): Do not use 'new' for side effects.")
        .with_labels([span0.into()])
}

#[derive(Debug, Default, Clone)]
pub struct NoNew;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow new operators outside of assignments or comparisons.
    ///
    /// ### Why is this bad?
    ///
    /// Calling new without assigning or comparing it the reference is thrown away and in many
    /// cases the constructor can be replaced with a function.
    ///
    /// ### Example
    /// ```javascript
    /// new Person();
    /// ```
    NoNew,
    suspicious,
);

impl Rule for NoNew {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(expr) = node.kind() else {
            return;
        };

        if let Some(parent_node) = ctx.nodes().parent_node(node.id()) {
            if matches!(parent_node.kind(), AstKind::ExpressionStatement(_)) {
                ctx.diagnostic(no_new_diagnostic(expr.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["var a = new Date()", "var a; if (a === new Date()) { a = false; }"];

    let fail = vec!["new Date()"];

    Tester::new(NoNew::NAME, pass, fail).test_and_snapshot();
}
