//! Support for the `workingDirectories` workspace option.
//!
//! A monorepo is usually opened at its root, which means the language server creates a single
//! [`WorkspaceWorker`](crate::worker::WorkspaceWorker) for it. Every tool then runs with the
//! repository root as its working directory, which is wrong for packages that ship their own
//! configuration: `configPath` resolves against the repository root, nested configs are merged
//! with the root config, and JS plugins / `tsgolint` are looked up from the repository root.
//!
//! `workingDirectories` is the server side equivalent of `eslint.workingDirectories`: it declares
//! additional project roots below a workspace folder. Every resolved directory becomes an ordinary
//! worker, so a file inside it is handled exactly as if the user had opened that directory as the
//! workspace folder.
//!
//! # Semantics
//!
//! These rules are the contract of the feature. Each one is covered by a named test which
//! references its number, so an alternative behaviour is a deliberate change of the contract and
//! not an oversight.
//!
//! 1. A working directory is a *virtual workspace folder*: it is served by an ordinary
//!    [`WorkspaceWorker`](crate::worker::WorkspaceWorker) rooted at that directory, and a file is
//!    routed to it by the longest matching root, exactly like a workspace folder the client
//!    opened itself. There is exactly one worker, and one watcher registration, per root: a
//!    workspace folder opened on a working directory replaces its sub worker, and removing that
//!    folder hands the root back to a sub worker.
//! 2. Its root configuration is found by walking **up** from the sub directory, and the first
//!    config found wins. This is what a nested client workspace folder does, and what
//!    `cd <dir> && <tool>` does. The configuration of the workspace folder and the one of the sub
//!    root are never merged.
//! 3. A file below a working directory is ignored when the tool ignores it with the workspace
//!    folder opened *without* `workingDirectories`, or with the working directory opened *alone*.
//!    Each tool decides with its own ignore code, the server evaluates nothing itself, and a
//!    working directory never un-ignores a file. A working directory which the workspace folder
//!    itself ignores is reported with a warning when its tool is built.
//! 4. A workspace folder excludes its working directories from its own eager discovery, and a
//!    working directory excludes the working directories nested below it. The worker of the
//!    parent is not responsible for the files of a working directory: it does not lint them, and
//!    it does not answer commands (`oxc.fixAll`) for them.
//! 5. The set of working directories is resolved when the server is initialized, when the
//!    configuration changes and when a workspace folder is added or removed, never on a watched
//!    file event. When it changes, the sub workers are created and removed, every worker whose
//!    exclusions changed is rebuilt once, the open documents are revalidated exactly once by their
//!    new owner, and a pull mode client is asked to refresh its diagnostics. A worker gets at most
//!    one watcher (un)registration per notification, and rewriting the option without changing the
//!    roots it resolves to rebuilds nothing.
//! 6. A watched file event is handed to every worker, like it is without `workingDirectories`.
//! 7. An entry is a string or `{ "directory": "..." }`: a path relative to the workspace folder,
//!    which has to exist, be a directory and stay inside the folder (the path is checked, a
//!    symbolic link is followed). `..`, absolute paths and `.`
//!    are rejected, and glob characters are literal. ESLint's `!cwd` is accepted: it is ignored, and
//!    reported once. An entry with `mode` or `pattern` is not supported in this version and is reported. Every entry
//!    is judged on its own: an invalid one never hides a valid one.
//! 8. The validation warnings of the option are reported when its value changes, not again on
//!    every later reconciliation.
//! 9. The default, an absent or empty option, changes nothing: one worker per workspace folder.
//! 10. The option is ignored entirely by a tool which does not
//!     [use working directories](crate::ToolBuilder::use_working_directories): no sub worker is
//!     created and no warning is reported.

use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tower_lsp_server::gen_lsp_types::{MessageType, Uri};
use tracing::debug;

use crate::{
    ClientMessage, ToolBuilder,
    uri_utils::{file_path_to_uri, uri_to_file_path},
};

/// Name of the workspace option holding the working directories.
pub const WORKING_DIRECTORIES_OPTION: &str = "workingDirectories";

