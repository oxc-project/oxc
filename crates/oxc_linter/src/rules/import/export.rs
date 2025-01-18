use std::path::PathBuf;

use rustc_hash::{FxHashMap, FxHashSet};

use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{context::LintContext, rule::Rule, ModuleRecord};

fn no_named_export(module_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("No named exports found in module '{module_name}'"))
        .with_label(span)
}

/// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/export.md>
#[derive(Debug, Default, Clone)]
pub struct Export;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports funny business with exports, like repeated exports of names or defaults.
    ///
    /// ### Why is this bad?
    ///
    /// Having multiple exports of the same name can lead to ambiguity and confusion
    /// in the codebase. It makes it difficult to track which export is being used
    /// and can result in runtime errors if the wrong export is referenced.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// let foo;
    /// export { foo }; // Multiple exports of name 'foo'.
    /// export * from "./export-all"; // Conflicts if export-all.js also exports foo
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// let foo;
    /// export { foo as foo1 }; // Renamed export to avoid conflict
    /// export * from "./export-all"; // No conflict if export-all.js also exports foo
    /// ```
    Export,

    import,
    nursery
);

impl Rule for Export {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();
        let named_export = &module_record.exported_bindings;

        let mut all_export_names = FxHashMap::default();
        let mut visited = FxHashSet::default();

        let loaded_modules = module_record.loaded_modules.read().unwrap();
        module_record.star_export_entries.iter().for_each(|star_export_entry| {
            if star_export_entry.is_type {
                return;
            }
            let mut export_names = FxHashSet::default();

            let Some(module_request) = &star_export_entry.module_request else {
                return;
            };
            let Some(remote_module_record) = loaded_modules.get(module_request.name()) else {
                return;
            };

            walk_exported_recursive(remote_module_record, &mut export_names, &mut visited);

            if export_names.is_empty() {
                ctx.diagnostic(no_named_export(module_request.name(), module_request.span()));
            } else {
                all_export_names.insert(star_export_entry.span, export_names);
            }
        });

        for (name, span) in named_export {
            let mut spans = all_export_names
                .iter()
                .filter_map(|(star_export_entry_span, export_names)| {
                    if export_names.contains(name) {
                        Some(*star_export_entry_span)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if !spans.is_empty() {
                spans.push(*span);
                let labels = spans.into_iter().map(LabeledSpan::underline).collect::<Vec<_>>();

                ctx.diagnostic(
                    OxcDiagnostic::warn(format!("Multiple exports of name '{name}'."))
                        .with_labels(labels),
                );
            }
        }
    }
}

fn walk_exported_recursive(
    module_record: &ModuleRecord,
    result: &mut FxHashSet<CompactStr>,
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
    let loaded_modules = module_record.loaded_modules.read().unwrap();
    for export_entry in &module_record.star_export_entries {
        let Some(module_request) = &export_entry.module_request else {
            continue;
        };
        let Some(remote_module_record) = loaded_modules.get(module_request.name()) else {
            continue;
        };
        walk_exported_recursive(remote_module_record, result, visited);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

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
            ("
                export default function foo(param: string): boolean;
                export default function foo(param: string, param1: number): boolean;
                export default function foo(param: string, param1?: number): boolean {
                    return param && param1;
                }
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
            ("
                export function fff(a: string);
                export function fff(a: number);
            "),
            ("
                export function fff(a: string);
                export function fff(a: number);
                export function fff(a: string|number) {};
            "),
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
            ("
                export function Foo();
                export namespace Foo { }
            "),
            ("
                export function Foo(a: string);
                export namespace Foo { }
            "),
            ("
                export function Foo(a: string);
                export function Foo(a: number);
                export namespace Foo { }
            "),
            ("
                export enum Foo { }
                export namespace Foo { }
            "),
            (r#"export * from "./file1.ts""#),
            ("
                export * as A from './named-export-collision/a';
                export * as B from './named-export-collision/b';
            "),
            (r#"
                declare module "a" {
                    const Foo = 1;
                    export {Foo as default};
                }
                declare module "b" {
                const Bar = 2;
                export {Bar as default};
                }
            "#),
            (r#"
                declare module "a" {
                    const Foo = 1;
                    export {Foo as default};
                }
                const Bar = 2;
                export {Bar as default};
            "#),
            "export type * from './export-props.js'",
        ];
        let fail = vec![
            (r#"let foo; export { foo }; export * from "./export-all""#),
            // (r#"export * from "./malformed.js""#),
            // This case has been comment out in eslint-plugin-import
            // https://github.com/import-js/eslint-plugin-import/blob/7a21f7e10f18c04473faadca94928af6b8e28009/tests/src/rules/export.js#L101-L109
            // (r#"export * from "./default-export""#),
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
            // ("
            //     declare module 'foo' {
            //         const Foo = 1;
            //         export default Foo;
            //         export default Foo;
            //     }
            // "),
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
            // ("
            //     export function Foo();
            //     export class Foo { }
            //     export namespace Foo { }
            // "),
            // ("
            //     export const Foo = 'bar';
            //     export function Foo();
            //     export namespace Foo { }
            // "),
            // ("
            //     export const Foo = 'bar';
            //     export namespace Foo { }
            // "),
        ];

        Tester::new(Export::NAME, Export::PLUGIN, pass, fail)
            .with_import_plugin(true)
            .change_rule_path("index.ts")
            .test_and_snapshot();
    }

    {
        let pass = vec!["export * from './module'"];
        let fail = vec![
            ("
                const Bar = 2;
                export {Bar as default};
                const Baz = 3;
                export {Baz as default};
            "),
        ];
        Tester::new(Export::NAME, Export::PLUGIN, pass, fail)
            .with_import_plugin(true)
            .change_rule_path("export-star-4/index.js")
            .test();
    }
}
