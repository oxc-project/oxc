use rustc_hash::FxHashMap;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{
    context::LintContext,
    module_record::{ExportImportName, ImportImportName},
    rule::Rule,
};

fn no_duplicate_imports_diagnostic(
    module_name: &str,
    span: Span,
    previous_span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{module_name}' import is duplicated"))
        .with_help("Merge the duplicated import into a single import statement")
        .with_labels([
            span.label("This import is duplicated"),
            previous_span.label("Can be merged with this import"),
        ])
}

fn no_duplicate_exports_diagnostic(
    module_name: &str,
    span: Span,
    previous_span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{module_name}' export is duplicated"))
        .with_help("Merge the duplicated exports into a single export statement")
        .with_labels([
            span.label("This export is duplicated"),
            previous_span.label("Can be merged with this"),
        ])
}

#[derive(Debug, Default, Clone)]
pub struct NoDuplicateImports {
    include_exports: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow duplicate module imports.
    ///
    /// ### Why is this bad?
    ///
    /// Using a single import statement per module will make the code clearer because you can see
    /// everything being imported from that module on one line.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// In the following example the module import on line 1 is repeated on line 3. These can be
    /// combined to make the list of imports more succinct.
    /// ```js
    /// import { merge } from 'module';
    /// import something from 'another-module';
    /// import { find } from 'module';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import { merge, find } from 'module';
    /// import something from 'another-module';
    /// ```
    ///
    /// ### Options
    ///
    /// #### includeExports
    ///
    /// `{ type: boolean, default: false }`
    ///
    /// When `true` this rule will also look at exports to see if there is both a re-export of a
    /// module as in `export ... from 'module'` and also a standard import statement for the same
    /// module. This would count as a rule violation because there are in a sense two statements
    /// importing from the same module.
    ///
    /// Examples of **incorrect** code when `includeExports` is set to `true`:
    /// ```js
    /// import { merge } from 'module';
    ///
    /// export { find } from 'module'; // re-export which is an import and an export.
    /// ```
    ///
    /// Examples of **correct** code when `includeExports` is set to `true`:
    ///
    /// If re-exporting from an imported module, you should add the imports to the
    /// `import` statement, and export that directly, not use `export ... from`.
    /// ```js
    /// import { merge } from "lodash-es";
    /// export { merge as lodashMerge }
    /// ```
    ///
    /// ```js
    /// import { merge, find } from 'module';
    ///
    /// // cannot be merged with the above import
    /// export * as something from 'module';
    ///
    /// // cannot be written differently
    /// export * from 'module';
    /// ```
    NoDuplicateImports,
    eslint,
    style,
    pending
);

#[derive(Debug, Clone, PartialEq)]
enum ImportType {
    Named,
    Default,
    Namespace,
    SideEffect,
    AllButDefault,
}

#[derive(Debug, Clone, PartialEq)]
enum ModuleType {
    Import,
    Export,
}

