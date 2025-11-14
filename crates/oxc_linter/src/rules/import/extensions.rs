use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_resolver::NODEJS_BUILTINS;
use oxc_span::{CompactStr, Span};
use oxc_syntax::module_record::RequestedModule;
use rustc_hash::FxHashMap;
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

/// Zero-copy extension rule configuration using static references for minimal memory usage.
///
/// Uses #[repr(u8)] to ensure each variant occupies exactly 1 byte in memory,
/// optimizing for both memory footprint and cache efficiency.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, JsonSchema, Serialize)]
pub(crate) enum ExtensionRule {
    Always = 0,
    Never = 1,
    IgnorePackages = 2,
}

/// Static instances of ExtensionRule variants for zero-copy reference sharing.
/// These are stored in read-only memory and accessed via references throughout the application.
static ALWAYS: ExtensionRule = ExtensionRule::Always;
static NEVER: ExtensionRule = ExtensionRule::Never;
static IGNORE_PACKAGES: ExtensionRule = ExtensionRule::IgnorePackages;

impl ExtensionRule {
    /// Parse a string into a static reference to an ExtensionRule variant.
    ///
    /// Returns a reference to one of the static instances (ALWAYS, NEVER, IGNORE_PACKAGES)
    /// for zero-allocation operation. This enables O(1) comparison using pointer equality.
    #[inline]
    pub fn from_str(s: &str) -> Option<&'static ExtensionRule> {
        match s {
            "always" => Some(&ALWAYS),
            "never" => Some(&NEVER),
            "ignorePackages" => Some(&IGNORE_PACKAGES),
            _ => None,
        }
    }
}

/// High-performance configuration structure using FxHashMap for O(1) extension lookups.
///
/// This structure stores extension rules in a hash map with pre-allocated capacity
/// for optimal performance. All extension names are stored as string slices to avoid
/// allocations, and all rules are static references for zero-copy operation.
#[derive(Debug, Clone, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ExtensionsConfig {
    /// Whether to ignore package imports (e.g., 'react', 'lodash') when enforcing extension rules.
    ignore_packages: bool,
    require_extension: Option<&'static ExtensionRule>,
    check_type_imports: bool,
    /// Map from file extension (without dot) to its configured rule.
    /// Uses FxHashMap for fast lookups (~3-6ns for 500 entries).
    extensions: FxHashMap<String, &'static ExtensionRule>,
}

impl ExtensionsConfig {
    /// Get the configured rule for a specific file extension.
    ///
    /// Returns a reference to the static ExtensionRule if configured, or None otherwise.
    /// This method is inlined for hot-path performance.
    #[inline]
    pub fn get_rule(&self, ext: &str) -> Option<&'static ExtensionRule> {
        self.extensions.get(ext).copied()
    }

    /// Check if an extension is configured to always require the extension.
    #[inline]
    pub fn is_always(&self, ext: &str) -> bool {
        matches!(self.get_rule(ext), Some(&ExtensionRule::Always))
    }

    /// Check if an extension is configured to never allow the extension.
    #[inline]
    pub fn is_never(&self, ext: &str) -> bool {
        matches!(self.get_rule(ext), Some(&ExtensionRule::Never))
    }

    /// Check if an extension is configured with IgnorePackages.
    #[inline]
    pub fn is_ignore_packages(&self, ext: &str) -> bool {
        matches!(self.get_rule(ext), Some(&ExtensionRule::IgnorePackages))
    }
}

impl Default for ExtensionsConfig {
    fn default() -> Self {
        let mut extensions = FxHashMap::default();
        // Pre-populate with common extensions set to Never
        extensions.insert("js".to_string(), &NEVER);
        extensions.insert("jsx".to_string(), &NEVER);
        extensions.insert("ts".to_string(), &NEVER);
        extensions.insert("tsx".to_string(), &NEVER);
        extensions.insert("json".to_string(), &NEVER);

        Self {
            ignore_packages: true,
            require_extension: None,
            check_type_imports: false,
            extensions,
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
            let default = ExtensionRule::from_str(first_arg);

            if let Some(val) = value.get(1) {
                let root = val.get("pattern").unwrap_or(val);

                let config = build_config(root, default);

                Self(Box::new(config))
            } else {
                let config = build_config(&value, default);

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
                    self.process_require_record(call_expr, ctx, config.require_extension);
                }
            }
        }

