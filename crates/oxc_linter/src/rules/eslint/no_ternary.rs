use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-ternary): Unexpected use of ternary expression")]
#[diagnostic(severity(warning), help("Do not use the ternary expression."))]
struct NoTernaryDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoTernary;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow ternary operators
    ///
    /// ### Why is this bad?
    /// The ternary operator is used to conditionally assign a value to a variable. Some believe that the use of ternary operators leads to unclear code.
    ///
    /// ### Example
    /// ```javascript
    /// var foo = isBar ? baz : qux;
    //
    // function quux() {
    //   return foo ? bar() : baz();
    // }
    /// ```
    NoTernary,
    style
);

impl Rule for NoTernary {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ConditionalExpression(cond_expr) = node.kind() {
            ctx.diagnostic(NoTernaryDiagnostic(Span::new(
                cond_expr.span.start,
                cond_expr.span.end,
            )));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![r#""x ? y";"#, "if (true) { thing() } else { stuff() };"];

    let fail = vec![
        "var foo = true ? thing : stuff;",
        "true ? thing() : stuff();",
        "function foo(bar) { return bar ? baz : qux; }",
    ];

    Tester::new(NoTernary::NAME, pass, fail).test_and_snapshot();
}
