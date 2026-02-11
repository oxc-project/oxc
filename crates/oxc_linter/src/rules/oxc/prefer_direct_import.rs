use std::path::{Path, PathBuf};

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span, VALID_EXTENSIONS};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    ModuleRecord,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    module_record::{ExportExportName, ExportImportName, ImportEntry, ImportImportName},
    rule::Rule,
};

const MAX_DIRECT_SOURCE_HELP_LIST: usize = 3;

fn prefer_direct_import(span: Span, direct_sources: &[String]) -> OxcDiagnostic {
    let direct_help = match direct_sources {
        [path] => format!("Import directly from `{path}`."),
        [] => "Import each binding from the module where it is defined.".to_string(),
        paths if paths.len() <= MAX_DIRECT_SOURCE_HELP_LIST => format!(
            "Import bindings from defining modules: {}.",
            paths.iter().map(|path| format!("`{path}`")).collect::<Vec<_>>().join(", ")
        ),
        paths => format!("Import bindings from defining modules ({} modules).", paths.len()),
    };

    OxcDiagnostic::warn("Prefer importing from defining modules.")
        .with_help(direct_help)
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferDirectImport;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows importing from a re-export module when the member can be
    /// imported directly from its source module.
    ///
    /// ### Why is this bad?
    ///
    /// Importing from re-export modules can pull in additional modules or cause
    /// unintended side effects.
    /// See the `oxc/no-barrel-file` rule for more details on why barrel files can
    /// be problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { User, Product, type Record } from './models';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { User } from './models/user';
    /// import { Product } from './models/product';
    /// import type { Record } from './models/record';
    ///
    /// // Namespace imports are allowed (intentional barrel usage)
    /// import * as models from './models';
    /// ```
    PreferDirectImport,
    oxc,
    style,
    suggestion
);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct ImportBinding {
    imported_name: CompactStr,
    local_name: CompactStr,
}

impl ImportBinding {
    fn new(imported_name: CompactStr, local_name: CompactStr) -> Self {
        Self { imported_name, local_name }
    }

