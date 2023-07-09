use oxc_ast::{
    ast::{ExportNamedDeclaration, ModuleDeclaration},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, ScopeId};
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(consistent-type-export): Consistent type exports")]
#[diagnostic(severity(error), help("Consistent type export"))]
struct ConsistentTypeExportDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct ConsistentTypeExports;

declare_oxc_lint!(
    /// ### What it does
    /// This rule enforces a set of restrictions on typescript files to enable "isolated declaration",
    /// i.e., .d.ts files can be generated from a single .ts file without resolving its dependencies.
    /// The typescript implementation is at `https://github.com/microsoft/TypeScript/pull/53463`
    /// The thread on isolated declaration is at `https://github.com/microsoft/TypeScript/issues/47947`
    ///
    /// ### Why is this bad?
    /// The restrictions allow .d.ts files to be generated based on one single .ts file, which improves the
    /// efficiency and possible parallelism of the declaration emitting process. Furthermore, it prevents syntax
    /// errors in one file from propagating to other dependent files.
    ///
    /// ### Example
    /// ```typescript
    /// export class Test {
    ///   x // error under isolated declarations
    ///   private y = 0; // no error, private field types are not serialized in declarations
    ///   #z = 1;// no error, fields is not present in declarations
    ///   constructor(x: number) {
    ///     this.x = 1;
    ///   }
    ///   get a() { // error under isolated declarations
    ///     return 1;
    ///   }
    ///   set a(value) { // error under isolated declarations
    ///     this.x = 1;
    ///   }
    /// }
    /// ```
    ConsistentTypeExports,
    nursery,
);

impl Rule for ConsistentTypeExports {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ModuleDeclaration(module_declaration) = node.kind() else { return };
        //let symbol_table = ctx.semantic().symbols()
        if let ModuleDeclaration::ExportNamedDeclaration(export_named_declaration) =
            module_declaration
        {
            check_export_named_declaration(node.scope_id(), export_named_declaration, ctx);
        }
    }
}

fn check_export_named_declaration(
    scope_id: ScopeId,
    export_named_declaration: &ExportNamedDeclaration,
    ctx: &LintContext,
) {
    let symbol_table = ctx.semantic().symbols();
    let scopes = ctx.scopes();
    for specifier in export_named_declaration.specifiers.iter() {
        // {export {foo} }
        let Some(symbol_id) = scopes.get_binding(scope_id, specifier.local.name()) else {
            continue;
        };
        // Symbol resolution should not be done in a rule it should be a global function...
        // Pretend this was TS equivalent check.getAliasedSymbol
        let symbol_is_type = symbol_table.get_flag(symbol_id).is_type();
        if symbol_is_type {
            ctx.diagnostic(ConsistentTypeExportDiagnostic(specifier.span));
        }

        match specifier.export_kind {
            oxc_ast::ast::ImportOrExportKind::Value => println!("Value"),
            oxc_ast::ast::ImportOrExportKind::Type => println!("Type"),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass: Vec<(&str, Option<serde_json::Value>)> = vec![
        //("export function foo(a: number): number { return a; }", None),
    ];

    let fail = vec![("type foo = number; export {foo}", None)];

    Tester::new(ConsistentTypeExports::NAME, pass, fail).test();
}
