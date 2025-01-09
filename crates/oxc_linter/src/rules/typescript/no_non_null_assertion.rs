use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

#[derive(Debug, Default, Clone)]
pub struct NoNonNullAssertion;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow non-null assertions using the ! postfix operator.
    ///
    /// ### Why is this bad?
    /// TypeScript's ! non-null assertion operator asserts to the type system that an expression is non-nullable, as in not null or undefined. Using assertions to tell the type system new information is often a sign that code is not fully type-safe. It's generally better to structure program logic so that TypeScript understands when values may be nullable.
    ///
    /// ### Example
    /// ```ts
    /// x!;
    /// x!.y;
    /// x.y!;
    /// ```
    NoNonNullAssertion,
    typescript,
    restriction,
);

fn no_non_null_assertion_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Forbidden non-null assertion.")
        .with_help("Consider using the optional chain operator `?.` instead. This operator includes runtime checks, so it is safer than the compile-only non-null assertion operator.")
        .with_label(span)
}

impl Rule for NoNonNullAssertion {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSNonNullExpression(expr) = node.kind() else { return };
        ctx.diagnostic(no_non_null_assertion_diagnostic(expr.span));
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["x;", "x.y;", "x.y.z;", "x?.y.z;", "x?.y?.z;", "!x;"];

    let fail = vec![
        "x!;",
        "x!.y;",
        "x.y!;",
        "!x!.y;",
        "x!.y?.z;",
        "x![y];",
        "x![y]?.z;",
        "x.y.z!();",
        "x.y?.z!();",
        "x!!!;",
        "x!!.y;",
        "x.y!!;",
        "x.y.z!!();",
        "x!?.[y].z;",
        "x!?.y.z;",
        "x.y.z!?.();",
        "
        	x!
        	.y
        	      ",
        "
        	x!
        	// comment
        	.y
        	      ",
        "
        	x!
        	 // comment
        	    . /* comment */
        	      y
        	      ",
        "
        	x!
        	 // comment
        	     /* comment */ ['y']
        	      ",
    ];

    Tester::new(NoNonNullAssertion::NAME, NoNonNullAssertion::PLUGIN, pass, fail)
        .test_and_snapshot();
}
