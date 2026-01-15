use crate::context::LintContext;
use crate::rule::Rule;
use lazy_regex::Regex;
use nodejs_built_in_modules::is_nodejs_builtin_module;
use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use schemars::JsonSchema;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
pub struct NoUnresolved(Box<NoUnresolvedConfig>);

#[derive(Debug, Clone, JsonSchema)]
pub struct NoUnresolvedConfig {
    commonjs: bool,
    amd: bool,
    ignore: Vec<Regex>,
    case_sensitive: bool,
    case_sensitive_strict: bool,
}

impl std::ops::Deref for NoUnresolved {
    type Target = NoUnresolvedConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for NoUnresolvedConfig {
    fn default() -> Self {
        Self {
            commonjs: false,
            amd: false,
            ignore: vec![],
            case_sensitive: false,
            case_sensitive_strict: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks for unresolved imports.
    ///
    /// ### Why is this bad?
    ///
    /// Unresolved imports can cause runtime errors.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// import { foo } from './bar.js';
    /// ```
    ///
    /// If `./bar.js` doesn't exist, this rule will flag this import.
    ///
    /// ### Options
    /// - `commonjs`: Whether to check for unresolved CommonJS imports.
    /// - `amd`: Whether to check for unresolved AMD imports.
    /// - `ignore`: A list of regex patterns to ignore.
    /// - `case_sensitive`: By default, this rule will report paths whose case do not match the underlying filesystem path, if the FS is not case-sensitive. To disable this behavior, set the caseSensitive option to false.
    /// - `case_sensitive_strict`: The caseSensitive option does not detect case for the current working directory. The caseSensitiveStrict option allows checking cwd in resolved path. By default, the option is disabled.
    ///
    /// ### Default configuration
    /// ```json
    /// {
    ///     "commonjs": false,
    ///     "amd": false,
    ///     "ignore": [],
    ///     "case_sensitive": false,
    ///     "case_sensitive_strict": false
    /// }
    /// ```
    NoUnresolved,
    import,
    nursery,
    config = NoUnresolvedConfig,
);

impl Rule for NoUnresolved {
    fn from_configuration(value: serde_json::Value) -> Result<NoUnresolved, serde_json::Error> {
        let cfg = value.get(0).cloned().unwrap_or(serde_json::Value::Null);

        let commonjs = cfg.get("commonjs").and_then(serde_json::Value::as_bool).unwrap_or(false);
        let amd = cfg.get("amd").and_then(serde_json::Value::as_bool).unwrap_or(false);
        let case_sensitive =
            cfg.get("caseSensitive").and_then(serde_json::Value::as_bool).unwrap_or(false);
        let case_sensitive_strict =
            cfg.get("caseSensitiveStrict").and_then(serde_json::Value::as_bool).unwrap_or(false);

        let ignore = cfg
            .get("ignore")
            .and_then(serde_json::Value::as_array)
            .unwrap_or(&vec![])
            .iter()
            .filter_map(serde_json::Value::as_str)
            .filter_map(|s| lazy_regex::Regex::new(s).ok())
            .collect::<Vec<lazy_regex::Regex>>();

        Ok(NoUnresolved(Box::new(NoUnresolvedConfig {
            commonjs,
            amd,
            ignore,
            case_sensitive,
            case_sensitive_strict,
        })))
    }

    fn run<'a>(&self, node: &oxc_semantic::AstNode<'a>, ctx: &LintContext<'a>) {
        fn no_unresolved_diagnostic(span: Span, module_name: &str) -> OxcDiagnostic {
            OxcDiagnostic::warn(format!("Unable to resolve path to module {module_name:?}"))
                .with_help("This module could not be found or resolved by the importer")
                .with_label(span)
        }

        fn is_builtin_or_global(name: &str, ctx: &LintContext) -> bool {
            // Check if it's a builtin, handling both with and without the node: prefix.
            // First check the original name in case a builtin legitimately has "node:" in it.
            if is_nodejs_builtin_module(name) || ctx.globals().is_enabled(name) {
                return true;
            } else {
                false
            }
        }

        let module_record = ctx.module_record();

        match node.kind() {
            AstKind::ImportDeclaration(import_decl) => {
                let name = import_decl.source.value.as_str();

                // Note: Both `assert { type: 'json' }` (legacy) and `with { type: 'json' }` (newer)
                // import assertion syntaxes are supported. The `with_clause` field on the
                // ImportDeclaration AST node handles both syntaxes transparently.

                if self.ignore.iter().any(|re| re.is_match(name)) {
                    return;
                }
                if is_builtin_or_global(name, ctx) {
                    return;
                }

                let loaded = module_record.get_loaded_module(name);
                let span = module_record
                    .requested_modules
                    .get(&CompactStr::from(name))
                    .and_then(|v| v.get(0))
                    .map(|r| r.statement_span)
                    .unwrap_or(import_decl.source.span);

                if loaded.is_none() {
                    // When the import plugin doesn't load JSON files, fall back to a direct
                    // filesystem existence check so valid JSON imports are not falsely reported.
                    if is_existing_json(name, &module_record.resolved_absolute_path) {
                        return;
                    }

                    ctx.diagnostic(no_unresolved_diagnostic(span, name));
                    return;
                }

                if self.case_sensitive {
                    if let Some(remote) = &loaded {
                        let resolved = &remote.resolved_absolute_path;
                        if name.starts_with('.') || name.starts_with('/') {
                            if !module_name_case_matches(
                                resolved,
                                &CompactStr::from(name),
                                self.case_sensitive_strict,
                            ) {
                                ctx.diagnostic(no_unresolved_diagnostic(span, name));
                            }
                        }
                    }
                }
            }
            AstKind::CallExpression(call_expr) => {
                let Expression::Identifier(ident) = &call_expr.callee else { return };
                let func_name = ident.name.as_str();
                let count = call_expr.arguments.len();

                if !matches!(func_name, "require" | "define") || count == 0 {
                    return;
                }

                match &call_expr.arguments[0] {
                    Argument::StringLiteral(str_lit) if func_name == "require" && self.commonjs => {
                        let name = str_lit.value.as_str();
                        if self.ignore.iter().any(|re| re.is_match(name)) {
                            return;
                        }
                        if is_builtin_or_global(name, ctx) {
                            return;
                        }
                        let span = module_record
                            .requested_modules
                            .get(&CompactStr::from(name))
                            .and_then(|v| v.get(0))
                            .map(|r| r.statement_span)
                            .unwrap_or(str_lit.span);
                        if module_record.get_loaded_module(name).is_none()
                            && !is_existing_json(name, &module_record.resolved_absolute_path)
                        {
                            ctx.diagnostic(no_unresolved_diagnostic(span, name));
                        }
                    }
                    Argument::ArrayExpression(arr_expr) if count == 2 && self.amd => {
                        for el in &arr_expr.elements {
                            if let Some(el_expr) = el.as_expression() {
                                if let Expression::StringLiteral(literal) = el_expr {
                                    let name = literal.value.as_str();
                                    if self.ignore.iter().any(|re| re.is_match(name)) {
                                        continue;
                                    }
                                    if is_builtin_or_global(name, ctx) {
                                        continue;
                                    }
                                    let span = module_record
                                        .requested_modules
                                        .get(&CompactStr::from(name))
                                        .and_then(|v| v.get(0))
                                        .map(|r| r.statement_span)
                                        .unwrap_or(literal.span);
                                    if module_record.get_loaded_module(name).is_none()
                                        && !is_existing_json(
                                            name,
                                            &module_record.resolved_absolute_path,
                                        )
                                    {
                                        ctx.diagnostic(no_unresolved_diagnostic(span, name));
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl crate::rule::RuleRunner for NoUnresolved {
    const NODE_TYPES: Option<&'static oxc_semantic::AstTypesBitset> = None;
    const RUN_FUNCTIONS: crate::rule::RuleRunFunctionsImplemented =
        crate::rule::RuleRunFunctionsImplemented::Run;
}

fn module_name_case_matches(resolved: &Path, module_name: &CompactStr, strict: bool) -> bool {
    // Compare the filename (and stem if no extension in the import) with the import specifier's last segment.
    let import = module_name.as_str();

    // Build a normalized absolute path for the import specifier using the resolved path's parent
    // so we can compare component-by-component without performing extra filesystem resolution.
    let Some(importer_dir) = resolved.parent() else { return true };

    let import_abspath = normalize_join(importer_dir, import);

    // Compare path components exactly (case-sensitive). If any component differs, return false.
    let import_comps: Vec<String> =
        import_abspath.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect();

    let resolved_comps: Vec<String> =
        resolved.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect();

    // If lengths differ, align from the end (to handle drive/root differences) and compare tail components.
    let (mut i, mut j) = (import_comps.len() as isize - 1, resolved_comps.len() as isize - 1);

    while i >= 0 && j >= 0 {
        if import_comps[i as usize] != resolved_comps[j as usize] {
            return false;
        }
        i -= 1;
        j -= 1;
    }

    // If strict, ensure we compared all components of the import path (i < 0). Otherwise, partial match is fine.
    if strict { i < 0 } else { true }
}

fn normalize_join(base: &Path, spec: &str) -> std::path::PathBuf {
    use std::path::PathBuf;

    let mut buf = PathBuf::new();
    buf.push(base);

    for part in spec.split('/') {
        match part {
            "" | "." => continue,
            ".." => {
                buf.pop();
            }
            p => buf.push(p),
        }
    }

    buf
}

fn is_existing_json(specifier: &str, importer_path: &Path) -> bool {
    if !specifier.ends_with(".json") {
        return false;
    }

    let candidate = if specifier.starts_with('/') {
        PathBuf::from(specifier)
    } else {
        importer_path.parent().map(|dir| dir.join(specifier)).unwrap_or_default()
    };

    !candidate.as_os_str().is_empty() && candidate.exists()
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        // Resolve: existing file
        (r#"import './default-export'"#, None),
        // Builtins: Node builtin
        (r#"import path from 'path'"#, None),
        // CommonJS: disabled config (should pass)
        (r#"var foo = require('./does-not-exist')"#, Some(json!([{ "commonjs": false }]))),
        // AMD: disabled config (should pass)
        (r#"require(['./does-not-exist'], function() {})"#, Some(json!([{ "amd": false }]))),
        // Ignore: simple pattern
        (r#"import './ignored-file'"#, Some(json!([{ "ignore": ["^\\./ignored"] }]))),
        // Ignore: exact match pattern
        (r#"import './ignored-file'"#, Some(json!([{ "ignore": ["^\\./ignored-file$"] }]))),
        // Case: insensitive (default)
        (r#"import './Default-Export'"#, Some(json!([{ "caseSensitive": false }]))),
        // Case: insensitive with strict flag
        (
            r#"import './Default-Export'"#,
            Some(json!([{ "caseSensitive": false, "caseSensitiveStrict": true }])),
        ),
        // JSON imports with import assertions (both assert and with syntaxes)
        (r#"import data from './data.json' assert { type: 'json' }"#, None),
        (r#"import foobar from './foobar.json' with { type: 'json' }"#, None),
    ];

    let fail = vec![
        // Resolve: missing relative import
        (r#"import './does-not-exist'"#, None),
        // CommonJS: enabled config (should fail)
        (r#"var foo = require('./does-not-exist')"#, Some(json!([{ "commonjs": true }]))),
        // AMD: enabled config (should fail)
        (r#"require(['./does-not-exist'], function() {})"#, Some(json!([{ "amd": true }]))),
        // Ignore: no ignore config
        (r#"import './ignored-file'"#, None),
        // Ignore: pattern doesn't match
        (r#"import './ignored-file'"#, Some(json!([{ "ignore": ["^\\./other$"] }]))),
        // Case: sensitive mismatch
        (r#"import './Default-Export'"#, Some(json!([{ "caseSensitive": true }]))),
        // Case: sensitive with strict
        (
            r#"import './Default-Export'"#,
            Some(json!([{ "caseSensitive": true, "caseSensitiveStrict": true }])),
        ),
    ];

    Tester::new(NoUnresolved::NAME, NoUnresolved::PLUGIN, pass, fail)
        .with_snapshot_suffix("basic")
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}

#[test]
fn test_node_protocol_builtins() {
    use crate::tester::Tester;

    let pass = vec![
        // Node builtin should be ignored/resolved
        (r#"import path from 'path'"#, None),
        // node: protocol should resolve to builtins
        (r#"import fs from 'node:fs'"#, None),
        (r#"import util from 'node:util'"#, None),
        // builtins with required node: prefix should resolve
        (r#"import test from 'node:test'"#, None),
        (r#"import sqlite from 'node:sqlite'"#, None),
        // JSON imports with assert { type: 'json' } (legacy syntax)
        (r#"import data from './default-export' assert { type: 'json' }"#, None),
        // JSON imports with with { type: 'json' } (newer syntax)
        (r#"import data from './default-export' with { type: 'json' }"#, None),
    ];

    let fail = vec![
        // unresolved package import
        (r#"import 'nonexistent-package'"#, None),
        // unresolved relative import
        (r#"import './does-not-exist'"#, None),
    ];

    Tester::new(NoUnresolved::NAME, NoUnresolved::PLUGIN, pass, fail)
        .with_snapshot_suffix("node-protocol")
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
