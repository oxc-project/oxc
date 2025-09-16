use oxc_ast::{AstKind, ast::BindingIdentifier};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn no_unsafe_declaration_merging_diagnostic(span: Span, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unsafe declaration merging between classes and interfaces.")
        .with_help("The TypeScript compiler doesn't check whether properties are initialized, which can lead to TypeScript not detecting code that will cause runtime errors.")
        .with_labels(if span < span1 { [span, span1]} else { [span1, span] })
}

#[derive(Debug, Default, Clone)]
pub struct NoUnsafeDeclarationMerging;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unsafe declaration merging.
    ///
    /// ### Why is this bad?
    ///
    /// Declaration merging between classes and interfaces is unsafe.
    /// The TypeScript compiler doesn't check whether properties are initialized, which can lead to TypeScript not detecting code that will cause runtime errors.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// interface Foo {}
    /// class Foo {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// interface Foo {}
    /// class Bar {}
    /// ```
    NoUnsafeDeclarationMerging,
    typescript,
    correctness
);

impl Rule for NoUnsafeDeclarationMerging {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Class(decl) => {
                if let Some(ident) = decl.id.as_ref() {
                    for symbol_id in ctx.scoping().get_bindings(node.scope_id()).values() {
                        if let AstKind::TSInterfaceDeclaration(scope_interface) =
                            get_symbol_kind(*symbol_id, ctx)
                        {
                            check_and_diagnostic(ident, &scope_interface.id, ctx);
                        }
                    }
                }
            }
            AstKind::TSInterfaceDeclaration(decl) => {
                for symbol_id in ctx.scoping().get_bindings(node.scope_id()).values() {
                    if let AstKind::Class(scope_class) = get_symbol_kind(*symbol_id, ctx) {
                        if let Some(scope_class_ident) = scope_class.id.as_ref() {
                            check_and_diagnostic(&decl.id, scope_class_ident, ctx);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn check_and_diagnostic(
    ident: &BindingIdentifier,
    scope_ident: &BindingIdentifier,
    ctx: &LintContext<'_>,
) {
    if scope_ident.name.as_str() == ident.name.as_str() {
        ctx.diagnostic(no_unsafe_declaration_merging_diagnostic(ident.span, scope_ident.span));
    }
}

fn get_symbol_kind<'a>(symbol_id: SymbolId, ctx: &LintContext<'a>) -> AstKind<'a> {
    ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id)).kind()
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			interface Foo {}
			class Bar implements Foo {}
			    ",
            None,
        ),
        (
            "
         			namespace Foo {}
         			namespace Foo {}
         			    ",
            None,
        ),
        (
            "
         			enum Foo {}
         			namespace Foo {}
         			    ",
            None,
        ),
        (
            "
         			namespace Fooo {}
         			function Foo() {}
         			    ",
            None,
        ),
        (
            "
         			const Foo = class {};
         			    ",
            None,
        ),
        (
            "
         			interface Foo {
         			  props: string;
         			}

         			function bar() {
         			  return class Foo {};
         			}
         			    ",
            None,
        ),
        (
            "
         			interface Foo {
         			  props: string;
         			}

         			(function bar() {
         			  class Foo {}
         			})();
         			    ",
            None,
        ),
        (
            "
         			declare global {
         			  interface Foo {}
         			}

         			class Foo {}
         			    ",
            None,
        ),
    ];

    let fail = vec![
        (
            "
			interface Foo {}
			class Foo {}
			      ",
            None,
        ),
        (
            "
         			class Foo {}
         			interface Foo {}
         			      ",
            None,
        ),
        (
            "
         			declare global {
         			  interface Foo {}
         			  class Foo {}
         			}
         			      ",
            None,
        ),
    ];

    Tester::new(NoUnsafeDeclarationMerging::NAME, NoUnsafeDeclarationMerging::PLUGIN, pass, fail)
        .test_and_snapshot();
}