impl Rule for NoDuplicateImports {
    fn from_configuration(value: serde_json::Value) -> Self {
        let value = value.get(0);
        Self {
            include_exports: value
                .and_then(|v| v.get("includeExports"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }
    }

    fn run_once(&self, ctx: &LintContext) {
        let module_record = ctx.module_record();
        let mut import_map: FxHashMap<&CompactStr, Vec<(ImportType, Span, ModuleType)>> =
            FxHashMap::default();
        let mut previous_span: Option<Span> = None;
        let mut side_effect_import_map: FxHashMap<&CompactStr, Vec<Span>> = FxHashMap::default();

        for entry in &module_record.import_entries {
            let source = &entry.module_request.name;
            let span = entry.module_request.span;

            let import_type = match &entry.import_name {
                ImportImportName::Name(_) => ImportType::Named,
                ImportImportName::NamespaceObject => ImportType::Namespace,
                ImportImportName::Default(_) => ImportType::Default,
            };

            if previous_span != Some(span) {
                previous_span = Some(span);

                if let Some(existing) = import_map.get(source) {
                    if can_merge_imports(&import_type, existing) {
                        ctx.diagnostic(no_duplicate_imports_diagnostic(
                            source,
                            span,
                            existing.first().unwrap().1,
                        ));
                        continue;
                    }
                }
            }

            import_map.entry(source).or_default().push((import_type, span, ModuleType::Import));
        }

        if module_record.import_entries.is_empty() {
            for (source, requests) in &module_record.requested_modules {
                for request in requests {
                    if request.is_import {
                        side_effect_import_map.entry(source).or_default().push(request.span);
                    }
                }
            }

            for (source, spans) in &side_effect_import_map {
                let mut spans_iter = spans.iter();
                if let Some(first_span) = spans_iter.next() {
                    for following_span in spans_iter {
                        ctx.diagnostic(no_duplicate_imports_diagnostic(
                            source,
                            *following_span,
                            *first_span,
                        ));
                    }
                }
            }
        }

        if self.include_exports {
            for entry in &module_record.star_export_entries {
                let Some(module_request) = &entry.module_request else {
                    continue;
                };
                let source = &module_request.name;
                let span = entry.span;

                if entry.import_name.is_all_but_default() {
                    if let Some(existing) = import_map.get(source) {
                        if existing.iter().any(|(t, _, _)| matches!(t, ImportType::AllButDefault)) {
                            ctx.diagnostic(no_duplicate_exports_diagnostic(
                                source,
                                span,
                                existing.first().unwrap().1,
                            ));
                            continue;
                        }
                    }
                    if let Some(existing) = side_effect_import_map.get(source) {
                        ctx.diagnostic(no_duplicate_exports_diagnostic(
                            source,
                            span,
                            *existing.first().unwrap(),
                        ));
                        continue;
                    }
                    import_map.entry(source).or_default().push((
                        ImportType::AllButDefault,
                        span,
                        ModuleType::Export,
                    ));
                    continue;
                }
                if let Some(existing) = import_map.get(source) {
                    if existing
                        .iter()
                        .any(|(t, _, _)| matches!(t, ImportType::Named | ImportType::SideEffect))
                    {
                        ctx.diagnostic(no_duplicate_exports_diagnostic(
                            source,
                            span,
                            existing.first().unwrap().1,
                        ));
                        continue;
                    }
                }

                import_map.entry(source).or_default().push((
                    ImportType::SideEffect,
                    span,
                    ModuleType::Export,
                ));
            }

            for entry in &module_record.indirect_export_entries {
                let Some(module_request) = &entry.module_request else {
                    continue;
                };
                let source = &module_request.name;
                let span = entry.span;

                if let Some(existing) = import_map.get(source) {
                    if entry.import_name == ExportImportName::All {
                        if existing.iter().any(|(t, _, _)| {
                            matches!(t, ImportType::Default | ImportType::Namespace)
                        }) {
                            ctx.diagnostic(no_duplicate_exports_diagnostic(
                                source,
                                span,
                                existing.first().unwrap().1,
                            ));
                        }

                        continue;
                    }

                    if existing.iter().any(|(t, import_span, module_type)| {
                        // import { a } from 'foo'; export { a as t };
                        if matches!(t, ImportType::Named) && module_request.span != *import_span {
                            return true;
                        }
                        (matches!(
                            t,
                            ImportType::Named | ImportType::SideEffect | ImportType::Default
                        ) && *module_type == ModuleType::Export)
                            || (matches!(t, ImportType::Default)
                                && *module_type == ModuleType::Import)
                    }) {
                        ctx.diagnostic(no_duplicate_exports_diagnostic(
                            source,
                            span,
                            existing.first().unwrap().1,
                        ));
                        continue;
                    }
                }

                import_map.entry(source).or_default().push((
                    ImportType::Named,
                    span,
                    ModuleType::Export,
                ));
            }
        }
    }
}

fn can_merge_imports(
    current_type: &ImportType,
    existing: &[(ImportType, Span, ModuleType)],
) -> bool {
    if *current_type == ImportType::AllButDefault {
        return false;
    }

    if *current_type == ImportType::SideEffect {
        return !existing.is_empty();
    }

    let namespace = existing.iter().find(|(t, _, _)| matches!(t, ImportType::Namespace));
    let named = existing.iter().find(|(t, _, _)| matches!(t, ImportType::Named));
    let default = existing.iter().find(|(t, _, _)| matches!(t, ImportType::Default));

    let has_namespace = namespace.is_some();
    let has_named = named.is_some();
    let has_default = default.is_some();

    match current_type {
        ImportType::Named => {
            has_named
                || (has_default
                    && !namespace.is_some_and(|(_, namespace_span, _)| {
                        default.unwrap().1 == *namespace_span
                    }))
        }
        ImportType::Namespace => {
            if has_named && has_default {
                if let Some((_, named_span, _)) = named {
                    if let Some((_, default_span, _)) = default {
                        if named_span == default_span {
                            return false;
                        }
                    }
                }
            }

            has_namespace || has_default
        }
        ImportType::Default => has_default || has_namespace || has_named,
        _ => unreachable!("other ImportType values should be already checked"),
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            r#"import os from "os";
    		import fs from "fs";"#,
            None,
        ),
        (r#"import { merge } from "lodash-es";"#, None),
        (r#"import _, { merge } from "lodash-es";"#, None),
        (r#"import * as Foobar from "async";"#, None),
        (r#"import "foo""#, None),
        (
            r#"import os from "os";
        export { something } from "os";"#,
            None,
        ),
        (
            r#"import * as bar from "os";
        import { baz } from "os";"#,
            None,
        ),
        (
            r#"import foo, * as bar from "os";
        import { baz } from "os";"#,
            None,
        ),
        (
            r#"import foo, { bar } from "os";
        import * as baz from "os";"#,
            None,
        ),
        (
            r#"import os from "os";
        export { hello } from "hello";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import os from "os";
        export * from "hello";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import os from "os";
        export { hello as hi } from "hello";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import os from "os";
        export default function(){};"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import { merge } from "lodash-es";
        export { merge as lodashMerge }"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"export { something } from "os";
        export * as os from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import { something } from "os";
        export * as os from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import * as os from "os";
        export { something } from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import os from "os";
        export * from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"export { something } from "os";
        export * from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            "
                import { a } from 'f';
                export { b as r };
            ",
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
    ];

    let fail = vec![
        (
            "
                export { a } from 'foo';
                import { f } from 'foo';
            ",
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import "fs";
        import "fs""#,
            None,
        ),
        (
            r#"import { merge } from "lodash-es";
        import { find } from "lodash-es";"#,
            None,
        ),
        (
            r#"import { merge } from "lodash-es";
          import _ from "lodash-es";"#,
            None,
        ),
        (
            r#"import os from "os";
          import { something } from "os";
          import * as foobar from "os";"#,
            None,
        ),
        (
            r#"import * as modns from "lodash-es";
          import { merge } from "lodash-es";
          import { baz } from "lodash-es";"#,
            None,
        ),
        (
            r#"export { os } from "os";
          export { something } from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import os from "os";
          export { os as foobar } from "os";
          export { something } from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import os from "os";
          export { something } from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import os from "os";
        export * as os from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"export * as os from "os";
        import os from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import * as modns from "mod";
        export * as  modns from "mod";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"export * from "os";
        export * from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import "os";
        export * from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            // Verifies that the error for the second line appears only once in the test snapshot.
            // https://github.com/oxc-project/oxc/pull/11320#issuecomment-2912286528
            r#"import type { PriorityDialogCustomClassNames, WeightDialogCustomClassNames } from "./HostEditDialogs";
            import { PriorityDialog, WeightDialog } from "./HostEditDialogs";"#,
            None,
        ),
        (
            "
                import b from 'foo';
                import { a } from 'foo';
            ",
            None,
        ),
        (
            "
                import * as bar from 'foo';
                import b from 'foo';
                import { a } from 'foo';
            ",
            None,
        ),
    ];

    Tester::new(NoDuplicateImports::NAME, NoDuplicateImports::PLUGIN, pass, fail)
        .test_and_snapshot();
}
