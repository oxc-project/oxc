use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{
    ModuleRecord,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    module_graph_visitor::{ModuleGraphVisitorBuilder, VisitFoldWhile},
    module_record::ImportImportName,
    rule::Rule,
};

fn prefer_direct_import(span: Span, loaded_modules: Option<usize>) -> OxcDiagnostic {
    let additional_modules =
        loaded_modules.and_then(|total| total.checked_sub(1)).filter(|n| *n > 0);
    let mut diagnostic = OxcDiagnostic::warn("Avoid importing from barrel files.");

    diagnostic = if let Some(total) = additional_modules {
        diagnostic.with_help(format!(
            "Import directly from the specific modules instead of the barrel file; this barrel loads {total} additional module{}.",
            if total == 1 { "" } else { "s" }
        ))
    } else {
        diagnostic.with_help(
            "Import directly from the specific modules instead of the barrel file.",
        )
    }
    .with_label(span);

    diagnostic
}

#[derive(Debug, Default, Clone)]
pub struct PreferDirectImport;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows importing from a barrel file when the member can be imported directly from its source module instead.
    ///
    /// ### Why is this bad?
    ///
    /// Importing from barrel files can pull in additional modules or cause unintended side effects.
    /// See the `oxc/no-barrel-file` rule for more details on why barrel files are problematic.
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

impl Rule for PreferDirectImport {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();

        if !module_record.has_module_syntax {
            return;
        }

        // Group imports by statement span (to handle multiple imports from same statement)
        let mut import_statements: HashMap<Span, Vec<&crate::module_record::ImportEntry>> =
            HashMap::new();

        for import_entry in &module_record.import_entries {
            import_statements.entry(import_entry.statement_span).or_default().push(import_entry);
        }

        // Process each import statement
        for (_span, entries) in import_statements {
            if entries.is_empty() {
                continue;
            }

            let mut has_namespace = false;
            let mut default_import: Option<&crate::module_record::ImportEntry> = None;

            for entry in &entries {
                if entry.import_name.is_namespace_object() {
                    has_namespace = true;
                }
                if entry.import_name.is_default() {
                    default_import = Some(*entry);
                }
            }

            // Skip namespace imports (allowed - intentional)
            if has_namespace {
                continue;
            }

            // All entries in a group share the same module_request and statement_span
            let first_entry = entries[0];
            let specifier = first_entry.module_request.name();

            // Get the module we're importing from
            let Some(remote_module) = module_record.get_loaded_module(specifier) else {
                continue;
            };

            // Only check barrel files
            if !is_barrel_file(&remote_module) {
                continue;
            }

            let loaded_modules = count_loaded_modules(&remote_module);

            // Collect names that are imported from the barrel (not directly exported)
            let star_exports = remote_module.exported_bindings_from_star_export();
            let mut imports_by_source: HashMap<(PathBuf, bool), Vec<CompactStr>> = HashMap::new();
            let mut has_barrel_imports = false;

            for entry in &entries {
                if let ImportImportName::Name(import_name) = &entry.import_name {
                    let name = import_name.name();

                    // Check if directly exported (good - skip)
                    if remote_module.exported_bindings.contains_key(name) {
                        continue;
                    }

                    // Check if re-exported via star exports
                    let mut fallback_source: Option<&PathBuf> = None;
                    let mut preferred_source: Option<&PathBuf> = None;
                    for (source_path, names) in star_exports.iter() {
                        if !names.iter().any(|binding| binding.as_str() == name) {
                            continue;
                        }

                        if fallback_source.is_none() {
                            fallback_source = Some(source_path);
                        }

                        if !is_barrel_source(&remote_module, source_path).unwrap_or(false) {
                            preferred_source = Some(source_path);
                            break;
                        }
                    }

                    if let Some(source_path) = preferred_source.or(fallback_source) {
                        has_barrel_imports = true;
                        let local_name = entry.local_name.name();
                        let binding = if local_name == name {
                            CompactStr::from(name)
                        } else {
                            CompactStr::from(format!("{name} as {local_name}"))
                        };
                        imports_by_source
                            .entry((source_path.clone(), entry.is_type))
                            .or_default()
                            .push(binding);
                    }
                }
            }

            // Report diagnostic if we found barrel imports
            if has_barrel_imports {
                ctx.diagnostic_with_suggestion(
                    prefer_direct_import(first_entry.module_request.span, loaded_modules),
                    |fixer| {
                        fix_barrel_imports(
                            fixer,
                            first_entry,
                            default_import,
                            &imports_by_source,
                            &module_record.resolved_absolute_path,
                        )
                    },
                );
            }
        }
    }
}