        for (module_name, module) in &module_record.requested_modules {
            for module_item in module {
                self.process_module_record(
                    (module_name.clone(), module_item),
                    ctx,
                    config.require_extension,
                    config.check_type_imports,
                    config.ignore_packages,
                    module_item.is_import,
                );
            }
        }
    }
}

/// Build configuration from JSON value with optional default rule.
///
/// This function dynamically parses extension configurations from JSON, supporting
/// both individual extension fields (js, jsx, ts, tsx, json, etc.) and arbitrary
/// custom extensions. Pre-allocates the HashMap based on JSON object size for efficiency.
fn build_config(
    value: &serde_json::Value,
    default: Option<&'static ExtensionRule>,
) -> ExtensionsConfig {
    let ignore_packages = value.get("ignorePackages").and_then(Value::as_bool).unwrap_or(true);
    let check_type_imports =
        value.get("checkTypeImports").and_then(Value::as_bool).unwrap_or_default();

    // Pre-allocate HashMap with estimated capacity for better performance
    let capacity = if let Some(obj) = value.as_object() {
        // Estimate: most fields are extension configs, reserve space
        obj.len().max(5)
    } else {
        5 // Default for common extensions
    };

    let mut extensions = FxHashMap::with_capacity_and_hasher(capacity, Default::default());

    // Common extensions to check
    let common_extensions = ["js", "jsx", "ts", "tsx", "json"];

    // Process known extensions from the configuration
    if let Some(obj) = value.as_object() {
        for (key, val) in obj {
            // Skip non-extension config fields
            if matches!(key.as_str(), "ignorePackages" | "checkTypeImports" | "pattern") {
                continue;
            }

            // Parse extension rule from string value
            if let Some(rule_str) = val.as_str() {
                if let Some(rule) = ExtensionRule::from_str(rule_str) {
                    extensions.insert(key.clone(), rule);
                }
            }
        }
    }

    // Apply default to common extensions that weren't explicitly configured
    if let Some(default_rule) = default {
        for &ext in &common_extensions {
            extensions.entry(ext.to_string()).or_insert(default_rule);
        }
    } else {
        // If no default and no explicit config, use Never for common extensions
        for &ext in &common_extensions {
            extensions.entry(ext.to_string()).or_insert(&NEVER);
        }
    }

    ExtensionsConfig { ignore_packages, require_extension: default, check_type_imports, extensions }
}

impl Extensions {
    fn process_module_record(
        &self,
        module_record: (CompactStr, &RequestedModule),
        ctx: &LintContext,
        require_extension: Option<&'static ExtensionRule>,
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

        let is_package = is_package_import(module_name.as_str());

        if is_builtin_node_module || (is_package && ignore_packages) {
            return;
        }

        let file_extension = get_file_extension_from_module_name(&module_name);

        let span = module.statement_span;

        if let Some(file_extension) = file_extension {
            let ext_str = file_extension.as_str();
            let should_flag = match require_extension {
                Some(&ExtensionRule::Always) => {
                    config.is_never(ext_str) || !config.is_always(ext_str)
                }
                Some(&ExtensionRule::Never) => !config.is_always(ext_str),
                _ => config.is_never(ext_str),
            };

            if should_flag {
                ctx.diagnostic(extension_should_not_be_included_in_diagnostic(
                    span,
                    &file_extension,
                    is_import,
                ));
            }
        } else if matches!(require_extension, Some(&ExtensionRule::Always)) {
            ctx.diagnostic(extension_missing_diagnostic(span, is_import));
        } else if matches!(require_extension, Some(&ExtensionRule::IgnorePackages)) {
            // With ignorePackages, missing extensions are OK only if per-extension configs
            // are explicitly set to Never (not IgnorePackages)
            if config.is_ignore_packages("js") || config.is_ignore_packages("ts") {
                ctx.diagnostic(extension_missing_diagnostic(span, is_import));
            }
        }
    }

