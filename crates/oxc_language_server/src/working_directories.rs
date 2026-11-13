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
//! These rules are the contract of the feature. Each one is covered by a test, so an alternative
//! behaviour is a deliberate change of the contract and not an oversight.
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
//! 3. A file below a working directory is ignored when an ignore file of the workspace folder
//!    (`.gitignore` and `.eslintignore` for oxlint, `.prettierignore` for oxfmt) ignores it, or
//!    when the `ignorePatterns` of the config which governs it from the working directory ignore
//!    it. With the nested configs on, that is the result of the workspace folder opened *without*
//!    `workingDirectories`. With `disableNestedConfig`, or a relative `configPath` /
//!    `fmt.configPath`, the `ignorePatterns` of the config of the workspace folder do not reach a
//!    working directory, which resolves its own root config. Each tool decides with its own
//!    ignore code and the server evaluates nothing itself. A working directory which the
//!    workspace folder itself ignores is reported with a warning on the first build of its
//!    worker.
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
//!    symbolic link is followed). Segments are separated by `/`, and by `\` on Windows only.
//!    `..`, absolute paths (a Windows drive included) and `.` are rejected, and glob characters are literal. ESLint's `!cwd`
//!    is accepted: it is ignored, and reported once. An entry with `mode` or `pattern` is not
//!    supported in this version and is reported. Every entry is judged on its own: an invalid one
//!    never hides a valid one.
//! 8. The validation warnings of the option are reported when its value changes, not again on
//!    every later reconciliation.
//! 9. The default, an absent or empty option, changes nothing: one worker per workspace folder.
//! 10. The option is ignored entirely by a tool which does not
//!     [use working directories](crate::ToolBuilder::use_working_directories): no sub worker is
//!     created and no warning is reported.