    fn to_specifier(&self) -> String {
        if self.imported_name == self.local_name {
            self.imported_name.to_string()
        } else {
            format!("{} as {}", self.imported_name, self.local_name)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ResolvedSource {
    path: PathBuf,
    imported_name: CompactStr,
}

fn is_reexport_module(module: &ModuleRecord) -> bool {
    !module.star_export_entries.is_empty()
        || module.indirect_export_entries.iter().any(|entry| entry.module_request.is_some())
}

fn has_local_export(module: &ModuleRecord, export_name: &str) -> bool {
    if export_name == "default" {
        return module.local_export_entries.iter().any(|entry| entry.export_name.is_default());
    }

    module.local_export_entries.iter().any(
        |entry| matches!(&entry.export_name, ExportExportName::Name(name) if name.name() == export_name),
    )
}

fn merge_candidate(candidate: &mut Option<ResolvedSource>, resolved: ResolvedSource) -> bool {
    match candidate {
        None => {
            *candidate = Some(resolved);
            true
        }
        Some(existing) => existing == &resolved,
    }
}

fn resolve_reexport_source(
    module: &ModuleRecord,
    export_name: &str,
    visited: &mut FxHashSet<(PathBuf, CompactStr)>,
    allow_local_export: bool,
) -> Option<ResolvedSource> {
    let state = (module.resolved_absolute_path.clone(), CompactStr::from(export_name));
    if !visited.insert(state.clone()) {
        return None;
    }

    let resolved = (|| -> Option<ResolvedSource> {
        if allow_local_export && has_local_export(module, export_name) {
            return Some(ResolvedSource {
                path: module.resolved_absolute_path.clone(),
                imported_name: CompactStr::from(export_name),
            });
        }

        let mut indirect_candidate = None;
        for export_entry in &module.indirect_export_entries {
            let ExportExportName::Name(exported_name) = &export_entry.export_name else {
                continue;
            };
            if exported_name.name() != export_name {
                continue;
            }

            let Some(module_request) = &export_entry.module_request else {
                continue;
            };
            let ExportImportName::Name(imported_name) = &export_entry.import_name else {
                continue;
            };
            let Some(remote_module) = module.get_loaded_module(module_request.name()) else {
                continue;
            };

            let imported_name = imported_name.name();
            if let Some(resolved) =
                resolve_reexport_source(&remote_module, imported_name, visited, true)
            {
                if !merge_candidate(&mut indirect_candidate, resolved) {
                    return None;
                }
            }
        }

        if let Some(indirect_candidate) = indirect_candidate {
            return Some(indirect_candidate);
        }

        if export_name == "default" {
            return None;
        }

        let mut star_candidate = None;
        for export_entry in &module.star_export_entries {
            let Some(module_request) = &export_entry.module_request else {
                continue;
            };
            let Some(remote_module) = module.get_loaded_module(module_request.name()) else {
                continue;
            };

            if let Some(resolved) =
                resolve_reexport_source(&remote_module, export_name, visited, true)
            {
                if !merge_candidate(&mut star_candidate, resolved) {
                    return None;
                }
            }
        }

        star_candidate
    })();

    visited.remove(&state);
    resolved
}

fn compute_relative_import_path(from: &Path, to: &Path) -> String {
    let from_dir = from.parent().unwrap_or(from);
    let from_components: Vec<_> = from_dir.components().collect();
    let to_components: Vec<_> = to.components().collect();
    let common_len =
        from_components.iter().zip(to_components.iter()).take_while(|(a, b)| a == b).count();

    let mut relative_parts = Vec::new();
    let up_count = from_components.len() - common_len;
    for _ in 0..up_count {
        relative_parts.push("..".to_string());
    }

    for component in &to_components[common_len..] {
        let part = component.as_os_str().to_string_lossy();
        if !part.is_empty() {
            relative_parts.push(part.into_owned());
        }
    }

    let mut path_str = relative_parts.join("/");

    if let Some((without_ext, ext)) = path_str.rsplit_once('.')
        && VALID_EXTENSIONS.contains(&ext)
    {
        path_str = without_ext.to_string();
    }

    if let Some(without_index) = path_str.strip_suffix("/index") {
        path_str = without_index.to_string();
    }

    if !path_str.starts_with('.') {
        path_str = format!("./{path_str}");
    }

    path_str
}

fn push_named_import(
    output: &mut Vec<String>,
    source: &str,
    is_type: bool,
    bindings: &[ImportBinding],
) {
    if bindings.is_empty() {
        return;
    }

    let mut specifiers = bindings.iter().map(ImportBinding::to_specifier).collect::<Vec<_>>();
    specifiers.sort();
    specifiers.dedup();
    let import_kind = if is_type { "import type" } else { "import" };
    output.push(format!("{import_kind} {{ {} }} from '{source}';", specifiers.join(", ")));
}

fn normalized_path_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn fix_imports(
    fixer: RuleFixer<'_, '_>,
    import_entry: &ImportEntry,
    default_import: Option<&ImportEntry>,
    remaining_named_imports: &FxHashMap<bool, Vec<ImportBinding>>,
    imports_by_source: &FxHashMap<(PathBuf, bool), Vec<ImportBinding>>,
    from: &Path,
) -> RuleFix {
    let mut new_imports = Vec::new();

    if let Some(default_import) = default_import {
        let import_kind = if default_import.is_type { "import type" } else { "import" };
        let local_name = default_import.local_name.name();
        let source = import_entry.module_request.name();
        new_imports.push(format!("{import_kind} {local_name} from '{source}';"));
    }

    for is_type in [false, true] {
        if let Some(bindings) = remaining_named_imports.get(&is_type) {
            push_named_import(
                &mut new_imports,
                import_entry.module_request.name(),
                is_type,
                bindings,
            );
        }
    }

    let mut sorted_sources = imports_by_source.iter().collect::<Vec<_>>();
    sorted_sources.sort_by_key(|((path, is_type), _)| (normalized_path_key(path), *is_type));
    for ((source_path, is_type), bindings) in sorted_sources {
        let source_import_path = compute_relative_import_path(from, source_path);
        push_named_import(&mut new_imports, &source_import_path, *is_type, bindings);
    }

    let replacement = new_imports.join("\n");
    fixer
        .replace(import_entry.statement_span, replacement)
        .with_message("Import from defining modules instead of re-export module")
}

impl Rule for PreferDirectImport {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();
        if !module_record.has_module_syntax {
            return;
        }

        let mut import_statements: FxHashMap<Span, Vec<&ImportEntry>> = FxHashMap::default();
        let mut statement_order = Vec::new();

        for import_entry in &module_record.import_entries {
            let entries =
                import_statements.entry(import_entry.statement_span).or_insert_with(|| {
                    statement_order.push(import_entry.statement_span);
                    Vec::new()
                });
            entries.push(import_entry);
        }

        for statement_span in statement_order {
            let Some(entries) = import_statements.get(&statement_span) else {
                continue;
            };
            if entries.iter().any(|entry| entry.import_name.is_namespace_object()) {
                continue;
            }

            let first_entry = entries[0];
            let Some(remote_module) =
                module_record.get_loaded_module(first_entry.module_request.name())
            else {
                continue;
            };
            if !is_reexport_module(&remote_module) {
                continue;
            }

            let mut default_import: Option<&ImportEntry> = None;
            let mut remaining_named_imports: FxHashMap<bool, Vec<ImportBinding>> =
                FxHashMap::default();
            let mut imports_by_source: FxHashMap<(PathBuf, bool), Vec<ImportBinding>> =
                FxHashMap::default();
            let mut has_barrel_imports = false;

            for entry in entries {
                match &entry.import_name {
                    ImportImportName::Name(import_name) => {
                        let imported_name = import_name.name();
                        let local_name = entry.local_name.name();
                        let mut visited = FxHashSet::default();
                        if let Some(source) = resolve_reexport_source(
                            &remote_module,
                            imported_name,
                            &mut visited,
                            false,
                        ) {
                            has_barrel_imports = true;
                            imports_by_source
                                .entry((source.path, entry.is_type))
                                .or_default()
                                .push(ImportBinding::new(
                                    source.imported_name,
                                    CompactStr::from(local_name),
                                ));
                        } else {
                            remaining_named_imports.entry(entry.is_type).or_default().push(
                                ImportBinding::new(
                                    CompactStr::from(imported_name),
                                    CompactStr::from(local_name),
                                ),
                            );
                        }
                    }
                    ImportImportName::Default(_) => {
                        default_import = Some(*entry);
                    }
                    ImportImportName::NamespaceObject => {
                        unreachable!("namespace imports are filtered above")
                    }
                }
            }

            if !has_barrel_imports {
                continue;
            }

            let mut direct_sources = imports_by_source
                .keys()
                .map(|(source_path, _)| {
                    compute_relative_import_path(&module_record.resolved_absolute_path, source_path)
                })
                .collect::<Vec<_>>();
            direct_sources.sort();
            direct_sources.dedup();

            ctx.diagnostic_with_suggestion(
                prefer_direct_import(first_entry.statement_span, &direct_sources),
                |fixer| {
                    fix_imports(
                        fixer,
                        first_entry,
                        default_import,
                        &remaining_named_imports,
                        &imports_by_source,
                        &module_record.resolved_absolute_path,
                    )
                },
            );
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"import { User } from './barrel/user';"#, None),
        (r#"import type { RecordType } from './barrel/deep/record/record';"#, None),
        (r#"import { fromIndex } from './barrel/deep';"#, None),
        (r#"import * as models from './barrel';"#, None),
        (r#"import { User as UserModel } from './barrel/user';"#, None),
    ];

    let fail = vec![
        (r#"import { User } from './barrel';"#, None),
        (r#"import { User, Product } from './barrel';"#, None),
        (r#"import { type RecordType, Product } from './barrel';"#, None),
        (
            r#"import {
  User,
  Product,
  type RecordType,
  UserModel as Person,
} from './barrel';"#,
            None,
        ),
        (r#"import { fromIndex, RecordType } from './barrel/deep';"#, None),
        (r#"import { UserModel } from './barrel';"#, None),
        (r#"import { UserModel as Person } from './barrel';"#, None),
        (r#"import type { ModelRecord } from './barrel';"#, None),
        (r#"import { User } from './barrel/index';"#, None),
        (r#"import { UserModel } from './barrel/renamed';"#, None),
    ];

    let fix = vec![
        ("import { User } from './barrel';", "import { User } from './barrel/user';", None),
        (
            "import { User, Product } from './barrel';",
            "import { Product } from './barrel/product';\nimport { User } from './barrel/user';",
            None,
        ),
        (
            "import { type RecordType, Product } from './barrel';",
            "import type { RecordType } from './barrel/deep/record/record';\nimport { Product } from './barrel/product';",
            None,
        ),
        (
            "import {\n  User,\n  Product,\n  type RecordType,\n  UserModel as Person,\n} from './barrel';",
            "import type { RecordType } from './barrel/deep/record/record';\nimport { Product } from './barrel/product';\nimport { User, User as Person } from './barrel/user';",
            None,
        ),
        (
            "import { fromIndex, RecordType } from './barrel/deep';",
            "import { fromIndex } from './barrel/deep';\nimport { RecordType } from './barrel/deep/record/record';",
            None,
        ),
        (
            "import { UserModel } from './barrel';",
            "import { User as UserModel } from './barrel/user';",
            None,
        ),
        (
            "import { UserModel as Person } from './barrel';",
            "import { User as Person } from './barrel/user';",
            None,
        ),
        (
            "import type { ModelRecord } from './barrel';",
            "import type { RecordType as ModelRecord } from './barrel/deep/record/record';",
            None,
        ),
        ("import { User } from './barrel/index';", "import { User } from './barrel/user';", None),
        (
            "import { UserModel } from './barrel/renamed';",
            "import { User as UserModel } from './barrel/user';",
            None,
        ),
    ];

    Tester::new(PreferDirectImport::NAME, PreferDirectImport::PLUGIN, pass, fail)
        .change_rule_path("index.ts")
        .with_import_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
