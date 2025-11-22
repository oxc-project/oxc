use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use fast_glob::glob_match;
use lazy_regex::{Regex, regex::Error as RegexError};
use oxc_ast::{
    AstKind,
    ast::{
        Argument, CallExpression, ExportAllDeclaration, ExportNamedDeclaration, Expression,
        ImportDeclaration, ImportDeclarationSpecifier, ImportExpression, ImportOrExportKind,
        StringLiteral, TSImportEqualsDeclaration, TSModuleReference,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_resolver::NODEJS_BUILTINS;
use oxc_span::Span;
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NoExtraneousDependenciesConfig {
    dev_dependencies: Option<BoolOrPatterns>,
    optional_dependencies: Option<BoolOrPatterns>,
    peer_dependencies: Option<BoolOrPatterns>,
    bundled_dependencies: Option<BoolOrPatterns>,
    include_internal: bool,
    include_types: bool,
    #[serde(
        default,
        deserialize_with = "deserialize_package_dir",
        alias = "packageDirs",
        alias = "package_dirs"
    )]
    package_dir: Vec<String>,
}

impl Default for NoExtraneousDependenciesConfig {
    fn default() -> Self {
        Self {
            dev_dependencies: None,
            optional_dependencies: None,
            peer_dependencies: None,
            bundled_dependencies: None,
            include_internal: false,
            include_types: false,
            package_dir: Vec::new(),
        }
    }
}

