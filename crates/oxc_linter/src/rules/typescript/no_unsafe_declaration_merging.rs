use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(no-unsafe-declaration-merging): Unsafe declaration merging between classes and interfaces.")]
#[diagnostic(severity(warning))]
struct NoUnsafeDeclarationMergingDiagnostic(#[label] pub Span);

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
        let scope_symbols = ctx.semantic().scopes().get_bindings(node.scope_id());

        match node.kind() {
            AstKind::Class(decl) => {
                if let Some(ident) = decl.id.as_ref() {
                    for scope_symbol in scope_symbols {
                        if let AstKind::TSInterfaceDeclaration(scope_decl) = ctx
                            .nodes()
                            .get_node(ctx.symbols().get_declaration(*scope_symbol.1))
                            .kind()
                        {
                            if ident.name.as_str() == scope_decl.id.name.as_str() {
                                ctx.diagnostic(NoUnsafeDeclarationMergingDiagnostic(decl.span));
                            }
                        }
                    }
                }
            }
            AstKind::TSInterfaceDeclaration(decl) => {
                for scope_symbol in scope_symbols {
                    if let AstKind::Class(scope_decl) =
                        ctx.nodes().get_node(ctx.symbols().get_declaration(*scope_symbol.1)).kind()
                    {
                        if let Some(scope_ident) = scope_decl.id.as_ref() {
                            if scope_ident.name.as_str() == decl.id.name.as_str() {
                                ctx.diagnostic(NoUnsafeDeclarationMergingDiagnostic(decl.span));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
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
