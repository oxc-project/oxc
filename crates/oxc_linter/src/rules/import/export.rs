use std::path::PathBuf;

use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ModuleRecord;
use oxc_span::{Atom, Span};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
enum ExportDiagnostic {
    #[error("eslint-plugin-import(export): Multiple exports of name '{1}'.")]
    #[diagnostic(severity(warning))]
    MultipleNamedExport(#[label] Span, Atom),
    #[error("eslint-plugin-import(export): No named exports found in module '{1}'")]
    #[diagnostic(severity(warning))]
    NoNamedExport(#[label] Span, Atom),
}

/// <https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/export.md>
#[derive(Debug, Default, Clone)]
pub struct Export;

declare_oxc_lint!(
    /// ### What it does
    /// Reports funny business with exports, like repeated exports of names or defaults.
    ///
    /// ### Example
    /// ```javascript
    /// let foo;
    /// export { foo }; // Multiple exports of name 'foo'.
    /// export * from "./export-all" // export-all.js also export foo
    /// ```
    Export,
    nursery
);

impl Rule for Export {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.semantic().module_record();
        let named_export = &module_record.exported_bindings;
        let mut duplicated_named_export = FxHashMap::default();
        if module_record.star_export_entries.is_empty() {
            return;
        }
        for export_entry in &module_record.star_export_entries {
            let Some(module_request) = &export_entry.module_request else {
                continue;
            };
            let Some(remote_module_record_ref) =
                module_record.loaded_modules.get(module_request.name())
            else {
                continue;
            };

            let remote_module_record = remote_module_record_ref.value();
            let mut all_export_names = FxHashSet::default();
            let mut visited = FxHashSet::default();
            walk_exported_recursive(remote_module_record, &mut all_export_names, &mut visited);
            if all_export_names.is_empty() {
                ctx.diagnostic(ExportDiagnostic::NoNamedExport(
                    module_request.span(),
                    module_request.name().clone(),
                ));
                continue;
            }

            for name in &all_export_names {
                if let Some(span) = named_export.get(name) {
                    duplicated_named_export.entry(*span).or_insert_with(|| name.clone());
                }
            }
        }

        for (span, name) in duplicated_named_export {
            ctx.diagnostic(ExportDiagnostic::MultipleNamedExport(span, name));
        }
    }
}

fn walk_exported_recursive(
    module_record: &ModuleRecord,
    result: &mut FxHashSet<Atom>,
    visited: &mut FxHashSet<PathBuf>,
) {
    let path = &module_record.resolved_absolute_path;
    if path.components().any(|c| match c {
        std::path::Component::Normal(p) => p == std::ffi::OsStr::new("node_modules"),
        _ => false,
    }) {
        return;
    }
    if !visited.insert(path.clone()) {
        return;
    }
    for name in module_record.exported_bindings.keys() {
        result.insert(name.clone());
    }
    for export_entry in &module_record.star_export_entries {
        let Some(module_request) = &export_entry.module_request else {
            continue;
        };
        let Some(remote_module_record_ref) =
            module_record.loaded_modules.get(module_request.name())
        else {
            continue;
        };
        walk_exported_recursive(remote_module_record_ref.value(), result, visited);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let mut tester =
        Tester::new_without_config::<String>(Export::NAME, vec![], vec![]).with_import_plugin(true);

    {
        let pass = vec![
            (r#"import "./malformed.js""#),
            (r#"var foo = "foo"; export default foo;"#),
            (r#"export var foo = "foo"; export var bar = "bar";"#),
            (r#"export var foo = "foo", bar = "bar";"#),
            ("export var { foo, bar } = object;"),
            ("export var [ foo, bar ] = array;"),
            ("let foo; export { foo, foo as bar }"),
            (r#"let bar; export { bar }; export * from "./export-all""#),
            (r#"export * from "./export-all""#),
            (r#"export * from "./does-not-exist""#),
            (r#"export default foo; export * from "./bar""#),
            // SYNTAX_CASES doesn't need to be tested
            ("
                import * as A from './named-export-collision/a';
                import * as B from './named-export-collision/b';
                export { A, B };
            "),
            ("
                export * as A from './named-export-collision/a';
                export * as B from './named-export-collision/b';
            "),
            // ("
            //     export default function foo(param: string): boolean;
            //     export default function foo(param: string, param1: number): boolean;
            //     export default function foo(param: string, param1?: number): boolean {
            //         return param && param1;
            //     }
            // "),
            // Typescript
            ("
                export const Foo = 1;
                export type Foo = number;
            "),
            ("
                export const Foo = 1;
                export interface Foo {}
            "),
            // ("
            //     export function fff(a: string);
            //     export function fff(a: number);
            // "),
            // ("
            //     export function fff(a: string);
            //     export function fff(a: number);
            //     export function fff(a: string|number) {};
            // "),
            ("
                export const Bar = 1;
                export namespace Foo {
                export const Bar = 1;
                }
            "),
            ("
                export type Bar = string;
                export namespace Foo {
                export type Bar = string;
                }
            "),
            ("
                export const Bar = 1;
                export type Bar = string;
                export namespace Foo {
                export const Bar = 1;
                export type Bar = string;
                }
            "),
            ("
                export namespace Foo {
                export const Foo = 1;
                export namespace Bar {
                    export const Foo = 2;
                }
                export namespace Baz {
                    export const Foo = 3;
                }
                }
            "),
            (" 
                export class Foo { }
                export namespace Foo { }
                export namespace Foo {
                export class Bar {}
                }
            "),
            // ("
            //     export function Foo();
            //     export namespace Foo { }
            // "),
            // ("
            //     export function Foo(a: string);
            //     export namespace Foo { }
            // "),
            // ("
            //     export function Foo(a: string);
            //     export function Foo(a: number);
            //     export namespace Foo { }
            // "),
            ("
                export enum Foo { }
                export namespace Foo { }
            "),
            (r#"export * from "./file1.ts""#),
            ("
                export * as A from './named-export-collision/a';
                export * as B from './named-export-collision/b';
            "),
            // (r#"
            //     declare module "a" {
            //         const Foo = 1;
            //         export {Foo as default};
            //     }
            //     declare module "b" {
            //     const Bar = 2;
            //     export {Bar as default};
            //     }
            // "#),
            // (r#"
            //     declare module "a" {
            //         const Foo = 1;
            //         export {Foo as default};
            //     }
            //     const Bar = 2;
            //     export {Bar as default};
            // "#),
        ];
        let fail = vec![
            (r#"let foo; export { foo }; export * from "./export-all""#),
            // (r#"export * from "./malformed.js""#),
            (r#"export * from "./default-export""#),
            (r#"let foo; export { foo as "foo" }; export * from "./export-all""#),
            (" 
                export type Foo = string;
                export type Foo = number;
            "),
            ("
                export const a = 1
                export namespace Foo {
                export const a = 2;
                export const a = 3;
                }
            "),
            ("
                declare module 'foo' {
                    const Foo = 1;
                    export default Foo;
                    export default Foo;
                }
            "),
            (" 
                export namespace Foo {
                    export namespace Bar {
                        export const Foo = 1;
                            export const Foo = 2;
                    }
                    export namespace Baz {
                        export const Bar = 3;
                        export const Bar = 4;
                    } 
                }
            "),
            ("
                export class Foo { }
                export class Foo { }
                export namespace Foo { }
            "),
            // ("
            //     export enum Foo { }
            //     export enum Foo { }
            //     export namespace Foo { }
            // "),
            ("
                export enum Foo { }
                export class Foo { }
                export namespace Foo { }
            "),
            ("
                export const Foo = 'bar';
                export class Foo { }
                export namespace Foo { }
            "),
            ("
                export function Foo();
                export class Foo { }
                export namespace Foo { }
            "),
            ("
                export const Foo = 'bar';
                export function Foo();
                export namespace Foo { }
            "),
            // ("
            //     export const Foo = 'bar';
            //     export namespace Foo { }
            // "),
            (r#"
                declare module "a" {
                    const Foo = 1;
                    export {Foo as default};
                }
                const Bar = 2;
                export {Bar as default};
                const Baz = 3;
                export {Baz as default};
            "#),
        ];

        tester = tester.change_rule_path("index.js").update_expect_pass_fail(pass, fail);
        tester.test();
    }

    {
        let pass = vec!["export * from './module'"];
        let fail = vec![];
        tester =
            tester.change_rule_path("export-star-4/index.js").update_expect_pass_fail(pass, fail);
        tester.test_and_snapshot();
    }
}