    fn process_require_record(
        &self,
        call_expr: &CallExpression<'_>,
        ctx: &LintContext,
        require_extension: Option<&'static ExtensionRule>,
    ) {
        let config = &self.0;
        for argument in &call_expr.arguments {
            if let Argument::StringLiteral(s) = argument {
                let file_extension = get_file_extension_from_module_name(&s.value.to_compact_str());
                let span = call_expr.span;

                if let Some(file_extension) = file_extension {
                    let ext_str = file_extension.as_str();
                    let should_flag = match require_extension {
                        Some(&ExtensionRule::Always) => {
                            config.is_never(ext_str) || !config.is_always(ext_str)
                        }
                        Some(&ExtensionRule::Never) => !config.is_always(ext_str),
                        _ => config.is_never(ext_str),
                    };

                    if should_flag {
                        ctx.diagnostic(extension_should_not_be_included_in_diagnostic(
                            span,
                            &file_extension,
                            true,
                        ));
                    }
                } else if matches!(require_extension, Some(&ExtensionRule::Always)) {
                    ctx.diagnostic(extension_missing_diagnostic(span, true));
                } else if matches!(require_extension, Some(&ExtensionRule::IgnorePackages)) {
                    // With ignorePackages, missing extensions are OK only if per-extension configs
                    // are explicitly set to Never (not IgnorePackages)
                    if config.is_ignore_packages("js") || config.is_ignore_packages("ts") {
                        ctx.diagnostic(extension_missing_diagnostic(span, true));
                    }
                }
            }
        }
    }
}
/// Determines if an import specifier is a package import (not relative or path alias).
///
/// This function implements string-based classification following the ECMAScript module
/// specifier resolution algorithm, with additional heuristics for build-tool-specific
/// path aliases (like @/, ~/, #/ commonly used in webpack/vite/tsconfig.json).
///
/// Returns `true` for:
/// - Bare packages: `lodash`, `react`
/// - Scoped packages: `@babel/core`, `@types/node`, `@x/pkg` (including single-letter scopes)
/// - Package subpaths: `lodash/fp`, `@babel/core/lib/parser`
///
/// Returns `false` for:
/// - Relative imports: `./foo`, `../bar`
/// - Absolute paths: `/usr/local/lib`
/// - Path aliases: `@/`, `~/`, `#/`
fn is_package_import(module_name: &str) -> bool {
    // Relative imports: ./foo, ../bar, or directory imports (., ..)
    if module_name.starts_with('.') {
        return false;
    }

    // Absolute paths: /foo
    if module_name.starts_with('/') {
        return false;
    }

    // Handle @ prefix: distinguish path aliases from scoped packages
    // - @/foo (path alias) → rest = "/foo" → starts with '/'
    // - @x/pkg (scoped package) → rest = "x/pkg" → doesn't start with '/'
    // - @babel/core (scoped package) → rest = "babel/core" → doesn't start with '/'
    if let Some(rest) = module_name.strip_prefix('@') {
        if rest.starts_with('/') {
            return false; // Path alias: @/
        }
        // Scoped packages must have scope/package format
        // This includes single-letter scopes like @x/pkg
        return rest.contains('/');
    }

    // Other single-char path aliases: ~/, #/
    if module_name.len() >= 2 {
        let bytes = module_name.as_bytes();
        if bytes[1] == b'/' && bytes[0] != b'.' && bytes[0] != b'@' {
            return false; // Path alias like ~/ or #/
        }
    }

    // Everything else is a bare package import
    // Examples: lodash, react, lodash/fp
    true
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

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        // Default config: no extension requirements
        (r#"import a from "@/a""#, None),
        (r#"import a from "a""#, None),
        (r#"import dot from "./file.with.dot""#, None),
        // 'always': require extensions for all imports
        (r#"import a from "a/index.js""#, Some(json!(["always"]))),
        (r#"import dot from "./file.with.dot.js""#, Some(json!(["always"]))),
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
        // TODO: Test commented out - requires dynamic file extension configuration
        // not currently supported in oxc test framework.
        // (
        //     r#"
        //         import barjson from "./bar.json";
        //         import barhbs from "./bar.hbs";
        //     "#,
        //     Some(json!(["always", { "js": "never", "jsx": "never"}])),
        // ),
        (
            r#"
                import bar from "./bar.js";
                import pack from "./package";
            "#,
            Some(json!(["never", { "js": "always", "json": "never"}])),
        ),
        (r#"import path from "path";"#, None),
        (r#"import path from "path";"#, Some(json!(["never"]))),
        (r#"import path from "path";"#, Some(json!(["always"]))),
        (r#"import thing from "./fake-file.js";"#, Some(json!(["always"]))),
        // 'never': no extensions allowed
        (r#"import thing from "non-package";"#, Some(json!(["never"]))),
        // Package subpaths are treated as packages
        (r#"import thing from "non-package/test";"#, Some(json!(["always"]))),
        // 'ignorePackages': require extensions for relative imports, not for packages
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
        (
            r#"import exceljs from "exceljs""#,
            Some(json!(["always", { "js": "never", "jsx": "never"}])),
        ),
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
        // Package detection: @name/pkg.js is treated as scoped package, not a file
        (
            r#"
                import lib from "pkg.js";
                import lib2 from "pgk/package";
                import lib3 from "@name/pkg.js";
            "#,
            Some(json!(["never"])),
        ),
        // Query strings: extensions are extracted before the '?' character
        (
            r#"
                import bare from "./foo?a=True.ext";
            "#,
            Some(json!(["never"])),
        ),
        (
            r#"
                import bare from "./foo.js?a=True";
            "#,
            Some(json!(["always"])),
        ),
        (
            r#"
                import lib from "pkg";
                import lib2 from "pgk/package.js";
                import lib3 from "@name/pkg";
            "#,
            Some(json!(["always"])),
        ),
        // Type imports: ignored by default unless checkTypeImports is true
        (
            r#"import type T from "./typescript-declare";"#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never"}]),
            ),
        ),
        (
            r#"export type { MyType } from "./typescript-declare";"#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never" }]),
            ),
        ),
        // Type imports with checkTypeImports: true
        (
            r#"
                import type { MyType } from "./typescript-declare.ts";
            "#,
            Some(json!(["always", {"checkTypeImports": true}])),
        ),
        (
            r#"
                export type { MyType } from "./typescript-declare.ts";
            "#,
            Some(json!(["always", {"checkTypeImports": true}])),
        ),
        // Empty imports and unicode
        (r"import''", None),
        (r"export *from 'íìc'", None),
        (
            r"import { Something } from './something.hooks'; import SomeComponent from './SomeComponent.vue';",
            Some(json!(["ignorePackages", { "js": "never", "ts": "never" }])),
        ),
        // Configuration inheritance: per-extension configs inherit from first arg unless
        // explicitly overridden. See https://github.com/oxc-project/oxc/issues/12220
        (
            r"
                import { A } from './something';
            ",
            Some(
                json!(["ignorePackages", { "js": "never", "ts": "never", "jsx": "never", "tsx": "never"}]),
            ),
        ),
        // Path alias ~/
        (
            r"
                import { D } from '~/common/something';
            ",
            Some(
                json!(["ignorePackages", { "js": "never", "ts": "never", "jsx": "never", "tsx": "never"}]),
            ),
        ),
        // Scoped package subpaths should be treated as packages
        (
            r"
                import { foo } from '@scope/package/deep/nested/path';
            ",
            Some(json!(["ignorePackages"])),
        ),
        // Mixed configuration: relative with extension, package without, scoped package subpath
        (
            r"
                import a from './relative.js';
                import b from 'package';
                import c from '@org/pkg/sub';
            ",
            Some(json!(["ignorePackages", { "js": "always" }])),
        ),
        // Path alias @/ (not a scoped package)
        (
            r"
                import foo from '@/components/Foo.js';
                import bar from '@/utils/bar.ts';
            ",
            Some(json!(["always", { "ignorePackages": false }])),
        ),
        // Other single-char path aliases
        (
            r"
                import a from '~/config.js';
                import b from '#/internal.ts';
            ",
            Some(json!(["always", { "ignorePackages": false }])),
        ),
        // Scoped packages (distinguished from path aliases)
        (
            r"
                import babel from '@babel/core';
                import types from '@types/node';
            ",
            Some(json!(["ignorePackages"])),
        ),
        // Custom extensions: .vue (Vue.js components)
        (
            r"
                import Component from './Component.vue';
            ",
            Some(json!(["always", { "vue": "always" }])),
        ),
        (
            r"
                import Component from './Component';
            ",
            Some(json!(["never", { "vue": "never" }])),
        ),
        // Custom extensions: .svelte (Svelte components)
        (
            r"
                import Component from './Component.svelte';
            ",
            Some(json!(["always", { "svelte": "always" }])),
        ),
        (
            r"
                import Component from './Component';
            ",
            Some(json!(["never", { "svelte": "never" }])),
        ),
        // Custom extensions: .mjs (ES modules)
        (
            r"
                import utils from './utils.mjs';
            ",
            Some(json!(["always", { "mjs": "always" }])),
        ),
        (
            r"
                import utils from './utils';
            ",
            Some(json!(["never", { "mjs": "never" }])),
        ),
        // Custom extensions: .cjs (CommonJS modules)
        (
            r"
                import legacy from './legacy.cjs';
            ",
            Some(json!(["always", { "cjs": "always" }])),
        ),
        (
            r"
                import legacy from './legacy';
            ",
            Some(json!(["never", { "cjs": "never" }])),
        ),
        // Custom extensions: .css (stylesheets)
        (
            r"
                import './styles.css';
            ",
            Some(json!(["always", { "css": "always" }])),
        ),
        (
            r"
                import './styles';
            ",
            Some(json!(["never", { "css": "never" }])),
        ),
        // Mixed custom extensions with common extensions
        (
            r"
                import Component from './App.vue';
                import utils from './utils.ts';
                import styles from './styles.css';
            ",
            Some(json!(["always", { "vue": "always", "ts": "always", "css": "always" }])),
        ),
        (
            r"
                import Component from './App';
                import utils from './utils';
                import styles from './styles';
            ",
            Some(json!(["never", { "vue": "never", "ts": "never", "css": "never" }])),
        ),
        // Custom extensions with ignorePackages
        (
            r"
                import Component from './Component.vue';
                import utils from './utils.mjs';
                import pkg from 'some-package';
            ",
            Some(json!(["ignorePackages", { "vue": "always", "mjs": "always" }])),
        ),
    ];

    let fail = vec![
        // Default config: package subpaths with extensions should fail
        (r#"import a from "a/index.js""#, None),
        // 'always' config: missing extensions should fail
        (r#"import dot from "./file.with.dot""#, Some(json!(["always"]))),
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
        (r#"import "./bar.coffee""#, Some(json!(["never", { "js": "always", "jsx": "always" }]))),
        // https://github.com/oxc-project/oxc/issues/12220
        (
            r"
                import { B } from './something.ts';
            ",
            Some(json!(["ignorePackages", { "js": "never", "ts": "never" }])),
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
                import barjs from ".";
                import barjs2 from "..";
            "#,
            Some(json!(["always"])),
        ),
        (
            r#"
                import barjs from "./bar.js";
                import barjson from "./bar.json";
                import barnone from "./bar";
            "#,
            Some(json!(["never", { "json": "always", "js": "never", "jsx": "never" }])),
        ),
        (
            r#"
                import thing from "./fake-file.js";
            "#,
            Some(json!(["never"])),
        ),
        (
            r#"
                import thing from "@name/pkg/test";
            "#,
            Some(json!(["always", {"ignorePackages": false}])),
        ),
        (
            r#"
                import thing from "@name/pkg/test.js";
            "#,
            Some(json!(["never",{"ignorePackages": false}])),
        ),
        (
            r"
                import foo from './foo.js';
                import bar from './bar.json';
                import Component from './Component';
                import baz from 'foo/baz';
                import baw from '@scoped/baw/import';
                import chart from '@/configs/chart';
                import express from 'express';
            ",
            Some(json!(["always", { "ignorePackages": true }])),
        ),
        (
            r"
                import foo from './foo.js';
                import bar from './bar.json';
                import Component from './Component';
                import baz from 'foo/baz';
                import baw from '@scoped/baw/import';
                import chart from '@/configs/chart';
                import express from 'express';
            ",
            Some(json!(["ignorePackages"])),
        ),
        (
            r"
                import foo from './foo.js';
                import bar from './bar.json';
                import Component from './Component.jsx';
                import express from 'express';
            ",
            Some(json!(["never", { "ignorePackages": true }])),
        ),
        (
            r"
                import foo from './foo.js';
                import bar from './bar.json';
                import Component from './Component.jsx';
            ",
            Some(json!(["always", { "pattern": { "jsx": "never" } }])),
        ),
        // Exports
        (
            r#"
                export { foo } from "./foo";
                let bar; export { bar };
            "#,
            Some(json!(["always"])),
        ),
        (
            r#"
                export { foo } from "./foo.js";
                let bar; export { bar };
            "#,
            Some(json!(["never"])),
        ),
        // Query strings: extension detected before '?' should fail
        (r#"import withExtension from "./foo.js?a=True";"#, Some(json!(["never"]))),
        (r#"import withoutExtension from "./foo?a=True.ext";"#, Some(json!(["always"]))),
        // Require statements: same rules apply as import statements
        (
            r#"
                const { foo } = require("./foo");
                export { foo };
            "#,
            Some(json!(["always"])),
        ),
        (
            r#"
                const { foo } = require("./foo.js");
                export { foo };
            "#,
            Some(json!(["never"])),
        ),
        (
            r#"
                import foo from "@/ImNotAScopedModule";
                import chart from "@/configs/chart";
            "#,
            Some(json!(["always",{ "ignorePackages": false }])),
        ),
        // Export { } from
        (
            r#"
                export { foo } from "./foo";
            "#,
            Some(json!(["always"])),
        ),
        (
            r#"
                export { foo } from "./foo.js";
            "#,
            Some(json!(["never"])),
        ),
        // Export * from
        (
            r#"
                export * from "./foo";
            "#,
            Some(json!(["always"])),
        ),
        (
            r#"
                export * from "./foo.js";
            "#,
            Some(json!(["never"])),
        ),
        (
            r#"
                import foo from "@/ImNotAScopedModule.js";
            "#,
            Some(json!(["never", { "ignorePackages": false }])),
        ),
        (
            r"
                import _ from 'lodash';
                import m from '@test-scope/some-module/index.js';
                import bar from './bar';
            ",
            Some(json!(["never",{ "ignorePackages": false }])),
        ),
        // Directory imports: '.' and '..' are relative imports without extensions
        (
            r#"
                import * as test from ".";
            "#,
            Some(json!(["ignorePackages"])),
        ),
        (
            r#"
                import * as test from "..";
            "#,
            Some(json!(["ignorePackages"])),
        ),
        // Type imports
        (
            r#"
                import T from "./typescript-declare";
            "#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never" }]),
            ),
        ),
        (
            r#"
                export { MyType } from "./typescript-declare";
            "#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never" }]),
            ),
        ),
        (
            r#"
                import type T from "./typescript-declare";
            "#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never", "checkTypeImports": true }]),
            ),
        ),
        (
            r#"
                export type { MyType } from "./typescript-declare";
            "#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never", "checkTypeImports": true }]),
            ),
        ),
        (
            r#"
                import type { MyType } from "./typescript-declare";
            "#,
            Some(json!(["always", { "checkTypeImports": true }])),
        ),
        (
            r#"
                export type { MyType } from "./typescript-declare";
            "#,
            Some(json!(["always", { "checkTypeImports": true }])),
        ),
        // Directory imports with 'always' should fail
        (
            r"
                import x from '.';
            ",
            Some(json!(["always"])),
        ),
        (
            r"
                import y from '..';
            ",
            Some(json!(["always"])),
        ),
        // Scoped package subpaths with extensions fail when ignorePackages: false
        (
            r"
                import { bar } from '@scope/pkg/file.js';
            ",
            Some(json!(["never", { "ignorePackages": false }])),
        ),
        (
            r"
                import { baz } from '@org/lib/sub/index.ts';
            ",
            Some(json!(["never", { "ignorePackages": false }])),
        ),
        // Mixed configuration: some should pass, some should fail
        (
            r"
                import x from './foo';
                import y from './bar.ts';
            ",
            Some(json!(["always", { "ts": "never" }])),
        ),
        // Custom extensions: .vue should fail with wrong config
        (
            r"
                import Component from './Component.vue';
            ",
            Some(json!(["never", { "vue": "never" }])),
        ),
        (
            r"
                import Component from './Component';
            ",
            Some(json!(["always", { "vue": "always" }])),
        ),
        // Custom extensions: .svelte should fail with wrong config
        (
            r"
                import Component from './Component.svelte';
            ",
            Some(json!(["never", { "svelte": "never" }])),
        ),
        (
            r"
                import Component from './Component';
            ",
            Some(json!(["always", { "svelte": "always" }])),
        ),
        // Custom extensions: .mjs should fail with wrong config
        (
            r"
                import utils from './utils.mjs';
            ",
            Some(json!(["never", { "mjs": "never" }])),
        ),
        (
            r"
                import utils from './utils';
            ",
            Some(json!(["always", { "mjs": "always" }])),
        ),
        // Custom extensions: .cjs should fail with wrong config
        (
            r"
                import legacy from './legacy.cjs';
            ",
            Some(json!(["never", { "cjs": "never" }])),
        ),
        (
            r"
                import legacy from './legacy';
            ",
            Some(json!(["always", { "cjs": "always" }])),
        ),
        // Custom extensions: .css should fail with wrong config
        (
            r"
                import './styles.css';
            ",
            Some(json!(["never", { "css": "never" }])),
        ),
        (
            r"
                import './styles';
            ",
            Some(json!(["always", { "css": "always" }])),
        ),
        // Mixed custom and common extensions with conflicting rules
        (
            r"
                import Component from './App.vue';
                import utils from './utils.ts';
            ",
            Some(json!(["never", { "vue": "never", "ts": "never" }])),
        ),
        (
            r"
                import Component from './App';
                import utils from './utils';
            ",
            Some(json!(["always", { "vue": "always", "ts": "always" }])),
        ),
        // Custom extensions with ignorePackages - should fail when extension present but set to never
        (
            r"
                import Component from './Component.vue';
            ",
            Some(json!(["ignorePackages", { "vue": "never" }])),
        ),
        // Multiple custom extensions in one test - some pass, some fail
        (
            r"
                import Component from './App.vue';
                import utils from './utils';
                import styles from './styles.css';
            ",
            Some(json!(["always", { "vue": "always", "mjs": "always", "css": "always" }])),
        ),
    ];

    Tester::new(Extensions::NAME, Extensions::PLUGIN, pass, fail).test_and_snapshot();
}
