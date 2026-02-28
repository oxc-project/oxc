use std::{
    fs,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use oxc_resolver::Resolver;
use oxc_span::CompactStr;
use rustc_hash::FxHashSet;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct TsconfigPathAlias {
    key_prefix: String,
    key_suffix: String,
    target_prefix: PathBuf,
    target_suffix: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct TsconfigPathAliases {
    aliases: Vec<TsconfigPathAlias>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TsconfigJson {
    #[serde(default)]
    compiler_options: TsconfigCompilerOptions,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TsconfigCompilerOptions {
    base_url: Option<PathBuf>,
    #[serde(default)]
    paths: serde_json::Map<String, serde_json::Value>,
}

/// Loads path aliases from a tsconfig.json for the prefer-shortest-imports rule.
///
/// Parses `compilerOptions.paths` and `baseUrl` into [`TsconfigPathAliases`], which
/// [`candidates_for_target`] uses to generate alias specifiers (e.g. `@/utils/helper`)
/// from resolved paths.
///
/// Returns `None` if the file can't be read, parsed, or has no parent directory.
/// Skips non-array path values and non-string target entries.
pub fn load_tsconfig_path_aliases(tsconfig_path: &Path) -> Option<TsconfigPathAliases> {
    let json = fs::read_to_string(tsconfig_path).ok()?;
    let mut bytes = json.into_bytes();
    _ = json_strip_comments::strip_slice(&mut bytes); // tsconfig allows JSONC comments
    let parsed: TsconfigJson = serde_json::from_slice(&bytes).ok()?;
    let tsconfig_dir = tsconfig_path.parent()?;
    let paths_base = parsed
        .compiler_options
        .base_url
        .as_ref()
        .map_or_else(|| tsconfig_dir.to_path_buf(), |base| tsconfig_dir.join(base));

    let mut aliases = Vec::new();
    for (key_pattern, raw_targets) in parsed.compiler_options.paths {
        let Some(targets) = raw_targets.as_array() else {
            continue; // paths values must be arrays, e.g. ["src/*"]
        };
        let (key_prefix, key_suffix) = split_wildcard(&key_pattern);
        for target in targets {
            let Some(target_pattern) = target.as_str() else {
                continue; // each target must be a string
            };
            let (target_prefix, target_suffix) = split_wildcard(target_pattern);
            aliases.push(TsconfigPathAlias {
                key_prefix: key_prefix.to_string(),
                key_suffix: key_suffix.to_string(),
                target_prefix: paths_base.join(target_prefix), // resolve relative to baseUrl
                target_suffix: PathBuf::from(target_suffix),
            });
        }
    }

    Some(TsconfigPathAliases { aliases })
}

/// Computes a relative import specifier from one directory to a target path.
///
/// Used by the runtime to generate relative candidates (e.g. `../../utils/helper`)
/// when comparing with path aliases for the prefer-shortest-imports rule.
///
/// Returns `None` if there is no common path prefix or the result would be empty.
/// Produces `./`-prefixed specifiers for same-directory targets.
pub fn relative_import_specifier(from_dir: &Path, target: &Path) -> Option<String> {
    let from_components: Vec<Component<'_>> = from_dir.components().collect();
    let target_components: Vec<Component<'_>> = target.components().collect();
    let shared =
        from_components.iter().zip(target_components.iter()).take_while(|(a, b)| a == b).count();

    if shared == 0 {
        return None;
    }

    let mut relative_parts = Vec::new();
    for component in &from_components[shared..] {
        if matches!(component, Component::Normal(_)) {
            relative_parts.push("..".to_string()); // one .. per directory above common prefix
        }
    }
    for component in &target_components[shared..] {
        if let Component::Normal(value) = component {
            relative_parts.push(value.to_string_lossy().to_string());
        }
    }

    if relative_parts.is_empty() {
        return None;
    }

    let mut specifier = relative_parts.join("/");
    if !specifier.starts_with("../") {
        specifier.insert_str(0, "./"); // same dir or below: ./foo not foo
    }
    Some(specifier)
}

/// Produces alternative forms of an import specifier for shortest-match comparison.
///
/// Returns the specifier plus variants without extension, without `/index`, and without
/// `index.<ext>`. Used by the runtime so `@/utils/helper` can match `@/utils/helper.ts`
/// and vice versa when resolving preferred specifiers.
pub fn candidate_variants(specifier: &str) -> Vec<String> {
    let normalized = specifier.replace('\\', "/");
    let mut variants = Vec::new();
    variants.push(normalized.clone());
    if let Some(without_extension) = strip_last_extension(&normalized) {
        variants.push(without_extension); // e.g. ./helper.ts → ./helper
    }
    if let Some(without_index) = normalized.strip_suffix("/index") {
        variants.push(without_index.to_string()); // e.g. @/utils/index → @/utils
    }
    if let Some(without_index_extension) = normalized
        .rsplit_once('/')
        .and_then(|(parent, tail)| tail.strip_prefix("index.").map(|_| parent.to_string()))
    {
        variants.push(without_index_extension); // e.g. @/utils/index.ts → @/utils
    }

    let mut seen = FxHashSet::default();
    variants
        .into_iter()
        .filter(|variant| !variant.is_empty())
        .filter_map(|variant| seen.insert(variant.clone()).then_some(variant))
        .collect()
}

impl TsconfigPathAliases {
    pub fn is_empty(&self) -> bool {
        self.aliases.is_empty()
    }

    /// Generates alias specifiers that resolve to the given target path.
    ///
    /// For each matching alias (e.g. `@/*` → `src/*`), produces the corresponding
    /// import string (e.g. `@/utils/helper` for `/repo/src/utils/helper.ts`).
    /// Used by the runtime when resolving preferred specifiers for prefer-shortest-imports.
    pub fn candidates_for_target(&self, target: &Path) -> Vec<String> {
        let mut candidates = Vec::new();

        for alias in &self.aliases {
            let Some(remainder) = target.strip_prefix(&alias.target_prefix).ok() else {
                continue; // target not under this alias's target prefix
            };

            let remainder_components: Vec<Component<'_>> = remainder.components().collect();
            let remainder_string = remainder_components
                .iter()
                .filter_map(|component| {
                    if let Component::Normal(value) = component {
                        Some(value.to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("/");
            let target_suffix = alias.target_suffix.to_string_lossy().replace('\\', "/");
            let middle = if target_suffix.is_empty() {
                remainder_string
            } else if let Some(value) = remainder_string.strip_suffix(&target_suffix) {
                value.trim_end_matches('/').to_string()
            } else {
                continue; // target suffix must match (e.g. .ts vs .js)
            };

            candidates.push(format!("{}{}{}", alias.key_prefix, middle, alias.key_suffix));
        }

        candidates
    }
}

fn split_wildcard(value: &str) -> (&str, &str) {
    if let Some(index) = value.find('*') {
        (&value[..index], &value[index + 1..])
    } else {
        (value, "")
    }
}

/// Lazily computes preferred (shortest) import specifiers for a module's imports.
///
/// Stored on [`ModuleRecord`](crate::ModuleRecord) and invoked on first access
/// of `preferred_specifier` / `preferred_non_relative_specifier`. Results are
/// cached in the module record's `RwLock` maps so each specifier is computed
/// at most once.
pub struct PreferredSpecifierComputer {
    resolver: Arc<Resolver>,
    tsconfig_path_aliases: Option<TsconfigPathAliases>,
}

impl PreferredSpecifierComputer {
    pub fn new(
        resolver: Arc<Resolver>,
        tsconfig_path_aliases: Option<TsconfigPathAliases>,
    ) -> Self {
        Self { resolver, tsconfig_path_aliases }
    }

    pub fn compute(
        &self,
        from_dir: &Path,
        specifier: &str,
        resolved_path: &Path,
    ) -> (Option<CompactStr>, Option<CompactStr>) {
        let mut candidates = Vec::new();

        if let Some(relative) = relative_import_specifier(from_dir, resolved_path) {
            candidates.extend(candidate_variants(&relative));
        }
        if let Some(path_aliases) = &self.tsconfig_path_aliases {
            for candidate in path_aliases.candidates_for_target(resolved_path) {
                candidates.extend(candidate_variants(&candidate));
            }
        }

        let mut preferred_specifier: Option<String> = None;
        let mut preferred_non_relative_specifier =
            (!is_relative_specifier(specifier)).then(|| specifier.to_string());

        for candidate in candidates {
            if candidate == specifier {
                continue;
            }
            if adds_explicit_extension(specifier, &candidate) {
                continue;
            }
            if !self.candidate_resolves_to_target(from_dir, &candidate, resolved_path) {
                continue;
            }

            if is_better_candidate(&candidate, specifier)
                && preferred_specifier
                    .as_ref()
                    .is_none_or(|current| is_better_candidate(&candidate, current))
            {
                preferred_specifier = Some(candidate.clone());
            }
            if !is_relative_specifier(&candidate)
                && preferred_non_relative_specifier
                    .as_ref()
                    .is_none_or(|current| is_better_candidate(&candidate, current))
            {
                preferred_non_relative_specifier = Some(candidate);
            }
        }

        let preferred_non_relative_specifier = preferred_non_relative_specifier
            .filter(|candidate| candidate != specifier)
            .map(CompactStr::from);

        (preferred_specifier.map(CompactStr::from), preferred_non_relative_specifier)
    }

    fn candidate_resolves_to_target(
        &self,
        from_dir: &Path,
        candidate: &str,
        resolved_path: &Path,
    ) -> bool {
        self.resolver
            .resolve(from_dir, candidate)
            .is_ok_and(|resolution| resolution.path() == resolved_path)
    }
}

fn is_better_candidate(candidate: &str, current: &str) -> bool {
    candidate.len() < current.len() || (candidate.len() == current.len() && candidate < current)
}

pub fn is_relative_specifier(specifier: &str) -> bool {
    specifier == "."
        || specifier == ".."
        || specifier.starts_with("./")
        || specifier.starts_with("../")
}

fn adds_explicit_extension(current: &str, candidate: &str) -> bool {
    !has_explicit_extension(current) && has_explicit_extension(candidate)
}

fn has_explicit_extension(specifier: &str) -> bool {
    let path_part = specifier.rsplit_once('?').map_or(specifier, |(path, _)| path);
    let segment = path_part.rsplit('/').next().unwrap_or(path_part);
    segment
        .rsplit_once('.')
        .is_some_and(|(name, extension)| !name.is_empty() && !extension.is_empty())
}

fn strip_last_extension(specifier: &str) -> Option<String> {
    let path_part = specifier.rsplit_once('?').map_or(specifier, |(path, _)| path);
    let (prefix, segment) = path_part.rsplit_once('/').map_or(("", path_part), |(p, s)| (p, s));
    let (name, extension) = segment.rsplit_once('.')?;
    if name.is_empty() || extension.is_empty() {
        return None;
    }

    if prefix.is_empty() { Some(name.to_string()) } else { Some(format!("{prefix}/{name}")) }
}

#[cfg(test)]
mod tests {
    use super::{
        TsconfigPathAlias, TsconfigPathAliases, candidate_variants, relative_import_specifier,
        split_wildcard, strip_last_extension,
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn candidate_variants_include_index_and_extension_forms() {
        let variants = candidate_variants("../utils/index.ts");
        assert_eq!(variants, vec!["../utils/index.ts", "../utils/index", "../utils"]);
    }

    #[test]
    fn candidate_variants_do_not_split_on_parent_directory_dots() {
        let variants = candidate_variants("@/folder.v2/helper.ts");
        assert!(variants.contains(&"@/folder.v2/helper".to_string()));
        assert!(!variants.contains(&"@/folder".to_string()));
    }

    #[test]
    fn candidate_variants_simple_relative_no_extension() {
        let variants = candidate_variants("./helper");
        assert_eq!(variants, vec!["./helper"]);
    }

    #[test]
    fn candidate_variants_normalizes_backslashes() {
        let variants = candidate_variants("..\\utils\\helper.ts");
        assert!(variants.contains(&"../utils/helper.ts".to_string()));
        assert!(variants.contains(&"../utils/helper".to_string()));
    }

    #[test]
    fn candidate_variants_index_without_extension() {
        let variants = candidate_variants("@/utils/index");
        assert_eq!(variants, vec!["@/utils/index", "@/utils"]);
    }

    #[test]
    fn candidate_variants_single_segment_with_extension() {
        let variants = candidate_variants("./helper.ts");
        assert_eq!(variants, vec!["./helper.ts", "./helper"]);
    }

    #[test]
    fn relative_import_specifier_same_dir_is_dot_slash() {
        let from_dir = Path::new("/repo/src/features");
        let target = Path::new("/repo/src/features/helper.ts");
        assert_eq!(relative_import_specifier(from_dir, target), Some("./helper.ts".to_string()));
    }

    #[test]
    fn relative_import_specifier_parent_traversal() {
        let from_dir = Path::new("/repo/src/features/deep");
        let target = Path::new("/repo/src/utils/helper.ts");
        assert_eq!(
            relative_import_specifier(from_dir, target),
            Some("../../utils/helper.ts".to_string())
        );
    }

    #[test]
    fn relative_import_specifier_no_common_prefix() {
        let from_dir = Path::new("/project-a/src");
        let target = Path::new("/project-b/lib/util.ts");
        // On Unix they share "/", on Windows this would differ.
        // The function should still produce a valid result when there's at least root in common.
        let result = relative_import_specifier(from_dir, target);
        assert!(result.is_some());
        assert!(result.unwrap().starts_with("../"));
    }

    #[test]
    fn strip_last_extension_ignores_query_suffix() {
        assert_eq!(
            strip_last_extension("../utils/helper.ts?raw"),
            Some("../utils/helper".to_string())
        );
    }

    #[test]
    fn strip_last_extension_no_extension() {
        assert_eq!(strip_last_extension("../utils/helper"), None);
    }

    #[test]
    fn strip_last_extension_dotfile_returns_none() {
        assert_eq!(strip_last_extension(".env"), None);
    }

    #[test]
    fn strip_last_extension_dotfile_in_path() {
        assert_eq!(strip_last_extension("./config/.env"), None);
    }

    #[test]
    fn strip_last_extension_multiple_dots() {
        assert_eq!(strip_last_extension("./helper.test.ts"), Some("./helper.test".to_string()));
    }

    #[test]
    fn split_wildcard_with_star() {
        assert_eq!(split_wildcard("@/*"), ("@/", ""));
    }

    #[test]
    fn split_wildcard_with_suffix() {
        assert_eq!(split_wildcard("~/lib/*.js"), ("~/lib/", ".js"));
    }

    #[test]
    fn split_wildcard_no_star() {
        assert_eq!(split_wildcard("@utils"), ("@utils", ""));
    }

    #[test]
    fn candidates_for_target_basic_wildcard() {
        let aliases = TsconfigPathAliases {
            aliases: vec![TsconfigPathAlias {
                key_prefix: "@/".to_string(),
                key_suffix: String::new(),
                target_prefix: PathBuf::from("/repo/src"),
                target_suffix: PathBuf::new(),
            }],
        };
        let candidates = aliases.candidates_for_target(Path::new("/repo/src/utils/helper.ts"));
        assert_eq!(candidates, vec!["@/utils/helper.ts"]);
    }

    #[test]
    fn candidates_for_target_exact_alias() {
        let aliases = TsconfigPathAliases {
            aliases: vec![TsconfigPathAlias {
                key_prefix: "@utils".to_string(),
                key_suffix: String::new(),
                target_prefix: PathBuf::from("/repo/src/utils/index.ts"),
                target_suffix: PathBuf::new(),
            }],
        };
        let candidates = aliases.candidates_for_target(Path::new("/repo/src/utils/index.ts"));
        assert_eq!(candidates, vec!["@utils"]);
    }

    #[test]
    fn candidates_for_target_no_match() {
        let aliases = TsconfigPathAliases {
            aliases: vec![TsconfigPathAlias {
                key_prefix: "@/".to_string(),
                key_suffix: String::new(),
                target_prefix: PathBuf::from("/repo/src"),
                target_suffix: PathBuf::new(),
            }],
        };
        let candidates = aliases.candidates_for_target(Path::new("/other/lib/util.ts"));
        assert!(candidates.is_empty());
    }

    #[test]
    fn candidates_for_target_multiple_aliases() {
        let aliases = TsconfigPathAliases {
            aliases: vec![
                TsconfigPathAlias {
                    key_prefix: "@/".to_string(),
                    key_suffix: String::new(),
                    target_prefix: PathBuf::from("/repo/src"),
                    target_suffix: PathBuf::new(),
                },
                TsconfigPathAlias {
                    key_prefix: "~/".to_string(),
                    key_suffix: String::new(),
                    target_prefix: PathBuf::from("/repo/src"),
                    target_suffix: PathBuf::new(),
                },
            ],
        };
        let candidates = aliases.candidates_for_target(Path::new("/repo/src/utils/helper.ts"));
        assert_eq!(candidates, vec!["@/utils/helper.ts", "~/utils/helper.ts"]);
    }

    #[test]
    fn candidates_for_target_with_key_suffix() {
        let aliases = TsconfigPathAliases {
            aliases: vec![TsconfigPathAlias {
                key_prefix: "#lib/".to_string(),
                key_suffix: ".js".to_string(),
                target_prefix: PathBuf::from("/repo/src/lib"),
                target_suffix: PathBuf::from(".ts"),
            }],
        };
        let candidates = aliases.candidates_for_target(Path::new("/repo/src/lib/utils.ts"));
        assert_eq!(candidates, vec!["#lib/utils.js"]);
    }

    #[test]
    fn candidates_for_target_suffix_mismatch() {
        let aliases = TsconfigPathAliases {
            aliases: vec![TsconfigPathAlias {
                key_prefix: "#lib/".to_string(),
                key_suffix: ".js".to_string(),
                target_prefix: PathBuf::from("/repo/src/lib"),
                target_suffix: PathBuf::from(".ts"),
            }],
        };
        let candidates = aliases.candidates_for_target(Path::new("/repo/src/lib/utils.js"));
        assert!(candidates.is_empty());
    }

    #[test]
    fn candidate_variants_dot_specifier() {
        let variants = candidate_variants(".");
        assert_eq!(variants, vec!["."]);
    }

    #[test]
    fn candidate_variants_dot_dot_specifier() {
        let variants = candidate_variants("..");
        assert_eq!(variants, vec![".."]);
    }

    #[test]
    fn relative_import_specifier_same_path_returns_none() {
        let dir = Path::new("/repo/src/features");
        assert_eq!(relative_import_specifier(dir, dir), None);
    }

    #[test]
    fn candidate_variants_deduplicates() {
        let variants = candidate_variants("./index");
        assert_eq!(variants.iter().filter(|v| *v == ".").count(), 1);
    }

    #[test]
    fn candidate_variants_empty_string_produces_nothing() {
        let variants = candidate_variants("");
        assert!(variants.is_empty());
    }

    #[test]
    fn strip_last_extension_bare_filename() {
        assert_eq!(strip_last_extension("helper.ts"), Some("helper".to_string()));
    }

    #[test]
    fn relative_import_specifier_deeper_target() {
        let from_dir = Path::new("/repo/src");
        let target = Path::new("/repo/src/utils/deep/helper.ts");
        assert_eq!(
            relative_import_specifier(from_dir, target),
            Some("./utils/deep/helper.ts".to_string())
        );
    }

    #[test]
    fn is_empty_reflects_aliases() {
        let empty = TsconfigPathAliases { aliases: vec![] };
        assert!(empty.is_empty());

        let non_empty = TsconfigPathAliases {
            aliases: vec![TsconfigPathAlias {
                key_prefix: "@/".to_string(),
                key_suffix: String::new(),
                target_prefix: PathBuf::from("/repo/src"),
                target_suffix: PathBuf::new(),
            }],
        };
        assert!(!non_empty.is_empty());
    }
}
