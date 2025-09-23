use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule};

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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// new Person();
    ///
    /// (() => { new Date() })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// var a = new Date()
    ///
    /// (() => new Date())
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

        let mut ancestors = ctx
            .nodes()
            .ancestors(node.id())
            .filter(|a| !matches!(a.kind(), AstKind::ParenthesizedExpression(_)));
        let Some(node) = ancestors.next() else { return };

        if matches!(node.kind(), AstKind::ExpressionStatement(_)) {
            ancestors.next(); // skip `FunctionBody`
            if let Some(node) = ancestors.next()
                && matches!(node.kind(), AstKind::ArrowFunctionExpression(e) if e.expression)
            {
                return;
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

    let fail = vec!["new Date()", "(() => { new Date() })", "(new Date())", "((new Date()))"];

    Tester::new(NoNew::NAME, NoNew::PLUGIN, pass, fail).test_and_snapshot();
}
