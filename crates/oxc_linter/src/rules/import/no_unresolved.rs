use crate::rule::{DefaultRuleConfig, Rule};
use lazy_regex::Regex;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_resolver::NODEJS_BUILTINS;
use oxc_span::{CompactStr, Span};
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct NoUnresolved(Box<NoUnresolvedConfig>);

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
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
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoUnresolved>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();

        fn no_unresolved_diagnostic(span: Span, module_name: &str) -> OxcDiagnostic {
            OxcDiagnostic::warn(format!("Unable to resolve path to module {module_name:?}"))
                .with_help("This module could not be found or resolved by the importer")
                .with_label(span)
        }

        for (module_name, requested_list) in &module_record.requested_modules {
            let name = module_name.as_str();

            // ignore patterns
            if self.ignore.iter().any(|re| re.is_match(name)) {
                continue;
            }

            // ignore Node builtins and user-provided globals
            if NODEJS_BUILTINS.binary_search(&name).is_ok() || ctx.globals().is_enabled(name)
            {
                continue;
            }

            // If there's a loaded module, it's resolved.
            let loaded = module_record.get_loaded_module(name);

            for requested in requested_list {
                // requested.is_import == true indicates ES import; for require/amd entries
                // only report when configured to check CommonJS/AMD
                if !requested.is_import && !(self.commonjs || self.amd) {
                    continue;
                }

                if loaded.is_none() {
                    ctx.diagnostic(no_unresolved_diagnostic(requested.statement_span, name));
                } else if self.case_sensitive {
                    // Check that the casing in the import matches the filesystem casing for the resolved file name.
                    if let Some(remote) = &loaded {
                        let resolved = &remote.resolved_absolute_path;

                        // Only check for relative or absolute path imports, skip package imports
                        if name.starts_with('.') || name.starts_with('/') {
                            if !module_name_case_matches(resolved, module_name, self.case_sensitive_strict)
                            {
                                ctx.diagnostic(no_unresolved_diagnostic(requested.statement_span, name));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn module_name_case_matches(resolved: &Path, module_name: &CompactStr, strict: bool) -> bool {
    // Compare the filename (and stem if no extension in the import) with the import specifier's last segment.
    let import = module_name.as_str();

    // Build a normalized absolute path for the import specifier using the resolved path's parent
    // so we can compare component-by-component without performing extra filesystem resolution.
    let Some(importer_dir) = resolved.parent() else { return true };

    let import_abspath = normalize_join(importer_dir, import);

    // Compare path components exactly (case-sensitive). If any component differs, return false.
    let import_comps: Vec<String> = import_abspath
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();

    let resolved_comps: Vec<String> = resolved
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();

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
    use std::path::{Component, PathBuf};

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

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;
    let pass = vec![
        // existing file
        (r#"import './default-export'"#, None),
        // builtin ignored
        (r#"import path from 'path'"#, None),
        // require allowed when commonjs check disabled
        (r#"var foo = require('./does-not-exist')"#, Some(json!([{ "commonjs": false }])),),
        // AMD require with amd disabled
        (r#"require(['./does-not-exist'], function() {})"#, Some(json!([{ "amd": false }])),),
        // ignored by regex
        (r#"import './ignored-file'"#, Some(json!([{ "ignore": ["^\\./ignored"] }])),),
        // case mismatch allowed when caseSensitive false
        (r#"import './Default-Export'"#, Some(json!([{ "caseSensitive": false }])),),
    ];

    let fail = vec![
        // missing ES import
        (r#"import './does-not-exist'"#, None),
        // missing require when commonjs enabled
        (r#"var foo = require('./does-not-exist')"#, Some(json!([{ "commonjs": true }])),),
        // AMD missing when amd enabled
        (r#"require(['./does-not-exist'], function() {})"#, Some(json!([{ "amd": true }])),),
        // ignored only when ignore provided; without ignore, it fails
        (r#"import './ignored-file'"#, None),
        // case mismatch reported when caseSensitive true
        (r#"import './Default-Export'"#, Some(json!([{ "caseSensitive": true }])),),
    ];

    Tester::new(NoUnresolved::NAME, NoUnresolved::PLUGIN, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}

#[test]
fn test_config_variations() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        // AMD array allowed when amd=false
        (r#"require(['./does-not-exist'], function() {})"#, Some(json!([{ "amd": false }]))),
        // exact ignore regex matches
        (r#"import './ignored-file'"#, Some(json!([{ "ignore": ["^\\./ignored-file$"] }]))),
        // case mismatch allowed when caseSensitive=false (strict irrelevant)
        (r#"import './Default-Export'"#, Some(json!([{ "caseSensitive": false, "caseSensitiveStrict": true }]))),
    ];

    let fail = vec![
        // AMD array reported when amd=true
        (r#"require(['./does-not-exist'], function() {})"#, Some(json!([{ "amd": true }]))),
        // ignore regex does not match -> report
        (r#"import './ignored-file'"#, Some(json!([{ "ignore": ["^\\./other$"] }]))),
        // strict case-sensitivity reports mismatch
        (r#"import './Default-Export'"#, Some(json!([{ "caseSensitive": true, "caseSensitiveStrict": true }]))),
    ];

    Tester::new(NoUnresolved::NAME, NoUnresolved::PLUGIN, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}

#[test]
fn test_package_vs_relative() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        // Node builtin should be ignored/resolved
        (r#"import path from 'path'"#, None),
    ];

    let fail = vec![
        // unresolved package import
        (r#"import 'nonexistent-package'"#, None),
        // unresolved relative import
        (r#"import './does-not-exist'"#, None),
    ];

    Tester::new(NoUnresolved::NAME, NoUnresolved::PLUGIN, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
