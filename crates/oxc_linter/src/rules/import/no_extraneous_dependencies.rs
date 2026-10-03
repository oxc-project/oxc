use std::{
    cell::OnceCell,
    path::{Path, PathBuf},
    sync::Arc,
};

use cow_utils::CowUtils;
use nodejs_built_in_modules::is_nodejs_builtin_module;
use oxc_ast::{
    AstKind,
    ast::{Expression, ImportDeclarationSpecifier},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_resolver::PathUtil;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    context::{ImportKind, LintContext, PackageJsonError},
    rule::{DefaultRuleConfig, Rule},
    utils::glob_match_with_extglobs,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum DependencyAllowance {
    /// Allow or forbid this dependency category in every file.
    Boolean(bool),
    /// Allow this dependency category only in files matching one of these globs.
    Globs(Vec<String>),
}

impl Default for DependencyAllowance {
    fn default() -> Self {
        Self::Boolean(true)
    }
}

impl DependencyAllowance {
    fn allows(&self, file: &Path, cwd: &Path) -> bool {
        match self {
            Self::Boolean(allow) => *allow,
            Self::Globs(globs) => {
                let filename = file.to_string_lossy();
                let filename = filename.cow_replace('\\', "/");
                globs.iter().any(|glob| {
                    glob_match_with_extglobs(glob, &filename)
                        || glob_match_with_extglobs(
                            cwd.join(glob)
                                .normalize()
                                .to_string_lossy()
                                .cow_replace('\\', "/")
                                .as_ref(),
                            &filename,
                        )
                })
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum PackageDirectories {
    /// Directory containing the package manifest.
    Single(PathBuf),
    /// Directories whose dependency declarations are combined.
    Multiple(Vec<PathBuf>),
}

impl Default for PackageDirectories {
    fn default() -> Self {
        Self::Multiple(Vec::new())
    }
}

impl PackageDirectories {
    fn as_slice(&self) -> &[PathBuf] {
        match self {
            Self::Single(path) => std::slice::from_ref(path),
            Self::Multiple(paths) => paths,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoExtraneousDependenciesConfig {
    /// Allow development dependencies. A glob array allows them only in matching files.
    dev_dependencies: DependencyAllowance,
    /// Allow optional dependencies. A glob array allows them only in matching files.
    optional_dependencies: DependencyAllowance,
    /// Allow peer dependencies. A glob array allows them only in matching files.
    peer_dependencies: DependencyAllowance,
    /// Allow bundled dependencies. A glob array allows them only in matching files.
    bundled_dependencies: DependencyAllowance,
    /// Directories containing package.json files. Relative paths are resolved from the working directory.
    /// By default, use the closest package.json above the linted file.
    package_dir: PackageDirectories,
    /// Also check imports resolved to internal modules. Relative imports are still ignored.
    include_internal: bool,
    /// Also check type-only imports and exports.
    include_types: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NoExtraneousDependencies(Box<NoExtraneousDependenciesConfig>);

impl std::ops::Deref for NoExtraneousDependencies {
    type Target = NoExtraneousDependenciesConfig;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbids importing packages that are not declared in package.json.
    /// Checks imports, re-exports, dynamic imports, and CommonJS require calls.
    /// Packages must resolve successfully to be checked. Node.js builtins and relative
    /// imports are ignored. Type-only imports are ignored unless `includeTypes` is enabled.
    ///
    /// By default, all dependency categories are allowed and the closest package.json
    /// is used. Use `packageDir` to select one or more manifests, and set
    /// `devDependencies`, `optionalDependencies`, `peerDependencies`, or
    /// `bundledDependencies` to false or an array of allowed filename globs.
    ///
    /// ### Why is this bad?
    ///
    /// Undeclared dependencies can disappear after a clean install or a dependency update.
    /// Depending on development packages in production code can also break deployments.
    ///
    /// ### Examples
    ///
    /// Given a package.json with `react` in dependencies and `vitest` in devDependencies,
    /// and the option `{ "devDependencies": false }`:
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import { test } from 'vitest';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import React from 'react';
    /// import fs from 'node:fs';
    /// ```
    NoExtraneousDependencies,
    import,
    nursery,
    config = NoExtraneousDependenciesConfig,
    version = "next",
    short_description = "Forbid importing undeclared dependencies.",
);

#[derive(Default)]
struct DependencyCategories<T> {
    production: T,
    development: T,
    optional: T,
    peer: T,
    bundled: T,
}

impl DependencyCategories<bool> {
    fn is_allowed(
        &self,
        rule: &NoExtraneousDependencies,
        file: &Path,
        cwd: &Path,
        allowances: &DependencyCategories<OnceCell<bool>>,
    ) -> bool {
        self.production
            || (self.development
                && *allowances.development.get_or_init(|| rule.dev_dependencies.allows(file, cwd)))
            || (self.optional
                && *allowances
                    .optional
                    .get_or_init(|| rule.optional_dependencies.allows(file, cwd)))
            || (self.peer
                && *allowances.peer.get_or_init(|| rule.peer_dependencies.allows(file, cwd)))
            || (self.bundled
                && *allowances.bundled.get_or_init(|| rule.bundled_dependencies.allows(file, cwd)))
    }
}

#[derive(Default)]
struct Dependencies(Vec<Arc<Value>>);

impl Dependencies {
    fn declaration(&self, name: &str, declared: &mut DependencyCategories<bool>) {
        for end in name.match_indices('/').map(|(i, _)| i).chain(std::iter::once(name.len())) {
            let ancestor = &name[..end];
            if ancestor.starts_with('@') && !ancestor.contains('/') {
                continue;
            }
            for package in &self.0 {
                for (field, declaration) in [
                    ("dependencies", &mut declared.production),
                    ("devDependencies", &mut declared.development),
                    ("optionalDependencies", &mut declared.optional),
                    ("peerDependencies", &mut declared.peer),
                ] {
                    *declaration |= package
                        .get(field)
                        .and_then(Value::as_object)
                        .is_some_and(|dependencies| dependencies.contains_key(ancestor));
                }
                declared.bundled |= match package
                    .get("bundleDependencies")
                    .or_else(|| package.get("bundledDependencies"))
                {
                    Some(Value::Object(map)) => map.contains_key(ancestor),
                    Some(Value::Array(names)) => {
                        names.iter().any(|name| name.as_str() == Some(ancestor))
                    }
                    _ => false,
                };
            }
        }
    }
}

fn nearest_manifest(file: &Path) -> Option<PathBuf> {
    file.parent()?.ancestors().map(|dir| dir.join("package.json")).find(|path| path.is_file())
}

fn package_name(specifier: &str) -> &str {
    let mut parts = specifier.split('/');
    let first = parts.next().unwrap_or_default();
    if first.starts_with('@') {
        parts.next().map_or(first, |second| &specifier[..first.len() + 1 + second.len()])
    } else {
        first
    }
}

fn import_source(
    kind: AstKind<'_>,
    include_types: bool,
    mut include_name: impl FnMut(&str) -> bool,
) -> Option<(&str, Span, ImportKind)> {
    let (name, span, import_kind) = match kind {
        AstKind::ImportDeclaration(import) => {
            if import.import_kind.is_type() && !include_types {
                return None;
            }
            (import.source.value.as_str(), import.source.span, ImportKind::Import)
        }
        AstKind::ExportFromDeclaration(export) => {
            if export.export_kind.is_type() && !include_types {
                return None;
            }
            (export.source.value.as_str(), export.source.span, ImportKind::Import)
        }
        AstKind::ExportAllDeclaration(export) => {
            if export.export_kind.is_type() && !include_types {
                return None;
            }
            (export.source.value.as_str(), export.source.span, ImportKind::Import)
        }
        AstKind::ImportExpression(import) => {
            let Expression::StringLiteral(source) = &import.source else {
                return None;
            };
            (source.value.as_str(), source.span, ImportKind::Import)
        }
        AstKind::CallExpression(call)
            if call.arguments.len() == 1
                && matches!(&call.callee, Expression::Identifier(id) if id.name == "require") =>
        {
            let argument = call.arguments[0].as_expression()?;
            let name = match argument {
                Expression::StringLiteral(source) => source.value.as_str(),
                Expression::TemplateLiteral(template) if template.is_no_substitution_template() => {
                    template.quasis[0].value.cooked.as_ref()?.as_str()
                }
                _ => return None,
            };
            (name, argument.span(), ImportKind::Require)
        }
        _ => return None,
    };
    if !include_name(name) {
        return None;
    }
    let type_only = match kind {
        AstKind::ImportDeclaration(import) => {
            import.import_kind.is_type()
                || import.specifiers.as_ref().is_some_and(|specifiers| {
                    !specifiers.is_empty() && specifiers.iter().all(|specifier| {
                        matches!(specifier, ImportDeclarationSpecifier::ImportSpecifier(specifier)
                            if specifier.import_kind.is_type())
                    })
                })
        }
        AstKind::ExportFromDeclaration(export) => {
            export.export_kind.is_type()
                || (!export.specifiers.is_empty()
                    && export.specifiers.iter().all(|specifier| specifier.export_kind.is_type()))
        }
        AstKind::ExportAllDeclaration(export) => export.export_kind.is_type(),
        _ => false,
    };
    if type_only && !include_types {
        return None;
    }
    let import_kind = if type_only { ImportKind::Type } else { import_kind };
    Some((name, span, import_kind))
}

fn is_external(path: &Path, package_root: &Path, name: &str, folders: Option<&[Value]>) -> bool {
    let matches_folder = |folder: &Path| {
        path.starts_with(package_root.join(folder))
            || (!path.starts_with(package_root)
                && !folder.is_absolute()
                && path.ancestors().any(|ancestor| ancestor.ends_with(folder)))
            || package_root.ancestors().any(|ancestor| ancestor.join(folder).join(name).exists())
    };
    match folders {
        Some(folders) => {
            folders.iter().filter_map(Value::as_str).map(Path::new).any(matches_folder)
        }
        None => matches_folder(Path::new("node_modules")),
    }
}

impl Rule for NoExtraneousDependencies {
    fn from_configuration(value: Value) -> Result<Self, serde_json::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let Some(import_context) = ctx.imports() else {
            return;
        };
        let settings = ctx.settings().json.as_ref();
        let internal_pattern =
            settings.and_then(|s| s.get("import/internal-regex")).and_then(Value::as_str);
        let internal_regex = OnceCell::new();
        let core_modules =
            settings.and_then(|s| s.get("import/core-modules")).and_then(Value::as_array);
        let mut imports = ctx
            .nodes()
            .iter()
            .filter_map(|node| {
                import_source(node.kind(), self.include_types, |name| {
                    let marked_internal = internal_regex
                        .get_or_init(|| {
                            internal_pattern
                                .and_then(|pattern| lazy_regex::Regex::new(pattern).ok())
                        })
                        .as_ref()
                        .is_some_and(|regex| regex.is_match(name));
                    if marked_internal {
                        return self.include_internal;
                    }
                    !(name.is_empty()
                        || name.starts_with('.')
                        || Path::new(name).is_absolute()
                        || name.starts_with("node:")
                        || is_nodejs_builtin_module(name)
                        || core_modules.is_some_and(|modules| {
                            modules.iter().any(|module| module.as_str() == Some(package_name(name)))
                        }))
                })
            })
            .peekable();
        let directories = self.package_dir.as_slice();
        if imports.peek().is_none() && directories.len() != 1 {
            return;
        }
        let Ok(cwd) = std::env::current_dir() else {
            return;
        };
        let file = if ctx.file_path().is_absolute() {
            ctx.file_path().to_path_buf()
        } else {
            cwd.join(ctx.file_path())
        };
        let nearest = OnceCell::new();
        let manifests = if directories.is_empty() {
            nearest.get_or_init(|| nearest_manifest(&file)).iter().cloned().collect::<Vec<_>>()
        } else {
            directories.iter().map(|dir| cwd.join(dir).join("package.json")).collect()
        };
        if manifests.is_empty() {
            return;
        }
        let mut dependencies = Dependencies::default();
        for manifest in &manifests {
            match import_context.package_json(manifest) {
                Ok(package) => dependencies.0.push(package),
                Err(error) if manifests.len() == 1 && !directories.is_empty() => {
                    let diagnostic = match error {
                        PackageJsonError::Read => OxcDiagnostic::warn(
                            "The package.json file could not be read.",
                        )
                        .with_help(
                            "Set packageDir to a directory containing a readable package.json.",
                        ),
                        PackageJsonError::Parse(error) => OxcDiagnostic::warn(format!(
                            "The package.json file could not be parsed: {error}"
                        ))
                        .with_help("Correct the JSON in the configured packageDir manifest."),
                    };
                    ctx.diagnostic(diagnostic.with_label(Span::empty(0)));
                    return;
                }
                Err(_) => {}
            }
        }
        if imports.peek().is_none() {
            return;
        }
        let allowances = DependencyCategories::default();
        let external_folders = settings
            .and_then(|s| s.get("import/external-module-folders"))
            .and_then(Value::as_array)
            .map(Vec::as_slice);
        for (name, span, import_kind) in imports {
            let original_name = package_name(name);
            let mut declared = DependencyCategories::default();
            dependencies.declaration(original_name, &mut declared);
            if declared.is_allowed(self, &file, &cwd, &allowances) {
                continue;
            }
            let Ok(resolved) = import_context.resolver(import_kind).resolve_file(&file, name)
            else {
                continue;
            };
            if !self.include_internal {
                let package_root = nearest
                    .get_or_init(|| nearest_manifest(&file))
                    .as_deref()
                    .and_then(Path::parent)
                    .unwrap_or(&cwd);
                if !is_external(resolved.path(), package_root, original_name, external_folders) {
                    continue;
                }
            }
            let real_name =
                resolved.package_json().and_then(|package| package.name()).unwrap_or(original_name);
            dependencies.declaration(real_name, &mut declared);
            if declared.is_allowed(self, &file, &cwd, &allowances) {
                continue;
            }
            let category = if declared.development {
                Some("devDependencies")
            } else if declared.optional {
                Some("optionalDependencies")
            } else {
                None
            };
            let message = category.map_or_else(
                || format!("Package `{real_name}` is not declared in dependencies."),
                |category| format!("Package `{real_name}` should be listed in dependencies instead of {category}."),
            );
            ctx.diagnostic(OxcDiagnostic::warn(message)
                .with_help(format!("Add `{real_name}` to dependencies in package.json or allow its dependency category in this rule's options."))
                .with_label(span));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoExtraneousDependencies;
    use crate::{rule::RuleMeta, tester::Tester};
    use serde_json::json;

    const FILE: &str = "no-extraneous-dependencies/index.ts";

    #[test]
    fn static_require_templates() {
        Tester::new(
            NoExtraneousDependencies::NAME,
            NoExtraneousDependencies::PLUGIN,
            vec!["require(`acorn`);", "require(`not-${name}`);", "require(`not-a-dependency`, 1);"],
            vec!["require(`not-a-dependency`);", r"require(`not-a-\u0064ependency`);"],
        )
        .with_import_plugin(true)
        .change_rule_path(FILE)
        .test();
    }

    #[test]
    fn builtin_named_packages() {
        Tester::new(
            NoExtraneousDependencies::NAME,
            NoExtraneousDependencies::PLUGIN,
            vec!["import 'buffer'; require('events'); import 'fs/promises'; import 'node:buffer';"],
            vec!["import 'buffer/';", "require('events/');", "require(`buffer/`);"],
        )
        .with_import_plugin(true)
        .change_rule_path(FILE)
        .test();
    }

    #[test]
    fn sibling_aliases() {
        Tester::new(
            NoExtraneousDependencies::NAME,
            NoExtraneousDependencies::PLUGIN,
            vec![("import 'internal/local';", None)],
            vec![("import 'internal/local';", Some(json!([{ "includeInternal": true }])))],
        )
        .with_import_plugin(true)
        .change_rule_path("no-extraneous-dependencies/nested/index.ts")
        .test();
    }

    #[test]
    fn node_exports_condition() {
        Tester::new(
            NoExtraneousDependencies::NAME,
            NoExtraneousDependencies::PLUGIN,
            vec![],
            vec!["import 'node-only';", "require('node-only');", "import('node-only');"],
        )
        .with_import_plugin(true)
        .change_rule_path(FILE)
        .test();
    }

    #[test]
    fn test() {
        let pass = vec![
            ("import 'acorn';", None),
            ("import 'chai';", None),
            ("import 'left-pad';", None),
            ("import 'react';", None),
            ("import '@generated/foo';", None),
            ("import 'rxjs/operators';", None),
            ("import 'esm-package/esm-module';", None),
            ("import 'fs'; import 'node:fs'; import 'fs/promises';", None),
            ("import './local'; import '../foo';", Some(json!([{ "includeInternal": true }]))),
            ("import 'package-that-is-not-installed';", None),
            ("const acorn = require('acorn');", None),
            ("export * from 'acorn';", None),
            ("export { parse } from 'acorn';", None),
            ("import('acorn');", None),
            ("require(6); require(name); require.resolve('not-a-dependency');", None),
            ("import(`not-a-dependency`); import(name);", None),
            ("import type T from 'not-a-dependency';", None),
            ("import { type T, type U } from 'not-a-dependency';", None),
            ("export type { T } from 'not-a-dependency';", None),
            ("export type * from 'not-a-dependency';", None),
            ("import type T from 'acorn';", Some(json!([{ "includeTypes": true }]))),
            ("import 'alias-acorn';", None),
            ("import 'internal/local';", None),
        ];
        let fail = vec![
            ("import 'not-a-dependency';", None),
            ("import value from 'not-a-dependency';", None),
            ("const value = require('not-a-dependency');", None),
            ("export * from 'not-a-dependency';", None),
            ("export { value } from 'not-a-dependency';", None),
            ("import('not-a-dependency');", None),
            ("import '@org/not-a-dependency/foo';", None),
            ("import 'chai';", Some(json!([{ "devDependencies": false }]))),
            ("import 'left-pad';", Some(json!([{ "optionalDependencies": false }]))),
            ("import 'react';", Some(json!([{ "peerDependencies": false }]))),
            ("import '@generated/foo';", Some(json!([{ "bundledDependencies": false }]))),
            ("import type T from 'not-a-dependency';", Some(json!([{ "includeTypes": true }]))),
            ("import { type T } from 'not-a-dependency';", Some(json!([{ "includeTypes": true }]))),
            ("import { type T, value } from 'not-a-dependency';", None),
            ("export type { T } from 'not-a-dependency';", Some(json!([{ "includeTypes": true }]))),
            ("export type * from 'not-a-dependency';", Some(json!([{ "includeTypes": true }]))),
            ("import 'alias-missing';", None),
            ("import 'internal/local';", Some(json!([{ "includeInternal": true }]))),
        ];
        Tester::new(NoExtraneousDependencies::NAME, NoExtraneousDependencies::PLUGIN, pass, fail)
            .with_import_plugin(true)
            .change_rule_path(FILE)
            .test_and_snapshot();
    }

    #[test]
    fn resolution() {
        let pass = vec![
            ("require('require-only');", None),
            ("import 'require-only';", Some(json!([{ "devDependencies": false }]))),
        ];
        let fail = vec![
            ("require('require-only');", Some(json!([{ "devDependencies": false }]))),
            ("import 'asset-package/style.css';", None),
            ("require('asset-package/style.css');", None),
        ];
        Tester::new(NoExtraneousDependencies::NAME, NoExtraneousDependencies::PLUGIN, pass, fail)
            .with_import_plugin(true)
            .change_rule_path(FILE)
            .test();
        Tester::new(
            NoExtraneousDependencies::NAME,
            NoExtraneousDependencies::PLUGIN,
            vec!["import 'workspace-alias';"],
            vec!["import 'alias-missing';"],
        )
        .with_import_plugin(true)
        .change_rule_path("no-extraneous-dependencies/nested/index.ts")
        .test();
    }

    #[test]
    fn type_only_exports() {
        let pass = vec![
            ("export { type T } from 'not-a-dependency';", None),
            ("export { type T, type U } from 'chai';", Some(json!([{ "devDependencies": false }]))),
        ];
        let fail = vec![
            (
                "export { type T } from 'chai';",
                Some(json!([{ "devDependencies": false, "includeTypes": true }])),
            ),
            ("export { type T, value } from 'chai';", Some(json!([{ "devDependencies": false }]))),
            ("export {} from 'chai';", Some(json!([{ "devDependencies": false }]))),
        ];
        Tester::new(NoExtraneousDependencies::NAME, NoExtraneousDependencies::PLUGIN, pass, fail)
            .with_import_plugin(true)
            .change_rule_path(FILE)
            .test();
    }

    #[test]
    fn type_only_resolution() {
        let options = Some(json!([{ "devDependencies": false, "includeTypes": true }]));
        let pass = vec![
            ("import type { T } from 'types-only';", None),
            ("import type { T } from 'types-only';", Some(json!([{ "includeTypes": true }]))),
            ("import 'types-only';", options.clone()),
            ("import { type T, value } from 'types-only';", options.clone()),
            ("export { type T, value } from 'types-only';", options.clone()),
            ("require('types-only');", options.clone()),
            ("import('types-only');", options.clone()),
        ];
        let fail = vec![
            ("import type { T } from 'types-only';", options.clone()),
            ("import { type T } from 'types-only';", options.clone()),
            ("export type { T } from 'types-only';", options.clone()),
            ("export { type T } from 'types-only';", options.clone()),
            ("export type * from 'types-only';", options),
            (
                "import type { T } from 'types-only';",
                Some(json!([{
                    "includeTypes": true,
                    "packageDir": "fixtures/import/no-extraneous-dependencies/nested"
                }])),
            ),
        ];
        Tester::new(NoExtraneousDependencies::NAME, NoExtraneousDependencies::PLUGIN, pass, fail)
            .with_import_plugin(true)
            .change_rule_path(FILE)
            .test();
    }

    #[test]
    fn dependency_globs() {
        for (option, source) in [
            ("devDependencies", "import 'chai';"),
            ("optionalDependencies", "import 'left-pad';"),
            ("peerDependencies", "import 'react';"),
            ("bundledDependencies", "import '@generated/foo';"),
        ] {
            let pass = vec![
                (source, Some(json!([{ option: true }]))),
                (source, Some(json!([{ option: ["**/*.test.ts"] }]))),
                (source, Some(json!([{ option: ["**/*.+(test|spec).ts"] }]))),
                (source, Some(json!([{ option: ["**/*.@(test|spec).ts"] }]))),
                (source, Some(json!([{ option: ["**/*.?(spec)test.ts"] }]))),
                (source, Some(json!([{ option: ["**/*.!(spec).ts"] }]))),
                (source, Some(json!([{ option: ["**/*.*(test|spec).ts"] }]))),
                (source, Some(json!([{ option: ["**/*.spec.ts", "**/*.test.ts"] }]))),
                (
                    source,
                    Some(
                        json!([{ option: ["fixtures/import/no-extraneous-dependencies/*.test.ts"] }]),
                    ),
                ),
                (
                    source,
                    Some(
                        json!([{ option: ["./fixtures/import/no-extraneous-dependencies/*.test.ts"] }]),
                    ),
                ),
                (
                    source,
                    Some(
                        json!([{ option: ["fixtures/import/../import/no-extraneous-dependencies/*.test.ts"] }]),
                    ),
                ),
            ];
            let fail = vec![
                (source, Some(json!([{ option: false }]))),
                (source, Some(json!([{ option: [] }]))),
                (source, Some(json!([{ option: ["**/*.spec.ts"] }]))),
                (source, Some(json!([{ option: ["**/*.+(spec|e2e).ts"] }]))),
                (source, Some(json!([{ option: ["**/*.!(test).ts"] }]))),
            ];
            Tester::new(
                NoExtraneousDependencies::NAME,
                NoExtraneousDependencies::PLUGIN,
                pass,
                fail,
            )
            .with_import_plugin(true)
            .change_rule_path("no-extraneous-dependencies/index.test.ts")
            .test();
        }
    }

    #[test]
    fn dependency_globs_hidden_paths() {
        for (file, glob) in
            [(".hidden/input.test.ts", "**/.hidden/*.ts"), (".input.test.ts", "**/.input.test.ts")]
        {
            Tester::new(
                NoExtraneousDependencies::NAME,
                NoExtraneousDependencies::PLUGIN,
                vec![("import 'chai';", Some(json!([{ "devDependencies": [glob] }])))],
                vec![
                    ("import 'chai';", Some(json!([{ "devDependencies": ["**/*.ts"] }]))),
                    (
                        "import 'chai';",
                        Some(json!([{ "devDependencies": ["**/*.@(test|spec).ts"] }])),
                    ),
                ],
            )
            .with_import_plugin(true)
            .change_rule_path(&format!("no-extraneous-dependencies/{file}"))
            .test();
        }
    }

    #[test]
    fn dependency_globs_brace_ranges() {
        for file in ["test1.ts", "test2.ts", "test3.ts"] {
            Tester::new(
                NoExtraneousDependencies::NAME,
                NoExtraneousDependencies::PLUGIN,
                vec![(
                    "import 'chai';",
                    Some(json!([{ "devDependencies": ["**/test{1..3}.@(js|ts)"] }])),
                )],
                vec![(
                    "import 'chai';",
                    Some(json!([{ "devDependencies": ["**/test{4..6}.@(js|ts)"] }])),
                )],
            )
            .with_import_plugin(true)
            .change_rule_path(&format!("no-extraneous-dependencies/{file}"))
            .test();
        }
    }

    #[test]
    fn negated_dependency_globs() {
        for file in ["prod.ts", "product.ts"] {
            Tester::new(
                NoExtraneousDependencies::NAME,
                NoExtraneousDependencies::PLUGIN,
                vec![("import 'chai';", Some(json!([{ "devDependencies": ["**/!(test)*.ts"] }])))],
                vec![("import 'chai';", Some(json!([{ "devDependencies": ["**/!(prod)*.ts"] }])))],
            )
            .with_import_plugin(true)
            .change_rule_path(&format!("no-extraneous-dependencies/{file}"))
            .test();
        }
    }

    #[test]
    fn package_directories() {
        let root =
            std::env::current_dir().unwrap().join("fixtures/import/no-extraneous-dependencies");
        let nested = root.join("nested");
        let other = root.join("other");
        let missing = root.join("missing");
        let pass = vec![
            ("import 'react';", None),
            ("import 'acorn';", Some(json!([{ "packageDir": root }]))),
            (
                "import 'acorn';",
                Some(json!([{ "packageDir": "fixtures/import/no-extraneous-dependencies" }])),
            ),
            ("import 'react'; import 'acorn';", Some(json!([{ "packageDir": [root, nested] }]))),
            (
                "import 'not-a-dependency'; import '@generated/bar';",
                Some(json!([{ "packageDir": [root, other] }])),
            ),
            ("import 'acorn';", Some(json!([{ "packageDir": [missing, root] }]))),
            ("import 'acorn';", Some(json!([{ "packageDir": [root, missing] }]))),
            (
                "import '@generated/foo'; import '@generated/bar';",
                Some(json!([{ "packageDir": [root, other] }])),
            ),
            ("import 'react';", Some(json!([{ "packageDir": [] }]))),
        ];
        let fail = vec![
            ("import 'acorn';", None),
            ("import 'react';", Some(json!([{ "packageDir": root.join("empty") }]))),
            ("import 'react';", Some(json!([{ "packageDir": root, "peerDependencies": false }]))),
            ("import 'acorn';", Some(json!([{ "packageDir": nested }]))),
            ("import 'not-a-dependency';", Some(json!([{ "packageDir": [root, nested] }]))),
            (
                "import '@generated/bar';",
                Some(json!([{ "packageDir": [root, other], "bundledDependencies": false }])),
            ),
        ];
        Tester::new(NoExtraneousDependencies::NAME, NoExtraneousDependencies::PLUGIN, pass, fail)
            .with_import_plugin(true)
            .change_rule_path("no-extraneous-dependencies/nested/index.ts")
            .test();
    }

    #[test]
    fn package_errors() {
        let fail = vec![
            (
                "import './local';",
                Some(
                    json!([{ "packageDir": "fixtures/import/no-extraneous-dependencies/missing" }]),
                ),
            ),
            (
                "import './local';",
                Some(
                    json!([{ "packageDir": "fixtures/import/no-extraneous-dependencies/broken" }]),
                ),
            ),
        ];
        Tester::new(NoExtraneousDependencies::NAME, NoExtraneousDependencies::PLUGIN, vec![], fail)
            .with_import_plugin(true)
            .change_rule_path(FILE)
            .with_snapshot_suffix("package_errors")
            .test_and_snapshot();
    }

    #[test]
    fn import_settings() {
        let pass = vec![
            (
                "import { value } from './local';",
                None,
                Some(json!({ "settings": { "import/internal-regex": "^\\./" } })),
            ),
            (
                "import 'not-a-dependency';",
                None,
                Some(json!({ "settings": { "import/core-modules": ["not-a-dependency"] } })),
            ),
            (
                "import '@org/not-a-dependency/foo';",
                None,
                Some(json!({ "settings": { "import/core-modules": ["@org/not-a-dependency"] } })),
            ),
            (
                "import 'not-a-dependency';",
                None,
                Some(json!({ "settings": { "import/internal-regex": "^not-a-dependency$" } })),
            ),
            (
                "import 'internal/local';",
                None,
                Some(json!({ "settings": { "import/external-module-folders": [] } })),
            ),
        ];
        let fail = vec![
            (
                "import { value } from './local';",
                Some(json!([{ "includeInternal": true }])),
                Some(json!({ "settings": { "import/internal-regex": "^\\./" } })),
            ),
            (
                "import 'not-a-dependency';",
                Some(json!([{ "includeInternal": true }])),
                Some(json!({ "settings": { "import/internal-regex": "^not-a-dependency$" } })),
            ),
            (
                "import 'not-a-dependency';",
                None,
                Some(json!({ "settings": { "import/internal-regex": "^different$" } })),
            ),
            (
                "import 'internal/local';",
                None,
                Some(json!({ "settings": { "import/external-module-folders": ["."] } })),
            ),
        ];
        Tester::new(NoExtraneousDependencies::NAME, NoExtraneousDependencies::PLUGIN, pass, fail)
            .with_import_plugin(true)
            .change_rule_path(FILE)
            .test();
    }
}
