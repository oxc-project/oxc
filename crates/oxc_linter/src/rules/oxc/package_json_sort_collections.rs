use std::ffi::OsStr;

use rustc_hash::FxHashSet;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

const DEFAULT_COLLECTIONS: &[&str] = &[
    "config",
    "dependencies",
    "devDependencies",
    "exports",
    "optionalDependencies",
    "overrides",
    "peerDependencies",
    "peerDependenciesMeta",
    "scripts",
];

const DEFAULT_NPM_SCRIPTS: &[&str] = &[
    "install",
    "pack",
    "prepare",
    "publish",
    "restart",
    "shrinkwrap",
    "start",
    "stop",
    "test",
    "uninstall",
    "version",
];

fn unsorted_keys_diagnostic(key: &str, span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Entries in `{key}` are not in lexicographical order."))
        .with_help("Sort the collection consistently to reduce noisy diffs.")
        .with_label(span)
}

fn unsorted_scripts_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Entries in `scripts` are not in lifecycle-aware lexicographical order.")
        .with_help(
            "Group `pre<name>`, `<name>`, and `post<name>` scripts together in a stable order.",
        )
        .with_label(span)
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, JsonSchema)]
#[serde(transparent)]
pub struct PackageJsonSortCollectionsConfig(Vec<String>);

#[derive(Debug, Default, Clone)]
pub struct PackageJsonSortCollections(Box<PackageJsonSortCollectionsConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces deterministic ordering for selected `package.json` collections.
    ///
    /// ### Why is this bad?
    ///
    /// Manually edited dependency and script objects tend to drift out of
    /// order, which creates noisy follow-up diffs.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "dependencies": { "zod": "^1.0.0", "bun": "^1.0.0" } }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "dependencies": { "bun": "^1.0.0", "zod": "^1.0.0" } }
    /// ```
    PackageJsonSortCollections,
    oxc,
    style,
    fix,
    config = PackageJsonSortCollectionsConfig
);

impl Rule for PackageJsonSortCollections {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<PackageJsonSortCollectionsConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let Ok(value) = serde_json::from_str::<Value>(source_text) else {
            return;
        };
        let configured_paths = if self.0.0.is_empty() {
            DEFAULT_COLLECTIONS.iter().map(|path| (*path).to_string()).collect::<Vec<_>>()
        } else {
            self.0.0.clone()
        };

        let Some((updated, first_unsorted_path)) = sort_collections(&value, &configured_paths)
        else {
            return;
        };
        let Some(expected) = serialize_updated_json(&updated, source_text) else {
            return;
        };

        #[expect(clippy::cast_possible_truncation)]
        let file_span = oxc_span::Span::new(0, source_text.len() as u32);
        let diagnostic = if path_targets_scripts(&first_unsorted_path) {
            unsorted_scripts_diagnostic(oxc_span::Span::new(0, 1))
        } else {
            unsorted_keys_diagnostic(&first_unsorted_path, oxc_span::Span::new(0, 1))
        };

        ctx.diagnostic_with_fix(diagnostic, |fixer| {
            fixer.replace_full_source_range(file_span, expected)
        });
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host()
            && ctx.file_extension().is_some_and(|ext| ext == OsStr::new("json"))
            && ctx.file_path().file_name().is_some_and(|name| name == "package.json")
    }
}

fn sort_collections(value: &Value, configured_paths: &[String]) -> Option<(Value, String)> {
    let mut first_unsorted_path: Option<String> = None;
    let (updated, changed) =
        sort_collections_in_value(value, configured_paths, "", &mut first_unsorted_path)?;
    if !changed {
        return None;
    }
    first_unsorted_path.map(|path| (updated, path))
}

fn sort_collections_in_value(
    value: &Value,
    configured_paths: &[String],
    current_path: &str,
    first_unsorted_path: &mut Option<String>,
) -> Option<(Value, bool)> {
    match value {
        Value::Object(object) => {
            let mut changed = false;
            let mut updated = Map::with_capacity(object.len());

            for (key, child_value) in object {
                let path = join_path(current_path, key);
                let (mut next_value, child_changed) = sort_collections_in_value(
                    child_value,
                    configured_paths,
                    &path,
                    first_unsorted_path,
                )?;
                changed |= child_changed;

                if configured_paths.iter().any(|candidate| candidate == &path)
                    && let Value::Object(collection) = &next_value
                    && let Some(sorted_collection) = maybe_sort_collection(&path, collection)
                {
                    next_value = Value::Object(sorted_collection);
                    changed = true;
                    if first_unsorted_path.is_none() {
                        *first_unsorted_path = Some(path.clone());
                    }
                }

                updated.insert(key.clone(), next_value);
            }

            if changed {
                Some((Value::Object(updated), true))
            } else {
                Some((value.clone(), false))
            }
        }
        _ => Some((value.clone(), false)),
    }
}

