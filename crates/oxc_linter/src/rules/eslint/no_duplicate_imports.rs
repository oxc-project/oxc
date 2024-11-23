use std::collections::HashMap;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use oxc_syntax::module_record::{ExportImportName, ImportImportName};

use crate::{context::LintContext, rule::Rule};

fn no_duplicate_imports_diagnostic(module_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{}' import is duplicated", module_name))
        .with_help("Merge the duplicated import into a single import statement")
        .with_label(span)
}

fn no_duplicate_exports_diagnostic(module_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{}' export is duplicated", module_name))
        .with_help("Merge the duplicated exports into a single export statement")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDuplicateImports {
    include_exports: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallow duplicate module imports
    ///
    /// ### Why is this bad?
    /// Using a single import statement per module will make the code clearer because you can see everything being imported from that module on one line.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
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
    nursery,
    pending);

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
        let Some(value) = value.get(0) else { return Self { include_exports: false } };
        Self {
            include_exports: value
                .get("includeExports")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }
    }

    fn run_once(&self, ctx: &LintContext) {
        let module_record = ctx.module_record();
        let mut import_map: HashMap<&CompactStr, Vec<(ImportType, Span, ModuleType)>> =
            HashMap::new(); // Added bool for same_statement
        let mut current_span: Option<Span> = None;

        println!("source_text: {:?}", ctx.source_text());

        // Handle bare imports first
        if module_record.import_entries.is_empty() {
            for (source, requests) in &module_record.requested_modules {
                for request in requests {
                    if request.is_import() {
                        if let Some(existing) = import_map.get(source) {
                            // Bare imports can't be duplicated at all
                            if !existing.is_empty() {
                                ctx.diagnostic(no_duplicate_imports_diagnostic(
                                    source,
                                    request.span(),
                                ));
                                continue;
                            }
                        }
                        import_map.entry(source).or_default().push((
                            ImportType::SideEffect,
                            request.span(),
                            ModuleType::Import,
                        ));
                    }
                }
            }
        }
        // Handle regular imports
        for entry in &module_record.import_entries {
            let source = entry.module_request.name();
            let span = entry.module_request.span();

            let same_statement = if let Some(curr_span) = current_span {
                curr_span == span
            } else {
                current_span = Some(span);
                true
            };

            let import_type = match &entry.import_name {
                ImportImportName::Name(_) => ImportType::Named,
                ImportImportName::NamespaceObject => ImportType::Namespace,
                ImportImportName::Default(_) => ImportType::Default,
            };

            println!("- source {source:?}, import_type {import_type:?},  same_statement: {same_statement}");
            if let Some(existing) = import_map.get(source) {
                let can_merge = can_merge_imports(&import_type, existing, same_statement);
                if can_merge {
                    ctx.diagnostic(no_duplicate_imports_diagnostic(source, span));
                    continue;
                }
            }

            import_map.entry(source).or_default().push((import_type, span, ModuleType::Import));

            if !same_statement {
                current_span = Some(span);
            }
        }

        // Handle exports if includeExports is true
        if self.include_exports {
            // Handle star exports
            for entry in &module_record.star_export_entries {
                println!("star_export_entry: {:?}", entry);
                if let Some(module_request) = &entry.module_request {
                    let source = module_request.name();
                    let span = entry.span;

                    if entry.import_name.is_all_but_default() {
                        if let Some(existing) = import_map.get(source) {
                            if existing
                                .iter()
                                .any(|(t, _, _)| matches!(t, ImportType::AllButDefault))
                            {
                                ctx.diagnostic(no_duplicate_exports_diagnostic(source, span));
                                continue;
                            }
                        }
                        import_map.entry(source).or_default().push((
                            ImportType::AllButDefault,
                            span,
                            ModuleType::Export,
                        ));
                        continue;
                    }
                    if let Some(existing) = import_map.get(source) {
                        if existing.iter().any(|(t, _, _)| {
                            matches!(t, ImportType::Named | ImportType::SideEffect)
                        }) {
                            ctx.diagnostic(no_duplicate_exports_diagnostic(source, span));
                            continue;
                        }
                    }

                    import_map.entry(source).or_default().push((
                        ImportType::SideEffect,
                        span,
                        ModuleType::Export,
                    ));
                }
            }

            // Handle indirect exports
            for entry in &module_record.indirect_export_entries {
                if let Some(module_request) = &entry.module_request {
                    let source = module_request.name();
                    let span = entry.span;
                    println!("- source: {source:?}, span: {span:?}, type: indirect_export_entries");

                    if let Some(existing) = import_map.get(source) {
                        if entry.import_name == ExportImportName::All {
                            if existing.iter().any(|(t, _, _)| {
                                matches!(t, ImportType::Default | ImportType::Namespace)
                            }) {
                                ctx.diagnostic(no_duplicate_exports_diagnostic(source, span));
                                continue;
                            }

                            continue;
                        }

                        if existing.iter().any(|(t, _, module_type)| {
                            (matches!(
                                t,
                                ImportType::Named | ImportType::SideEffect | ImportType::Default
                            ) && *module_type == ModuleType::Export)
                                || (matches!(t, ImportType::Default)
                                    && *module_type == ModuleType::Import)
                        }) {
                            ctx.diagnostic(no_duplicate_exports_diagnostic(source, span));
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
}

fn can_merge_imports(
    current_type: &ImportType,
    existing: &[(ImportType, Span, ModuleType)],
    same_statement: bool,
) -> bool {
    println!("existing: {existing:?}");

    // Allow multiple imports in the same statement
    if same_statement {
        return false;
    }

    // this is the (namespace, span) if exsiting. None else

    let namespace = existing.iter().find(|(t, _, _)| matches!(t, ImportType::Namespace));
    let named = existing.iter().find(|(t, _, _)| matches!(t, ImportType::Named));
    let default = existing.iter().find(|(t, _, _)| matches!(t, ImportType::Default));

    let has_namespace = namespace.is_some();
    let has_named = named.is_some();
    let has_default = default.is_some();

    // For duplicate named imports
    if matches!(current_type, ImportType::Named) {
        if has_named {
            return true;
        }
    }

    // For namespace imports
    if matches!(current_type, ImportType::Namespace) {
        if has_named && has_default {
            if let Some((_, named_span, _)) = named {
                if let Some((_, default_span, _)) = default {
                    if named_span == default_span {
                        return false;
                    }
                }
            }
        }

        if has_namespace {
            return true;
        }
        if has_default && !same_statement {
            return true;
        }
    }

    // For default imports
    if matches!(current_type, ImportType::Default) {
        if has_default {
            return true;
        }
        if has_named && !same_statement {
            return true;
        }
        if has_namespace {
            return true;
        }
    }

    // For side effect imports
    if matches!(current_type, ImportType::SideEffect) {
        // Any existing import means it's a duplicate
        if !existing.is_empty() {
            return true;
        }
    }

    false
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
        // (
        //     r#"import "os";
        // export * from "os";"#,
        //     Some(serde_json::json!([{ "includeExports": true }])),
        // ),
    ];

    Tester::new(NoDuplicateImports::NAME, pass, fail).test_and_snapshot();
}
