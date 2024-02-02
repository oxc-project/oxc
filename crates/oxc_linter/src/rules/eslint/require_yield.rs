use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(require-yield): This generator function does not have 'yield'")]
#[diagnostic(severity(warning))]
struct RequireYieldDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct RequireYield;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule generates warnings for generator functions that do not have the yield keyword.
    ///
    /// ### Why is this bad?
    ///
    /// Probably a mistake.
    ///
    /// ### Example
    /// ```javascript
    /// function* foo() {
    ///   return 10;
    /// }
    /// ```
    RequireYield,
    correctness
);

impl Rule for RequireYield {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let kind = node.kind();

        if node.flags().has_yield() {
            return;
        }

        let span = match kind {
            AstKind::Function(func)
                if func.generator
                    && func.body.as_ref().is_some_and(|body| !body.statements.is_empty()) =>
            {
                func.id.as_ref().map_or_else(|| kind.span(), |ident| ident.span)
            }
            AstKind::ArrowExpression(arrow) if !arrow.body.statements.is_empty() => arrow.span,
            _ => return,
        };

        ctx.diagnostic(RequireYieldDiagnostic(span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function foo() { return 0; }", None),
        ("function* foo() { yield 0; }", None),
        ("function* foo() { }", None),
        ("(function* foo() { yield 0; })();", None),
        ("(function* foo() { })();", None),
        ("var obj = { *foo() { yield 0; } };", None),
        ("var obj = { *foo() { } };", None),
        ("class A { *foo() { yield 0; } };", None),
        ("class A { *foo() { } };", None),
    ];

    let fail = vec![
        ("function* foo() { return 0; }", None),
        ("(function* foo() { return 0; })();", None),
        ("var obj = { *foo() { return 0; } }", None),
        ("class A { *foo() { return 0; } }", None),
        ("function* foo() { function* bar() { yield 0; } }", None),
        ("function* foo() { function* bar() { return 0; } yield 0; }", None),
    ];

    Tester::new(RequireYield::NAME, pass, fail).test_and_snapshot();
}
