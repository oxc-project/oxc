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
//! 3. The ignore files of the directories between the workspace folder and the sub root are
//!    honoured. The deepest file wins, and a `!pattern` in it can re-include a path which a file
//!    above it excluded, like `git` does.
//! 4. A workspace folder excludes its working directories from its own eager discovery, and a
//!    working directory excludes the working directories nested below it.
//! 5. When the resolved set changes (a configuration change, an `auto` reconciliation after a
//!    watched file event, or a workspace folder being added or removed), the sub workers are
//!    created and removed, every worker whose exclusions changed is rebuilt and has its watchers
//!    re-registered if its patterns changed, the open documents are revalidated exactly once by
//!    their new owner, and a pull mode client is asked to refresh its diagnostics. Rewriting the
//!    option without changing the roots it resolves to rebuilds nothing.
//! 6. A watched file event reaches the workers the change can affect: the ones whose root
//!    contains the file, the ones rooted below the directory holding it (a config there governs
//!    them), and the ones which registered it as an absolute watcher pattern because they extend
//!    it. A file inside a working directory belongs to that worker, so its workspace folder does
//!    not see the event unless it extends that very file. An event which reaches none of them is
//!    handed to every worker.
//! 7. `[{ "mode": "auto" }]` detects a directory holding both a `package.json` and a tool
//!    configuration file, and follows the `.gitignore` of the workspace folder, while a directory
//!    listed explicitly (a path or a glob) is a working directory even when it is gitignored.
//!    `node_modules` and `.git` are never walked, and an event below them only reaches a worker
//!    whose tool explicitly watches that exact file, for example a config it extends from a
//!    dependency. A `package.json` change never reaches a tool at all: the workspace folder
//!    watches it for the detection alone. The set is recomputed when a file named `package.json`
//!    or one of the configuration files of the tool is created or deleted, which is decided by
//!    the file name and not by the watcher patterns: a setup with an explicit `configPath`
//!    watches one file and still gains a working directory when a config appears elsewhere.
//! 8. The validation warnings of the option are reported when its value changes, not again on
//!    every later reconciliation.
//! 9. The default, an absent or empty option, changes nothing: one worker per workspace folder.
//!
//! An entry may be a string, `{ "directory": "..." }` or `{ "pattern": "..." }`. ESLint's `!cwd`
//! is accepted and ignored: every working directory already gets its own worker rooted at it.

use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tower_lsp_server::ls_types::{MessageType, Uri};
use tracing::debug;

use crate::{ClientMessage, ToolBuilder};

/// Name of the workspace option holding the working directories.
pub const WORKING_DIRECTORIES_OPTION: &str = "workingDirectories";

/// Directory names which are never part of a project tree.
const IGNORED_DIRECTORIES: [&str; 2] = ["node_modules", ".git"];

/// Whether `path` is below a `node_modules` or `.git` directory *of* `root`.
///
/// Only the components below `root` are inspected. A workspace folder may itself live inside a
/// `node_modules` directory, and everything in it is then relevant.
///
/// The `auto` mode watches `**/package.json`, and the LSP glob syntax has no negation, so events
/// below these directories are filtered out by the server instead.
pub fn is_below_ignored_directory(root: &Path, path: &Path) -> bool {
    path.strip_prefix(root).is_ok_and(|relative| {
        relative.components().any(|component| {
            component.as_os_str().to_str().is_some_and(|name| IGNORED_DIRECTORIES.contains(&name))
        })
    })
}

/// A single `workingDirectories` entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum WorkingDirectory {
    /// A path or a glob, relative to the workspace folder. Example: `"packages/*"`.
    Path(String),
    /// An object entry, either `{ "directory": "client" }` (ESLint compatible) or
    /// `{ "mode": "auto" }`.
    Entry(WorkingDirectoryEntry),
}

/// The object form of a [`WorkingDirectory`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkingDirectoryEntry {
    /// A path or a glob, relative to the workspace folder.
    ///
    /// `pattern` is accepted as an alias, because that is the name some clients use for the same
    /// thing in their own settings.
    #[serde(default, alias = "pattern", skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,
    /// When set to `auto`, the working directories are detected instead of being listed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<WorkingDirectoryMode>,
    /// ESLint's `!cwd`, which asks the client not to change the working directory of the linter
    /// process. The server gives every working directory its own worker instead, so there is no
    /// process working directory to keep, and the flag is ignored.
    #[serde(default, rename = "!cwd", skip_serializing_if = "Option::is_none")]
    pub not_cwd: Option<bool>,
}

