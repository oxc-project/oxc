use rustc_hash::FxHashMap;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    module_record::{ExportImportName, ImportImportName},
    rule::{DefaultRuleConfig, Rule},
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

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoDuplicateImports {
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
    include_exports: bool,
    /// When `true`, imports with only type specifiers (inline types or type imports) are
    /// considered separate from imports with value specifiers, so they can be imported from the
    /// same module on separate import statements.
    ///
    /// Examples of **correct** code when `allowSeparateTypeImports` is set to `true`:
    /// ```js
    /// import { foo } from "module";
    /// import type { Bar } from "module";
    /// ```
    ///
    /// ```js
    /// import { type Foo } from "module";
    /// import type { Bar } from "module";
    /// ```
    allow_separate_type_imports: bool,
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
    NoDuplicateImports,
    eslint,
    style,
    pending,
    config = NoDuplicateImports,
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
        serde_json::from_value::<DefaultRuleConfig<NoDuplicateImports>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run_once(&self, ctx: &LintContext) {
        let module_record = ctx.module_record();
        let mut import_map: FxHashMap<&CompactStr, Vec<(ImportType, Span, ModuleType, bool)>> =
            FxHashMap::default();
        let mut previous_span: Option<Span> = None;
        let mut side_effect_import_map: FxHashMap<&CompactStr, Vec<Span>> = FxHashMap::default();

        for entry in &module_record.import_entries {
            let source = &entry.module_request.name;
            let span = entry.module_request.span;

            let requested_module_is_type_import = module_record.requested_modules
                [&entry.module_request.name]
                .iter()
                .find(|requested_module| requested_module.span == entry.module_request.span)
                .unwrap()
                .is_type;

            let import_type = match &entry.import_name {
                ImportImportName::Name(_) => ImportType::Named,
                ImportImportName::NamespaceObject => ImportType::Namespace,
                ImportImportName::Default(_) => ImportType::Default,
            };

            if previous_span != Some(span) {
                previous_span = Some(span);

                if let Some(existing) = import_map.get(source)
                    && can_merge_imports(
                        &import_type,
                        existing,
                        self.allow_separate_type_imports,
                        requested_module_is_type_import,
                        entry.is_type,
                    )
                {
                    ctx.diagnostic(no_duplicate_imports_diagnostic(
                        source,
                        span,
                        existing.first().unwrap().1,
                    ));
                    continue;
                }
            }

            import_map.entry(source).or_default().push((
                import_type,
                span,
                ModuleType::Import,
                requested_module_is_type_import,
            ));
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
                    if let Some(existing) = import_map.get(source)
                        && existing
                            .iter()
                            .any(|(t, _, _, _)| matches!(t, ImportType::AllButDefault))
                    {
                        ctx.diagnostic(no_duplicate_exports_diagnostic(
                            source,
                            span,
                            existing.first().unwrap().1,
                        ));
                        continue;
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
                        false,
                    ));
                    continue;
                }
                if let Some(existing) = import_map.get(source)
                    && existing
                        .iter()
                        .any(|(t, _, _, _)| matches!(t, ImportType::Named | ImportType::SideEffect))
                {
                    ctx.diagnostic(no_duplicate_exports_diagnostic(
                        source,
                        span,
                        existing.first().unwrap().1,
                    ));
                    continue;
                }

                import_map.entry(source).or_default().push((
                    ImportType::SideEffect,
                    span,
                    ModuleType::Export,
                    entry.is_type,
                ));
            }

            for entry in &module_record.indirect_export_entries {
                let Some(module_request) = &entry.module_request else {
                    continue;
                };
                let source = &module_request.name;
                let span = entry.span;

                // Check if this export is from a statement-level type export (export type)
                let requested_module = module_record.requested_modules[source]
                    .iter()
                    .find(|rm| rm.span == module_request.span)
                    .unwrap();
                let is_statement_type_export = requested_module.is_type;

                if let Some(existing) = side_effect_import_map.get(source) {
                    ctx.diagnostic(no_duplicate_exports_diagnostic(
                        source,
                        span,
                        *existing.first().unwrap(),
                    ));
                    continue;
                }

                if let Some(existing) = import_map.get(source) {
                    if entry.import_name == ExportImportName::All {
                        if existing.iter().any(|(t, _, _, _)| {
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

                    // Special case: type export with type imports (default + named)
                    // This applies regardless of allowSeparateTypeImports
                    if entry.is_type {
                        let all_existing_are_type =
                            existing.iter().all(|(_, _, _, is_stmt_type)| *is_stmt_type);
                        if all_existing_are_type {
                            let has_default = existing
                                .iter()
                                .any(|(t, _, _, _)| matches!(t, ImportType::Default));
                            // Export is named, cannot merge with default type import
                            if has_default {
                                import_map.entry(source).or_default().push((
                                    ImportType::Named,
                                    span,
                                    ModuleType::Export,
                                    entry.is_type,
                                ));
                                continue;
                            }
                        }
                    }

                    // Handle allowSeparateTypeImports for exports
                    if self.allow_separate_type_imports {
                        let imports_only = existing
                            .iter()
                            .filter(|(_, _, module_type, _)| *module_type == ModuleType::Import)
                            .collect::<Vec<_>>();

                        if !imports_only.is_empty() {
                            // Only consider statement-level type exports/imports
                            // Inline type specifiers (export { type Foo }) are treated as value exports
                            if is_statement_type_export && entry.is_type {
                                // Statement-level type export: check if there are value imports
                                let has_value_imports = imports_only
                                    .iter()
                                    .any(|(_, _, _, is_stmt_type)| !is_stmt_type);
                                if has_value_imports {
                                    // Don't report, allow separation from value imports
                                    import_map.entry(source).or_default().push((
                                        ImportType::Named,
                                        span,
                                        ModuleType::Export,
                                        is_statement_type_export,
                                    ));
                                    continue;
                                }
                            } else if !is_statement_type_export {
                                // Value/inline type export: check if there are only statement-level type imports
                                let all_imports_are_statement_type = imports_only
                                    .iter()
                                    .all(|(_, _, _, is_stmt_type)| *is_stmt_type);

                                if all_imports_are_statement_type {
                                    // Don't report, allow separation from statement-level type imports
                                    import_map.entry(source).or_default().push((
                                        ImportType::Named,
                                        span,
                                        ModuleType::Export,
                                        is_statement_type_export,
                                    ));
                                    continue;
                                }
                            }
                        }
                    }

                    if existing.iter().any(|(t, import_span, module_type, _)| {
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
                    is_statement_type_export,
                ));
            }
        }
    }
}

fn can_merge_imports(
    current_type: &ImportType,
    existing: &[(ImportType, Span, ModuleType, bool)],
    allow_separate_type_imports: bool,
    is_current_statement_type_import: bool,
    is_current_specifier_type: bool,
) -> bool {
    if *current_type == ImportType::AllButDefault {
        return false;
    }

    if *current_type == ImportType::SideEffect {
        return !existing.is_empty();
    }

    // Special case: both are type imports (statement-level)
    // Check if one is default and one is named (cannot be merged per ESLint)
    let all_existing_are_type = existing.iter().all(|(_, _, _, is_stmt_type)| *is_stmt_type);
    if is_current_statement_type_import && is_current_specifier_type && all_existing_are_type {
        let current_is_default = matches!(current_type, ImportType::Default);
        let current_is_named = matches!(current_type, ImportType::Named);
        let has_default = existing.iter().any(|(t, _, _, _)| matches!(t, ImportType::Default));
        let has_named = existing.iter().any(|(t, _, _, _)| matches!(t, ImportType::Named));

        // Cannot merge if one is default and the other is named
        if (current_is_default && has_named) || (current_is_named && has_default) {
            return false;
        }
    }

    // Handle allowSeparateTypeImports option
    if allow_separate_type_imports {
        let has_value_statement_imports =
            existing.iter().any(|(_, _, _, is_stmt_type)| !is_stmt_type);
        let has_type_statement_imports =
            existing.iter().any(|(_, _, _, is_stmt_type)| *is_stmt_type);

        // If current is a statement-level type import (import type)
        if is_current_statement_type_import {
            // Don't merge with value/mixed statements (allow separation)
            // But DO merge with other statement-level type imports (report as duplicate)
            if has_value_statement_imports && !has_type_statement_imports {
                return false;
            }
            // If there are other type imports, continue to check if they can be merged
        } else {
            // Current is a value or mixed import (not statement-level type)
            // Don't merge with statement-level type imports (allow separation)
            if has_type_statement_imports {
                return false;
            }
        }
    }

    let namespace = existing.iter().find(|(t, _, _, _)| matches!(t, ImportType::Namespace));
    let named = existing.iter().find(|(t, _, _, _)| matches!(t, ImportType::Named));
    let default = existing.iter().find(|(t, _, _, _)| matches!(t, ImportType::Default));

    let has_namespace = namespace.is_some();
    let has_named = named.is_some();
    let has_default = default.is_some();

    match current_type {
        ImportType::Named => {
            has_named
                || (has_default
                    && !namespace.is_some_and(|(_, namespace_span, _, _)| {
                        default.unwrap().1 == *namespace_span
                    }))
        }
        ImportType::Namespace => {
            if has_named
                && has_default
                && let Some((_, named_span, _, _)) = named
                && let Some((_, default_span, _, _)) = default
                && named_span == default_span
            {
                return false;
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
            r#"import type { Os } from "os";
			import type { Fs } from "fs";"#,
            None,
        ),
        (
            r#"import { type Os } from "os";
			import type { Fs } from "fs";"#,
            None,
        ),
        (r#"import type { Merge } from "lodash-es";"#, None),
        (r#"import _, { type Merge } from "lodash-es";"#, None),
        (r#"import type * as Foobar from "async";"#, None),
        (
            r#"import type Os from "os";
			export type { Something } from "os";"#,
            None,
        ),
        (
            r#"import type Os from "os";
			export { type Something } from "os";"#,
            None,
        ),
        (
            r#"import type * as Bar from "os";
			import { type Baz } from "os";"#,
            None,
        ),
        (
            r#"import foo, * as bar from "os";
			import { type Baz } from "os";"#,
            None,
        ),
        (
            r#"import foo, { type bar } from "os";
			import type * as Baz from "os";"#,
            None,
        ),
        (
            r#"import type { Merge } from "lodash-es";
			import type _ from "lodash-es";"#,
            None,
        ),
        (
            r#"import type Os from "os";
			export { type Hello } from "hello";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import type Os from "os";
			export type * from "hello";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import type Os from "os";
			export { type Hello as Hi } from "hello";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import type Os from "os";
			export default function(){};"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import { type Merge } from "lodash-es";
			export { Merge as lodashMerge }"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"export type { Something } from "os";
			export * as os from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import { type Something } from "os";
			export * as os from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import type * as Os from "os";
			export { something } from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import type Os from "os";
			export * from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import type Os from "os";
			export type { Something } from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"export type { Something } from "os";
			export * from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import { foo, type Bar } from "module";"#,
            Some(serde_json::json!([{ "allowSeparateTypeImports": true }])),
        ),
        (
            r#"import { foo } from "module";
			import type { Bar } from "module";"#,
            Some(serde_json::json!([{ "allowSeparateTypeImports": true }])),
        ),
        (
            r#"import { type Foo } from "module";
			import type { Bar } from "module";"#,
            Some(serde_json::json!([{ "allowSeparateTypeImports": true }])),
        ),
        (
            r#"import { foo, type Bar } from "module";
			export { type Baz } from "module2";"#,
            Some(serde_json::json!([{ "allowSeparateTypeImports": true, "includeExports": true }])),
        ),
        (
            r#"import type { Foo } from "module";
			export { bar, type Baz } from "module";"#,
            Some(serde_json::json!([{ "allowSeparateTypeImports": true, "includeExports": true }])),
        ),
        (
            r#"import { type Foo } from "module";
			export type { Bar } from "module";"#,
            Some(serde_json::json!([{ "allowSeparateTypeImports": true, "includeExports": true }])),
        ),
        (
            r#"import type * as Foo from "module";
			export { type Bar } from "module";"#,
            Some(serde_json::json!([{ "allowSeparateTypeImports": true, "includeExports": true }])),
        ),
        (
            r#"import { type Foo } from "module";
			export type * as Bar from "module";"#,
            Some(serde_json::json!([{ "allowSeparateTypeImports": true, "includeExports": true }])),
        ),
    ];

    let fail = vec![
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
            r#"import "fs";
			import "fs""#,
            None,
        ),
        (
            r#"import { type Merge } from "lodash-es";
			import { type Find } from "lodash-es";"#,
            None,
        ),
        (
            r#"import { type Merge } from "lodash-es";
			import type { Find } from "lodash-es";"#,
            None,
        ),
        (
            r#"import type { Merge } from "lodash-es";
			import type { Find } from "lodash-es";"#,
            None,
        ),
        (
            r#"import type Os from "os";
			import type { Something } from "os";
			import type * as Foobar from "os";"#,
            None,
        ),
        (
            r#"import type * as Modns from "lodash-es";
			import type { Merge } from "lodash-es";
			import type { Baz } from "lodash-es";"#,
            None,
        ),
        (
            r#"import { type Foo } from "module";
			export type { Bar } from "module";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"export { os } from "os";
			export type { Something } from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"export type { Os } from "os";
			export type { Something } from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import type { Os } from "os";
			export type { Os as Foobar } from "os";
			export type { Something } from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import type { Os } from "os";
			export type { Something } from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import type Os from "os";
			export type * as Os from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import type * as Modns from "mod";
			export type * as Modns from "mod";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"export type * from "os";
			export type * from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            r#"import "os";
			export type { Os } from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
        (
            "import { someValue } from 'module';
			import { anotherValue } from 'module';",
            Some(serde_json::json!([{ "allowSeparateTypeImports": true }])),
        ),
        (
            r#"import type { Merge } from "lodash-es";
			import type { Find } from "lodash-es";"#,
            Some(serde_json::json!([{ "allowSeparateTypeImports": true }])),
        ),
        (
            "import { someValue, type Foo } from 'module';
			import type { SomeType } from 'module';
			import type { AnotherType } from 'module';",
            Some(serde_json::json!([{ "allowSeparateTypeImports": true }])),
        ),
        (
            "import { type Foo } from 'module';
			import { type Bar } from 'module';",
            Some(serde_json::json!([{ "allowSeparateTypeImports": true }])),
        ),
        (
            r#"export type { Foo } from "module";
			export type { Bar } from "module";"#,
            Some(serde_json::json!([{ "allowSeparateTypeImports": true, "includeExports": true }])),
        ),
        (
            r#"import { type Foo } from "module";
			export { type Bar } from "module";
			export { type Baz } from "module";"#,
            Some(serde_json::json!([{ "allowSeparateTypeImports": true, "includeExports": true }])),
        ),
        (
            r#"import { type Foo } from "module";
			export { type Bar } from "module";
			export { regular } from "module";"#,
            Some(serde_json::json!([{ "allowSeparateTypeImports": true, "includeExports": true }])),
        ),
        (
            r#"import { type Foo } from "module";
			import { regular } from "module";
			export { type Bar } from "module";
			export { regular as other } from "module";"#,
            Some(serde_json::json!([{ "allowSeparateTypeImports": true, "includeExports": true }])),
        ),
    ];

    Tester::new(NoDuplicateImports::NAME, NoDuplicateImports::PLUGIN, pass, fail)
        .test_and_snapshot();
}
