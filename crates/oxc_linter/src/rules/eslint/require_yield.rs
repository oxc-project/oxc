use oxc_ast::{
    AstKind,
    ast::{ArrowFunctionExpression, Function, YieldExpression},
};
use oxc_ast_visit::VisitJs;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::scope::ScopeFlags;

use crate::{AstNode, context::LintContext, rule::Rule};

fn require_yield_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("This generator function does not have `yield`")
        .with_help("Add a `yield` expression inside the generator body, or convert it to a regular function if iteration behavior is not needed.")
        .with_label(span)
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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function* foo() {
    ///   return 10;
    /// }
    /// ```
    RequireYield,
    eslint,
    correctness,
    version = "0.0.4",
    short_description = "This rule generates warnings for generator functions that do not have the yield keyword.",
);

impl Rule for RequireYield {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::Function(func) = node.kind()
            && func.generator
            && let Some(body) = func.body.as_ref()
            && !body.statements.is_empty()
        {
            let mut finder = YieldFinder::default();
            if ctx.source_range(body.span).contains("yield") {
                finder.visit_function_body(body);
            }

            if !finder.found {
                let span = func.id.as_ref().map_or_else(|| func.span, |ident| ident.span);
                ctx.diagnostic(require_yield_diagnostic(span));
            }
        }
    }
}

#[derive(Default)]
struct YieldFinder {
    found: bool,
}

impl<'a> VisitJs<'a> for YieldFinder {
    fn visit_yield_expression(&mut self, _expr: &YieldExpression<'a>) {
        self.found = true;
    }

    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _expr: &ArrowFunctionExpression<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function foo() { return 0; }",
        "function* foo() { yield 0; }",
        "function* foo() { yield* foo; }",
        "function* foo() { return; yield 0; }",
        "function* foo() { class Bar { [yield 0]() {} } }",
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