fn maybe_sort_collection(path: &str, object: &Map<String, Value>) -> Option<Map<String, Value>> {
    let current_keys = object.keys().cloned().collect::<Vec<_>>();
    let expected_keys = if path_targets_scripts(path) {
        sort_script_names(&current_keys)
    } else {
        let mut keys = current_keys.clone();
        keys.sort_unstable();
        keys
    };

    if current_keys == expected_keys {
        return None;
    }

    let mut sorted = Map::with_capacity(object.len());
    for key in expected_keys {
        sorted.insert(key.clone(), object.get(&key)?.clone());
    }
    Some(sorted)
}

fn path_targets_scripts(path: &str) -> bool {
    path.split('.').next_back().is_some_and(|segment| segment == "scripts")
}

fn join_path(parent: &str, key: &str) -> String {
    if parent.is_empty() { key.to_string() } else { format!("{parent}.{key}") }
}

fn sort_script_names(keys: &[String]) -> Vec<String> {
    let default_scripts = DEFAULT_NPM_SCRIPTS.iter().copied().collect::<FxHashSet<_>>();
    let original_keys = keys.iter().cloned().collect::<FxHashSet<_>>();
    let mut transformed = Vec::with_capacity(keys.len());
    let mut prefixable = FxHashSet::<String>::default();

    for key in keys {
        let omitted: String =
            key.strip_prefix("pre").or_else(|| key.strip_prefix("post")).unwrap_or(key).to_string();

        if default_scripts.contains(omitted.as_str()) || original_keys.contains(&omitted) {
            prefixable.insert(omitted.clone());
            transformed.push(omitted);
        } else {
            transformed.push(key.clone());
        }
    }

    let names = sort_script_groups(transformed);
    let mut expanded = Vec::new();
    let mut seen = FxHashSet::<String>::default();
    for key in names {
        if prefixable.contains(&key) {
            for candidate in [format!("pre{key}"), key.clone(), format!("post{key}")] {
                if original_keys.contains(&candidate) && seen.insert(candidate.clone()) {
                    expanded.push(candidate);
                }
            }
        } else if original_keys.contains(&key) && seen.insert(key.clone()) {
            expanded.push(key);
        }
    }

    expanded
}

fn sort_script_groups(keys: Vec<String>) -> Vec<String> {
    let mut group_map = std::collections::BTreeMap::<String, Vec<String>>::new();

    for key in keys {
        let base = key.split_once(':').map_or_else(|| key.clone(), |(head, _)| head.to_string());
        group_map.entry(base).or_default().push(key);
    }

    let mut result = Vec::new();
    for (_group, mut children) in group_map {
        let has_nested = children.iter().any(|key| key.contains(':'));
        if has_nested && children.len() > 1 {
            let mut direct =
                children.iter().filter(|key| !key.contains(':')).cloned().collect::<Vec<_>>();
            direct.sort_unstable();

            let nested = children.into_iter().filter(|key| key.contains(':')).collect::<Vec<_>>();

            result.extend(direct);
            result.extend(sort_script_groups(nested));
        } else {
            children.sort_unstable();
            result.extend(children);
        }
    }

    result
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
        (r#"{"dependencies":{"a":"1","b":"1"}}"#, None),
        (r#"{"scripts":{"prebuild":"x","build":"x","postbuild":"x","lint":"x"}}"#, None),
        (
            r#"{"pnpm":{"patchedDependencies":{"a":"patch","b":"patch"}}}"#,
            Some(json!([["pnpm.patchedDependencies"]])),
        ),
    ];

    let fail = vec![
        (r#"{"dependencies":{"b":"1","a":"1"}}"#, None),
        (r#"{"scripts":{"build":"x","postbuild":"x","prebuild":"x"}}"#, None),
        (
            r#"{"pnpm":{"patchedDependencies":{"b":"patch","a":"patch"}}}"#,
            Some(json!([["pnpm.patchedDependencies"]])),
        ),
    ];

    let fix = vec![
        (
            r#"{"dependencies":{"b":"1","a":"1"}}"#,
            "{\n  \"dependencies\": {\n    \"a\": \"1\",\n    \"b\": \"1\"\n  }\n}",
            None,
        ),
        (
            r#"{"scripts":{"build":"x","postbuild":"x","prebuild":"x"}}"#,
            "{\n  \"scripts\": {\n    \"prebuild\": \"x\",\n    \"build\": \"x\",\n    \"postbuild\": \"x\"\n  }\n}",
            None,
        ),
        (
            r#"{"pnpm":{"patchedDependencies":{"b":"patch","a":"patch"}}}"#,
            "{\n  \"pnpm\": {\n    \"patchedDependencies\": {\n      \"a\": \"patch\",\n      \"b\": \"patch\"\n    }\n  }\n}",
            Some(json!([["pnpm.patchedDependencies"]])),
        ),
    ];

    Tester::new(PackageJsonSortCollections::NAME, PackageJsonSortCollections::PLUGIN, pass, fail)
        .expect_fix(fix)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
