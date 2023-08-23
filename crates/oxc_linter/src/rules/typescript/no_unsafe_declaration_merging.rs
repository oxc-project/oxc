use oxc_ast::{ast::BindingIdentifier, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(no-unsafe-declaration-merging): Unsafe declaration merging between classes and interfaces.")]
#[diagnostic(severity(warning), help("The TypeScript compiler doesn't check whether properties are initialized, which can cause lead to TypeScript not detecting code that will cause runtime errors."))]
struct NoUnsafeDeclarationMergingDiagnostic(#[label] Span, #[label] Span);

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
    /// The TypeScript compiler doesn't check whether properties are initialized, which can cause lead to TypeScript not detecting code that will cause runtime errors.
    ///
    /// ### Example
    /// ```javascript
    /// interface Foo {}
    /// class Foo {}
    /// ```
    NoUnsafeDeclarationMerging,
    correctness
);

impl Rule for NoUnsafeDeclarationMerging {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !ctx.source_type().is_typescript() {
            return;
        }

        match node.kind() {
            AstKind::Class(decl) => {
                if let Some(ident) = decl.id.as_ref() {
                    for (_, symbol_id) in ctx.semantic().scopes().get_bindings(node.scope_id()) {
                        if let AstKind::TSInterfaceDeclaration(scope_interface) =
                            get_symbol_kind(*symbol_id, ctx)
                        {
                            check_and_diagnostic(ident, &scope_interface.id, ctx);
                        }
                    }
                }
            }
            AstKind::TSInterfaceDeclaration(decl) => {
                for (_, symbol_id) in ctx.semantic().scopes().get_bindings(node.scope_id()) {
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
}

fn check_and_diagnostic(
    ident: &BindingIdentifier,
    scope_ident: &BindingIdentifier,
    ctx: &LintContext<'_>,
) {
    if scope_ident.name.as_str() == ident.name.as_str() {
        ctx.diagnostic(NoUnsafeDeclarationMergingDiagnostic(ident.span, scope_ident.span));
    }
}

fn get_symbol_kind<'a>(symbol_id: SymbolId, ctx: &LintContext<'a>) -> AstKind<'a> {
    return ctx.nodes().get_node(ctx.symbols().get_declaration(symbol_id)).kind();
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

    Tester::new(NoUnsafeDeclarationMerging::NAME, pass, fail).test_and_snapshot();
}
