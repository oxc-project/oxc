use std::collections::BTreeMap;

use fast_glob::glob_match;
use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_resolver::NODEJS_BUILTINS;
use oxc_span::{CompactStr, Span};
use oxc_syntax::module_record::RequestedModule;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

fn extension_should_not_be_included_in_diagnostic(
    span: Span,
    extension: &CompactStr,
    is_import: bool,
) -> OxcDiagnostic {
    let import_or_export = if is_import { "import" } else { "export" };

    OxcDiagnostic::warn(format!(
        r#"File extension "{extension}" should not be included in the {import_or_export} declaration."#
    ))
    .with_help(format!("Remove the file extension from this {import_or_export}."))
    .with_label(span)
}

fn extension_missing_diagnostic(span: Span, is_import: bool) -> OxcDiagnostic {
    let import_or_export = if is_import { "import" } else { "export" };

    OxcDiagnostic::warn(format!("Missing file extension in {import_or_export} declaration"))
        .with_help(format!("Add a file extension to this {import_or_export}."))
        .with_label(span)
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum FileExtensionConfig {
    Always,
    #[default]
    Never,
    IgnorePackages,
}

impl FileExtensionConfig {
    pub fn from(s: &str) -> FileExtensionConfig {
        match s {
            "always" => FileExtensionConfig::Always,
            "never" => FileExtensionConfig::Never,
            "ignorePackages" => FileExtensionConfig::IgnorePackages,
            _ => FileExtensionConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
enum OverrideAction {
    #[default]
    Enforce,
    Ignore,
}

#[derive(Debug, Clone, Default, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
struct PathGroupOverride {
    pattern: String,
    action: OverrideAction,
}

#[derive(Debug, Clone, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ExtensionsConfig {
    /// Whether to ignore package imports (e.g., 'react', 'lodash') when enforcing extension rules.
    ignore_packages: bool,
    /// Configuration for requiring or disallowing file extensions in import/require statements.
    require_extension: Option<FileExtensionConfig>,
    /// Whether to check type imports when enforcing extension rules.
    check_type_imports: bool,
    extensions: BTreeMap<String, FileExtensionConfig>,
    default_config: FileExtensionConfig,
    path_group_overrides: Vec<PathGroupOverride>,
    has_wildcard: bool, // Whether "*" pattern was explicitly set
}

impl ExtensionsConfig {
    fn is_never(&self, ext: &str) -> bool {
        self.extensions.get(ext).is_some_and(|config| matches!(config, FileExtensionConfig::Never))
    }

    fn get_modifier(&self, ext: &str) -> &FileExtensionConfig {
        self.extensions.get(ext).unwrap_or(&self.default_config)
    }
}

impl Default for ExtensionsConfig {
    fn default() -> Self {
        // Pre-populate standard extensions with "Never" to match the original behavior
        // when no configuration is provided
        let mut extensions = BTreeMap::new();
        for ext in ["js", "jsx", "ts", "tsx", "json"] {
            extensions.insert(ext.to_string(), FileExtensionConfig::Never);
        }

        Self {
            ignore_packages: false, // ESLint default is false
            require_extension: None,
            check_type_imports: false,
            extensions,
            default_config: FileExtensionConfig::Never,
            path_group_overrides: Vec::new(),
            has_wildcard: false,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Extensions(Box<ExtensionsConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Some file resolve algorithms allow you to omit the file extension within the import source path.
    /// For example the node resolver (which does not yet support ESM/import) can resolve ./foo/bar to the absolute path /User/someone/foo/bar.js because the .js extension is resolved automatically by default in CJS.
    /// Depending on the resolver you can configure more extensions to get resolved automatically.
    /// In order to provide a consistent use of file extensions across your code base, this rule can enforce or disallow the use of certain file extensions.
    ///
    /// ### Why is this bad?
    ///
    /// ESM-based file resolve algorithms (e.g., the one that Vite provides) recommend specifying the file extension to improve performance.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// The following patterns are considered problems when configuration set to "always":
    /// ```js
    /// import foo from './foo';
    /// import bar from './bar';
    /// import Component from './Component';
    /// import foo from '@/foo';
    /// ```
    ///
    /// The following patterns are considered problems when configuration set to "never":
    /// ```js
    /// import foo from './foo.js';
    /// import bar from './bar.json';
    /// import Component from './Component.jsx';
    /// import express from 'express/index.js';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// The following patterns are not considered problems when configuration set to "always":
    ///
    /// ```js
    /// import foo from './foo.js';
    /// import bar from './bar.json';
    /// import Component from './Component.jsx';
    /// import * as path from 'path';
    /// import foo from '@/foo.js';
    /// ```
    ///
    /// The following patterns are not considered problems when configuration set to "never":
    /// ```js
    /// import foo from './foo';
    /// import bar from './bar';
    /// import Component from './Component';
    /// import express from 'express/index';
    /// import * as path from 'path';
    /// ```
    Extensions,
    import,
    restriction,
    config = ExtensionsConfig,
);

impl Rule for Extensions {
    fn from_configuration(value: serde_json::Value) -> Self {
        if let Some(first_arg) = value.get(0).and_then(Value::as_str) {
            let default = FileExtensionConfig::from(first_arg);

            if let Some(val) = value.get(1) {
                let root = val.get("pattern").unwrap_or(val);

                let config = build_config(root, Some(&default));

                Self(Box::new(config))
            } else {
                let config = build_config(&value, Some(&default));

                Self(Box::new(config))
            }
        } else {
            let config = build_config(&value, None);
            Self(Box::new(config))
        }
    }

    fn run_once(&self, ctx: &LintContext) {
        let module_record = ctx.module_record();

        let config = self.0.clone();

        for node in ctx.nodes().iter() {
            if let AstKind::CallExpression(call_expr) = node.kind() {
                let Expression::Identifier(ident) = &call_expr.callee else {
                    return;
                };
                let func_name = ident.name.as_str();
                let count = call_expr.arguments.len();

                if matches!(func_name, "require") && count > 0 {
                    self.process_require_record(call_expr, ctx, config.require_extension.as_ref());
                }
            }
        }

        for (module_name, module) in &module_record.requested_modules {
            for module_item in module {
                self.process_module_record(
                    (module_name.clone(), module_item),
                    ctx,
                    config.require_extension.as_ref(),
                    config.check_type_imports,
                    config.ignore_packages,
                    module_item.is_import,
                );
            }
        }
    }
}

fn build_config(
    value: &serde_json::Value,
    default: Option<&FileExtensionConfig>,
) -> ExtensionsConfig {
    let mut extensions = BTreeMap::new();
    let default_config = default.cloned().unwrap_or_default();

    // If no explicit mode provided, pre-populate standard extensions with default
    // This preserves backward compatibility with the original behavior
    if default.is_none() {
        for ext in ["js", "jsx", "ts", "tsx", "json"] {
            extensions.insert(ext.to_string(), default_config.clone());
        }
    }

    // Parse extension-specific configurations
    if let Some(obj) = value.as_object() {
        for (key, val) in obj {
            // Skip known non-extension keys
            if matches!(
                key.as_str(),
                "ignorePackages" | "checkTypeImports" | "pattern" | "pathGroupOverrides"
            ) {
                continue;
            }

            // Handle wildcard "*" as default config
            if key == "*" {
                // Wildcard will be handled separately
                continue;
            }

            // Parse extension config
            if let Some(config_str) = val.as_str() {
                extensions.insert(key.clone(), FileExtensionConfig::from(config_str));
            }
        }
    }

    // Check for wildcard "*" pattern to set default
    let has_wildcard = value.get("*").is_some();
    let default_config = value
        .get("*")
        .and_then(Value::as_str)
        .map(FileExtensionConfig::from)
        .unwrap_or(default_config);

    // Parse pathGroupOverrides
    let mut path_group_overrides = Vec::new();
    if let Some(overrides_array) = value.get("pathGroupOverrides").and_then(Value::as_array) {
        for override_obj in overrides_array {
            let Some(obj) = override_obj.as_object() else {
                continue;
            };
            let Some(pattern) = obj.get("pattern").and_then(Value::as_str) else {
                continue;
            };
            let Some(action) = obj.get("action").and_then(Value::as_str) else {
                continue;
            };
            let action_enum = match action {
                "enforce" => OverrideAction::Enforce,
                "ignore" => OverrideAction::Ignore,
                _ => continue,
            };
            path_group_overrides
                .push(PathGroupOverride { pattern: pattern.to_string(), action: action_enum });
        }
    }

    // Handle ignorePackages flag
    // When first arg is "ignorePackages", it means: require_extension="always" + ignore_packages=true
    // When it's a boolean in config object, use that value
    let ignore_packages = if matches!(default, Some(FileExtensionConfig::IgnorePackages)) {
        // First arg was "ignorePackages" string - this means ignore_packages=true
        true
    } else {
        // Check for explicit boolean value in config object
        value.get("ignorePackages").and_then(Value::as_bool).unwrap_or(false)
    };

    // Transform "ignorePackages" mode to "always" (matching ESLint behavior)
    let require_extension = if matches!(default, Some(FileExtensionConfig::IgnorePackages)) {
        Some(FileExtensionConfig::Always)
    } else {
        default.cloned()
    };

    ExtensionsConfig {
        ignore_packages,
        require_extension,
        check_type_imports: value
            .get("checkTypeImports")
            .and_then(Value::as_bool)
            .unwrap_or_default(),
        extensions,
        default_config,
        path_group_overrides,
        has_wildcard,
    }
}

impl Extensions {
    fn process_module_record(
        &self,
        module_record: (CompactStr, &RequestedModule),
        ctx: &LintContext,
        require_extension: Option<&FileExtensionConfig>,
        check_type_imports: bool,
        ignore_packages: bool,
        is_import: bool,
    ) {
        let config = &self.0;
        let (module_name, module) = module_record;

        if module.is_type && !check_type_imports {
            return;
        }

        let is_builtin_node_module = NODEJS_BUILTINS.binary_search(&module_name.as_str()).is_ok()
            || ctx.globals().is_enabled(module_name.as_str());

        // Determine if this is a package import (external module)
        // Since oxc doesn't do file resolution, we use heuristics:
        // - Scoped packages: "@scope/package" or "@scope/package/subpath" (scope is alphanumeric)
        // - Regular packages: "package" or "package/subpath"
        // - Relative paths: start with "." or ".."
        // - Path aliases like "@/" are NOT packages (@ must be followed by alphanumeric)

        let is_relative = module_name.as_str().starts_with('.');
        let starts_with_word_char =
            module_name.chars().next().is_some_and(|c| c.is_alphanumeric() || c == '_');

        // Check if this is a scoped package (must be @scope/... where scope is alphanumeric)
        let is_scoped = if module_name.as_str().starts_with('@') {
            // Must have a scope name after @, not just @/ or @.
            module_name
                .as_str()
                .get(1..)
                .and_then(|s| s.chars().next())
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
        } else {
            false
        };

        // This is a package if it's scoped or starts with a word character (not relative)
        let is_package_root = is_scoped || (!is_relative && starts_with_word_char);

        if is_builtin_node_module {
            return;
        }

        // For root package imports without subpaths, skip extension checking entirely
        // because package names can contain dots (e.g., "decimal.js", "pkg.config.js")
        let has_subpath = if is_scoped {
            // For scoped packages like "@scope/pkg", the first '/' is part of the package name
            // A subpath would be "@scope/pkg/subpath" (more than one '/')
            module_name.matches('/').count() > 1
        } else {
            // For regular packages, any '/' indicates a subpath
            module_name.contains('/')
        };

        // Check pathGroupOverrides FIRST, before any package ignoring logic
        // This allows "enforce" to override ignorePackages for specific patterns
        let mut should_enforce = false;
        for override_item in &config.path_group_overrides {
            if glob_match(&override_item.pattern, module_name.as_str()) {
                match override_item.action {
                    OverrideAction::Ignore => {
                        // Skip validation for this import
                        return;
                    }
                    OverrideAction::Enforce => {
                        // Mark that we should enforce validation even if ignorePackages=true
                        should_enforce = true;
                        break;
                    }
                }
            }
        }

        if is_package_root && !has_subpath {
            // Root package name only, no subpath
            if ignore_packages && !should_enforce {
                return;
            }
            // Even without ignore_packages, we can't reliably extract extensions from package names
            return;
        }

        // For package subpaths (e.g., "lodash/map", "@babel/core/lib/index"),
        // handle based on ignore_packages flag (unless overridden by enforce)
        if is_package_root && ignore_packages && !should_enforce {
            return;
        }

        // At this point, it's either:
        // - A relative path (always validate)
        // - A package subpath with ignore_packages=false (validate)
        // - A package subpath with should_enforce=true (validate)
        // Continue to extension checking...

        let file_extension = get_file_extension_from_module_name(&module_name);

        let span = module.statement_span;

        if let Some(file_extension) = file_extension {
            let ext_str = file_extension.as_str();
            let modifier = config.get_modifier(ext_str);

            let should_flag = match require_extension {
                Some(FileExtensionConfig::Always) => {
                    // In "always" mode, flag only if modifier says "never"
                    // Unknown extensions inherit the "always" default
                    matches!(modifier, FileExtensionConfig::Never)
                }
                Some(FileExtensionConfig::Never) => {
                    // In "never" mode, we can't use file resolution like ESLint does
                    // to determine which extensions are resolvable. To avoid false positives,
                    // we only flag explicitly configured extensions or standard JS/TS extensions
                    // that are likely to be resolvable.
                    //
                    // However, if wildcard "*" is set to "never", we should check ALL extensions
                    //
                    // ESLint behavior: only flags resolvable extensions (determined by resolver config)
                    // oxc behavior: flag standard extensions, explicitly configured ones, or all if wildcard set
                    if matches!(modifier, FileExtensionConfig::Always) {
                        false // Explicitly allowed
                    } else if config.extensions.contains_key(ext_str) {
                        // Explicitly configured - respect the configuration
                        matches!(modifier, FileExtensionConfig::Never)
                    } else if config.has_wildcard
                        && matches!(config.default_config, FileExtensionConfig::Never)
                    {
                        // Wildcard "*" is set to "never" - flag all extensions
                        true
                    } else {
                        // Not explicitly configured, no wildcard - only flag standard JS/TS extensions
                        matches!(ext_str, "js" | "ts" | "mjs" | "cjs" | "mts" | "cts")
                    }
                }
                Some(FileExtensionConfig::IgnorePackages) | None => {
                    // In ignorePackages or default mode, only flag explicitly configured "never"
                    config.is_never(ext_str)
                }
            };

            if should_flag {
                ctx.diagnostic(extension_should_not_be_included_in_diagnostic(
                    span,
                    &file_extension,
                    is_import,
                ));
            }
        } else {
            // Missing extension - check if it should be required
            let should_require = matches!(
                require_extension,
                Some(FileExtensionConfig::Always | FileExtensionConfig::IgnorePackages)
            );

            if should_require {
                ctx.diagnostic(extension_missing_diagnostic(span, is_import));
            }
        }
    }

    fn process_require_record(
        &self,
        call_expr: &CallExpression<'_>,
        ctx: &LintContext,
        require_extension: Option<&FileExtensionConfig>,
    ) {
        let config = &self.0;
        for argument in &call_expr.arguments {
            if let Argument::StringLiteral(s) = argument {
                let file_extension = get_file_extension_from_module_name(&s.value.to_compact_str());
                let span = call_expr.span;

                if let Some(file_extension) = file_extension {
                    let ext_str = file_extension.as_str();
                    let modifier = config.get_modifier(ext_str);

                    let should_flag = match require_extension {
                        Some(FileExtensionConfig::Always) => {
                            // In "always" mode, flag only if modifier says "never"
                            // Unknown extensions inherit the "always" default
                            matches!(modifier, FileExtensionConfig::Never)
                        }
                        Some(FileExtensionConfig::Never) => {
                            // In "never" mode, flag unless modifier says "always"
                            // Unknown extensions inherit the "never" default
                            !matches!(modifier, FileExtensionConfig::Always)
                        }
                        Some(FileExtensionConfig::IgnorePackages) | None => {
                            // In ignorePackages or default mode, only flag explicitly configured "never"
                            config.is_never(ext_str)
                        }
                    };

                    if should_flag {
                        ctx.diagnostic(extension_should_not_be_included_in_diagnostic(
                            span,
                            &file_extension,
                            true,
                        ));
                    }
                } else {
                    // Missing extension - check if it should be required
                    let should_require = matches!(
                        require_extension,
                        Some(FileExtensionConfig::Always | FileExtensionConfig::IgnorePackages)
                    );

                    if should_require {
                        ctx.diagnostic(extension_missing_diagnostic(span, true));
                    }
                }
            }
        }
    }
}
fn get_file_extension_from_module_name(module_name: &CompactStr) -> Option<CompactStr> {
    if let Some((_, extension)) =
        module_name.split('?').next().unwrap_or(module_name).rsplit_once('.')
        && !extension.is_empty()
        && !extension.starts_with('/')
    {
        return Some(CompactStr::from(extension));
    }

    None
}

// Test suite based on eslint-plugin-import's extensions rule tests
// Source: https://github.com/import-js/eslint-plugin-import/blob/main/tests/src/rules/extensions.js
//
// Key differences from eslint-plugin-import due to lack of file resolution:
// 1. In "never" mode, oxc only flags standard JS/TS extensions (.js, .ts, .mjs, .cjs, .mts, .cts)
//    unless extensions are explicitly configured or wildcard "*" is set
// 2. Package root names (e.g., "pkg.js", "@name/pkg.js") are never checked since dots in
//    package names are valid and we can't resolve whether they're packages
// 3. For file paths like "./file.with.dot" we can't determine the "real" extension without
//    resolution, so we parse based on the last dot
//
#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        // ============================================================================
        // Basic cases - from eslint-plugin-import
        // ============================================================================
        (r#"import a from "@/a""#, None),
        (r#"import a from "a""#, None),
        (r#"import dot from "./file.with.dot""#, None),
        // ============================================================================
        // "always" mode - extensions required
        // ============================================================================
        (r#"import a from "a/index.js""#, Some(json!(["always"]))),
        (r#"import dot from "./file.with.dot.js""#, Some(json!(["always"]))),
        (r#"import thing from "./fake-file.js""#, Some(json!(["always"]))),
        (r#"import bare from "./foo.js?a=True""#, Some(json!(["always"]))),
        // ============================================================================
        // "never" mode - no extensions
        // ============================================================================
        (r#"import thing from "non-package""#, Some(json!(["never"]))),
        (r#"import lib from "./bar""#, Some(json!(["never"]))),
        (r#"import bare from "./foo?a=True.ext""#, Some(json!(["never"]))),
        // oxc difference: In "never" mode without explicit config, we only flag standard JS/TS
        // extensions. Non-standard extensions like .css, .hbs pass to avoid false positives.
        // ESLint would only flag extensions in its resolver config.
        (
            r#"
                import component from "./bar.jsx";
                import data from "./bar.json";
                import styles from "./styles.css";
                import template from "./template.hbs";
            "#,
            Some(json!(["never"])),
        ),
        // ============================================================================
        // Mixed extension policies - from eslint-plugin-import
        // ============================================================================
        (
            r#"
                import a from "a";
                import packageConfig from "./package.json";
            "#,
            Some(json!({"json": "always", "js": "never"})),
        ),
        (
            r#"
                import lib from "./bar";
                import component from "./bar.jsx";
                import data from "./bar.json";
            "#,
            Some(json!(["never", { "jsx": "always", "json": "always"}])),
        ),
        (
            r#"
                import bar from "./bar.js";
                import pack from "./package";
            "#,
            Some(json!(["never", { "js": "always", "json": "never"}])),
        ),
        // ============================================================================
        // Unresolved paths (Node builtins, etc) - from eslint-plugin-import
        // ============================================================================
        (r#"import path from "path""#, None),
        (r#"import path from "path""#, Some(json!(["never"]))),
        (r#"import path from "path""#, Some(json!(["always"]))),
        // ============================================================================
        // ignorePackages mode - from eslint-plugin-import
        // ============================================================================
        (
            r#"
                import foo from "./foo.js";
                import bar from "./bar.json";
                import Component from "./Component.jsx";
                import express from "express";
            "#,
            Some(json!(["ignorePackages"])),
        ),
        (
            r#"
                import foo from "./foo.js";
                import bar from "./bar.json";
                import Component from "./Component.jsx";
                import express from "express";
            "#,
            Some(json!(["always", { "ignorePackages": true}])),
        ),
        (
            r#"
                import foo from "./foo";
                import bar from "./bar";
                import Component from "./Component";
                import express from "express";
            "#,
            Some(json!(["never", { "ignorePackages": true}])),
        ),
        // ============================================================================
        // Export statements - from eslint-plugin-import
        // ============================================================================
        (
            r#"
                export { foo } from "./foo.js";
                let bar; export { bar };
            "#,
            Some(json!(["always"])),
        ),
        (
            r#"
                export { foo } from "./foo";
                let bar; export { bar };
            "#,
            Some(json!(["never"])),
        ),
        // ============================================================================
        // Package root names - from eslint-plugin-import
        // oxc treats these as package names (not file paths), so dots are ignored
        // ============================================================================
        (
            r#"
                import lib from "pkg.js";
                import lib2 from "pgk/package";
                import lib3 from "@name/pkg.js";
            "#,
            Some(json!(["never"])),
        ),
        (
            r#"
                import lib from "pkg";
                import lib2 from "pgk/package.js";
                import lib3 from "@name/pkg";
            "#,
            Some(json!(["always"])),
        ),
        // ============================================================================
        // TypeScript type-only imports - from eslint-plugin-import
        // Without checkTypeImports, type imports are ignored
        // ============================================================================
        (
            r#"import type T from "./typescript-declare""#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never"}]),
            ),
        ),
        (
            r#"export type { MyType } from "./typescript-declare""#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never" }]),
            ),
        ),
        // With checkTypeImports, type imports ARE checked
        (
            r#"import type { MyType } from "./typescript-declare.ts""#,
            Some(json!(["always", {"checkTypeImports": true}])),
        ),
        (
            r#"export type { MyType } from "./typescript-declare.ts""#,
            Some(json!(["always", {"checkTypeImports": true}])),
        ),
        // ============================================================================
        // Edge cases
        // ============================================================================
        (r"import''", None),
        (r"export *from 'íìc'", None),
        (
            r"import { Something } from './something.hooks'; import SomeComponent from './SomeComponent.vue';",
            Some(json!(["ignorePackages", { "js": "never", "ts": "never" }])),
        ),
        // ============================================================================
        // Modern module extensions (.mts, .cts, .mjs, .cjs)
        // ============================================================================
        (
            r#"
                import foo from "./foo.mts";
                import bar from "./bar.cts";
            "#,
            Some(json!(["always", { "mts": "always", "cts": "always" }])),
        ),
        (
            r#"
                import foo from "./foo.mjs";
                import bar from "./bar.cjs";
            "#,
            Some(json!(["always", { "mjs": "always", "cjs": "always" }])),
        ),
        (
            r#"
                import foo from "./foo";
                import bar from "./bar";
            "#,
            Some(
                json!(["never", { "mts": "never", "cts": "never", "mjs": "never", "cjs": "never" }]),
            ),
        ),
        // ============================================================================
        // Wildcard pattern support
        // ============================================================================
        (
            r#"
                import foo from "./foo.mts";
                import bar from "./bar.custom";
            "#,
            Some(json!(["always", { "pattern": { "*": "always", "js": "never" } }])),
        ),
        (
            r#"
                import foo from "./foo";
                import bar from "./bar";
            "#,
            Some(json!(["never", { "pattern": { "*": "never" } }])),
        ),
        // ============================================================================
        // pathGroupOverrides - from eslint-plugin-import (TypeScript resolver tests)
        // ============================================================================
        (
            r#"
                import foo from "@auditboard/api/something";
                import bar from "@playwright-tests/fixtures";
            "#,
            Some(json!(["always", {
                "pathGroupOverrides": [
                    { "pattern": "@auditboard/**", "action": "ignore" },
                    { "pattern": "@playwright-tests/**", "action": "ignore" }
                ]
            }])),
        ),
        (
            r#"
                import { ErrorMessage } from "@black-flag/core/util";
                import { $instances } from "rootverse+debug:src.ts";
                import { $exists } from "rootverse+bfe:src/symbols.ts";
                import type { Entries } from "type-fest";
            "#,
            Some(json!(["always", {
                "ignorePackages": true,
                "checkTypeImports": true,
                "pathGroupOverrides": [
                    { "pattern": "multiverse{*,*/**}", "action": "enforce" }
                ]
            }])),
        ),
        (
            r#"
                import { ErrorMessage } from "@black-flag/core/util";
                import { $instances } from "rootverse+debug:src.ts";
                import { $exists } from "rootverse+bfe:src/symbols.ts";
                import type { Entries } from "type-fest";
            "#,
            Some(json!(["always", {
                "ignorePackages": true,
                "checkTypeImports": true,
                "pathGroupOverrides": [
                    { "pattern": "rootverse{*,*/**}", "action": "enforce" }
                ]
            }])),
        ),
        (
            r#"
                import { ErrorMessage } from "@black-flag/core/util";
                import { $instances } from "rootverse+debug:src";
                import { $exists } from "rootverse+bfe:src/symbols";
                import type { Entries } from "type-fest";
            "#,
            Some(json!(["always", {
                "ignorePackages": true,
                "checkTypeImports": true,
                "pathGroupOverrides": [
                    { "pattern": "multiverse{*,*/**}", "action": "enforce" },
                    { "pattern": "rootverse{*,*/**}", "action": "ignore" }
                ]
            }])),
        ),
    ];

    let fail = vec![
        // ============================================================================
        // Default mode (no config) - from eslint-plugin-import
        // ============================================================================
        (r#"import a from "a/index.js""#, None),
        // ============================================================================
        // "always" mode - missing extensions - from eslint-plugin-import
        // ============================================================================
        (r#"import barjs from ".""#, Some(json!(["always"]))),
        (r#"import barjs2 from "..""#, Some(json!(["always"]))),
        (r#"import thing from "non-package/test""#, Some(json!(["always"]))),
        (
            r#"import thing from "@name/pkg/test""#,
            Some(json!(["always", {"ignorePackages": false}])),
        ),
        (r#"export { foo } from "./foo""#, Some(json!(["always"]))),
        (r#"export * from "./foo""#, Some(json!(["always"]))),
        (r#"import withoutExtension from "./foo?a=True.ext""#, Some(json!(["always"]))),
        (r#"const { foo } = require("./foo")"#, Some(json!(["always"]))),
        (
            r#"
                import foo from "@/ImNotAScopedModule";
                import chart from "@/configs/chart";
            "#,
            Some(json!(["always", { "ignorePackages": false }])),
        ),
        // ============================================================================
        // "never" mode - unexpected extensions - from eslint-plugin-import
        // ============================================================================
        (r#"import lib from "./bar.js""#, Some(json!(["never"]))),
        (r#"import thing from "./fake-file.js""#, Some(json!(["never"]))),
        (
            r#"import thing from "@name/pkg/test.js""#,
            Some(json!(["never", {"ignorePackages": false}])),
        ),
        (r#"export { foo } from "./foo.js""#, Some(json!(["never"]))),
        (r#"export * from "./foo.js""#, Some(json!(["never"]))),
        (r#"import withExtension from "./foo.js?a=True""#, Some(json!(["never"]))),
        (r#"const { foo } = require("./foo.js")"#, Some(json!(["never"]))),
        (
            r#"import foo from "@/ImNotAScopedModule.js""#,
            Some(json!(["never", { "ignorePackages": false }])),
        ),
        (
            r#"import m from "@test-scope/some-module/index.js""#,
            Some(json!(["never", {"ignorePackages": false}])),
        ),
        // ============================================================================
        // Mixed extension policies - from eslint-plugin-import
        // ============================================================================
        (
            r#"
                import a from "a/index.js";
                import packageConfig from "./package";
            "#,
            Some(json!([{ "json": "always", "js": "never"}])),
        ),
        (
            r#"
                import lib from "./bar.js";
                import component from "./bar.jsx";
                import data from "./bar.json";
            "#,
            Some(json!(["never"])),
        ),
        (
            r#"
                import lib from "./bar.js";
                import component from "./bar.jsx";
                import data from "./bar.json";
            "#,
            Some(json!([{ "json": "always", "js": "never", "jsx": "never" }])),
        ),
        (
            r#"
                import component from "./bar.jsx";
                import data from "./bar.json";
            "#,
            Some(json!([{ "json": "always", "js": "never", "jsx": "never" }])),
        ),
        (
            r#"
                import barjs from "./bar.js";
                import barjson from "./bar.json";
                import barnone from "./bar";
            "#,
            Some(json!(["always", { "json": "always", "js": "never", "jsx": "never" }])),
        ),
        (
            r#"
                import barjs from "./bar.js";
                import barjson from "./bar.json";
                import barnone from "./bar";
            "#,
            Some(json!(["never", { "json": "always", "js": "never", "jsx": "never" }])),
        ),
        // ============================================================================
        // ignorePackages mode - from eslint-plugin-import
        // Should flag local files but not packages
        // ============================================================================
        (
            r#"
                import foo from "./foo.js";
                import bar from "./bar.json";
                import Component from "./Component";
                import baz from "foo/baz";
                import baw from "@scoped/baw/import";
                import chart from "@/configs/chart";
            "#,
            Some(json!(["always", { "ignorePackages": true }])),
        ),
        (
            r#"
                import foo from "./foo.js";
                import bar from "./bar.json";
                import Component from "./Component";
                import baz from "foo/baz";
                import baw from "@scoped/baw/import";
                import chart from "@/configs/chart";
            "#,
            Some(json!(["ignorePackages"])),
        ),
        (
            r#"
                import foo from "./foo.js";
                import bar from "./bar.json";
                import Component from "./Component.jsx";
            "#,
            Some(json!(["never", { "ignorePackages": true }])),
        ),
        (r#"import * as test from ".""#, Some(json!(["ignorePackages"]))),
        (r#"import * as test from "..""#, Some(json!(["ignorePackages"]))),
        // ============================================================================
        // Wildcard pattern violations
        // ============================================================================
        (
            r#"import foo from "./foo.js""#,
            Some(json!(["always", { "pattern": { "*": "always", "js": "never" } }])),
        ),
        (
            r#"
                import foo from "./foo.js";
                import bar from "./bar.json";
                import Component from "./Component.jsx";
            "#,
            Some(json!(["always", { "pattern": { "jsx": "never" } }])),
        ),
        (
            r#"import foo from "./foo.custom""#,
            Some(json!(["never", { "pattern": { "*": "never" } }])),
        ),
        // ============================================================================
        // TypeScript type imports - from eslint-plugin-import
        // ============================================================================
        (
            r#"import T from "./typescript-declare""#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never" }]),
            ),
        ),
        (
            r#"export { MyType } from "./typescript-declare""#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never" }]),
            ),
        ),
        // With checkTypeImports, type imports ARE checked
        (
            r#"import type T from "./typescript-declare""#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never", "checkTypeImports": true }]),
            ),
        ),
        (
            r#"export type { MyType } from "./typescript-declare""#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never", "checkTypeImports": true }]),
            ),
        ),
        (
            r#"import type { MyType } from "./typescript-declare""#,
            Some(json!(["always", { "checkTypeImports": true }])),
        ),
        (
            r#"export type { MyType } from "./typescript-declare""#,
            Some(json!(["always", { "checkTypeImports": true }])),
        ),
        // ============================================================================
        // Modern module extensions - violations
        // ============================================================================
        (
            r#"import foo from "./foo""#,
            Some(
                json!(["always", { "mts": "always", "cts": "always", "mjs": "always", "cjs": "always" }]),
            ),
        ),
        (r#"import foo from "./foo.mts""#, Some(json!(["never", { "mts": "never" }]))),
        // ============================================================================
        // pathGroupOverrides - from eslint-plugin-import (TypeScript resolver tests)
        // enforce action overrides ignorePackages
        // ============================================================================
        (
            r#"
                import foo from "@auditboard/api/something.js";
                import bar from "@auditboard/auth/handler.ts";
            "#,
            Some(json!(["never", {
                "ignorePackages": true,
                "pathGroupOverrides": [
                    {
                        "pattern": "@auditboard/{api,auth,compliance}/**",
                        "action": "enforce"
                    }
                ]
            }])),
        ),
        (
            r#"
                import { ErrorMessage } from "@black-flag/core/util";
                import { $instances } from "rootverse+debug:src";
                import { $exists } from "rootverse+bfe:src/symbols";
                import type { Entries } from "type-fest";
            "#,
            Some(json!(["always", {
                "ignorePackages": true,
                "checkTypeImports": true,
                "pathGroupOverrides": [
                    { "pattern": "rootverse{*,*/**}", "action": "enforce" },
                    { "pattern": "universe{*,*/**}", "action": "ignore" }
                ]
            }])),
        ),
    ];

    Tester::new(Extensions::NAME, Extensions::PLUGIN, pass, fail).test_and_snapshot();
}
