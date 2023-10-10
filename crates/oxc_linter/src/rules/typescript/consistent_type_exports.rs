use oxc_ast::{
    ast::{ExportNamedDeclaration, ImportOrExportKind, ModuleDeclaration},
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

        if let ModuleDeclaration::ExportNamedDeclaration(export_named_declaration) =
            module_declaration
        {
            if export_named_declaration.export_kind == ImportOrExportKind::Type {
                return;
            }
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
    for specifier in &export_named_declaration.specifiers {
        if matches!(specifier.export_kind, ImportOrExportKind::Type) {
            continue;
        }
        let Some(symbol_id) = scopes.get_binding(scope_id, specifier.local.name()) else {
            continue;
        };

        let symbol_is_type = symbol_table.get_flag(symbol_id).is_type();
        if symbol_is_type {
            ctx.diagnostic(ConsistentTypeExportDiagnostic(specifier.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass: Vec<(&str, Option<serde_json::Value>)> = vec![
        ("type foo = number; export {type foo}", None),
        ("type foo = number; export type {foo}", None),
        // ("type foo = number; const foo = 3; export {foo}", None),
        // from rule
        ("export { Foo } from 'foo';", None),
        ("export type { Type1 } from './consistent-type-exports';", None),
        ("export { value1 } from './consistent-type-exports';", None),
        ("export type { value1 } from './consistent-type-exports';", None),
        (
            "
            const variable = 1;
            class Class {}
            enum Enum {}
            function Func() {}
            namespace ValueNS {
              export const x = 1;
            }
            export { variable, Class, Enum, Func, ValueNS };
                ",
            None,
        ),
        (
            "type Alias = 1;
            interface IFace {}
            namespace TypeNS {
              export type x = 1;
            }
            export type { Alias, IFace, TypeNS };",
            None,
        ),
        (
            "const foo = 1;
            export type { foo };",
            None,
        ),
        (
            "namespace NonTypeNS {
                export const x = 1;
              }

              export { NonTypeNS };",
            None,
        ),
    ];

    let fail = vec![
        ("type foo = number; export {foo}", None),
        // namespace check
        (
            "
        namespace TypeNS {
            type foo = 1
        }
        export {TypeNS}
        ",
            None,
        ),
        // from rule
        (
            "type Alias = 1;
        interface IFace {}
        namespace TypeNS {
          export type x = 1;
          export const f = 1;
        }

        export { Alias, IFace, TypeNS };",
            None,
        ),
    ];

    Tester::new(ConsistentTypeExports::NAME, pass, fail).test();
}
