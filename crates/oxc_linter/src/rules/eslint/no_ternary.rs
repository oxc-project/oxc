use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_ternary_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected use of ternary expression")
        .with_help("Do not use the ternary expression.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoTernary;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow ternary operators
    ///
    /// ### Why is this bad?
    ///
    /// The ternary operator is used to conditionally assign a value to a
    /// variable. Some believe that the use of ternary operators leads to
    /// unclear code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var foo = isBar ? baz : qux;
    /// ```
    ///
    /// ```javascript
    /// function quux() {
    ///   return foo ? bar() : baz();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// let foo;
    ///
    /// if (isBar) {
    ///     foo = baz;
    /// } else {
    ///     foo = qux;
    /// }
    /// ```
    ///
    /// ```javascript
    /// function quux() {
    ///     if (foo) {
    ///         return bar();
    ///     } else {
    ///         return baz();
    ///     }
    /// }
    /// ```
    NoTernary,
    eslint,
    style
);

impl Rule for NoTernary {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ConditionalExpression(cond_expr) = node.kind() {
            ctx.diagnostic(no_ternary_diagnostic(cond_expr.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#""x ? y";"#,
        "if (true) { thing() } else { stuff() };",
        "let foo; if (isBar) { foo = baz; } else { foo = qux; };",
        "function quux() { if (foo) { return bar(); } else { return baz(); } }",
    ];

    let fail = vec![
        "var foo = true ? thing : stuff;",
        "true ? thing() : stuff();",
        "function foo(bar) { return bar ? baz : qux; }",
    ];

    Tester::new(NoTernary::NAME, NoTernary::PLUGIN, pass, fail).test_and_snapshot();
}