/// Detection mode of a [`WorkingDirectoryEntry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum WorkingDirectoryMode {
    /// Detect every directory which contains both a `package.json` and a tool configuration file.
    Auto,
}

/// The outcome of [`resolve_working_directories`].
#[derive(Debug, Default)]
pub struct ResolvedWorkingDirectories {
    /// The resolved project roots, sorted and deduplicated. Never contains the workspace folder.
    pub roots: Vec<Uri>,
    /// Human readable problems which should be reported to the client.
    pub warnings: Vec<String>,
    /// Whether automatic detection is enabled, which requires watching `package.json` files.
    pub auto: bool,
}

impl ResolvedWorkingDirectories {
    /// Turn the warnings into messages which the server shows to the user.
    pub fn client_messages(&self) -> Vec<ClientMessage> {
        self.warnings
            .iter()
            .map(|warning| ClientMessage { message: warning.clone(), r#type: MessageType::WARNING })
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

/// Whether resolving this option walks the workspace folder.
///
/// Only a glob and the `auto` detection do. A literal entry is a handful of `is_dir()` calls, and
/// an absent or empty option is no work at all, so the caller can resolve those inline instead of
/// moving them to a blocking thread.
pub fn working_directories_need_walk(options: &serde_json::Value) -> bool {
    let Some(value) = options.get(WORKING_DIRECTORIES_OPTION) else {
        return false;
    };
    let Ok(entries) = serde_json::from_value::<Vec<WorkingDirectory>>(value.clone()) else {
        return false;
    };

    entries.iter().any(|entry| match entry {
        WorkingDirectory::Path(path) => is_glob(path),
        WorkingDirectory::Entry(entry) => {
            entry.mode == Some(WorkingDirectoryMode::Auto)
                || entry.directory.as_deref().is_some_and(is_glob)
        }
    })
}

/// Resolve the `workingDirectories` option of a workspace folder into absolute project roots.
///
/// Returns an empty result when the option is absent or empty, which is the default and keeps the
/// previous behaviour of one worker per workspace folder.
/// `nested_roots` holds the workspace folders the client opened below this one. The resolution
/// never descends *into* them, because everything there belongs to their own worker, but the
/// directories themselves stay in the result: this root still has to exclude them from its own
/// discovery, regardless of which worker serves them.
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

    let entries = match serde_json::from_value::<Vec<WorkingDirectory>>(value.clone()) {
        Ok(entries) => entries,
        Err(err) => {
            resolved
                .warnings
                .push(format!("`{WORKING_DIRECTORIES_OPTION}` is invalid and was ignored: {err}"));
            return resolved;
        }
    };
    if entries.is_empty() {
        return resolved;
    }

    let Some(root_path) = root_uri.to_file_path() else {
        resolved.warnings.push(format!(
            "`{WORKING_DIRECTORIES_OPTION}` is only supported for `file://` workspace folders, it was ignored for `{}`",
            root_uri.as_str()
        ));
        return resolved;
    };

    let mut patterns = Vec::with_capacity(entries.len());
    let mut has_not_cwd = false;
    for entry in entries {
        let directory = match entry {
            WorkingDirectory::Path(path) => path,
            WorkingDirectory::Entry(entry) => {
                has_not_cwd |= entry.not_cwd == Some(true);

                if entry.mode == Some(WorkingDirectoryMode::Auto) {
                    resolved.auto = true;
                    continue;
                }
                let Some(directory) = entry.directory else {
                    resolved.warnings.push(format!(
                        "`{WORKING_DIRECTORIES_OPTION}` entry without `directory` or `mode` was ignored"
                    ));
                    continue;
                };
                directory
            }
        };

        match validate_entry(&directory) {
            Ok(pattern) => patterns.push(pattern),
            Err(err) => resolved.warnings.push(err),
        }
    }

    if has_not_cwd {
        // reported once, however many entries carry the flag
        resolved.warnings.push(format!(
            "`{WORKING_DIRECTORIES_OPTION}` ignores `!cwd`: every working directory already gets its own worker rooted at it"
        ));
    }

    // The caller resolves its roots through `ResolvedPath`, the walk starts from the path of the
    // workspace folder URI. Both are brought into the same form before anything is compared.
    let nested_roots = &align_nested_roots(&root_path, nested_roots);

    let mut directories = if resolved.auto {
        if !patterns.is_empty() {
            resolved.warnings.push(format!(
                "`{WORKING_DIRECTORIES_OPTION}` uses `{{ \"mode\": \"auto\" }}`, the other entries were ignored"
            ));
        }
        // The whole tree has to be walked, a project root can be at any depth. Detection follows
        // the ignore files: a generated or vendored package is not a project the user works in.
        walk_directories(&root_path, None, true, nested_roots, |dir| builder.is_project_root(dir))
    } else {
        resolve_patterns(&root_path, &patterns, nested_roots, &mut resolved.warnings)
    };

    directories.sort_unstable();
    directories.dedup();

    resolved.roots = directories
        .into_iter()
        // a working directory must stay below the workspace folder and can not be the folder itself
        .filter(|dir| dir != &root_path && dir.starts_with(&root_path))
        .filter_map(Uri::from_file_path)
        .collect();

    debug!("resolved working directories for {}: {:?}", root_uri.as_str(), resolved.roots);

    resolved
}

/// Express the roots of the nested workspace folders in the path form of `root_path`.
///
/// The [`WorkerManager`](crate::worker_manager::WorkerManager) resolves its roots through
/// [`ResolvedPath`](crate::file_system::ResolvedPath), which canonicalizes on macOS and Windows,
/// while the resolution here walks the path of the workspace folder URI. A temporary directory on
/// macOS is `/var/folders/...` in one form and `/private/var/folders/...` in the other, and a
/// symbolic link or a Windows short name has the same effect: comparing the two forms never
/// matches, and a nested workspace folder would not be skipped. The output keeps the form of the
/// workspace folder URI, so the roots this function returns stay comparable to it.
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
        // the workspace folder itself is not one of its nested folders: keeping it would prune the
        // whole walk
        .filter(|nested| nested != root_path)
        .collect()
}

/// Validate a single entry and return it with normalized separators.
///
/// Absolute paths and paths escaping the workspace folder are rejected. ESLint's `!cwd` syntax is
/// not supported.
fn validate_entry(entry: &str) -> Result<String, String> {
    let trimmed = entry.trim();
    if trimmed.is_empty() {
        return Err(format!("`{WORKING_DIRECTORIES_OPTION}` entry is empty and was ignored"));
    }

    #[expect(clippy::disallowed_methods)] // a glob always uses `/`, even on Windows
    let normalized = trimmed.replace('\\', "/");

    if Path::new(&normalized).is_absolute() || normalized.starts_with('/') {
        return Err(format!(
            "`{WORKING_DIRECTORIES_OPTION}` entry `{entry}` is an absolute path and was ignored, use a path relative to the workspace folder"
        ));
    }

    let normalized = normalized.strip_prefix("./").unwrap_or(&normalized);

    if normalized == "." || normalized.is_empty() {
        return Err(format!(
            "`{WORKING_DIRECTORIES_OPTION}` entry `{entry}` points to the workspace folder itself and was ignored"
        ));
    }

    if normalized.split('/').any(|part| part == "..") {
        return Err(format!(
            "`{WORKING_DIRECTORIES_OPTION}` entry `{entry}` escapes the workspace folder and was ignored"
        ));
    }

    Ok(normalized.trim_end_matches('/').to_string())
}

/// Resolve literal entries and glob entries into existing directories.
fn resolve_patterns(
    root_path: &Path,
    patterns: &[String],
    nested_roots: &[PathBuf],
    warnings: &mut Vec<String>,
) -> Vec<PathBuf> {
    let mut directories = Vec::with_capacity(patterns.len());
    let mut globs: Vec<&String> = vec![];

    for pattern in patterns {
        if is_glob(pattern) {
            globs.push(pattern);
            continue;
        }
        let directory = root_path.join(pattern);
        if nested_roots.iter().any(|nested| directory != *nested && directory.starts_with(nested)) {
            warnings.push(format!(
                "`{WORKING_DIRECTORIES_OPTION}` entry `{pattern}` is inside a workspace folder of its own and was ignored"
            ));
        } else if directory.is_dir() {
            directories.push(directory);
        } else {
            warnings.push(format!(
                "`{WORKING_DIRECTORIES_OPTION}` entry `{pattern}` is not an existing directory and was ignored"
            ));
        }
    }

    if globs.is_empty() {
        return directories;
    }

    // A directory the user listed explicitly is a working directory even when it is gitignored,
    // for example a generated package which is still linted.
    directories.extend(walk_directories(
        root_path,
        glob_max_depth(&globs),
        false,
        nested_roots,
        |directory| {
            let Ok(relative) = directory.strip_prefix(root_path) else {
                return false;
            };
            #[expect(clippy::disallowed_methods)] // a glob always uses `/`, even on Windows
            let relative = relative.to_string_lossy().replace('\\', "/");
            globs.iter().any(|glob| fast_glob::glob_match(glob.as_str(), relative.as_str()))
        },
    ));

    directories
}

/// The deepest directory a set of globs can match, in path segments.
///
/// Returns `None` when a glob can not be bounded: `**` matches any number of segments, and a brace
/// alternation (`{a,b/c}`) can hide a different number of separators in each branch.
fn glob_max_depth(globs: &[&String]) -> Option<usize> {
    if globs.iter().any(|glob| glob.contains("**") || glob.contains('{')) {
        return None;
    }
    globs.iter().map(|glob| glob.split('/').count()).max()
}

fn is_glob(pattern: &str) -> bool {
    pattern.contains(['*', '?', '[', '{'])
}

/// Collect the directories below `root` (excluding `root` itself) which `is_match` accepts.
///
/// Uses the same walker settings as the nested config discovery of `oxlint`, so both agree on what
/// belongs to a workspace folder. `max_depth` bounds the walk when the caller knows how deep a
/// match can be, and `honour_git_ignore` is only set for the `auto` detection: a directory the
/// user listed explicitly is a working directory even when it is gitignored.
fn walk_directories(
    root: &Path,
    max_depth: Option<usize>,
    honour_git_ignore: bool,
    nested_roots: &[PathBuf],
    is_match: impl Fn(&Path) -> bool,
) -> Vec<PathBuf> {
    let nested_roots = nested_roots.to_vec();
    let mut builder = ignore::WalkBuilder::new(root);
    builder
        .hidden(false) // don't skip hidden files
        .parents(false) // disable gitignore from parent dirs
        .ignore(false) // disable .ignore files
        .git_global(false) // disable global gitignore
        .git_ignore(honour_git_ignore) // only the detection follows the in-tree `.gitignore`
        .git_exclude(honour_git_ignore)
        // a workspace folder is not always a repository, the ignore files still apply
        .require_git(false)
        .follow_links(true)
        .max_depth(max_depth)
        .filter_entry(move |entry| {
            if !entry.file_type().is_some_and(|file_type| file_type.is_dir()) {
                return true;
            }
            // Never descend into `node_modules`, `.git`, or a workspace folder of its own. The
            // nested folder itself is kept: this root still excludes it from its own discovery.
            !entry.file_name().to_str().is_some_and(|name| IGNORED_DIRECTORIES.contains(&name))
                && !nested_roots
                    .iter()
                    .any(|nested| entry.path() != nested && entry.path().starts_with(nested))
        });

    builder
        .build()
        .flatten()
        .filter(|entry| entry.file_type().is_some_and(|file_type| file_type.is_dir()))
        .map(ignore::DirEntry::into_path)
        .filter(|path| path != root && is_match(path))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use serde_json::json;
    use tower_lsp_server::ls_types::Uri;

    use crate::{ToolBuildResult, ToolBuilder, tests::FakeToolBuilder};

    use super::{
        WORKING_DIRECTORIES_OPTION, glob_max_depth, is_below_ignored_directory,
        resolve_working_directories, sub_worker_options, validate_entry,
        working_directories_need_walk,
    };

    /// A builder which considers a directory a project root when it contains both a `package.json`
    /// and a `tool.config` file.
    #[derive(Default)]
    struct ProjectRootToolBuilder(FakeToolBuilder);

    impl ToolBuilder for ProjectRootToolBuilder {
        fn build(&self, root_uri: &Uri, options: serde_json::Value) -> ToolBuildResult {
            self.0.build(root_uri, options)
        }

        fn is_project_root(&self, dir: &Path) -> bool {
            dir.join("package.json").is_file() && dir.join("tool.config").is_file()
        }
    }

    fn builder() -> ProjectRootToolBuilder {
        ProjectRootToolBuilder::default()
    }

    fn uri(path: &Path) -> Uri {
        Uri::from_file_path(path).unwrap()
    }

    #[test]
    fn test_no_option() {
        let dir = tempfile::tempdir().unwrap();
        let resolved = resolve_working_directories(&uri(dir.path()), &json!({}), &builder(), &[]);
        assert!(resolved.roots.is_empty());
        assert!(resolved.warnings.is_empty());
        assert!(!resolved.auto);

        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: [] }),
            &builder(),
            &[],
        );
        assert!(resolved.roots.is_empty());
        assert!(resolved.warnings.is_empty());
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
            &json!({ WORKING_DIRECTORIES_OPTION: ["packages/*/sub"] }),
            &builder(),
            &[real.join("packages/a")],
        );

        // `packages/a` is a workspace folder of its own, regardless of the spelling used for it
        assert_eq!(resolved.roots, vec![uri(&link.join("packages/b/sub"))], "{:?}", resolved.roots);
    }

    #[test]
    fn test_literal_and_glob_entries() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("packages/a/src")).unwrap();
        fs::create_dir_all(dir.path().join("packages/b")).unwrap();
        fs::create_dir_all(dir.path().join("client")).unwrap();

        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: ["packages/*", { "directory": "client" }] }),
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

    #[test]
    fn test_explicit_entries_ignore_gitignore() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), "generated/\n").unwrap();
        fs::create_dir_all(dir.path().join("generated/pkg")).unwrap();
        fs::write(dir.path().join("generated/pkg/package.json"), "{}").unwrap();
        fs::write(dir.path().join("generated/pkg/tool.config"), "{}").unwrap();

        // an explicit glob matches a gitignored directory
        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: ["generated/*"] }),
            &builder(),
            &[],
        );
        assert_eq!(resolved.roots, vec![uri(&dir.path().join("generated/pkg"))]);

        // the `auto` detection does not, a gitignored package is not a project the user works in
        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: [{ "mode": "auto" }] }),
            &builder(),
            &[],
        );
        assert!(resolved.roots.is_empty(), "{:?}", resolved.roots);
    }

    #[test]
    fn test_glob_only_matches_directories() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("packages/a")).unwrap();
        fs::write(dir.path().join("packages/readme.md"), "").unwrap();

        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: ["packages/*"] }),
            &builder(),
            &[],
        );

        assert_eq!(resolved.roots, vec![uri(&dir.path().join("packages/a"))]);
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
    fn test_pattern_alias_and_not_cwd() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("packages/a")).unwrap();

        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: [
                { "pattern": "./packages/*" },
                { "directory": "packages/a", "!cwd": true },
            ] }),
            &builder(),
            &[],
        );

        // `pattern` is an alias of `directory` and a leading `./` is stripped
        assert_eq!(resolved.roots, vec![uri(&dir.path().join("packages/a"))]);
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
        assert!(resolved.roots.is_empty());
        assert_eq!(resolved.warnings.len(), 1);
        assert!(resolved.warnings[0].contains("not an existing directory"));
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
        assert!(resolved.roots.is_empty());
        assert_eq!(resolved.warnings.len(), 1);
        assert!(resolved.warnings[0].contains("is invalid"));
    }

    #[test]
    fn test_auto_mode() {
        let dir = tempfile::tempdir().unwrap();
        // a project root: `package.json` + config
        fs::create_dir_all(dir.path().join("packages/a")).unwrap();
        fs::write(dir.path().join("packages/a/package.json"), "{}").unwrap();
        fs::write(dir.path().join("packages/a/tool.config"), "{}").unwrap();
        // a config without `package.json` is not a project root
        fs::create_dir_all(dir.path().join("packages/b")).unwrap();
        fs::write(dir.path().join("packages/b/tool.config"), "{}").unwrap();
        // a `package.json` without config is not a project root
        fs::create_dir_all(dir.path().join("packages/c")).unwrap();
        fs::write(dir.path().join("packages/c/package.json"), "{}").unwrap();
        // `node_modules` is never walked
        fs::create_dir_all(dir.path().join("node_modules/x")).unwrap();
        fs::write(dir.path().join("node_modules/x/package.json"), "{}").unwrap();
        fs::write(dir.path().join("node_modules/x/tool.config"), "{}").unwrap();

        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: [{ "mode": "auto" }] }),
            &builder(),
            &[],
        );

        assert!(resolved.auto);
        assert!(resolved.warnings.is_empty(), "{:?}", resolved.warnings);
        assert_eq!(resolved.roots, vec![uri(&dir.path().join("packages/a"))]);
    }

    #[test]
    fn test_auto_mode_ignores_other_entries() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("client")).unwrap();

        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: [{ "mode": "auto" }, "client"] }),
            &builder(),
            &[],
        );

        assert!(resolved.auto);
        assert!(resolved.roots.is_empty());
        assert_eq!(resolved.warnings.len(), 1);
        assert!(resolved.warnings[0].contains("the other entries were ignored"));
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
    fn test_is_below_ignored_directory() {
        let repo = Path::new("/repo");
        assert!(is_below_ignored_directory(repo, Path::new("/repo/node_modules/x/package.json")));
        assert!(is_below_ignored_directory(repo, Path::new("/repo/.git/config")));
        assert!(!is_below_ignored_directory(repo, Path::new("/repo/packages/a/package.json")));
        // a directory which only starts with the ignored name is not ignored
        assert!(!is_below_ignored_directory(repo, Path::new("/repo/node_modules_2/package.json")));
        // only the components below the root count: a workspace folder can live inside
        // `node_modules` itself
        assert!(!is_below_ignored_directory(
            Path::new("/repo/node_modules/pkg"),
            Path::new("/repo/node_modules/pkg/src/index.ts")
        ));
        // a path outside the root is not below it
        assert!(!is_below_ignored_directory(repo, Path::new("/other/node_modules/x")));
    }

    #[test]
    fn test_working_directories_need_walk() {
        assert!(!working_directories_need_walk(&json!({})));
        assert!(!working_directories_need_walk(&json!({ WORKING_DIRECTORIES_OPTION: [] })));
        assert!(!working_directories_need_walk(
            &json!({ WORKING_DIRECTORIES_OPTION: ["packages/a", { "directory": "client" }] })
        ));
        assert!(working_directories_need_walk(
            &json!({ WORKING_DIRECTORIES_OPTION: ["packages/*"] })
        ));
        assert!(working_directories_need_walk(
            &json!({ WORKING_DIRECTORIES_OPTION: [{ "mode": "auto" }] })
        ));
    }

    #[test]
    fn test_glob_max_depth() {
        assert_eq!(glob_max_depth(&[&"packages/*".to_string()]), Some(2));
        assert_eq!(
            glob_max_depth(&[&"packages/*".to_string(), &"apps/*/lib".to_string()]),
            Some(3)
        );
        // `**` matches any number of segments, the walk can not be bounded
        assert_eq!(glob_max_depth(&[&"packages/**".to_string()]), None);
        // neither can a brace alternation, its branches can hold different depths
        assert_eq!(glob_max_depth(&[&"{packages/a,client}".to_string()]), None);
    }

    #[test]
    fn test_sub_worker_options() {
        let options = json!({ "run": "onType", WORKING_DIRECTORIES_OPTION: ["packages/*"] });
        assert_eq!(sub_worker_options(&options), json!({ "run": "onType" }));

        // non object options are passed through
        assert_eq!(sub_worker_options(&serde_json::Value::Null), serde_json::Value::Null);
    }

    #[test]
    fn test_validate_entry() {
        assert_eq!(validate_entry("packages/a"), Ok("packages/a".to_string()));
        assert_eq!(validate_entry("./packages/a/"), Ok("packages/a".to_string()));
        assert_eq!(validate_entry("packages\\a"), Ok("packages/a".to_string()));
        assert!(validate_entry("../a").is_err());
        assert!(validate_entry("packages/../../a").is_err());
        assert!(validate_entry("/a").is_err());
        assert!(validate_entry("  ").is_err());
        assert!(validate_entry(".").is_err());
    }
}
