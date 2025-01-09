use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn require_yield_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("This generator function does not have 'yield'").with_label(span)
}

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
    eslint,
    correctness
);

impl Rule for RequireYield {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::Function(func) = node.kind() {
            if !node.flags().has_yield()
                && func.generator
                && func.body.as_ref().is_some_and(|body| !body.statements.is_empty())
            {
                let span = func.id.as_ref().map_or_else(|| func.span, |ident| ident.span);
                ctx.diagnostic(require_yield_diagnostic(span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function foo() { return 0; }",
        "function* foo() { yield 0; }",
        "function* foo() { }",
        "(function* foo() { yield 0; })();",
        "(function* foo() { })();",
        "function* foo() { while (true) { yield 0; } }",
        "var obj = { *foo() { yield 0; } };",
        "var obj = { *foo() { } };",
        "class A { *foo() { yield 0; } };",
        "class A { *foo() { } };",
        "() => {}",
    ];

    let fail = vec![
        "function* foo() { return 0; }",
        "(function* foo() { return 0; })();",
        "var obj = { *foo() { return 0; } }",
        "class A { *foo() { return 0; } }",
        "function* foo() { function* bar() { yield 0; } }",
        "function* foo() { function* bar() { return 0; } yield 0; }",
    ];

    Tester::new(RequireYield::NAME, RequireYield::PLUGIN, pass, fail).test_and_snapshot();
}
