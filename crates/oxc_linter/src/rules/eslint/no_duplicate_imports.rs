use rustc_hash::FxHashMap;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{
    context::LintContext,
    module_record::{ExportImportName, ImportImportName},
    rule::Rule,
};

fn no_duplicate_imports_diagnostic(module_name: &str, span: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{module_name}' import is duplicated"))
        .with_help("Merge the duplicated import into a single import statement")
        .with_labels([
            span.label("This import is duplicated"),
            span2.label("Can be merged with this import"),
        ])
}

fn no_duplicate_exports_diagnostic(module_name: &str, span: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{module_name}' export is duplicated"))
        .with_help("Merge the duplicated exports into a single export statement")
        .with_labels([
            span.label("This export is duplicated"),
            span2.label("Can be merged with this"),
        ])
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
    eslint,
    style,
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
        let mut import_map: FxHashMap<&CompactStr, Vec<(ImportType, Span, ModuleType)>> =
            FxHashMap::default();
        let mut current_span: Option<Span> = None;
        let mut side_effect_import_map: FxHashMap<&CompactStr, Vec<Span>> = FxHashMap::default();

        for entry in &module_record.import_entries {
            let source = &entry.module_request.name;
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

            if let Some(existing) = import_map.get(source) {
                let can_merge = can_merge_imports(&import_type, existing, same_statement);
                if can_merge {
                    ctx.diagnostic(no_duplicate_imports_diagnostic(
                        source,
                        span,
                        existing.first().unwrap().1,
                    ));
                    continue;
                }
            }

            import_map.entry(source).or_default().push((import_type, span, ModuleType::Import));

            if !same_statement {
                current_span = Some(span);
            }
        }

        for (source, requests) in &module_record.requested_modules {
            for request in requests {
                if request.is_import && module_record.import_entries.is_empty() {
                    side_effect_import_map.entry(source).or_default().push(request.span);
                }
            }
        }

        for (source, spans) in &side_effect_import_map {
            if spans.len() > 1 {
                for span in spans {
                    let i = spans.iter().position(|s| s == span).unwrap();
                    if i > 0 {
                        ctx.diagnostic(no_duplicate_imports_diagnostic(
                            source,
                            *span,
                            *spans.first().unwrap(),
                        ));
                    }
                }
            }
        }

        if self.include_exports {
            for entry in &module_record.star_export_entries {
                if let Some(module_request) = &entry.module_request {
                    let source = &module_request.name;
                    let span = entry.span;

                    if entry.import_name.is_all_but_default() {
                        if let Some(existing) = import_map.get(source) {
                            if existing
                                .iter()
                                .any(|(t, _, _)| matches!(t, ImportType::AllButDefault))
                            {
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
                        if existing.iter().any(|(t, _, _)| {
                            matches!(t, ImportType::Named | ImportType::SideEffect)
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
                        ImportType::SideEffect,
                        span,
                        ModuleType::Export,
                    ));
                }
            }

            for entry in &module_record.indirect_export_entries {
                if let Some(module_request) = &entry.module_request {
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
}

fn can_merge_imports(
    current_type: &ImportType,
    existing: &[(ImportType, Span, ModuleType)],
    same_statement: bool,
) -> bool {
    if same_statement {
        return false;
    }

    let namespace = existing.iter().find(|(t, _, _)| matches!(t, ImportType::Namespace));
    let named = existing.iter().find(|(t, _, _)| matches!(t, ImportType::Named));
    let default = existing.iter().find(|(t, _, _)| matches!(t, ImportType::Default));

    let has_namespace = namespace.is_some();
    let has_named = named.is_some();
    let has_default = default.is_some();

    if matches!(current_type, ImportType::Named) && has_named {
        return true;
    }

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

    if matches!(current_type, ImportType::SideEffect) && !existing.is_empty() {
        return true;
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
        (
            r#"import "os";
        export * from "os";"#,
            Some(serde_json::json!([{ "includeExports": true }])),
        ),
    ];

    Tester::new(NoDuplicateImports::NAME, NoDuplicateImports::PLUGIN, pass, fail)
        .test_and_snapshot();
}
