use std::{collections::HashSet, ffi::OsStr};

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

const LEGACY_ORDER: &[&str] = &[
    "name",
    "version",
    "private",
    "publishConfig",
    "description",
    "main",
    "exports",
    "browser",
    "files",
    "bin",
    "directories",
    "man",
    "scripts",
    "repository",
    "keywords",
    "author",
    "license",
    "bugs",
    "homepage",
    "config",
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
    "bundledDependencies",
    "engines",
    "os",
    "cpu",
];

const SORT_PACKAGE_JSON_ORDER: &[&str] = &[
    "$schema",
    "name",
    "displayName",
    "version",
    "stableVersion",
    "private",
    "description",
    "categories",
    "keywords",
    "homepage",
    "bugs",
    "repository",
    "funding",
    "license",
    "qna",
    "author",
    "maintainers",
    "contributors",
    "publisher",
    "sideEffects",
    "type",
    "imports",
    "exports",
    "main",
    "svelte",
    "umd:main",
    "jsdelivr",
    "unpkg",
    "module",
    "source",
    "jsnext:main",
    "browser",
    "react-native",
    "types",
    "typesVersions",
    "typings",
    "style",
    "example",
    "examplestyle",
    "assets",
    "bin",
    "man",
    "directories",
    "files",
    "workspaces",
    "binary",
    "scripts",
    "betterScripts",
    "l10n",
    "contributes",
    "activationEvents",
    "husky",
    "simple-git-hooks",
    "pre-commit",
    "commitlint",
    "lint-staged",
    "nano-staged",
    "config",
    "nodemonConfig",
    "browserify",
    "babel",
    "browserslist",
    "xo",
    "prettier",
    "eslintConfig",
    "eslintIgnore",
    "npmpkgjsonlint",
    "npmPackageJsonLintConfig",
    "npmpackagejsonlint",
    "release",
    "remarkConfig",
    "stylelint",
    "ava",
    "jest",
    "jest-junit",
    "jest-stare",
    "mocha",
    "nyc",
    "c8",
    "tap",
    "oclif",
    "resolutions",
    "overrides",
    "dependencies",
    "devDependencies",
    "dependenciesMeta",
    "peerDependencies",
    "peerDependenciesMeta",
    "optionalDependencies",
    "bundledDependencies",
    "bundleDependencies",
    "extensionPack",
    "extensionDependencies",
    "flat",
    "packageManager",
    "engines",
    "engineStrict",
    "devEngines",
    "volta",
    "languageName",
    "os",
    "cpu",
    "preferGlobal",
    "publishConfig",
    "icon",
    "badges",
    "galleryBanner",
    "preview",
    "markdown",
    "pnpm",
];

