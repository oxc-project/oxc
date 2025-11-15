use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_resolver::NODEJS_BUILTINS;
use oxc_span::{CompactStr, Span};
use oxc_syntax::module_record::RequestedModule;
use rustc_hash::{FxBuildHasher, FxHashMap};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

/// Action to take for path group overrides.
///
/// Determines how import extensions are validated for matching bespoke import specifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PathGroupAction {
    /// Enforce extension validation for matching imports (require extensions based on config)
    Enforce,
    /// Ignore matching imports entirely (skip all extension validation)
    Ignore,
}

/// Path group override configuration for bespoke import specifiers.
///
/// Allows fine-grained control over extension validation for custom import protocols
/// (e.g., monorepo tools, custom resolvers, framework-specific imports).
///
/// # Pattern Matching
///
/// Uses fast-glob patterns to match import specifiers:
/// - `*`: Match any characters except `/`
/// - `**`: Match any characters including `/` (recursive)
/// - `{a,b}`: Match alternatives
///
/// # Examples
///
/// ```json
/// {
///   "pattern": "rootverse{*,*/**}",
///   "action": "enforce"
/// }
/// ```
///
/// Matches: `rootverse+debug:src`, `rootverse+bfe:src/symbols`
#[derive(Debug, Clone, Deserialize, JsonSchema, Serialize)]
pub struct PathGroupOverride {
    /// Glob pattern to match import specifiers
    pattern: String,
    /// Action to take when pattern matches
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

/// High-performance configuration structure using FxHashMap for O(1) extension lookups.
///
/// This structure stores extension rules in a hash map with pre-allocated capacity
/// for optimal performance. All extension names are stored as string slices to avoid
/// allocations, and all rules are static references for zero-copy operation.
#[derive(Debug, Clone, JsonSchema, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ExtensionsConfig {
    /// Whether to ignore package imports (e.g., 'react', 'lodash') when enforcing extension rules.
    ignore_packages: bool,
    require_extension: Option<&'static ExtensionRule>,
    check_type_imports: bool,
    /// Map from file extension (without dot) to its configured rule.
    /// Uses FxHashMap for fast lookups (~3-6ns for 500 entries).
    extensions: FxHashMap<String, &'static ExtensionRule>,
    /// Path group overrides for bespoke import specifiers.
    /// First matching pattern wins (precedence order).
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
    /// ### Configuration
    ///
    /// This rule accepts three types of configuration:
    ///
    /// 1. **Global rule** (string): `"always"`, `"never"`, or `"ignorePackages"`
    /// 2. **Per-extension rules** (object): `{ "js": "always", "jsx": "never", ... }`
    /// 3. **Combined** (array): `["always", { "js": "never" }]` or `[{ "js": "always" }]`
    ///
    /// **Default behavior (no configuration)**: All imports pass. Unconfigured file extensions are ignored to avoid false positives.
    ///
    /// **ignorePackages option**:
    /// - A boolean option (not per-extension) that exempts package imports from the "always" rule
    /// - Can be set in the config object: `["always", { "ignorePackages": true }]`
    /// - Legacy shorthand: `["ignorePackages"]` is equivalent to `["always", { "ignorePackages": true }]`
    /// - **Default: `false`** (matches ESLint behavior)
    /// - **With "always"**: When `true`, package imports (e.g., `lodash`, `@babel/core`) don't require extensions
    /// - **With "never"**: This option has no effect; extensions are still forbidden on package imports
    /// - Example: `["always", { "ignorePackages": true }]` allows `import foo from "lodash"` but requires `import bar from "./bar.js"`
    ///
    /// **pathGroupOverrides option** (bespoke import specifiers):
    /// - Array of pattern-action pairs for custom import protocols (monorepo tools, custom resolvers)
    /// - Each override has: `{ "pattern": "<glob-pattern>", "action": "enforce" | "ignore" }`
    /// - **Pattern matching**: Uses glob patterns (`*`, `**`, `{a,b}`) to match import specifiers
    /// - **Actions**:
    ///   - `"enforce"`: Apply normal extension validation (respect global/per-extension rules)
    ///   - `"ignore"`: Skip all extension validation for matching imports
    /// - **Precedence**: First matching pattern wins
    /// - Example: `["always", { "pathGroupOverrides": [{ "pattern": "rootverse{*,*/**}", "action": "ignore" }] }]`
    ///   - Allows `import { x } from 'rootverse+debug:src'` without extension
    ///   - Still requires extensions for standard imports
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
        } else if let Some(first_obj) = value.get(0) {
            // First element is not a string, but is present (e.g., [{ "json": "always" }])
            let config = build_config(first_obj, None);
            Self(Box::new(config))
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
    // Legacy behavior: if default is IgnorePackages, convert to "always" with ignorePackages: true
    // This matches ESLint's behavior where "ignorePackages" string converts to this config
    let (default, default_ignore_packages) =
        if matches!(default, Some(&ExtensionRule::IgnorePackages)) {
            (Some(&ALWAYS), true)
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

    ExtensionsConfig {
        ignore_packages,
        require_extension: default,
        check_type_imports,
        extensions,
        path_group_overrides,
    }
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

        // Check pathGroupOverrides first (highest precedence)
        // If a pattern matches, apply the action: ignore or enforce
        let path_group_action = config.check_path_group_overrides(module_name.as_str());

        if path_group_action == Some(PathGroupAction::Ignore) {
            // Skip all validation for this import
            return;
        }

        let is_builtin_node_module = NODEJS_BUILTINS.binary_search(&module_name.as_str()).is_ok()
            || ctx.globals().is_enabled(module_name.as_str());

        let is_package = is_package_import(module_name.as_str());

        // Built-in Node modules are always skipped
        if is_builtin_node_module {
            return;
        }

        // ignorePackages only affects "always" rule (exempts packages from requiring extensions)
        // It does NOT affect "never" rule (extensions are still forbidden on packages)
        // Skip validation only when: it's a package, ignorePackages is true, AND we're enforcing "always"
        if is_package
            && ignore_packages
            && matches!(
                require_extension,
                Some(&ExtensionRule::Always | &ExtensionRule::IgnorePackages)
            )
        {
            return;
        }

        // Try to get the actual file extension from the resolved module path
        let resolved_extension = get_resolved_extension(ctx.module_record(), module_name.as_str());

        // Get what's written in the import statement
        // For ROOT packages, don't extract extensions - dots are part of package names
        // e.g., "pkg.js" or "@babel/core.js" is a package name, not a file with ".js" extension
        // But for package SUBPATHS like "@babel/core/lib/parser.js", DO extract the extension
        // EXCEPTION: For bespoke imports with "enforce" action, ALWAYS extract the extension
        let is_root_package = is_root_package_import(module_name.as_str());
        let is_enforced_bespoke = matches!(path_group_action, Some(PathGroupAction::Enforce));
        let written_extension = if is_root_package && !is_enforced_bespoke {
            None
        } else {
            get_file_extension_from_module_name(&module_name)
        };

        let span = module.statement_span;

        // Determine which extension to check against the configuration
        // Prefer resolved extension (actual file), fallback to written extension (import text)
        let extension_to_check = resolved_extension.as_deref().or(written_extension.as_deref());

        if let Some(ext_str) = extension_to_check {
            // Standard JS/TS extensions that are implicitly recognized
            let is_standard_extension =
                matches!(ext_str, "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "json");

            // Skip validation if this extension is not explicitly configured,
            // UNLESS there's a global require_extension rule (always/never)
            // This prevents false positives for unconfigured extensions
            if !config.has_rule(ext_str) && require_extension.is_none() && !is_standard_extension {
                return;
            }

            // Check if the extension is present in the import statement
            let extension_is_written = written_extension.is_some();

            // Check if we have a resolved extension (from module resolution) or just written extension
            let has_resolved_extension = resolved_extension.is_some();

            let should_flag = match require_extension {
                Some(&ExtensionRule::Always) => {
                    // Extension should always be present
                    if !extension_is_written {
                        // Missing extension - check if THIS specific extension is configured as "never"
                        // If so, the per-extension "never" rule overrides the global "always"
                        // Also allow if we have a resolved extension (from module resolution)
                        if has_resolved_extension {
                            !config.is_never(ext_str)
                        } else {
                            // No module resolution - be lenient if ANY standard extensions have "never" rules
                            // This handles the case where the user configured "always" with specific "never" overrides
                            let has_never_rules = config.is_never("js")
                                || config.is_never("jsx")
                                || config.is_never("ts")
                                || config.is_never("tsx")
                                || config.is_never("mjs")
                                || config.is_never("cjs");
                            !has_never_rules
                        }
                    } else if config.is_never(ext_str) {
                        // Extension is explicitly configured as "never"
                        true
                    } else {
                        // Extension is present and not explicitly "never" - allow it
                        false
                    }
                }
                Some(&ExtensionRule::Never) => {
                    // Extension should never be present
                    extension_is_written && !config.is_always(ext_str)
                }
                _ => {
                    // Default behavior: flag if extension violates per-extension rules
                    if extension_is_written {
                        // Extension is present, check if it should not be
                        config.is_never(ext_str)
                    } else {
                        // Extension is missing, check if it should be present
                        config.is_always(ext_str)
                    }
                }
            };

            if should_flag {
                if extension_is_written {
                    ctx.diagnostic(extension_should_not_be_included_in_diagnostic(
                        span,
                        written_extension.as_ref().unwrap(),
                        is_import,
                    ));
                } else {
                    ctx.diagnostic(extension_missing_diagnostic(span, is_import));
                }
            }
        } else if matches!(require_extension, Some(&ExtensionRule::Always)) {
            // No extension found (neither resolved nor written), but always is required
            // However, if standard extensions have "never" rules, be lenient (without module resolution, we can't know the actual extension)
            let has_never_rules = config.is_never("js")
                || config.is_never("jsx")
                || config.is_never("ts")
                || config.is_never("tsx")
                || config.is_never("mjs")
                || config.is_never("cjs");

            if !has_never_rules {
                ctx.diagnostic(extension_missing_diagnostic(span, is_import));
            }
        } else if matches!(require_extension, Some(&ExtensionRule::IgnorePackages)) {
            // With ignorePackages, extensions are required for relative imports
            // UNLESS all standard extensions are configured (which means the user
            // has explicitly set rules for them, typically "never")
            let has_standard_extension_rules = config.has_rule("js")
                || config.has_rule("jsx")
                || config.has_rule("ts")
                || config.has_rule("tsx");

            if !has_standard_extension_rules {
                // No per-extension rules, so missing extensions should be flagged
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
                let module_name = s.value.to_compact_str();
                let span = call_expr.span;

                // Try to get the actual file extension from the resolved module path
                let resolved_extension =
                    get_resolved_extension(ctx.module_record(), module_name.as_str());

                // Get what's written in the require statement
                let written_extension = get_file_extension_from_module_name(&module_name);

                // Determine which extension to check against the configuration
                // Prefer resolved extension (actual file), fallback to written extension (require text)
                let extension_to_check =
                    resolved_extension.as_deref().or(written_extension.as_deref());

                if let Some(ext_str) = extension_to_check {
                    // Standard JS/TS extensions that are implicitly recognized
                    let is_standard_extension =
                        matches!(ext_str, "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "json");

                    // Skip validation if this extension is not explicitly configured,
                    // UNLESS there's a global require_extension rule (always/never)
                    // This prevents false positives for unconfigured extensions
                    if !config.has_rule(ext_str)
                        && require_extension.is_none()
                        && !is_standard_extension
                    {
                        continue;
                    }

                    // Check if the extension is present in the require statement
                    let extension_is_written = written_extension.is_some();

                    // Check if we have a resolved extension (from module resolution) or just written extension
                    let has_resolved_extension = resolved_extension.is_some();

                    let should_flag = match require_extension {
                        Some(&ExtensionRule::Always) => {
                            // Extension should always be present
                            if !extension_is_written {
                                // Missing extension - check if THIS specific extension is configured as "never"
                                // If so, the per-extension "never" rule overrides the global "always"
                                // Also allow if we have a resolved extension (from module resolution)
                                if has_resolved_extension {
                                    !config.is_never(ext_str)
                                } else {
                                    // No module resolution - be lenient if ANY standard extensions have "never" rules
                                    // This handles the case where the user configured "always" with specific "never" overrides
                                    let has_never_rules = config.is_never("js")
                                        || config.is_never("jsx")
                                        || config.is_never("ts")
                                        || config.is_never("tsx")
                                        || config.is_never("mjs")
                                        || config.is_never("cjs");
                                    !has_never_rules
                                }
                            } else if config.is_never(ext_str) {
                                // Extension is explicitly configured as "never"
                                true
                            } else {
                                // Extension is present and not explicitly "never" - allow it
                                false
                            }
                        }
                        Some(&ExtensionRule::Never) => {
                            // Extension should never be present
                            extension_is_written && !config.is_always(ext_str)
                        }
                        _ => {
                            // Default behavior: flag if extension violates per-extension rules
                            if extension_is_written {
                                // Extension is present, check if it should not be
                                config.is_never(ext_str)
                            } else {
                                // Extension is missing, check if it should be present
                                config.is_always(ext_str)
                            }
                        }
                    };

                    if should_flag {
                        if extension_is_written {
                            ctx.diagnostic(extension_should_not_be_included_in_diagnostic(
                                span,
                                written_extension.as_ref().unwrap(),
                                true,
                            ));
                        } else {
                            ctx.diagnostic(extension_missing_diagnostic(span, true));
                        }
                    }
                } else if matches!(require_extension, Some(&ExtensionRule::Always)) {
                    // No extension found (neither resolved nor written), but always is required
                    // However, if standard extensions have "never" rules, be lenient (without module resolution, we can't know the actual extension)
                    let has_never_rules = config.is_never("js")
                        || config.is_never("jsx")
                        || config.is_never("ts")
                        || config.is_never("tsx")
                        || config.is_never("mjs")
                        || config.is_never("cjs");

                    if !has_never_rules {
                        ctx.diagnostic(extension_missing_diagnostic(span, true));
                    }
                } else if matches!(require_extension, Some(&ExtensionRule::IgnorePackages)) {
                    // With ignorePackages, extensions are required for relative imports
                    // UNLESS all standard extensions are configured (which means the user
                    // has explicitly set rules for them, typically "never")
                    let has_standard_extension_rules = config.has_rule("js")
                        || config.has_rule("jsx")
                        || config.has_rule("ts")
                        || config.has_rule("tsx");

                    if !has_standard_extension_rules {
                        // No per-extension rules, so missing extensions should be flagged
                        ctx.diagnostic(extension_missing_diagnostic(span, true));
                    }
                }
            }
        }
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
fn get_file_extension_from_module_name(module_name: &CompactStr) -> Option<CompactStr> {
    if let Some((_, extension)) =
        module_name.split('?').next().unwrap_or(module_name).rsplit_once('.')
        && !extension.is_empty()
        && !extension.starts_with('/')
    {
        return Some(CompactStr::from(extension.to_ascii_lowercase()));
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
    module_record.get_loaded_module(module_name).and_then(|loaded_module| {
        loaded_module
            .resolved_absolute_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_ascii_lowercase())
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
            r#"import { x } from 'rootverse+debug:src.ts';"#,
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
            r#"import { x } from 'rootverse+debug:src';"#,
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
            r#"import { x } from 'rootverse+debug:src.ts';"#,
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
            r#"import { x } from 'rootverse+debug:src';"#,
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
            r#"import { x } from './regular-import.js';"#,
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
            r#"
                import { a } from './standard.js';
                import { b } from 'rootverse+debug:custom';
            "#,
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
            r#"import { x } from 'rootverse+debug:src';"#,
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
            r#"import { x } from 'rootverse+debug:src';"#,
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
            r#"import { x } from 'custom+protocol:module';"#,
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
            r#"
                import { a } from './standard';
                import { b } from 'custom+protocol:module';
            "#,
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
            r#"import { x } from 'workspace:packages/core/src';"#,
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
            r#"import { x } from 'MyCustom+Protocol:src';"#,
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
    ];

    Tester::new(Extensions::NAME, Extensions::PLUGIN, pass, fail).test_and_snapshot();
}
