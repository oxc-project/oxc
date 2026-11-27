use oxc_ast::{AstKind, ast::BindingIdentifier};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn no_unsafe_declaration_merging_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unsafe declaration merging between classes and interfaces.")
        .with_help("The TypeScript compiler doesn't check whether properties are initialized, which can lead to TypeScript not detecting code that will cause runtime errors.")
        .with_label(span)
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
    correctness,
    recommended,
    version = "0.0.11",
    short_description = "Disallow unsafe declaration merging.",
);

impl Rule for NoUnsafeDeclarationMerging {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let ident = match node.kind() {
            AstKind::Class(class) if class.is_declaration() => {
                let Some(ident) = &class.id else { return };
                if !has_declaration(ident, ctx, |kind| {
                    matches!(kind, AstKind::TSInterfaceDeclaration(_))
                }) {
                    return;
                }
                ident
            }
            AstKind::TSInterfaceDeclaration(decl) => {
                if !has_declaration(
                    &decl.id,
                    ctx,
                    |kind| matches!(kind, AstKind::Class(class) if class.is_declaration()),
                ) {
                    return;
                }
                &decl.id
            }
            _ => return,
        };
        ctx.diagnostic(no_unsafe_declaration_merging_diagnostic(ident.span));
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

/// Whether the symbol bound by `ident` has a declaration for which `predicate` returns `true`.
fn has_declaration(
    ident: &BindingIdentifier,
    ctx: &LintContext<'_>,
    predicate: impl Fn(AstKind) -> bool,
) -> bool {
    ctx.scoping()
        .symbol_declarations(ident.symbol_id())
        .any(|declaration| predicate(ctx.nodes().kind(declaration)))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
            interface Foo {}
            class Bar implements Foo {}
                ",
        "
                     namespace Foo {}
                     namespace Foo {}
                         ",
        "
                     enum Foo {}
                     namespace Foo {}
                         ",
        "
                     namespace Fooo {}
                     function Foo() {}
                         ",
        "
                     const Foo = class {};
                         ",
        "
                     interface Foo {
                       props: string;
                     }

                     function bar() {
                       return class Foo {};
                     }
                         ",
        "
                     interface Foo {
                       props: string;
                     }

                     (function bar() {
                       class Foo {}
                     })();
                         ",
        "
                     declare global {
                       interface Foo {}
                     }

                     class Foo {}
                         ",
        "
interface Foo { value: string }
const Factory = class Foo {};
export {};
        ",
    ];

    let fail = vec![
        "
            interface Foo {}
            class Foo {}
                  ",
        "
                     class Foo {}
                     interface Foo {}
                           ",
        "
                     declare global {
                       interface Foo {}
                       class Foo {}
                     }
                           ",
        "
namespace Foo { export interface Options {} }
class Foo {}
interface Foo { value: string }
export {};
        ",
        "
// eslint-disable-next-line @typescript-eslint/no-unsafe-declaration-merging
interface Foo { value: string }
class Foo {}
export {};
        ",
    ];

    Tester::new(NoUnsafeDeclarationMerging::NAME, NoUnsafeDeclarationMerging::PLUGIN, pass, fail)
        .test_and_snapshot();
}