fn is_barrel_file(module: &crate::ModuleRecord) -> bool {
    !module.star_export_entries.is_empty()
}

fn count_loaded_modules(module_record: &ModuleRecord) -> Option<usize> {
    if module_record.loaded_modules().is_empty() {
        return None;
    }
    Some(
        ModuleGraphVisitorBuilder::default()
            .visit_fold(0, module_record, |acc, _, _| VisitFoldWhile::Next(acc + 1))
            .result,
    )
}

fn is_barrel_source(remote_module: &ModuleRecord, source_path: &Path) -> Option<bool> {
    for module in remote_module.loaded_modules().values() {
        let Some(module) = module.upgrade() else {
            continue;
        };
        if module.resolved_absolute_path == source_path {
            return Some(is_barrel_file(&module));
        }
    }
    None
}

fn compute_relative_import_path(from: &Path, to: &Path) -> String {
    // Get the directory of the importing file
    let from_dir = from.parent().unwrap_or(from);

    // Get path components
    let from_components: Vec<_> = from_dir.components().collect();
    let to_components: Vec<_> = to.components().collect();

    // Find common prefix length
    let common_len =
        from_components.iter().zip(to_components.iter()).take_while(|(a, b)| a == b).count();

    // Build relative path
    let mut relative_parts = Vec::new();

    // Add ".." for each directory we need to go up
    let up_count = from_components.len() - common_len;
    for _ in 0..up_count {
        relative_parts.push("..".to_string());
    }

    // Add the remaining path components from 'to'
    for component in &to_components[common_len..] {
        if let Some(s) = component.as_os_str().to_str() {
            relative_parts.push(s.to_string());
        }
    }

    let mut path_str = relative_parts.join("/");

    // Remove file extension
    if let Some(without_ext) = path_str
        .strip_suffix(".ts")
        .or_else(|| path_str.strip_suffix(".tsx"))
        .or_else(|| path_str.strip_suffix(".js"))
        .or_else(|| path_str.strip_suffix(".jsx"))
        .or_else(|| path_str.strip_suffix(".mts"))
        .or_else(|| path_str.strip_suffix(".cts"))
        .or_else(|| path_str.strip_suffix(".mjs"))
        .or_else(|| path_str.strip_suffix(".cjs"))
    {
        path_str = without_ext.to_string();
    }

    // Ensure it starts with ./ or ../
    if !path_str.starts_with('.') {
        path_str = format!("./{}", path_str);
    }

    path_str
}