fn deserialize_package_dir<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        String(String),
        Vec(Vec<String>),
    }

    let dirs = StringOrVec::deserialize(deserializer)?;
    Ok(match dirs {
        StringOrVec::String(dir) => vec![dir],
        StringOrVec::Vec(dirs) => dirs,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
enum BoolOrPatterns {
    Bool(bool),
    Patterns(Vec<String>),
}

#[derive(Debug, Default, Clone)]
pub struct NoExtraneousDependencies {
    config: NoExtraneousDependenciesConfig,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures every imported or required external module is listed in the nearest `package.json`.
    ///
    /// ### Why is this bad?
    ///
    /// Undeclared dependencies make installs and deployments non-deterministic and often fail when
    /// the package manager does not automatically hoist a transient dependency.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import leftPad from "left-pad";
    /// const semver = require("semver");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import React from "react"; // listed in dependencies
    /// import type { Props } from "optional-types"; // ignored when `includeTypes` is false
    /// ```
    ///
    /// Port of [eslint-plugin-import/no-extraneous-dependencies](https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/no-extraneous-dependencies.md).
    NoExtraneousDependencies,
    import,
    restriction,
    config = NoExtraneousDependenciesConfig,
);

impl Rule for NoExtraneousDependencies {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config =
            serde_json::from_value::<DefaultRuleConfig<NoExtraneousDependenciesConfig>>(value)
                .map(DefaultRuleConfig::into_inner)
                .unwrap_or_default();
        Self { config }
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let file_path = ctx.file_path();
        let Some(file_dir) = file_path.parent() else {
            return;
        };

        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let package_dirs: Option<Vec<PathBuf>> = if self.config.package_dir.is_empty() {
            None
        } else {
            Some(
                self.config
                    .package_dir
                    .iter()
                    .map(|dir| {
                        let path = Path::new(dir);
                        if path.is_absolute() { path.to_path_buf() } else { cwd.join(path) }
                    })
                    .collect(),
            )
        };

        let deps = match load_dependencies(file_dir, package_dirs.as_deref()) {
            Ok(Some(deps)) => deps,
            Ok(None) => return,
            Err(err) => {
                ctx.diagnostic(package_json_error_diagnostic(&err));
                return;
            }
        };

        let file_glob_path = normalize_path(file_path);
        let cwd_glob_path = normalize_path(&cwd);

        let options = DependencyOptions {
            allow_dev: evaluate_allowance(
                self.config.dev_dependencies.as_ref(),
                &file_glob_path,
                &cwd_glob_path,
            ),
            allow_optional: evaluate_allowance(
                self.config.optional_dependencies.as_ref(),
                &file_glob_path,
                &cwd_glob_path,
            ),
            allow_peer: evaluate_allowance(
                self.config.peer_dependencies.as_ref(),
                &file_glob_path,
                &cwd_glob_path,
            ),
            allow_bundled: evaluate_allowance(
                self.config.bundled_dependencies.as_ref(),
                &file_glob_path,
                &cwd_glob_path,
            ),
            verify_internal: self.config.include_internal,
            verify_types: self.config.include_types,
        };

        let internal_regex = ctx
            .settings()
            .json
            .as_ref()
            .and_then(|settings| settings.get("import/internal-regex"))
            .and_then(Value::as_str)
            .and_then(|pattern| match Regex::new(pattern) {
                Ok(regex) => Some(regex),
                Err(err) => {
                    ctx.diagnostic(invalid_internal_regex_diagnostic(pattern, &err));
                    None
                }
            });

        DependencyChecker { ctx, deps, options, internal_regex }.run();
    }
}

struct DependencyChecker<'a, 'ctx> {
    ctx: &'ctx LintContext<'a>,
    deps: DependencySets,
    options: DependencyOptions,
    internal_regex: Option<Regex>,
}

impl<'a, 'ctx> DependencyChecker<'a, 'ctx> {
    fn run(&self) {
        for node in self.ctx.nodes().iter() {
            match node.kind() {
                AstKind::ImportDeclaration(decl) => self.visit_import_declaration(decl),
                AstKind::ExportNamedDeclaration(decl) => self.visit_export_named(decl),
                AstKind::ExportAllDeclaration(decl) => self.visit_export_all(decl),
                AstKind::ImportExpression(expr) => self.visit_import_expression(expr),
                AstKind::TSImportEqualsDeclaration(decl) => self.visit_ts_import_equals(decl),
                AstKind::CallExpression(call) => self.visit_call_expression(call),
                _ => {}
            }
        }
    }

    fn visit_import_declaration(&self, decl: &ImportDeclaration<'a>) {
        if !self.options.verify_types && import_is_type_only(decl) {
            return;
        }
        self.check_literal(&decl.source);
    }

    fn visit_export_named(&self, decl: &ExportNamedDeclaration<'a>) {
        let Some(source) = decl.source.as_ref() else { return };
        if !self.options.verify_types && export_named_is_type_only(decl) {
            return;
        }
        self.check_literal(source);
    }

    fn visit_export_all(&self, decl: &ExportAllDeclaration<'a>) {
        if !self.options.verify_types && matches!(decl.export_kind, ImportOrExportKind::Type) {
            return;
        }
        self.check_literal(&decl.source);
    }

    fn visit_import_expression(&self, expr: &ImportExpression<'a>) {
        if let Some((value, span)) = string_from_expression(&expr.source) {
            self.check_specifier(value, span);
        }
    }

    fn visit_ts_import_equals(&self, decl: &TSImportEqualsDeclaration<'a>) {
        if !self.options.verify_types && matches!(decl.import_kind, ImportOrExportKind::Type) {
            return;
        }

        if let TSModuleReference::ExternalModuleReference(ext) = &decl.module_reference {
            self.check_literal(&ext.expression);
        }
    }

    fn visit_call_expression(&self, call: &CallExpression<'a>) {
        if call.arguments.is_empty() {
            return;
        }

        if !is_commonjs_require(call) {
            return;
        }

        if let Some((value, span)) = argument_to_string(&call.arguments[0]) {
            self.check_specifier(value, span);
        }
    }

    fn check_literal(&self, literal: &StringLiteral<'a>) {
        self.check_specifier(literal.value.as_str(), literal.span);
    }

    fn check_specifier(&self, raw: &str, span: Span) {
        let Some(normalized) = normalize_specifier(raw) else { return };

        if is_builtin(&normalized) {
            return;
        }

        let is_internal = self
            .internal_regex
            .as_ref()
            .is_some_and(|regex| regex.is_match(normalized.full.as_str()));

        if is_internal && !self.options.verify_internal {
            return;
        }

        let status = self.deps.classify(normalized.full.as_str());

        if status.satisfies(&self.options) {
            return;
        }

        let diagnostic = if status.is_in_dev && !self.options.allow_dev {
            dev_dependency_diagnostic(span, normalized.display_name())
        } else if status.is_in_optional && !self.options.allow_optional {
            optional_dependency_diagnostic(span, normalized.display_name())
        } else if status.is_in_peer && !self.options.allow_peer {
            peer_dependency_diagnostic(span, normalized.display_name())
        } else if status.is_in_bundled && !self.options.allow_bundled {
            bundled_dependency_diagnostic(span, normalized.display_name())
        } else {
            missing_dependency_diagnostic(span, normalized.display_name())
        };

        self.ctx.diagnostic(diagnostic);
    }
}

#[derive(Debug, Default)]
struct DependencySets {
    dependencies: FxHashSet<String>,
    dev_dependencies: FxHashSet<String>,
    optional_dependencies: FxHashSet<String>,
    peer_dependencies: FxHashSet<String>,
    bundled_dependencies: FxHashSet<String>,
}

impl DependencySets {
    fn merge(&mut self, other: DependencySets) {
        self.dependencies.extend(other.dependencies);
        self.dev_dependencies.extend(other.dev_dependencies);
        self.optional_dependencies.extend(other.optional_dependencies);
        self.peer_dependencies.extend(other.peer_dependencies);
        self.bundled_dependencies.extend(other.bundled_dependencies);
    }

    fn classify(&self, package_path: &str) -> DeclarationStatus {
        let mut status = DeclarationStatus::default();
        for ancestor in package_hierarchy(package_path) {
            if self.dependencies.contains(ancestor.as_str()) {
                status.is_in_deps = true;
            }
            if self.dev_dependencies.contains(ancestor.as_str()) {
                status.is_in_dev = true;
            }
            if self.optional_dependencies.contains(ancestor.as_str()) {
                status.is_in_optional = true;
            }
            if self.peer_dependencies.contains(ancestor.as_str()) {
                status.is_in_peer = true;
            }
            if self.bundled_dependencies.contains(ancestor.as_str()) {
                status.is_in_bundled = true;
            }
        }
        status
    }
}

#[derive(Debug, Default)]
struct DeclarationStatus {
    is_in_deps: bool,
    is_in_dev: bool,
    is_in_optional: bool,
    is_in_peer: bool,
    is_in_bundled: bool,
}

impl DeclarationStatus {
    fn satisfies(&self, options: &DependencyOptions) -> bool {
        self.is_in_deps
            || (self.is_in_dev && options.allow_dev)
            || (self.is_in_optional && options.allow_optional)
            || (self.is_in_peer && options.allow_peer)
            || (self.is_in_bundled && options.allow_bundled)
    }
}

struct DependencyOptions {
    allow_dev: bool,
    allow_optional: bool,
    allow_peer: bool,
    allow_bundled: bool,
    verify_internal: bool,
    verify_types: bool,
}

fn import_is_type_only(decl: &ImportDeclaration<'_>) -> bool {
    if matches!(decl.import_kind, ImportOrExportKind::Type) {
        return true;
    }

    let Some(specifiers) = decl.specifiers.as_ref() else {
        // Side-effect imports (no specifiers) are never considered type-only.
        return false;
    };

    !specifiers.is_empty()
        && specifiers.iter().all(|specifier| {
            matches!(specifier, ImportDeclarationSpecifier::ImportSpecifier(spec) if matches!(spec.import_kind, ImportOrExportKind::Type))
        })
}

fn export_named_is_type_only(decl: &ExportNamedDeclaration<'_>) -> bool {
    if matches!(decl.export_kind, ImportOrExportKind::Type) {
        return true;
    }
    !decl.specifiers.is_empty()
        && decl
            .specifiers
            .iter()
            .all(|specifier| matches!(specifier.export_kind, ImportOrExportKind::Type))
}

fn string_from_expression<'a>(expr: &'a Expression<'a>) -> Option<(&'a str, Span)> {
    match expr.without_parentheses() {
        Expression::StringLiteral(literal) => Some((literal.value.as_str(), literal.span)),
        Expression::TemplateLiteral(template) => {
            let template = template.as_ref();
            if template.expressions.is_empty() && template.quasis.len() == 1 {
                let quasi = &template.quasis[0];
                let value = quasi
                    .value
                    .cooked
                    .as_ref()
                    .map_or_else(|| quasi.value.raw.as_str(), |atom| atom.as_str());
                Some((value, quasi.span))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn argument_to_string<'a>(arg: &'a Argument<'a>) -> Option<(&'a str, Span)> {
    let expression = arg.as_expression()?;
    string_from_expression(expression)
}

fn is_commonjs_require(call: &CallExpression<'_>) -> bool {
    matches!(
        call.callee.without_parentheses(),
        Expression::Identifier(ident) if ident.name == "require"
    )
}

fn normalize_specifier(raw: &str) -> Option<NormalizedModuleName> {
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || trimmed.starts_with('.')
        || trimmed.starts_with('/')
        || trimmed.starts_with('#')
    {
        return None;
    }

    let trimmed = trimmed.strip_prefix("node:").unwrap_or(trimmed);
    if trimmed.is_empty() || trimmed.contains("://") || trimmed.starts_with("data:") {
        return None;
    }

    let sanitized = trimmed.split(|c| c == '?' || c == '#').next().unwrap_or("").to_string();
    if sanitized.is_empty() {
        return None;
    }

    let package_base = {
        let mut segments = sanitized.split('/');
        let first = segments.next()?;

        if first.starts_with('@') {
            let second = segments.next()?;
            if second.is_empty() {
                return None;
            }
            format!("{first}/{second}")
        } else {
            if first.contains(':') {
                // A colon indicates a scheme (e.g. data:, https:) which Node resolves via URL
                // semantics, so treat those as unsupported specifiers.
                return None;
            }
            first.to_string()
        }
    };

    Some(NormalizedModuleName { full: sanitized, package_base })
}

fn is_builtin(name: &NormalizedModuleName) -> bool {
    NODEJS_BUILTINS.binary_search(&name.package_base.as_str()).is_ok()
}

struct NormalizedModuleName {
    full: String,
    package_base: String,
}

impl NormalizedModuleName {
    fn display_name(&self) -> &str {
        &self.package_base
    }
}

/// Builds every ancestor of the import specifier, skipping the bare `@scope` prefix because it
/// is never a complete npm package on its own. For example `@scope/pkg/sub` becomes
/// `["@scope/pkg", "@scope/pkg/sub"]`.
fn package_hierarchy(specifier: &str) -> Vec<String> {
    let parts: Vec<_> = specifier.split('/').collect();
    let mut ancestors = Vec::new();

    for (index, part) in parts.iter().enumerate() {
        if part.starts_with('@') {
            continue;
        }
        ancestors.push(parts[..=index].join("/"));
    }

    ancestors
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Returns `true` when a dependency type is permitted for the given file. Unconfigured types (`None`)
/// default to allowed, matching ESLint's behavior.
fn evaluate_allowance(option: Option<&BoolOrPatterns>, file_path: &str, cwd: &str) -> bool {
    match option {
        None => true,
        Some(BoolOrPatterns::Bool(value)) => *value,
        Some(BoolOrPatterns::Patterns(patterns)) => patterns.iter().any(|pattern| {
            if glob_match(pattern, file_path) {
                return true;
            }

            if Path::new(pattern).is_absolute() {
                return false;
            }

            let joined = join_pattern(cwd, pattern);
            glob_match(&joined, file_path)
        }),
    }
}

fn join_pattern(cwd: &str, pattern: &str) -> String {
    let base = Path::new(cwd);
    let pattern_path = Path::new(pattern);
    let joined = if pattern_path.is_absolute() {
        pattern_path.to_path_buf()
    } else {
        base.join(pattern_path)
    };
    normalize_path(&joined)
}

fn load_dependencies(
    start_dir: &Path,
    package_dirs: Option<&[PathBuf]>,
) -> Result<Option<DependencySets>, PackageJsonError> {
    if let Some(dirs) = package_dirs {
        let mut combined = DependencySets::default();
        let mut found_any = false;
        let mut first_missing: Option<PathBuf> = None;

        for dir in dirs {
            let path = dir.join("package.json");
            match read_package_json(&path)? {
                Some(dep) => {
                    combined.merge(dep);
                    found_any = true;
                }
                None => {
                    if first_missing.is_none() {
                        first_missing = Some(path);
                    }
                }
            }
        }

        if found_any {
            return Ok(Some(combined));
        }

        let missing_path = first_missing
            .or_else(|| dirs.first().map(|dir| dir.join("package.json")))
            .unwrap_or_else(|| PathBuf::from("package.json"));
        return Err(PackageJsonError::Missing(missing_path));
    }

    let mut current = Some(start_dir);
    while let Some(dir) = current {
        let candidate = dir.join("package.json");
        match read_package_json(&candidate)? {
            Some(dep) => return Ok(Some(dep)),
            None => {
                current = dir.parent();
            }
        }
    }

    Ok(None)
}

fn read_package_json(path: &Path) -> Result<Option<DependencySets>, PackageJsonError> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(PackageJsonError::Io(path.to_path_buf(), err)),
    };

    let value: Value = serde_json::from_str(&contents)
        .map_err(|err| PackageJsonError::Parse(path.to_path_buf(), err))?;

    let object = value.as_object().ok_or_else(|| {
        PackageJsonError::Parse(
            path.to_path_buf(),
            serde_json::Error::io(io::Error::new(
                io::ErrorKind::InvalidData,
                "package.json must be an object",
            )),
        )
    })?;

    Ok(Some(build_dependency_sets(object)))
}

fn build_dependency_sets(map: &Map<String, Value>) -> DependencySets {
    let mut deps = DependencySets::default();
    deps.dependencies = collect_dependency_keys(map.get("dependencies"));
    deps.dev_dependencies = collect_dependency_keys(map.get("devDependencies"));
    deps.optional_dependencies = collect_dependency_keys(map.get("optionalDependencies"));
    deps.peer_dependencies = collect_dependency_keys(map.get("peerDependencies"));
    deps.bundled_dependencies = collect_bundled_dependencies(map);
    deps
}

fn collect_dependency_keys(value: Option<&Value>) -> FxHashSet<String> {
    let mut set = FxHashSet::default();
    if let Some(Value::Object(obj)) = value {
        set.extend(obj.keys().cloned());
    }
    set
}

fn collect_bundled_dependencies(map: &Map<String, Value>) -> FxHashSet<String> {
    let mut set = FxHashSet::default();
    let value = map.get("bundledDependencies").or_else(|| map.get("bundleDependencies"));

    match value {
        Some(Value::Array(arr)) => {
            for item in arr {
                if let Some(name) = item.as_str() {
                    set.insert(name.to_string());
                }
            }
        }
        Some(Value::Object(obj)) => {
            set.extend(obj.keys().cloned());
        }
        _ => {}
    }

    set
}

#[derive(Debug)]
enum PackageJsonError {
    Missing(PathBuf),
    Io(PathBuf, std::io::Error),
    Parse(PathBuf, serde_json::Error),
}

fn package_json_error_diagnostic(err: &PackageJsonError) -> OxcDiagnostic {
    match err {
        PackageJsonError::Missing(path) => OxcDiagnostic::warn(format!(
            "The package.json file could not be found at '{}'.",
            path.display()
        ))
        .with_label(Span::new(0, 0)),
        PackageJsonError::Io(path, error) => {
            OxcDiagnostic::warn(format!("Failed to read '{}': {}", path.display(), error))
                .with_label(Span::new(0, 0))
        }
        PackageJsonError::Parse(path, error) => OxcDiagnostic::warn(format!(
            "The package.json file at '{}' could not be parsed: {}",
            path.display(),
            error
        ))
        .with_label(Span::new(0, 0)),
    }
}

fn missing_dependency_diagnostic(span: Span, package: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{package}' should be listed in the project's dependencies."))
        .with_help(format!(
            "Run `npm install --save {package}` (or the equivalent for your package manager)."
        ))
        .with_label(span)
}

fn dev_dependency_diagnostic(span: Span, package: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "'{package}' should be listed in dependencies, not devDependencies."
    ))
    .with_label(span)
}