/// A single `workingDirectories` entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum WorkingDirectory {
    /// A path relative to the workspace folder. Example: `"packages/a"`.
    Path(String),
    /// An object entry, `{ "directory": "client" }` (ESLint compatible).
    Entry(WorkingDirectoryEntry),
}

/// The object form of a [`WorkingDirectory`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkingDirectoryEntry {
    /// A path relative to the workspace folder.
    pub directory: String,
    /// ESLint's `!cwd`, which asks the client not to change the working directory of the linter
    /// process. The server gives every working directory its own worker instead, so there is no
    /// process working directory to keep, and the flag is ignored.
    #[serde(default, rename = "!cwd", skip_serializing_if = "Option::is_none")]
    pub not_cwd: Option<bool>,
}

/// The outcome of [`resolve_working_directories`].
#[derive(Debug, Default)]
pub struct ResolvedWorkingDirectories {
    /// The resolved project roots, sorted and deduplicated. Never contains the workspace folder.
    pub roots: Vec<Uri>,
    /// Human readable problems which should be reported to the client.
    pub warnings: Vec<String>,
}

impl ResolvedWorkingDirectories {
    /// Turn the warnings into messages which the server shows to the user.
    pub fn client_messages(&self) -> Vec<ClientMessage> {
        self.warnings
            .iter()
            .map(|warning| ClientMessage { message: warning.clone(), r#type: MessageType::Warning })
            .collect()
    }
}

/// Returns the options a sub worker should be started with.
///
/// These are the options of its workspace folder without the `workingDirectories` key, so a
/// working directory never spawns further working directories.
pub fn sub_worker_options(options: &serde_json::Value) -> serde_json::Value {
    let mut options = options.clone();
    if let Some(object) = options.as_object_mut() {
        object.remove(WORKING_DIRECTORIES_OPTION);
    }
    options
}

/// Resolve the `workingDirectories` option of a workspace folder into absolute project roots.
///
/// Returns an empty result when the option is absent or empty (the default: one worker per
/// workspace folder), or when the tool does not
/// [use working directories](ToolBuilder::use_working_directories).
/// `nested_roots` holds the workspace folders the client opened below this one: a working
/// directory inside one of them belongs to that folder and is skipped. The directories themselves
/// stay in the result: this root still has to exclude them from its own discovery, regardless of
/// which worker serves them.
pub fn resolve_working_directories(
    root_uri: &Uri,
    options: &serde_json::Value,
    builder: &dyn ToolBuilder,
    nested_roots: &[PathBuf],
) -> ResolvedWorkingDirectories {
    let mut resolved = ResolvedWorkingDirectories::default();

    let Some(value) = options.get(WORKING_DIRECTORIES_OPTION) else {
        return resolved;
    };
    if value.is_null() {
        return resolved;
    }
    if !builder.use_working_directories() {
        debug!("`{WORKING_DIRECTORIES_OPTION}` is disabled, ignored for {}", root_uri.as_str());
        return resolved;
    }

    let Some(entries) = value.as_array() else {
        resolved.warnings.push(format!(
            "`{WORKING_DIRECTORIES_OPTION}` is invalid and was ignored: expected an array of entries"
        ));
        return resolved;
    };
    if entries.is_empty() {
        return resolved;
    }

    let Some(root_path) = uri_to_file_path(root_uri) else {
        resolved.warnings.push(format!(
            "`{WORKING_DIRECTORIES_OPTION}` is only supported for `file://` workspace folders, it was ignored for `{}`",
            root_uri.as_str()
        ));
        return resolved;
    };

    let mut has_not_cwd = false;
    let mut directories = vec![];
    // the entries are judged one by one, an invalid one must not hide a valid one
    for entry in entries {
        let directory = match entry {
            serde_json::Value::String(path) => path.as_str(),
            serde_json::Value::Object(object) => {
                has_not_cwd |=
                    object.get("!cwd").and_then(serde_json::Value::as_bool) == Some(true);

                if let Some(key) =
                    ["mode", "pattern"].into_iter().find(|key| object.contains_key(*key))
                {
                    resolved.warnings.push(format!(
                        "`{WORKING_DIRECTORIES_OPTION}` entry with `{key}` is not supported in this version and was ignored"
                    ));
                    continue;
                }
                let Some(directory) = object.get("directory").and_then(serde_json::Value::as_str)
                else {
                    resolved.warnings.push(format!(
                        "`{WORKING_DIRECTORIES_OPTION}` entry without a `directory` string was ignored"
                    ));
                    continue;
                };
                directory
            }
            _ => {
                resolved.warnings.push(format!(
                    "`{WORKING_DIRECTORIES_OPTION}` entry is neither a string nor an object and was ignored"
                ));
                continue;
            }
        };

        match validate_entry(directory) {
            Ok(entry) => directories.push(entry),
            Err(err) => resolved.warnings.push(err),
        }
    }

    if has_not_cwd {
        // reported once, however many entries carry the flag
        resolved.warnings.push(format!(
            "`{WORKING_DIRECTORIES_OPTION}` ignores `!cwd`: every working directory already gets its own worker rooted at it"
        ));
    }

    // The caller resolves its roots through `ResolvedPath`, the entries are joined to the path of
    // the workspace folder URI. Both are brought into the same form before anything is compared.
    let nested_roots = &align_nested_roots(&root_path, nested_roots);

    let mut roots = Vec::with_capacity(directories.len());
    for entry in &directories {
        let directory = root_path.join(entry);
        if nested_roots.iter().any(|nested| directory != *nested && directory.starts_with(nested)) {
            resolved.warnings.push(format!(
                "`{WORKING_DIRECTORIES_OPTION}` entry `{entry}` is inside a workspace folder of its own and was ignored"
            ));
        } else if directory.is_dir() {
            roots.push(directory);
        } else {
            resolved.warnings.push(format!(
                "`{WORKING_DIRECTORIES_OPTION}` entry `{entry}` is not an existing directory and was ignored"
            ));
        }
    }

    roots.sort_unstable();
    roots.dedup();

    resolved.roots = roots
        .into_iter()
        // a working directory must stay below the workspace folder and can not be the folder itself
        .filter(|dir| dir != &root_path && dir.starts_with(&root_path))
        .filter_map(file_path_to_uri)
        .collect();

    debug!("resolved working directories for {}: {:?}", root_uri.as_str(), resolved.roots);

    resolved
}

/// Express the roots of the nested workspace folders in the path form of `root_path`.
///
/// The [`WorkerManager`](crate::worker_manager::WorkerManager) resolves its roots through
/// [`ResolvedPath`](crate::file_system::ResolvedPath), which canonicalizes on macOS and Windows,
/// while the resolution here joins the path of the workspace folder URI. A temporary directory on
/// macOS is `/var/folders/...` in one form and `/private/var/folders/...` in the other, and a
/// symbolic link or a Windows short name has the same effect: comparing the two forms never
/// matches, and an entry inside a nested workspace folder would not be skipped. The output keeps
/// the form of the workspace folder URI, so the roots this function returns stay comparable to it.
fn align_nested_roots(root_path: &Path, nested_roots: &[PathBuf]) -> Vec<PathBuf> {
    if nested_roots.is_empty() {
        return vec![];
    }

    let canonical_root =
        std::fs::canonicalize(root_path).unwrap_or_else(|_| root_path.to_path_buf());

    nested_roots
        .iter()
        .map(|nested| {
            let canonical = std::fs::canonicalize(nested).unwrap_or_else(|_| nested.clone());
            canonical
                .strip_prefix(&canonical_root)
                .map_or_else(|_| nested.clone(), |relative| root_path.join(relative))
        })
        // the workspace folder itself is not one of its nested folders
        .filter(|nested| nested != root_path)
        .collect()
}

/// Validate a single entry and return it with normalized separators.
///
/// Absolute paths and paths escaping the workspace folder are rejected. The entry is a literal
/// path: glob characters have no special meaning.
fn validate_entry(entry: &str) -> Result<String, String> {
    let trimmed = entry.trim();
    if trimmed.is_empty() {
        return Err(format!("`{WORKING_DIRECTORIES_OPTION}` entry is empty and was ignored"));
    }

    #[expect(clippy::disallowed_methods)] // the normalized entry always uses `/`, even on Windows
    let normalized = trimmed.replace('\\', "/");

    if Path::new(&normalized).is_absolute() || normalized.starts_with('/') {
        return Err(format!(
            "`{WORKING_DIRECTORIES_OPTION}` entry `{entry}` is an absolute path and was ignored, use a path relative to the workspace folder"
        ));
    }

    // `.` and empty segments (`a//b`, `a/./b`, `a/b/.`) name the same directory, the entry becomes
    // one canonical spelling so equal directories give equal roots
    let segments = normalized
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect::<Vec<_>>();

    if segments.is_empty() {
        return Err(format!(
            "`{WORKING_DIRECTORIES_OPTION}` entry `{entry}` points to the workspace folder itself and was ignored"
        ));
    }

    if segments.contains(&"..") {
        return Err(format!(
            "`{WORKING_DIRECTORIES_OPTION}` entry `{entry}` escapes the workspace folder and was ignored"
        ));
    }

    Ok(segments.join("/"))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use serde_json::json;
    use tower_lsp_server::gen_lsp_types::Uri;

    use crate::{ToolBuildResult, ToolBuilder, tests::FakeToolBuilder};

    use super::{
        WORKING_DIRECTORIES_OPTION, resolve_working_directories, sub_worker_options, validate_entry,
    };

    /// A builder which does not override [`ToolBuilder::use_working_directories`].
    struct NotOptedInToolBuilder;

    impl ToolBuilder for NotOptedInToolBuilder {
        fn build(&self, root_uri: &Uri, options: serde_json::Value) -> ToolBuildResult {
            FakeToolBuilder::default().build(root_uri, options)
        }
    }

    fn builder() -> FakeToolBuilder {
        FakeToolBuilder::default()
    }

    fn uri(path: &Path) -> Uri {
        crate::uri_utils::file_path_to_uri(path).unwrap()
    }

    #[test]
    fn test_no_option() {
        let dir = tempfile::tempdir().unwrap();
        let resolved = resolve_working_directories(&uri(dir.path()), &json!({}), &builder(), &[]);
        assert_eq!(resolved.roots.len(), 0);
        assert_eq!(resolved.warnings.len(), 0);

        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: [] }),
            &builder(),
            &[],
        );
        assert_eq!(resolved.roots.len(), 0);
        assert_eq!(resolved.warnings.len(), 0);
    }

    /// Semantics rules 1 and 4: the nested workspace folders come from the worker manager, which
    /// resolves them through `ResolvedPath`. That form is not the one of the workspace folder URI
    /// (macOS resolves `/var` to `/private/var`, Windows the short names and the casing), so both
    /// sides are brought into one form before they are compared. A symbolic link reproduces the
    /// same mismatch everywhere.
    #[cfg(unix)]
    #[test]
    fn test_a_nested_root_in_another_spelling_is_still_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        fs::create_dir_all(real.join("packages/a/sub")).unwrap();
        fs::create_dir_all(real.join("packages/b/sub")).unwrap();

        // the client opened the workspace folder through the link, the manager resolved the
        // nested folder to the directory it points at
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let resolved = resolve_working_directories(
            &uri(&link),
            &json!({ WORKING_DIRECTORIES_OPTION: ["packages/a/sub", "packages/b/sub"] }),
            &builder(),
            &[real.join("packages/a")],
        );

        // `packages/a` is a workspace folder of its own, regardless of the spelling used for it
        assert_eq!(resolved.roots, vec![uri(&link.join("packages/b/sub"))], "{:?}", resolved.roots);
        assert_eq!(resolved.warnings.len(), 1, "{:?}", resolved.warnings);
        assert!(resolved.warnings[0].contains("inside a workspace folder of its own"));
    }

    /// Semantics rule 7: strings and `{ "directory": ... }` entries are literal paths.
    #[test]
    fn test_literal_entries() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("packages/a/src")).unwrap();
        fs::create_dir_all(dir.path().join("packages/b")).unwrap();
        fs::create_dir_all(dir.path().join("client")).unwrap();

        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: [
                "packages/a",
                "./packages/b/",
                { "directory": "client" },
            ] }),
            &builder(),
            &[],
        );

        assert!(resolved.warnings.is_empty(), "{:?}", resolved.warnings);
        let roots: Vec<Uri> = [
            dir.path().join("client"),
            dir.path().join("packages/a"),
            dir.path().join("packages/b"),
        ]
        .iter()
        .map(|path| uri(path))
        .collect();
        assert_eq!(resolved.roots, roots);
    }

    /// Semantics rule 7: glob characters are literal, `apps/[locale]` is a directory like any other.
    #[test]
    fn test_glob_characters_are_literal() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("apps/[locale]")).unwrap();
        fs::create_dir_all(dir.path().join("apps/l")).unwrap();
        fs::create_dir_all(dir.path().join("packages/a")).unwrap();

        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: ["apps/[locale]", "packages/*", "apps/{l}"] }),
            &builder(),
            &[],
        );

        assert_eq!(resolved.roots, vec![uri(&dir.path().join("apps/[locale]"))]);
        // the globs match nothing, they are directories which do not exist
        assert_eq!(resolved.warnings.len(), 2, "{:?}", resolved.warnings);
        assert!(
            resolved.warnings.iter().all(|warning| warning.contains("not an existing directory"))
        );
    }

    /// Semantics rule 7: `mode` and `pattern` are not supported in this version, the entries are
    /// reported and create nothing, while a valid sibling entry still resolves.
    #[test]
    fn test_unsupported_entries_are_reported() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("packages/a")).unwrap();
        fs::create_dir_all(dir.path().join("client")).unwrap();

        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: [
                { "mode": "auto" },
                { "pattern": "packages/a" },
                { "directory": "packages/a", "pattern": "packages/*" },
                "client",
            ] }),
            &builder(),
            &[],
        );

        assert_eq!(resolved.roots, vec![uri(&dir.path().join("client"))]);
        assert_eq!(resolved.warnings.len(), 3, "{:?}", resolved.warnings);
        assert!(resolved.warnings[0].contains("`mode` is not supported in this version"));
        assert!(resolved.warnings[1].contains("`pattern` is not supported in this version"));
        assert!(resolved.warnings[2].contains("`pattern` is not supported in this version"));
    }

    /// Semantics rule 7: an invalid entry never hides a valid one.
    #[test]
    fn test_an_invalid_entry_does_not_hide_a_valid_one() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("packages/a")).unwrap();

        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: [42, null, { "directory": 1 }, {}, "packages/a"] }),
            &builder(),
            &[],
        );

        assert_eq!(resolved.roots, vec![uri(&dir.path().join("packages/a"))]);
        assert_eq!(resolved.warnings.len(), 4, "{:?}", resolved.warnings);
        assert!(resolved.warnings[0].contains("neither a string nor an object"));
        assert!(resolved.warnings[1].contains("neither a string nor an object"));
        assert!(resolved.warnings[2].contains("without a `directory` string"));
        assert!(resolved.warnings[3].contains("without a `directory` string"));
    }

    #[test]
    fn test_rejected_entries() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("packages/a")).unwrap();

        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: ["../outside", "/absolute", ".", "", "packages/a"] }),
            &builder(),
            &[],
        );

        assert_eq!(resolved.roots, vec![uri(&dir.path().join("packages/a"))]);
        assert_eq!(resolved.warnings.len(), 4, "{:?}", resolved.warnings);
        assert!(resolved.warnings[0].contains("escapes the workspace folder"));
        assert!(resolved.warnings[1].contains("absolute path"));
        assert!(resolved.warnings[2].contains("workspace folder itself"));
        assert!(resolved.warnings[3].contains("is empty"));
    }

    #[test]
    fn test_not_cwd_is_accepted_and_ignored() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("packages/a")).unwrap();
        fs::create_dir_all(dir.path().join("packages/b")).unwrap();

        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: [
                { "directory": "packages/a", "!cwd": true },
                { "directory": "packages/b", "!cwd": true },
            ] }),
            &builder(),
            &[],
        );

        assert_eq!(
            resolved.roots,
            vec![uri(&dir.path().join("packages/a")), uri(&dir.path().join("packages/b"))]
        );
        // `!cwd` is reported once and ignored
        assert_eq!(resolved.warnings.len(), 1, "{:?}", resolved.warnings);
        assert!(resolved.warnings[0].contains("ignores `!cwd`"));
    }

    #[test]
    fn test_unknown_directory_warns() {
        let dir = tempfile::tempdir().unwrap();
        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: ["packages/a"] }),
            &builder(),
            &[],
        );
        assert_eq!(resolved.roots.len(), 0);
        assert_eq!(resolved.warnings.len(), 1);
        assert!(resolved.warnings[0].contains("not an existing directory"));
    }

    /// Semantics rule 10: nothing is resolved by a tool which does not opt in, not even the
    /// warnings of an invalid value.
    #[test]
    fn test_ignored_when_the_tool_does_not_use_working_directories() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("packages/a")).unwrap();

        let disabled = FakeToolBuilder::default().with_working_directories_disabled();
        for option in
            [json!(["packages/a"]), json!(["../outside"]), json!([{ "mode": "auto" }]), json!(42)]
        {
            for builder in [&disabled as &dyn ToolBuilder, &NotOptedInToolBuilder] {
                let resolved = resolve_working_directories(
                    &uri(dir.path()),
                    &json!({ WORKING_DIRECTORIES_OPTION: option }),
                    builder,
                    &[],
                );
                assert_eq!(resolved.roots.len(), 0);
                assert!(resolved.warnings.is_empty(), "{:?}", resolved.warnings);
            }
        }
    }

    #[test]
    fn test_invalid_option_shape() {
        let dir = tempfile::tempdir().unwrap();
        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: 42 }),
            &builder(),
            &[],
        );
        assert_eq!(resolved.roots.len(), 0);
        assert_eq!(resolved.warnings.len(), 1);
        assert!(resolved.warnings[0].contains("is invalid"));
    }

    #[test]
    fn test_nested_working_directories_are_allowed() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("packages/a/nested")).unwrap();

        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: ["packages/a", "packages/a/nested"] }),
            &builder(),
            &[],
        );

        assert_eq!(
            resolved.roots,
            vec![uri(&dir.path().join("packages/a")), uri(&dir.path().join("packages/a/nested")),]
        );
    }

    #[test]
    fn test_sub_worker_options() {
        let options = json!({ "run": "onType", WORKING_DIRECTORIES_OPTION: ["packages/a"] });
        assert_eq!(sub_worker_options(&options), json!({ "run": "onType" }));

        // non object options are passed through
        assert_eq!(sub_worker_options(&serde_json::Value::Null), serde_json::Value::Null);
    }

    #[test]
    fn test_validate_entry() {
        assert_eq!(validate_entry("packages/a"), Ok("packages/a".to_string()));
        assert_eq!(validate_entry("./packages/a/"), Ok("packages/a".to_string()));
        assert_eq!(validate_entry("packages\\a"), Ok("packages/a".to_string()));
        assert_eq!(validate_entry("apps/[locale]"), Ok("apps/[locale]".to_string()));
        assert!(validate_entry("../a").is_err());
        assert!(validate_entry("packages/../../a").is_err());
        assert!(validate_entry("/a").is_err());
        // a second separator after the `./` must not turn the entry into an absolute path
        assert_eq!(validate_entry(".//a"), Ok("a".to_string()));
        assert_eq!(validate_entry("././a"), Ok("a".to_string()));
        // equal directories give equal entries
        assert_eq!(validate_entry("a//b"), Ok("a/b".to_string()));
        assert_eq!(validate_entry("a/./b/."), Ok("a/b".to_string()));
        assert!(validate_entry(".//").is_err());
        assert!(validate_entry("  ").is_err());
        assert!(validate_entry(".").is_err());
    }
}
