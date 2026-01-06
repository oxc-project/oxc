use nodejs_built_in_modules::is_nodejs_builtin_module;
use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::{FxBuildHasher, FxHashMap};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use serde_json::Value;

use crate::{AstNode, context::LintContext, rule::Rule};

fn extension_should_not_be_included_in_diagnostic(
    span: Span,
    extension: &str,
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

    OxcDiagnostic::warn(format!("Missing file extension in {import_or_export} declaration."))
        .with_help(format!("Add a file extension to this {import_or_export}."))
        .with_label(span)
}

/// Extension rule configuration; Copy to avoid extra indirection.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExtensionRule {
    Always = 0,
    Never = 1,
    IgnorePackages = 2,
}

impl ExtensionRule {
    /// Parse a string into an ExtensionRule variant.
    #[inline]
    pub fn from_str(s: &str) -> Option<ExtensionRule> {
        match s {
            "always" => Some(ExtensionRule::Always),
            "never" => Some(ExtensionRule::Never),
            "ignorePackages" => Some(ExtensionRule::IgnorePackages),
            _ => None,
        }
    }
}

/// Action to take for path group overrides.
///
/// Determines how import extensions are validated for matching bespoke import specifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PathGroupAction {
    /// Enforce extension validation for matching imports (require extensions based on config).
    Enforce,
    /// Ignore matching imports entirely (skip all extension validation).
    Ignore,
}

#[derive(Debug, Clone, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
pub struct PathGroupOverride {
    /// Glob pattern to match import specifiers. This uses Rust's fast-glob library for matching.
    pattern: String,
    /// Action to take when pattern matches.
    action: PathGroupAction,
}

impl PathGroupOverride {
    /// Check if this override matches the given import path.
    ///
    /// Uses fast-glob pattern matching for flexible, performant matching.
    #[inline]
    pub fn matches(&self, import_path: &str) -> bool {
        fast_glob::glob_match(&self.pattern, import_path)
    }

    /// Get the action to take for this override.
    #[inline]
    pub fn action(&self) -> PathGroupAction {
        self.action
    }
}

/// This rule accepts three types of configuration:
///
/// 1. **Global rule** (string): `"always"`, `"never"`, or `"ignorePackages"`
///
/// ```jsonc
/// {
///   "rules": {
///     // this would require extensions for all imports, *including from packages*
///     // e.g. `import React from 'react';` would be disallowed.
///     // You should generally always set `ignorePackages` to `true` when using `always`.
///     "import/extensions": ["error", "always"]
///   }
/// }
/// ```
///
/// 2. **Per-extension rules** (object): `{ "js": "always", "jsx": "never", ... }`
///
/// ```jsonc
/// {
///   "rules": {
///     "import/extensions": [
///       "error",
///       // per-extension rules:
///       // require extensions for .js imports and disallow them for .ts imports
///       { "js": "always", "ts": "never", "ignorePackages": true }
///     ]
///   }
/// }
/// ```
///
/// 3. **Combined** (array): `["error", "always", { "js": "never" }]` or `["error", { "js": "always" }]`
///
/// ```jsonc
/// {
///   "rules": {
///     "import/extensions": [
///       "error",
///       "always", // by default, require extensions for all imports
///       {
///         "ts": "never", // override the global value and disallow extensions on imports for specific file types
///         "ignorePackages": true
///       }
///     ]
///   }
/// }
/// ```
///
/// **Default behavior (no configuration)**: All imports - of all kinds - pass.
/// Unconfigured file extensions are ignored, to avoid false positives.
#[derive(Debug, Clone, JsonSchema, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ExtensionsConfig {
    /// Whether to ignore package imports when enforcing extension rules.
    ///
    /// > [!IMPORTANT]
    /// > When setting this rule to `always`, you should also set `ignorePackages` to `true`.
    /// > Otherwise, package imports without extensions (such as `import React from 'react';`)
    /// > will be disallowed, which is not desirable and is not fixable.
    ///
    /// A boolean option (not per-extension) that exempts package imports from the "always" rule.
    ///
    /// Can be set in the config object: `["error", "always", { "ignorePackages": true }]`
    ///
    /// Legacy shorthand: `["error", "ignorePackages"]` is equivalent to `["error", "always", { "ignorePackages": true }]`
    ///
    /// - **With "always"**: When `true`, package imports (e.g., `lodash`, `@babel/core`) don't require extensions
    /// - **With "never"**: This option has no effect; extensions are still forbidden on package imports
    ///
    /// Example: `["error", "always", { "ignorePackages": true }]` allows `import foo from "lodash"` but requires `import bar from "./bar.js"`
    ignore_packages: bool,
    #[serde(skip)] // skipped because it cannot actually be set directly in the config object.
    require_extension: Option<ExtensionRule>,
    /// Whether to check type imports when enforcing extension rules.
    ///
    /// ```ts
    /// // If checkTypeImports is `false`, we don't care about
    /// // whether these imports have file extensions or not, both are always allowed:
    /// import type { Foo } from './foo';
    /// import type { Foo } from './foo.ts';
    /// ```
    check_type_imports: bool,
    /// Map from file extension (without dot) to its configured rule.
    // skipped because we build this dynamically and it is not an actual named field.
    #[serde(skip)]
    extensions: FxHashMap<String, ExtensionRule>,
    /// Path group overrides for bespoke import specifiers.
    ///
    /// Array of pattern-action pairs for custom import protocols (monorepo tools, custom resolvers).
    /// Each override has: `{ "pattern": "<glob-pattern>", "action": "enforce" | "ignore" }`
    ///
    /// **Pattern matching**: Uses glob patterns (`*`, `**`, `{a,b}`) to match import specifiers.
    /// Note that the pattern matching is done in Rust with the fast-glob library, and so may differ
    /// from the JavaScript glob library used by the original ESLint rule.
    ///
    /// **Actions**:
    /// - `"enforce"`: Apply normal extension validation (respect global/per-extension rules)
    /// - `"ignore"`: Skip all extension validation for matching imports
    ///
    /// **Precedence**: First matching pattern wins.
    ///
    /// **Examples:**
    ///
    /// ```json
    /// {
    ///   "pattern": "rootverse{*,*/**}",
    ///   "action": "ignore"
    /// }
    /// ```
    ///
    /// Matches imports from `rootverse+debug:src`, `rootverse+bfe:src/symbols` and
    /// ignores whether or not they have an extension.
    path_group_overrides: Vec<PathGroupOverride>,
}

