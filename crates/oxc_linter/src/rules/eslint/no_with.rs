use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_with_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected use of `with` statement.")
        .with_help("Do not use the `with` statement.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoWith;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow `with` statements
    ///
    /// ### Why is this bad?
    /// The with statement is potentially problematic because it adds members of an object to the current scope, making it impossible to tell what a variable inside the block actually refers to.
    ///
    /// ### Example
    /// ```javascript
    /// with (point) {
    ///     r = Math.sqrt(x * x + y * y); // is r a member of point?
    /// }
    /// ```
    NoWith,
    eslint,
    correctness
);

impl Rule for NoWith {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::WithStatement(with_statement) = node.kind() {
            ctx.diagnostic(no_with_diagnostic(Span::new(
                with_statement.span.start,
                with_statement.span.start + 4,
            )));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["foo.bar()"];

    let fail = vec!["with(foo) { bar() }"];

    Tester::new(NoWith::NAME, NoWith::PLUGIN, pass, fail).test_and_snapshot();
}
