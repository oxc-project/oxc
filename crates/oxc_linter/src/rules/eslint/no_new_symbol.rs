use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-new-symbol): Disallow new operators with the Symbol object")]
#[diagnostic(
    severity(warning),
    help(
        "Symbol is not intended to be used with the new operator, but to be called as a function. Consider removing the new operator."
    )
)]
struct NoNewSymbolDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoNewSymbol;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow new operators with the Symbol object
    ///
    ///
    /// ### Why is this bad?
    ///
    /// Symbol is not intended to be used with the new operator, but to be called as a function.
    ///
    /// ### Example
    /// ```javascript
    /// var foo = new Symbol('foo');
    /// ```
    NoNewSymbol,
    correctness
);

impl Rule for NoNewSymbol {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(expr) = node.kind() else { return };
        let Expression::Identifier(ident) = &expr.callee else { return };
        if ident.name == "Symbol" && ctx.semantic().is_reference_to_global_variable(ident) {
            let start = expr.span.start;
            let end = start + 3;
            ctx.diagnostic(NoNewSymbolDiagnostic(Span::new(start, end)));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var foo = Symbol('foo');", None),
        ("function bar(Symbol) { var baz = new Symbol('baz');}", None),
        ("function Symbol() {} new Symbol();", None),
        ("new foo(Symbol);", None),
        ("new foo(bar, Symbol);", None),
    ];

    let fail = vec![
        ("var foo = new Symbol('foo');", None),
        ("function bar() { return function Symbol() {}; } var baz = new Symbol('baz');", None),
    ];

    Tester::new(NoNewSymbol::NAME, pass, fail).test_and_snapshot();
}