use std::path::{Component, Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tower_lsp_server::gen_lsp_types::{MessageType, Uri};
use tracing::debug;

use crate::{
    ClientMessage, ToolBuilder,
    file_system::ResolvedPath,
    uri_utils::{file_path_to_uri, uri_to_file_path},
    utils::find_root_for_uri,
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

/// The `workingDirectories` entries of the options object of a workspace folder.
///
/// An invalid entry is left out: it is reported by [`resolve_working_directories`], and does not
/// hide the other ones.
pub fn parse_working_directories(
    options: &serde_json::Map<String, serde_json::Value>,
) -> Vec<WorkingDirectory> {
    options.get(WORKING_DIRECTORIES_OPTION).and_then(serde_json::Value::as_array).map_or_default(
        |entries| {
            entries.iter().filter_map(|entry| WorkingDirectory::deserialize(entry).ok()).collect()
        },
    )
}

/// Whether `path` is below `root`, and not `root` itself.
pub fn strictly_below(path: &Path, root: &Path) -> bool {
    path != root && path.starts_with(root)
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

    let resolved_root = ResolvedPath::from(root_path.to_path_buf());
    let mut roots = Vec::with_capacity(directories.len());
    for entry in &directories {
        // in the form of the nested roots, which the entry is compared with
        let directory = align_to_folder(&root_path, resolved_root.as_path(), root_path.join(entry));
        if !strictly_below(&directory, &root_path) {
            // e.g. a symbolic link to the workspace folder, which no path check can tell
            resolved.warnings.push(points_to_folder_itself(entry));
        } else if nested_roots.iter().any(|nested| strictly_below(&directory, nested)) {
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
        .filter(|dir| strictly_below(dir, &root_path))
        .filter_map(file_path_to_uri)
        .collect();

    debug!("resolved working directories for {}: {:?}", root_uri.as_str(), resolved.roots);

    resolved
}

/// The worker which the language server routes a file to, see [`route_to_worker`].
#[derive(Debug)]
pub struct RoutedWorker {
    /// The root of the worker.
    pub root: Uri,
    /// The options the worker is started with.
    pub options: serde_json::Value,
    /// The workspace folder of a sub worker, whose ignore files it still honours.
    pub parent_root: Option<PathBuf>,
    /// The roots of the workers below this one, which it must not load anything from.
    pub excluded_roots: Vec<PathBuf>,
}

/// The worker which the language server routes `uri` to, for the workspace folder `root_uri` with
/// the given options.
///
/// The `workingDirectories` option is resolved like the server does, and the worker with the
/// longest matching root path wins. `None` when no worker is responsible for `uri`.
///
/// For the tests of a tool, which build the tool of that worker directly instead of running a
/// [`WorkerManager`](crate::worker_manager::WorkerManager).
pub fn route_to_worker(
    root_uri: &Uri,
    options: &serde_json::Value,
    builder: &dyn ToolBuilder,
    uri: &Uri,
) -> Option<RoutedWorker> {
    let mut roots = vec![root_uri.clone()];
    roots.extend(resolve_working_directories(root_uri, options, builder, &[]).roots);

    let index = find_root_for_uri(&roots, uri)?;
    let root = roots[index].clone();
    let root_path = uri_to_file_path(&root)?.into_owned();
    // a worker must not load anything owned by a worker below it, which includes a working
    // directory nested inside another working directory
    let excluded_roots = roots
        .iter()
        .filter_map(|other| uri_to_file_path(other).map(std::borrow::Cow::into_owned))
        .filter(|other_path| strictly_below(other_path, &root_path))
        .collect();

    // a sub worker never sees the `workingDirectories` option itself, but it still honours the
    // ignore files of its workspace folder
    let (options, parent_root) = if index == 0 {
        (options.clone(), None)
    } else {
        (sub_worker_options(options), uri_to_file_path(root_uri).map(std::borrow::Cow::into_owned))
    };

    Some(RoutedWorker { root, options, parent_root, excluded_roots })
}

/// Express the roots of the nested workspace folders in the path form of `root_path`.
///
/// The [`WorkerManager`](crate::worker_manager::WorkerManager) resolves its roots through
/// [`ResolvedPath`], which canonicalizes on macOS and Windows,
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

/// `directory` as the file system spells it, below `root_path`.
///
/// Two entries which name one directory differently, by casing on a case-insensitive disk or
/// through a symbolic link, resolve to the same [`ResolvedPath`]. The roots are compared and
/// deduplicated, and a worker is matched against the files, in that form, so the spelling of the
/// entry must not decide how many workers a directory gets. The result keeps the form of
/// `root_path`, like [`align_nested_roots`]. A directory which resolves outside of the workspace
/// folder keeps the spelling it was given. Where [`ResolvedPath`] does not resolve anything, this
/// returns `directory` unchanged.
fn align_to_folder(root_path: &Path, resolved_root: &Path, directory: PathBuf) -> PathBuf {
    let resolved = ResolvedPath::from(directory.clone());
    resolved
        .as_path()
        .strip_prefix(resolved_root)
        .map_or(directory, |relative| root_path.join(relative))
}

/// The warning for an entry which names the workspace folder itself.
fn points_to_folder_itself(entry: &str) -> String {
    format!(
        "`{WORKING_DIRECTORIES_OPTION}` entry `{entry}` points to the workspace folder itself and was ignored"
    )
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

    // `\` separates segments on Windows only: elsewhere it is an ordinary character of a name.
    #[expect(clippy::disallowed_methods)] // the normalized entry always uses `/`, even on Windows
    let normalized = if cfg!(windows) { trimmed.replace('\\', "/") } else { trimmed.to_string() };

    if Path::new(&normalized).is_absolute() || normalized.starts_with('/') {
        return Err(format!(
            "`{WORKING_DIRECTORIES_OPTION}` entry `{entry}` is an absolute path and was ignored, use a path relative to the workspace folder"
        ));
    }

    // `C:dir` is relative to the current directory of the drive `C:`, joining it to the path of
    // the workspace folder would replace that path
    if matches!(Path::new(&normalized).components().next(), Some(Component::Prefix(_))) {
        return Err(format!(
            "`{WORKING_DIRECTORIES_OPTION}` entry `{entry}` names a drive and was ignored, use a path relative to the workspace folder"
        ));
    }

    // `.` and empty segments (`a//b`, `a/./b`, `a/b/.`) name the same directory, the entry becomes
    // one canonical spelling so equal directories give equal roots
    let segments = normalized
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect::<Vec<_>>();

    if segments.is_empty() {
        return Err(points_to_folder_itself(entry));
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

    /// The nested workspace folders come from the worker manager, which
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

    /// Strings and `{ "directory": ... }` entries are literal paths.
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

    /// The entries which name one directory give one root, however they are
    /// spelled.
    #[test]
    fn test_a_directory_named_twice_is_one_root() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("packages/a")).unwrap();

        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: [
                "packages/a",
                "./packages//a/",
                { "directory": "packages/a" },
            ] }),
            &builder(),
            &[],
        );

        assert!(resolved.warnings.is_empty(), "{:?}", resolved.warnings);
        assert_eq!(resolved.roots, vec![uri(&dir.path().join("packages/a"))]);
    }

    /// Two casings of one directory, on a disk which ignores the casing, are one
    /// root, spelled like the file system spells it.
    #[cfg(any(target_os = "macos", windows))]
    #[test]
    fn test_two_casings_of_one_directory_are_one_root() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("packages/a")).unwrap();
        assert!(
            dir.path().join("Packages/A").is_dir(),
            "this test needs a volume which ignores the case"
        );

        for entries in [json!(["packages/a", "Packages/A"]), json!(["Packages/A", "packages/a"])] {
            let resolved = resolve_working_directories(
                &uri(dir.path()),
                &json!({ WORKING_DIRECTORIES_OPTION: entries }),
                &builder(),
                &[],
            );

            assert!(resolved.warnings.is_empty(), "{:?}", resolved.warnings);
            assert_eq!(resolved.roots, vec![uri(&dir.path().join("packages/a"))], "{entries}");
        }
    }

    /// Glob characters are literal, `apps/[locale]` is a directory like any other.
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

    /// `mode` and `pattern` are not supported in this version, the entries are
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

    /// An invalid entry never hides a valid one.
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

    /// An entry which resolves to the workspace folder, here through a symbolic link, is reported
    /// like the other shapes which name it, and never becomes a root. Only where `ResolvedPath`
    /// resolves links, so the entry is not the folder after the alignment elsewhere.
    #[cfg(target_os = "macos")]
    #[test]
    fn test_an_entry_resolving_to_the_workspace_folder_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("packages/a")).unwrap();
        std::os::unix::fs::symlink(dir.path(), dir.path().join("loop")).unwrap();

        let resolved = resolve_working_directories(
            &uri(dir.path()),
            &json!({ WORKING_DIRECTORIES_OPTION: ["loop", "packages/a"] }),
            &builder(),
            &[],
        );

        assert_eq!(resolved.roots, vec![uri(&dir.path().join("packages/a"))]);
        assert_eq!(resolved.warnings.len(), 1, "{:?}", resolved.warnings);
        assert!(resolved.warnings[0].contains("`loop` points to the workspace folder itself"));
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

    /// Nothing is resolved by a tool which does not opt in, not even the
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
        // `\` separates segments on Windows only
        #[cfg(windows)]
        assert_eq!(validate_entry("packages\\a"), Ok("packages/a".to_string()));
        #[cfg(not(windows))]
        assert_eq!(validate_entry("weird\\name"), Ok("weird\\name".to_string()));
        assert_eq!(validate_entry("apps/[locale]"), Ok("apps/[locale]".to_string()));
        assert!(validate_entry("../a").is_err());
        assert!(validate_entry("packages/../../a").is_err());
        assert!(validate_entry("/a").is_err());
        // a drive without a separator is relative to the current directory of that drive
        #[cfg(windows)]
        assert!(validate_entry("C:foo").is_err());
        #[cfg(not(windows))]
        assert_eq!(validate_entry("C:foo"), Ok("C:foo".to_string()));
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
