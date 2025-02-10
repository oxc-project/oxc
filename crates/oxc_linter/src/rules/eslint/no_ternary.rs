use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_ternary_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected use of ternary expression")
        .with_help("Do not use the ternary expression.")
        .with_label(span)
}

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
    eslint,
    style
);

impl Rule for NoTernary {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ConditionalExpression(cond_expr) = node.kind() {
            ctx.diagnostic(no_ternary_diagnostic(Span::new(
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

    Tester::new(NoTernary::NAME, NoTernary::PLUGIN, pass, fail).test_and_snapshot();
}
