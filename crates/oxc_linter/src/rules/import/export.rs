use rustc_hash::{FxHashMap, FxHashSet};

use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;

use crate::{
    ModuleRecord,
    context::LintContext,
    module_record::{ExportEntry, ExportExportName},
    rule::Rule,
};

fn no_named_export(module_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("No named exports found in module '{module_name}'"))
        .with_help("Remove the `export *` re-export, or add named exports to the target module.")
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
    nursery,
    version = "0.0.21",
    short_description = "Reports funny business with exports, like repeated exports of names or defaults.",
);

impl Rule for Export {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();
        let named_export = &module_record.exported_bindings;

        diagnose_duplicate_named_exports(ctx, module_record);

        let mut all_export_names = FxHashMap::default();
        module_record.star_export_entries.iter().for_each(|star_export_entry| {
            if star_export_entry.is_type {
                return;
            }
            let Some(module_request) = &star_export_entry.module_request else {
                return;
            };
            let Some(remote_module_record) = module_record.get_loaded_module(module_request.name())
            else {
                return;
            };

            // Include direct exports as well as names forwarded through `export *`.
            // The shared module-record traversal handles cycles and caches the result.
            let export_names: FxHashSet<_> = remote_module_record
                .exported_bindings
                .keys()
                .chain(remote_module_record.exported_bindings_from_star_export().values().flatten())
                // `export *` never forwards the default export, including named aliases.
                .filter(|name| name.as_str() != "default")
                .cloned()
                .collect();

            if export_names.is_empty() {
                ctx.diagnostic(no_named_export(module_request.name(), module_request.span));
            } else {
                all_export_names.insert(star_export_entry.span, export_names);
            }
        });

        for (name, span) in named_export {
            let mut spans = all_export_names
                .iter()
                .filter_map(|(star_export_entry_span, export_names)| {
                    if export_names.contains(name) { Some(*star_export_entry_span) } else { None }
                })
                .collect::<Vec<_>>();

            if !spans.is_empty() {
                spans.push(*span);
                let labels = spans.into_iter().map(LabeledSpan::underline).collect::<Vec<_>>();

                ctx.diagnostic(
                    OxcDiagnostic::warn(format!("Multiple exports of name '{name}'."))
                        .with_help("Rename or remove the duplicate export so each name is exported only once.")
                        .with_labels(labels),
                );
            }
        }
    }
}

struct ExportNameSpans {
    spans: Vec<Span>,
    has_named_specifier: bool,
}

fn diagnose_duplicate_named_exports(ctx: &LintContext<'_>, module_record: &ModuleRecord) {
    let mut export_names: FxHashMap<(CompactStr, bool), ExportNameSpans> = FxHashMap::default();

    module_record
        .local_export_entries
        .iter()
        .chain(&module_record.indirect_export_entries)
        .for_each(|export_entry| {
            let Some((name, span)) = export_name(export_entry) else {
                return;
            };

            let entry = export_names
                .entry((CompactStr::from(name), export_entry.is_type))
                .or_insert(ExportNameSpans { spans: Vec::new(), has_named_specifier: false });
            entry.spans.push(span);
            entry.has_named_specifier |= is_named_export_specifier(ctx, export_entry);
        });

    for ((name, _), entry) in export_names {
        if entry.spans.len() <= 1 || !entry.has_named_specifier {
            continue;
        }

        let labels = entry.spans.into_iter().map(LabeledSpan::underline).collect::<Vec<_>>();
        ctx.diagnostic(
            OxcDiagnostic::warn(format!("Multiple exports of name '{name}'."))
                .with_help(
                    "Rename or remove the duplicate export so each name is exported only once.",
                )
                .with_labels(labels),
        );
    }
}

fn export_name(export_entry: &ExportEntry) -> Option<(&str, Span)> {
    match &export_entry.export_name {
        ExportExportName::Name(name) => Some((name.name(), name.span)),
        ExportExportName::Default(span) => Some(("default", *span)),
        ExportExportName::Null => None,
    }
}

fn is_named_export_specifier(ctx: &LintContext<'_>, export_entry: &ExportEntry) -> bool {
    // Resolved indirect exports use the import statement span, which can appear after the export.
    if export_entry.statement_span.start > export_entry.span.start {
        return true;
    }

    ctx.find_next_token_within(export_entry.statement_span.start, export_entry.span.start, "{")
        .is_some()
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
            "export const foo = 1;\nimport { foo } from 'mod';",
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
            ("export const foo = function () {};\nfunction bar() {}\nexport { bar as foo };"),
            ("export const foo = 1;\nexport /* comment */ { foo };"),
            ("export const value = 1;\nexport { value };"),
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

#[test]
fn test_external_star_exports() {
    use crate::tester::Tester;

    let pass = vec![
        // Named exports in dependencies must be collected, not skipped.
        "export * from 'export-star-regression';",
        // Repeated and overlapping paths must not produce an empty export set.
        "export * from 'export-star-regression'; export * from 'export-star-regression';",
        "export * from 'export-star-regression'; export * from 'export-star-regression/nested.js';",
        // A dependency's default alias is not forwarded by `export *`.
        "export * from 'export-star-regression'; const own = 1; export { own as default };",
        // Preserve both explicit type-only and ordinary TypeScript re-exports.
        "export type * from 'export-star-regression/types.ts';",
        "export * from 'export-star-regression/types.ts';",
        // Cyclic graphs terminate and remain valid through multiple entry points.
        "export * from 'export-star-regression/cycle-a.js';",
        "export * from 'export-star-regression/cycle-a.js'; export * from 'export-star-regression/cycle-b.js';",
    ];
    let fail = vec![
        // Verify names found directly and through transitive star exports.
        "export * from 'export-star-regression'; export const value = 2;",
        "export * from 'export-star-regression'; export const nested = 3;",
        // A default-only module has no names that `export *` can forward.
        "export * from 'export-star-regression/default-only.js';",
        // The traversal must find names across the cycle, not merely terminate.
        "export * from 'export-star-regression/cycle-a.js'; export const b = 3;",
    ];
    Tester::new(Export::NAME, Export::PLUGIN, pass, fail)
        .with_import_plugin(true)
        .change_rule_path("index.ts")
        .with_snapshot_suffix("external_star_exports")
        .test_and_snapshot();
}