fn optional_dependency_diagnostic(span: Span, package: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "'{package}' should be listed in dependencies, not optionalDependencies."
    ))
    .with_label(span)
}

fn peer_dependency_diagnostic(span: Span, package: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "'{package}' should be listed in dependencies, not peerDependencies."
    ))
    .with_label(span)
}

fn bundled_dependency_diagnostic(span: Span, package: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "'{package}' should be listed in dependencies, not bundledDependencies."
    ))
    .with_label(span)
}

fn invalid_internal_regex_diagnostic(pattern: &str, err: &RegexError) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Invalid 'import/internal-regex' pattern '{pattern}': {err}"))
        .with_label(Span::new(0, 0))
}

#[test]
fn package_hierarchy_handles_scoped_packages() {
    assert_eq!(
        package_hierarchy("@scope/pkg/sub"),
        vec![String::from("@scope/pkg"), String::from("@scope/pkg/sub")]
    );
    assert_eq!(package_hierarchy("pkg/sub"), vec![String::from("pkg"), String::from("pkg/sub")]);
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass: Vec<(&str, Option<Value>, Option<Value>)> = vec![
        ("import React from 'react';", None, None),
        // devDependencies are allowed by default when no `devDependencies` config override is set.
        ("const dev = require('dev-only');", None, None),
        // Type-only imports are ignored until `includeTypes` is enabled.
        ("import type { Foo } from 'optional-only';", None, None),
        ("import '@scope/pkg/lib';", None, None),
        ("import('@scope/pkg');", None, None),
        (
            "import internal from '@internal/utils';",
            None,
            Some(json!({ "settings": { "import/internal-regex": "^@internal/" } })),
        ),
        (
            "import custom from 'custom-only';",
            Some(
                json!([{ "packageDir": ["fixtures/import/no_extraneous_dependencies/custom_pkg"] }]),
            ),
            None,
        ),
        (
            "import custom from 'custom-only';",
            Some(
                json!([{ "packageDir": "fixtures/import/no_extraneous_dependencies/custom_pkg" }]),
            ),
            None,
        ),
    ];

    let fail: Vec<(&str, Option<Value>, Option<Value>)> = vec![
        ("import missing from 'left-pad';", None, None),
        ("const foo = require('left-pad');", None, None),
        ("import type { Foo } from 'left-pad';", Some(json!([{ "includeTypes": true }])), None),
        ("export * from 'left-pad';", None, None),
        ("import devOnly from 'dev-only';", Some(json!([{ "devDependencies": false }])), None),
        (
            "import opt from 'optional-only';",
            Some(json!([{ "optionalDependencies": false }])),
            None,
        ),
        ("import peer from 'peer-only';", Some(json!([{ "peerDependencies": false }])), None),
        (
            "import bundled from 'bundled-only';",
            Some(json!([{ "bundledDependencies": false }])),
            None,
        ),
        (
            "import internal from '@internal/utils';",
            Some(json!([{ "includeInternal": true }])),
            Some(json!({ "settings": { "import/internal-regex": "^@internal/" } })),
        ),
        ("import custom from 'custom-only';", None, None),
    ];

    Tester::new(NoExtraneousDependencies::NAME, NoExtraneousDependencies::PLUGIN, pass, fail)
        .change_rule_path("no_extraneous_dependencies/src/index.ts")
        .with_import_plugin(true)
        .test_and_snapshot();

    let glob_pass: Vec<(&str, Option<Value>, Option<Value>)> = vec![(
        "import devOnly from 'dev-only';",
        Some(json!([{ "devDependencies": ["**/__tests__/**"] }])),
        None,
    )];
    let glob_fail: Vec<(&str, Option<Value>, Option<Value>)> = vec![(
        "import devOnly from 'dev-only';",
        Some(json!([{ "devDependencies": ["src/**"] }])),
        None,
    )];

    Tester::new(
        NoExtraneousDependencies::NAME,
        NoExtraneousDependencies::PLUGIN,
        glob_pass,
        glob_fail,
    )
    .change_rule_path("no_extraneous_dependencies/__tests__/example.test.ts")
    .with_snapshot_suffix("_glob")
    .with_import_plugin(true)
    .test_and_snapshot();
}