impl ExtensionsConfig {
    /// Check if a specific extension has an explicit configuration.
    ///
    /// Returns `true` if the extension is configured, `false` if it should be ignored.
    /// This is used to implement "only check configured extensions" behavior.
    #[inline]
    pub fn has_rule(&self, ext: &str) -> bool {
        self.extensions.contains_key(ext)
    }

    /// Get the configured rule for a specific file extension.
    ///
    /// Returns the configured ExtensionRule if present, or None otherwise.
    /// This method is inlined for hot-path performance.
    #[inline]
    pub fn get_rule(&self, ext: &str) -> Option<ExtensionRule> {
        self.extensions.get(ext).copied()
    }

    /// Check if an extension is configured to always require the extension.
    #[inline]
    pub fn is_always(&self, ext: &str) -> bool {
        matches!(self.get_rule(ext), Some(ExtensionRule::Always))
    }

    /// Check if an extension is configured to never allow the extension.
    #[inline]
    pub fn is_never(&self, ext: &str) -> bool {
        matches!(self.get_rule(ext), Some(ExtensionRule::Never))
    }

    /// Check if the extension is a standard JS/TS extension.
    #[inline]
    pub fn is_standard_extension(ext: &str) -> bool {
        matches!(ext, "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "json")
    }

    /// Check if any standard extension has a "never" rule configured.
    /// Used for lenient behavior when module resolution is unavailable.
    #[inline]
    pub fn has_any_never_rules(&self) -> bool {
        // Fast path: if global rule is Never, all extensions default to Never
        matches!(self.require_extension, Some(ExtensionRule::Never))
            || self.is_never("js")
            || self.is_never("jsx")
            || self.is_never("ts")
            || self.is_never("tsx")
            || self.is_never("mjs")
            || self.is_never("cjs")
            || self.is_never("json")
    }

    /// Check path group overrides for the given import path.
    ///
    /// Returns the action to take if a pattern matches, or None if no patterns match.
    /// First matching pattern wins (precedence order).
    #[inline]
    pub fn check_path_group_overrides(&self, import_path: &str) -> Option<PathGroupAction> {
        self.path_group_overrides
            .iter()
            .find(|override_| override_.matches(import_path))
            .map(PathGroupOverride::action)
    }

    /// Determine if an extension violation should be flagged.
    ///
    /// Returns `true` if the import violates the configured extension rules.
    /// Per-extension rules override global rules (e.g., `{ "js": "never" }` overrides global "always").
    pub fn should_flag_extension(
        &self,
        ext_str: &str,
        extension_is_written: bool,
        has_resolved_extension: bool,
        require_extension: Option<ExtensionRule>,
    ) -> bool {
        match (extension_is_written, require_extension) {
            // Extension is written - check if it should be forbidden
            (true, Some(ExtensionRule::Never)) => !self.is_always(ext_str),
            (true, _) => self.is_never(ext_str),

            // Extension is missing - check if it should be required
            (false, Some(ExtensionRule::Always)) => {
                // Per-extension "never" overrides global "always"
                // Lenient: when no module resolution, check if any standard extension has "never"
                if has_resolved_extension {
                    !self.is_never(ext_str)
                } else {
                    !self.has_any_never_rules()
                }
            }
            (false, _) => self.is_always(ext_str),
        }
    }

    /// Build configuration from JSON value with optional default rule.
    ///
    /// This function dynamically parses extension configurations from JSON, supporting
    /// both individual extension fields (js, jsx, ts, tsx, json, etc.) and arbitrary
    /// custom extensions.
    pub fn from_json_value(value: &serde_json::Value, default: Option<ExtensionRule>) -> Self {
        // If default is IgnorePackages, convert to "always" with ignorePackages: true
        // This matches ESLint's behavior where "ignorePackages" string converts to this config
        let (default, default_ignore_packages) =
            if matches!(default, Some(ExtensionRule::IgnorePackages)) {
                (Some(ExtensionRule::Always), true)
            } else {
                (default, false)
            };

        let ignore_packages =
            value.get("ignorePackages").and_then(Value::as_bool).unwrap_or(default_ignore_packages);
        let check_type_imports =
            value.get("checkTypeImports").and_then(Value::as_bool).unwrap_or_default();

        // Parse pathGroupOverrides if present
        let path_group_overrides = value
            .get("pathGroupOverrides")
            .and_then(Value::as_array)
            .map(|overrides| {
                overrides
                    .iter()
                    .filter_map(|override_val| {
                        serde_json::from_value::<PathGroupOverride>(override_val.clone()).ok()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // Pre-allocate HashMap with estimated capacity for better performance
        let capacity = if let Some(obj) = value.as_object() {
            // Estimate: most fields are extension configs, reserve space
            obj.len()
        } else {
            0
        };

        let mut extensions = FxHashMap::with_capacity_and_hasher(capacity, FxBuildHasher);

        // Process known extensions from the configuration
        if let Some(obj) = value.as_object() {
            for (key, val) in obj {
                // Skip non-extension config fields
                if matches!(
                    key.as_str(),
                    "ignorePackages" | "checkTypeImports" | "pattern" | "pathGroupOverrides"
                ) {
                    continue;
                }

                // Parse extension rule from string value
                if let Some(rule_str) = val.as_str()
                    && let Some(rule) = ExtensionRule::from_str(rule_str)
                {
                    extensions.insert(key.clone(), rule);
                }
            }
        }

        Self {
            ignore_packages,
            require_extension: default,
            check_type_imports,
            extensions,
            path_group_overrides,
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
    ///
    /// **Per-extension configuration examples**:
    /// ```js
    /// // Configuration: { "vue": "always", "ts": "never" }
    /// import Component from './Component.vue'; // ✓ OK - .vue configured as "always"
    /// import utils from './utils';              // ✓ OK - .ts configured as "never"
    /// import styles from './styles.css';        // ✓ OK - .css not configured, ignored
    ///
    /// // Configuration: ["ignorePackages", { "js": "never", "ts": "never" }]
    /// import foo from './foo';                  // ✓ OK - no extension
    /// import bar from 'lodash/fp';              // ✓ OK - package import, ignored (ignorePackages sets this to true)
    /// ```
    Extensions,
    import,
    restriction,
    config = ExtensionsConfig,
);

impl Rule for Extensions {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        if let Some(first_arg) = value.get(0).and_then(Value::as_str) {
            let default = ExtensionRule::from_str(first_arg);

            if let Some(val) = value.get(1) {
                let root = val.get("pattern").unwrap_or(val);

                let config = ExtensionsConfig::from_json_value(root, default);

                Ok(Self(Box::new(config)))
            } else {
                let config = ExtensionsConfig::from_json_value(&value, default);

                Ok(Self(Box::new(config)))
            }
        } else if let Some(first_obj) = value.get(0) {
            // First element is not a string, but is present (e.g., [{ "json": "always" }])
            let config = ExtensionsConfig::from_json_value(first_obj, None);
            Ok(Self(Box::new(config)))
        } else {
            let config = ExtensionsConfig::from_json_value(&value, None);
            Ok(Self(Box::new(config)))
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Process require() calls
        let AstKind::CallExpression(call_expr) = node.kind() else { return };
        let Expression::Identifier(ident) = &call_expr.callee else { return };
        if ident.name.as_str() != "require" {
            return;
        }
        for argument in &call_expr.arguments {
            if let Argument::StringLiteral(s) = argument {
                self.process_import(
                    ctx,
                    s.value.as_str(),
                    call_expr.span,
                    false, // require() is never a type import
                    true,  // treat require as import for diagnostics
                );
            }
        }
    }

    fn run_once(&self, ctx: &LintContext) {
        // Process import/export statements
        for (module_name, modules) in &ctx.module_record().requested_modules {
            for module in modules {
                self.process_import(
                    ctx,
                    module_name.as_str(),
                    module.statement_span,
                    module.is_type,
                    module.is_import,
                );
            }
        }
    }
}

impl Extensions {
    /// Core validation logic for extension checking.
    fn validate_extension(
        &self,
        ctx: &LintContext,
        resolved_extension: Option<&str>,
        written_extension: Option<&str>,
        span: Span,
        is_import: bool,
        require_extension: Option<ExtensionRule>,
    ) {
        let config = &self.0;

        // Prefer resolved extension (actual file), fallback to written extension (import text)
        let extension_to_check = resolved_extension.or(written_extension);

        if let Some(ext_str) = extension_to_check {
            // Skip validation for unconfigured extensions (prevents false positives)
            // unless there's a global rule or it's a standard extension
            // (cheapest checks first: is_none, matches!, then hash lookup)
            if require_extension.is_none()
                && !ExtensionsConfig::is_standard_extension(ext_str)
                && !config.has_rule(ext_str)
            {
                return;
            }

            if config.should_flag_extension(
                ext_str,
                written_extension.is_some(),
                resolved_extension.is_some(),
                require_extension,
            ) {
                if let Some(ext) = written_extension {
                    ctx.diagnostic(extension_should_not_be_included_in_diagnostic(
                        span, ext, is_import,
                    ));
                } else {
                    ctx.diagnostic(extension_missing_diagnostic(span, is_import));
                }
            }
        } else if matches!(require_extension, Some(ExtensionRule::Always)) {
            // No extension found but "always" requires one
            // Lenient: skip if any standard extension has "never" rule
            if !config.has_any_never_rules() {
                ctx.diagnostic(extension_missing_diagnostic(span, is_import));
            }
        }
        // Note: IgnorePackages is converted to Always in build_config, so no branch needed
    }

    /// Unified import/require processing with all pre-validation checks.
    ///
    /// Handles both ESM imports/exports and CommonJS require() calls with consistent
    /// validation logic. This ensures require() now correctly checks pathGroupOverrides,
    /// built-in modules, and ignorePackages (which was previously missing).
    fn process_import(
        &self,
        ctx: &LintContext,
        module_name: &str,
        span: Span,
        is_type_import: bool,
        is_import: bool,
    ) {
        let config = &self.0;

        // Type imports check (only for ESM, always false for require)
        if is_type_import && !config.check_type_imports {
            return;
        }

        // Check pathGroupOverrides first (highest precedence)
        let path_group_action = config.check_path_group_overrides(module_name);
        if path_group_action == Some(PathGroupAction::Ignore) {
            return;
        }

        // Built-in Node modules are always skipped
        if is_nodejs_builtin_module(module_name) || ctx.globals().is_enabled(module_name) {
            return;
        }

        // ignorePackages only affects "always" rule
        if config.ignore_packages
            && is_package_import(module_name)
            && matches!(
                config.require_extension,
                Some(ExtensionRule::Always | ExtensionRule::IgnorePackages)
            )
        {
            return;
        }

        // Get extensions
        let resolved_extension = get_resolved_extension(ctx.module_record(), module_name);

        // For ROOT packages, don't extract extensions - dots are part of package names
        // Exception: if pathGroupOverrides explicitly enforces validation
        let written_extension = if is_root_package_import(module_name)
            && !matches!(path_group_action, Some(PathGroupAction::Enforce))
        {
            None
        } else {
            get_file_extension_from_module_name(module_name)
        };

        self.validate_extension(
            ctx,
            resolved_extension.as_deref(),
            written_extension.as_deref(),
            span,
            is_import,
            config.require_extension,
        );
    }
}

/// Determines if an import specifier is a ROOT package (package name without subpath).
///
/// Root packages are package names where dots are part of the package name itself,
/// not file extensions. Extension validation should be skipped for root packages.
///
/// Returns `true` for:
/// - Bare packages: `lodash`, `react`, `pkg.js` (no `/` means `.js` is part of package name)
/// - Scoped packages: `@babel/core`, `@types/node`, `@x/pkg.js` (only one `/` separating scope/name)
///
/// Returns `false` for:
/// - Package subpaths: `lodash/fp`, `@babel/core/lib/parser.js` (files within packages, have actual extensions)
/// - Relative imports: `./foo`, `../bar`
/// - Absolute paths: `/usr/local/lib`
/// - Path aliases: `@/`, `~/`, `#/`
fn is_root_package_import(module_name: &str) -> bool {
    // First check if it's a package at all
    if !is_package_import(module_name) {
        return false;
    }

    // For scoped packages (@scope/package), only count as root if there's exactly one '/'
    // @babel/core → root package (one '/')
    // @babel/core/lib/parser.js → subpath (more than one '/')
    if module_name.starts_with('@') {
        return module_name.matches('/').count() == 1;
    }

    // For bare packages, only count as root if there's no '/'
    // lodash → root package (no '/')
    // lodash/fp → subpath (has '/')
    !module_name.contains('/')
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
    // Use slice pattern for single bounds check
    if let [first, b'/', ..] = module_name.as_bytes()
        && *first != b'.'
        && *first != b'@'
    {
        return false; // Path alias like ~/ or #/
    }

    // Everything else is a bare package import
    // Examples: lodash, react, lodash/fp
    true
}

/// Get the file extension from the import/export string specifier.
///
/// This parses the extension from the written import statement text,
/// handling query parameters and edge cases. Extensions are normalized
/// to lowercase for case-insensitive matching.
///
/// # Examples
/// - `"./foo.js"` → `Some("js")`
/// - `"./foo.JS"` → `Some("js")` (normalized to lowercase)
/// - `"./foo.js?v=123"` → `Some("js")` (query params stripped)
/// - `"./foo"` → `None`
/// - `"./foo."` → `None` (empty extension)
/// - `"./foo.bar/"` → `None` (directory path)
fn get_file_extension_from_module_name(module_name: &str) -> Option<String> {
    use cow_utils::CowUtils;
    if let Some((_, extension)) =
        module_name.split('?').next().unwrap_or(module_name).rsplit_once('.')
        && !extension.is_empty()
        && !extension.starts_with('/')
    {
        return Some(extension.cow_to_ascii_lowercase().into_owned());
    }

    None
}

/// Get the actual file extension from the resolved module path.
///
/// This uses the module record's resolved absolute path to determine
/// the actual extension of the file on the filesystem. Returns `None`
/// if the module is not resolved (e.g., package imports, unresolved modules).
/// Extensions are normalized to lowercase for case-insensitive matching.
///
/// # Examples
/// - Resolved `./foo.ts` → `Some("ts")`
/// - Resolved `./foo.TS` → `Some("ts")` (normalized to lowercase)
/// - Package import `lodash` → `None` (not resolved locally)
/// - Path alias `@/utils/foo.js` → `Some("js")` (if resolved)
fn get_resolved_extension(
    module_record: &crate::module_record::ModuleRecord,
    module_name: &str,
) -> Option<String> {
    use cow_utils::CowUtils;
    module_record.get_loaded_module(module_name).and_then(|loaded_module| {
        loaded_module
            .resolved_absolute_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.cow_to_ascii_lowercase().into_owned())
    })
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        // Default config: no extension requirements, unconfigured extensions are ignored
        (r#"import a from "@/a""#, None),
        (r#"import a from "a""#, None),
        (r#"import dot from "./file.with.dot""#, None),
        (r#"import a from "a/index.js""#, None),
        // 'always': require extensions for all imports
        (r#"import a from "a/index.js""#, Some(json!(["always"]))),
        (r#"import dot from "./file.with.dot.js""#, Some(json!(["always"]))),
        (
            r#"
                import a from "a";
                import packageConfig from "./package.json";
            "#,
            Some(json!([{"json": "always", "js": "never"}])),
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
                import barjson from "./bar.json";
                import barhbs from "./bar.hbs";
            "#,
            Some(json!(["always", { "js": "never", "jsx": "never"}])),
        ),
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
        // Package subpaths are treated as packages (when ignorePackages is true)
        (
            r#"import thing from "non-package/test";"#,
            Some(json!(["always", { "ignorePackages": true }])),
        ),
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
            Some(json!(["always", { "js": "never", "jsx": "never", "ignorePackages": true }])),
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
            Some(json!(["always", { "ignorePackages": true }])),
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
        // Root packages: .js in package name is NOT treated as file extension
        // These are package names, not files, so "never" doesn't apply
        (
            r#"
                import lib from "pkg.js";
                import lib2 from "pgk/package";
                import lib3 from "@name/pkg.js";
            "#,
            Some(json!(["never", { "ignorePackages": true }])),
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
        // pathGroupOverrides: Enforce action with extension present
        (
            r"import { x } from 'rootverse+debug:src.ts';",
            Some(json!([
                "always",
                {
                    "pathGroupOverrides": [
                        { "pattern": "rootverse{*,*/**}", "action": "enforce" }
                    ]
                }
            ])),
        ),
        // pathGroupOverrides: Ignore action without extension
        (
            r"import { x } from 'rootverse+debug:src';",
            Some(json!([
                "always",
                {
                    "pathGroupOverrides": [
                        { "pattern": "rootverse{*,*/**}", "action": "ignore" }
                    ]
                }
            ])),
        ),
        // pathGroupOverrides: Ignore action with extension (also valid)
        (
            r"import { x } from 'rootverse+debug:src.ts';",
            Some(json!([
                "always",
                {
                    "pathGroupOverrides": [
                        { "pattern": "rootverse{*,*/**}", "action": "ignore" }
                    ]
                }
            ])),
        ),
        // pathGroupOverrides: Multiple patterns with precedence (ignore first)
        (
            r"import { x } from 'rootverse+debug:src';",
            Some(json!([
                "always",
                {
                    "pathGroupOverrides": [
                        { "pattern": "rootverse{*,*/**}", "action": "ignore" },
                        { "pattern": "rootverse*", "action": "enforce" }
                    ]
                }
            ])),
        ),
        // pathGroupOverrides: No pattern match, standard validation applies
        (
            r"import { x } from './regular-import.js';",
            Some(json!([
                "always",
                {
                    "pathGroupOverrides": [
                        { "pattern": "rootverse{*,*/**}", "action": "ignore" }
                    ]
                }
            ])),
        ),
        // pathGroupOverrides: Mixed standard and bespoke imports
        (
            r"
                import { a } from './standard.js';
                import { b } from 'rootverse+debug:custom';
            ",
            Some(json!([
                "always",
                {
                    "pathGroupOverrides": [
                        { "pattern": "rootverse*", "action": "ignore" }
                    ]
                }
            ])),
        ),
        // Edge case: Query strings with multiple ? characters
        (r"import x from './foo.js?v=1?extra=2';", Some(json!(["always"]))),
        // Edge case: Fragment identifiers
        (r"import x from './foo.js#section';", Some(json!(["always"]))),
        // Edge case: Combined query + fragment
        (r"import x from './foo.js?v=1#top';", Some(json!(["always"]))),
        // Edge case: Encoded characters in paths
        (r"import x from './foo%20bar.js';", Some(json!(["always"]))),
        // Edge case: Unicode in file names (Chinese)
        (r"import x from './文件.js';", Some(json!(["always"]))),
        // Edge case: Spaces in paths
        (r"import x from './my file.js';", Some(json!(["always"]))),
        // Edge case: Special chars (brackets)
        (r"import x from './file[1].js';", Some(json!(["always"]))),
        // Edge case: Nested path aliases
        (r"import x from '@/components/ui/Button.js';", Some(json!(["always"]))),
        // Edge case: Multiple alias types in one import block
        (
            r"
                import a from '@/utils.js';
                import b from '~/config.ts';
                import c from '#/internal.mjs';
            ",
            Some(json!(["always"])),
        ),
        // Edge case: Path aliases with query strings
        (
            r"import styles from '@/styles.css?inline';",
            Some(json!(["always", { "css": "always" }])),
        ),
        // Edge case: Path aliases with dots in filename
        (r"import x from '@/file.with.dots.js';", Some(json!(["always"]))),
        // Edge case: Scoped packages vs aliases
        (
            r"
                import babel from '@babel/core';
                import local from '@/babel/core.js';
            ",
            Some(json!(["ignorePackages"])),
        ),
        // Edge case: Single-letter scopes
        (r"import x from '@x/pkg';", Some(json!(["ignorePackages"]))),
        // Edge case: Deep nesting in scoped package
        (
            r"import x from '@org/pkg/a/b/c/d/e/f.js';",
            Some(json!(["always", { "ignorePackages": true }])),
        ),
        // Edge case: Monorepo workspace protocol
        (r"import x from 'workspace:*';", Some(json!(["ignorePackages"]))),
        // Edge case: File protocol
        (r"import x from 'file:../relative/path.js';", Some(json!(["always"]))),
        // Edge case: Link protocol
        (r"import x from 'link:../../package';", Some(json!(["ignorePackages"]))),
        // Edge case: Multiple dots in package names
        (r"import x from 'pkg.name.with.dots';", Some(json!(["ignorePackages"]))),
        // Edge case: Numbers in package names
        (
            r"
                import vue from 'vue3';
                import babel from '@babel/core-7';
            ",
            Some(json!(["ignorePackages"])),
        ),
        // Edge case: Uppercase in package names
        (
            r"import x from 'MyPackage/index.js';",
            Some(json!(["always", { "ignorePackages": true }])),
        ),
        // Edge case: export * as namespace from
        (r"export * as namespace from './module.js';", Some(json!(["always"]))),
        // Edge case: export { default as name } from
        (r"export { default as name } from './module.js';", Some(json!(["always"]))),
        // Edge case: Re-exports with type
        (
            r"export type * from './types.ts';",
            Some(json!(["always", { "checkTypeImports": true }])),
        ),
        // Edge case: Aggregate exports (multiple exports from same module)
        (
            r"
                export { foo } from './module.js';
                export { bar } from './module.js';
            ",
            Some(json!(["always"])),
        ),
        // Edge case: Case-insensitive extension matching (uppercase extension)
        (r"import x from './foo.JS';", Some(json!(["always", { "js": "always" }]))),
        // Edge case: Case-insensitive extension matching (mixed case)
        (r"import x from './foo.Ts';", Some(json!(["always", { "ts": "always" }]))),
        // 'react' import when ignorePackages is true should be allowed
        (r"import React from 'react';", Some(json!(["always", { "ignorePackages": true }]))),
        (
            r"import React from 'react';",
            Some(json!(["ignorePackages"])), // equivalent to the above
        ),
        (
            r#"import React from "react";"#, // works with double-quotes as well
            Some(json!(["ignorePackages"])),
        ),
        // This should also be allowed when set to `never`.
        (r"import React from 'react';", Some(json!(["never"]))),
        // Built-in Node modules should always be allowed without extension
        (r"import fs from 'fs';", Some(json!(["always"]))),
        (r"import crypto from 'crypto';", Some(json!(["always"]))),
        // import starting with 'node:' should be treated specially and always be allowed without extension.
        (r"import fs from 'node:fs';", Some(json!(["always"]))),
        (r"import crypto from 'node:crypto';", Some(json!(["always"]))),
        (r"import crypto from 'node:crypto';", Some(json!(["always", { "ignorePackages": true }]))),
        (r"import crypto from 'node:crypto';", Some(json!(["never"]))),
        (r"import * as crypto from 'node:crypto';", Some(json!(["always"]))),
        (r"import { default as foo } from 'node:crypto';", Some(json!(["always"]))),
        // Ensure that import attributes like "type: json" work fine
        (
            r#"import data from "./data.json" with { type: "json" };"#,
            Some(json!(["always", { "json": "always" }])),
        ),
        (r#"import data from "./data" with { type: "json" };"#, Some(json!(["never"]))),
        (r#"import data from "./data.json" with { type: "json" };"#, Some(json!(["always"]))),
        (
            r#"import data from "./data.json" with { type: "json" };"#,
            Some(json!(["ignorePackages"])),
        ),
        // Subpath imports
        // https://nodejs.org/api/packages.html#subpath-imports
        // (
        //     r##"import internalZ from "#internal/z";"##,
        //     Some(json!(["never"])),
        // ),
        // (
        //     r##"import internalZ from "#internal/z.js";"##,
        //     Some(json!(["always", { "ignorePackages": true }])),
        // ),
        // (
        //     r##"import internalZ from "#internal/z.js";"##,
        //     Some(json!(["always"])),
        // ),
    ];

    let fail = vec![
        // NOTE: The test `import dot from "./file.with.dot"` with config ["always"] is omitted
        // because without module resolution, we cannot distinguish between:
        // 1. A valid `.dot` file extension (should pass)
        // 2. A filename with `.dot` in it where `.js` is the real extension (should fail)
        // With module resolution, this would be correctly handled.
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
        // Scoped package subpaths with extensions fail, regardless of ignorePackages setting
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
        (
            r"
                import { bar } from '@scope/pkg/file.js';
            ",
            Some(json!(["never", { "ignorePackages": true }])),
        ),
        (
            r"
                import { baz } from '@org/lib/sub/index.ts';
            ",
            Some(json!(["never", { "ignorePackages": true }])),
        ),
        (
            r"
                import { baz } from '@org/lib/sub/index.ts';
            ",
            Some(json!(["never"])),
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
        // pathGroupOverrides: Enforce action missing extension
        (
            r"import { x } from 'rootverse+debug:src';",
            Some(json!([
                "always",
                {
                    "pathGroupOverrides": [
                        { "pattern": "rootverse{*,*/**}", "action": "enforce" }
                    ]
                }
            ])),
        ),
        // pathGroupOverrides: Multiple patterns violation (enforce first)
        (
            r"import { x } from 'rootverse+debug:src';",
            Some(json!([
                "always",
                {
                    "pathGroupOverrides": [
                        { "pattern": "rootverse{*,*/**}", "action": "enforce" },
                        { "pattern": "rootverse*", "action": "ignore" }
                    ]
                }
            ])),
        ),
        // pathGroupOverrides: Pattern precedence violation (enforce before ignore)
        // First pattern matches and enforces, missing extension should fail
        (
            r"import { x } from 'custom+protocol:module';",
            Some(json!([
                "always",
                {
                    "pathGroupOverrides": [
                        { "pattern": "custom*", "action": "enforce" },
                        { "pattern": "custom**", "action": "ignore" }
                    ]
                }
            ])),
        ),
        // pathGroupOverrides: Bespoke + regular violations (both fail)
        (
            r"
                import { a } from './standard';
                import { b } from 'custom+protocol:module';
            ",
            Some(json!([
                "always",
                {
                    "pathGroupOverrides": [
                        { "pattern": "custom*", "action": "enforce" }
                    ]
                }
            ])),
        ),
        // pathGroupOverrides: Complex nested pattern
        (
            r"import { x } from 'workspace:packages/core/src';",
            Some(json!([
                "always",
                {
                    "pathGroupOverrides": [
                        { "pattern": "workspace:**", "action": "enforce" }
                    ]
                }
            ])),
        ),
        // pathGroupOverrides: Case sensitivity test (pattern should match)
        (
            r"import { x } from 'MyCustom+Protocol:src';",
            Some(json!([
                "always",
                {
                    "pathGroupOverrides": [
                        { "pattern": "MyCustom*", "action": "enforce" }
                    ]
                }
            ])),
        ),
        // Edge case fail: Query string without extension before ?
        (r"import x from './foo?v=1';", Some(json!(["always"]))),
        // Edge case fail: Fragment without extension
        (r"import x from './foo#section';", Some(json!(["always"]))),
        // Edge case fail: Path alias without extension
        (r"import x from '@/components/Button';", Some(json!(["always"]))),
        // Edge case fail: Mixed - some with, some without extensions
        (
            r"
                import a from '@/utils.js';
                import b from '~/config';
            ",
            Some(json!(["always"])),
        ),
        // Edge case fail: Special protocol without extension
        (r"import x from 'workspace:packages/core';", Some(json!(["always"]))),
        // Edge case fail: Trailing slash (directory import) without extension
        (r"import x from '@/utils/';", Some(json!(["always"]))),
        // Edge case fail: Case-insensitive - uppercase extension should still fail with "never"
        (r"import x from './foo.JS';", Some(json!(["never", { "js": "never" }]))),
        // Edge case fail: Case-insensitive - mixed case extension should still fail with "never"
        (r"import x from './foo.Ts';", Some(json!(["never", { "ts": "never" }]))),
        // Fails because ignorePackages defaults to false. (not great default behavior, but matches ESLint)
        (r"import React from 'react';", Some(json!(["always"]))),
        // Not a real node: protocol import, so should fail when extension is required and ignorePackages is false.
        (r"import crypto from 'node_crypto';", Some(json!(["always"]))),
        // If an import has the same name as a built-in module, but isn't actually _from_ a built-in module,
        // it should fail (assuming ignorePackages is false).
        (r"import fs from 'some-package';", Some(json!(["always"]))),
        (r"import fs from '@fs/some-package';", Some(json!(["always"]))),
        (r"import { default as crypto } from '@fs/some-package';", Some(json!(["always"]))),
        (r"import * as fs from '@fs/some-package';", Some(json!(["always"]))),
        // When requiring no extension but ignoring packages, still fail when the import is a package subpath.
        (
            r"import useState from 'react/useState.ts';",
            Some(json!(["never", { "ignorePackages": true }])),
        ),
        (
            r"import useState from '@foo/bar/useState.ts';",
            Some(json!(["never", { "ignorePackages": true }])),
        ),
        // TODO: This should probably fail? Needs further investigation.
        // (
        //     r"import useState from '@foo/bar/useState';",
        //     Some(json!(["always", { "ignorePackages": true }])),
        // ),
        // TODO: This is not handled yet.
        // For subpath imports that start with `#`, they should not be considered
        // packages and should fail based on whether they have an extension.
        // https://nodejs.org/api/packages.html#subpath-imports
        // (
        //     r##"import internalZ from "#internal/z";"##,
        //     Some(json!(["always", { "ignorePackages": true }])),
        // ),
        // (
        //     r##"import internalZ from "#internal/z.js";"##,
        //     Some(json!(["never"])),
        // ),
    ];

    Tester::new(Extensions::NAME, Extensions::PLUGIN, pass, fail).test_and_snapshot();
}