fn fix_barrel_imports(
    fixer: RuleFixer<'_, '_>,
    import_entry: &crate::module_record::ImportEntry,
    default_import: Option<&crate::module_record::ImportEntry>,
    imports_by_source: &HashMap<(PathBuf, bool), Vec<CompactStr>>,
    from: &Path,
) -> RuleFix {
    // Generate new import statements for each source file
    // Sort by source path for deterministic output
    let mut sorted_sources: Vec<_> = imports_by_source.iter().collect();
    sorted_sources
        .sort_by_key(|((path, is_type), _)| (path.to_string_lossy().to_string(), *is_type));

    let mut new_imports = Vec::new();

    if let Some(default_import) = default_import {
        let import_kind = if default_import.is_type { "import type" } else { "import" };
        let local_name = default_import.local_name.name();
        let specifier = import_entry.module_request.name();
        new_imports.push(format!("{import_kind} {local_name} from '{specifier}';"));
    }

    for ((source_path, is_type), names) in sorted_sources {
        let mut source_import_path = compute_relative_import_path(from, source_path);

        // Remove '/index' from the end (importing from dir is same as from dir/index)
        if source_import_path.ends_with("/index") {
            source_import_path = source_import_path.strip_suffix("/index").unwrap().to_string();
        }

        // Also sort names for determinism
        let mut sorted_names = names.clone();
        sorted_names.sort();
        let names_str =
            sorted_names.iter().map(|name| name.as_str()).collect::<Vec<_>>().join(", ");

        let import_kind = if *is_type { "import type" } else { "import" };

        new_imports
            .push(format!("{} {{ {} }} from '{}';", import_kind, names_str, source_import_path));
    }

    let replacement = new_imports.join("\n");

    fixer
        .replace(import_entry.statement_span, replacement)
        .with_message("Import from source files instead of barrel")
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Direct import from source
        (r#"import { User } from './barrel/user';"#, None),
        // Namespace imports (allowed - intentional)
        (r#"import * as models from './barrel';"#, None),
        // Direct imports from different sources
        (
            r#"import { User } from './barrel/user'; import { Product } from './barrel/product';"#,
            None,
        ),
        // Direct import with extension
        (r#"import { User } from './barrel/user.js';"#, None),
    ];

    let fail = vec![
        // Single named import from barrel
        (r#"import { User } from './barrel';"#, None),
        // Multiple imports from same source
        (r#"import { User, UserRole } from './barrel';"#, None),
        // Multiple imports from different sources
        (r#"import { User, Product } from './barrel';"#, None),
        // Alias import from barrel
        (r#"import { User as UserModel } from './barrel';"#, None),
        // Mixed type/value imports from barrel
        (r#"import { type User, Product } from './barrel';"#, None),
        // Default + named import from barrel
        (r#"import Order, { User } from './barrel';"#, None),
        // Import from barrel/index (same as barrel)
        (r#"import { User } from './barrel/index';"#, None),
        // Import from barrel/index.js with extension
        (r#"import { Product } from './barrel/index.js';"#, None),
        // Two-level deep barrel file
        (r#"import { User } from './barrel/deep';"#, None),
    ];

    let fix = vec![
        // Single import fix
        (r#"import { User } from './barrel';"#, r#"import { User } from './barrel/user';"#, None),
        // Multiple imports from same source
        (
            r#"import { User, UserRole } from './barrel';"#,
            r#"import { User, UserRole } from './barrel/user';"#,
            None,
        ),
        // Multiple imports from different sources
        (
            r#"import { User, Product } from './barrel';"#,
            r#"import { Product } from './barrel/product';
import { User } from './barrel/user';"#,
            None,
        ),
        // Alias import fix
        (
            r#"import { User as UserModel } from './barrel';"#,
            r#"import { User as UserModel } from './barrel/user';"#,
            None,
        ),
        // Mixed type/value import fix
        (
            r#"import { type User, Product } from './barrel';"#,
            r#"import { Product } from './barrel/product';
import type { User } from './barrel/user';"#,
            None,
        ),
        // Default + named import fix (keep default from barrel)
        (
            r#"import Order, { User } from './barrel';"#,
            r#"import Order from './barrel';
import { User } from './barrel/user';"#,
            None,
        ),
        // Import from barrel/index
        (
            r#"import { User } from './barrel/index';"#,
            r#"import { User } from './barrel/user';"#,
            None,
        ),
        // Import from barrel/index.js with extension
        (
            r#"import { Product } from './barrel/index.js';"#,
            r#"import { Product } from './barrel/product';"#,
            None,
        ),
        // Two-level deep barrel file - resolves to intermediate barrel
        // (Note: ideally would resolve to './barrel-deep/models/user', but the intermediate
        // barrel './barrel/deep/models' is still an improvement over the top-level barrel)
        (
            r#"import { User } from './barrel/deep';"#,
            r#"import { User } from './barrel/deep/models';"#,
            None,
        ),
    ];

    Tester::new(PreferDirectImport::NAME, PreferDirectImport::PLUGIN, pass, fail)
        .change_rule_path("index.ts")
        .with_import_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