fn incorrect_order_diagnostic(property: &str, span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Top-level property `{property}` is not ordered in the standard way."
    ))
    .with_help("Reorder top-level package.json properties consistently.")
    .with_label(span)
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PackageJsonPropertyOrderPreset {
    Legacy,
    #[default]
    SortPackageJson,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum PackageJsonPropertyOrder {
    Preset(PackageJsonPropertyOrderPreset),
    Custom(Vec<String>),
}

impl Default for PackageJsonPropertyOrder {
    fn default() -> Self {
        Self::Preset(PackageJsonPropertyOrderPreset::SortPackageJson)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PackageJsonOrderPropertiesConfig {
    order: PackageJsonPropertyOrder,
}

impl Default for PackageJsonOrderPropertiesConfig {
    fn default() -> Self {
        Self { order: PackageJsonPropertyOrder::default() }
    }
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonOrderProperties(Box<PackageJsonOrderPropertiesConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a conventional order for top-level `package.json` properties.
    ///
    /// ### Why is this bad?
    ///
    /// Consistent top-level package metadata order improves readability and
    /// reduces noisy diffs across many packages.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "version": "1.0.0", "name": "demo" }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "name": "demo", "version": "1.0.0" }
    /// ```
    PackageJsonOrderProperties,
    oxc,
    style,
    fix,
    config = PackageJsonOrderPropertiesConfig
);

impl Rule for PackageJsonOrderProperties {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<PackageJsonOrderPropertiesConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let Ok(value) = serde_json::from_str::<Value>(source_text) else {
            return;
        };
        let Some(object) = value.as_object() else {
            return;
        };

        let ordered_keys = ordered_keys_for_config(object, &self.0.order);
        let current_keys = object.keys().map(String::as_str).collect::<Vec<_>>();
        let expected_keys = ordered_keys.iter().map(String::as_str).collect::<Vec<_>>();
        if current_keys == expected_keys {
            return;
        }

        let Some(first_mismatch_index) = current_keys
            .iter()
            .zip(expected_keys.iter())
            .position(|(current, expected)| current != expected)
        else {
            return;
        };

        let property = current_keys[first_mismatch_index];
        let Some(updated) = reorder_top_level_object(object, &ordered_keys) else {
            return;
        };
        let Some(expected) = serialize_updated_json(&updated, source_text) else {
            return;
        };

        let file_span = oxc_span::Span::new(0, source_text.len() as u32);
        ctx.diagnostic_with_fix(
            incorrect_order_diagnostic(property, oxc_span::Span::new(0, 1)),
            |fixer| fixer.replace_full_source_range(file_span, expected),
        );
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host()
            && ctx.file_extension().is_some_and(|ext| ext == OsStr::new("json"))
            && ctx.file_path().file_name().is_some_and(|name| name == "package.json")
    }
}

fn ordered_keys_for_config(
    object: &Map<String, Value>,
    order: &PackageJsonPropertyOrder,
) -> Vec<String> {
    let all_keys = object.keys().map(String::as_str).collect::<Vec<_>>();
    let explicit_order = match order {
        PackageJsonPropertyOrder::Preset(PackageJsonPropertyOrderPreset::Legacy) => {
            LEGACY_ORDER.iter().copied().map(str::to_string).collect::<Vec<_>>()
        }
        PackageJsonPropertyOrder::Preset(PackageJsonPropertyOrderPreset::SortPackageJson) => {
            SORT_PACKAGE_JSON_ORDER.iter().copied().map(str::to_string).collect::<Vec<_>>()
        }
        PackageJsonPropertyOrder::Custom(custom) => {
            let mut order = custom.clone();
            order.extend(SORT_PACKAGE_JSON_ORDER.iter().copied().map(str::to_string));
            order
        }
    };

    let mut seen = HashSet::new();
    let mut ordered_keys = explicit_order
        .into_iter()
        .filter(|key| seen.insert(key.clone()) && object.contains_key(key))
        .collect::<Vec<_>>();

    let mut remainder = all_keys.into_iter().filter(|key| !seen.contains(*key)).collect::<Vec<_>>();
    remainder.sort_unstable();

    ordered_keys.extend(remainder.into_iter().map(str::to_string));
    ordered_keys
}

fn reorder_top_level_object(object: &Map<String, Value>, ordered_keys: &[String]) -> Option<Value> {
    let mut reordered = Map::with_capacity(object.len());
    for key in ordered_keys {
        reordered.insert(key.clone(), object.get(key)?.clone());
    }
    Some(Value::Object(reordered))
}

fn serialize_updated_json(value: &Value, source_text: &str) -> Option<String> {
    let indent = vec![b' '; 2];
    let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent);
    let mut output = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut output, formatter);
    value.serialize(&mut serializer).ok()?;
    let mut formatted = String::from_utf8(output).ok()?;
    if source_text.ends_with('\n') {
        formatted.push('\n');
    }
    Some(formatted)
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r#"{"name":"demo","version":"1.0.0"}"#, None),
        (
            r#"{"name":"demo","version":"1.0.0","publishConfig":{},"description":"x"}"#,
            Some(json!([{ "order": "legacy" }])),
        ),
        (
            r#"{"name":"demo","version":"1.0.0","foo":"x","zzz":"y"}"#,
            Some(json!([{ "order": ["name", "version"] }])),
        ),
    ];

    let fail = vec![
        (r#"{"version":"1.0.0","name":"demo"}"#, None),
        (
            r#"{"description":"x","name":"demo","version":"1.0.0"}"#,
            Some(json!([{ "order": "legacy" }])),
        ),
        (
            r#"{"foo":"x","name":"demo","version":"1.0.0"}"#,
            Some(json!([{ "order": ["name", "version"] }])),
        ),
    ];

    let fix = vec![
        (
            r#"{"version":"1.0.0","name":"demo"}"#,
            "{\n  \"name\": \"demo\",\n  \"version\": \"1.0.0\"\n}",
            None,
        ),
        (
            r#"{"description":"x","name":"demo","version":"1.0.0"}"#,
            "{\n  \"name\": \"demo\",\n  \"version\": \"1.0.0\",\n  \"description\": \"x\"\n}",
            Some(json!([{ "order": "legacy" }])),
        ),
        (
            r#"{"foo":"x","name":"demo","version":"1.0.0"}"#,
            "{\n  \"name\": \"demo\",\n  \"version\": \"1.0.0\",\n  \"foo\": \"x\"\n}",
            Some(json!([{ "order": ["name", "version"] }])),
        ),
    ];

    Tester::new(PackageJsonOrderProperties::NAME, PackageJsonOrderProperties::PLUGIN, pass, fail)
        .expect_fix(fix)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
