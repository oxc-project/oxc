use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_new_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use 'new' for side effects.").with_label(span)
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
    eslint,
    suspicious,
);

impl Rule for NoNew {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(expr) = node.kind() else {
            return;
        };

        let mut ancestors = ctx.nodes().ancestor_ids(node.id()).skip(1);
        let Some(node_id) = ancestors.next() else { return };

        let kind = ctx.nodes().kind(node_id);
        if matches!(kind, AstKind::ExpressionStatement(_)) {
            ancestors.next(); // skip `FunctionBody`
            if let Some(node_id) = ancestors.next() {
                let kind = ctx.nodes().kind(node_id);
                if matches!(kind, AstKind::ArrowFunctionExpression(e) if e.expression) {
                    return;
                }
            }
            let span = Span::new(expr.span.start, expr.callee.span().end);
            ctx.diagnostic(no_new_diagnostic(span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var a = new Date()",
        "var a; if (a === new Date()) { a = false; }",
        "(() => new Date())",
    ];

    let fail = vec!["new Date()", "(() => { new Date() })"];

    Tester::new(NoNew::NAME, NoNew::PLUGIN, pass, fail).test_and_snapshot();
}
