use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::Span;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

#[derive(Debug, Default, Clone)]
pub struct NoNonNullAssertedNullishCoalescing;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow non-null assertions in the left operand of a nullish coalescing operator.
    ///
    /// ### Why is this bad?
    /// The ?? nullish coalescing runtime operator allows providing a default value when dealing with null or undefined. Using a ! non-null assertion type operator in the left operand of a nullish coalescing operator is redundant, and likely a sign of programmer error or confusion over the two operators.
    ///
    /// ### Example
    /// ```ts
    /// foo! ?? bar;
    ///
    /// let x: string;
    /// x! ?? '';
    /// ```
    NoNonNullAssertedNullishCoalescing,
    typescript,
    restriction,
);

fn no_non_null_asserted_nullish_coalescing_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("'Disallow non-null assertions in the left operand of a nullish coalescing operator")
        .with_help("The nullish coalescing operator is designed to handle undefined and null - using a non-null assertion is not needed.")
        .with_label(span)
}

impl Rule for NoNonNullAssertedNullishCoalescing {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::LogicalExpression(expr) = node.kind() else { return };
        let Expression::TSNonNullExpression(ts_non_null_expr) = &expr.left else { return };
        if let Expression::Identifier(ident) = &ts_non_null_expr.expression {
            if let Some(symbol_id) = ctx.scopes().get_binding(node.scope_id(), &ident.name) {
                if !has_assignment_before_node(symbol_id, ctx, expr.span.end) {
                    return;
                }
            }
        }

        ctx.diagnostic(no_non_null_asserted_nullish_coalescing_diagnostic(ts_non_null_expr.span));
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}
fn has_assignment_before_node(
    symbol_id: SymbolId,
    ctx: &LintContext,
    parent_span_end: u32,
) -> bool {
    let symbol_table = ctx.semantic().symbols();

    for reference in symbol_table.get_resolved_references(symbol_id) {
        if reference.is_write() && ctx.semantic().reference_span(reference).end < parent_span_end {
            return true;
        }
    }

    let declaration_id = symbol_table.get_declaration(symbol_id);
    let AstKind::VariableDeclarator(decl) = ctx.nodes().kind(declaration_id) else {
        return false;
    };
    decl.definite || decl.init.is_some()
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo ?? bar;",
        "foo ?? bar!;",
        "foo.bazz ?? bar;",
        "foo.bazz ?? bar!;",
        "foo!.bazz ?? bar;",
        "foo!.bazz ?? bar!;",
        "foo() ?? bar;",
        "foo() ?? bar!;",
        "(foo ?? bar)!;",
        "
        	      let x: string;
        	      x! ?? '';
        	    ",
        "
        	      let x: string;
        	      x ?? '';
        	    ",
        "
        	      let x!: string;
        	      x ?? '';
        	    ",
        "
        	      let x: string;
        	      foo(x);
        	      x! ?? '';
        	    ",
        "
        	      let x: string;
        	      x! ?? '';
        	      x = foo();
        	    ",
        "
        	      let x: string;
        	      foo(x);
        	      x! ?? '';
        	      x = foo();
        	    ",
        "
        	      let x = foo();
        	      x ?? '';
        	    ",
        "
        	      function foo() {
        	        let x: string;
        	        return x ?? '';
        	      }
        	    ",
        "
        	      let x: string;
        	      function foo() {
        	        return x ?? '';
        	      }
        	    ",
    ];

    let fail = vec![
        "foo! ?? bar;",
        "foo! ?? bar!;",
        "foo.bazz! ?? bar;",
        "foo.bazz! ?? bar!;",
        "foo!.bazz! ?? bar;",
        "foo!.bazz! ?? bar!;",
        "foo()! ?? bar;",
        "foo()! ?? bar!;",
        "
        	let x!: string;
        	x! ?? '';
        	      ",
        "
        	let x: string;
        	x = foo();
        	x! ?? '';
        	      ",
        "
        	let x: string;
        	x = foo();
        	x! ?? '';
        	x = foo();
        	      ",
        "
        	let x = foo();
        	x! ?? '';
        	      ",
        "
        	function foo() {
        	  let x!: string;
        	  return x! ?? '';
        	}
        	      ",
        "
        	let x!: string;
        	function foo() {
        	  return x! ?? '';
        	}
        	      ",
        "
        	let x = foo();
        	x  ! ?? '';
        	      ",
    ];

    Tester::new(
        NoNonNullAssertedNullishCoalescing::NAME,
        NoNonNullAssertedNullishCoalescing::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
