use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
    sync::mpsc,
};

use ignore::{DirEntry, overrides::Override};
use oxc_linter::LINTABLE_EXTENSIONS;
use rustc_hash::FxHashMap;

use crate::cli::IgnoreOptions;

#[derive(Debug, Clone)]
pub struct Extensions(pub Vec<&'static str>);

impl Default for Extensions {
    fn default() -> Self {
        Self(LINTABLE_EXTENSIONS.to_vec())
    }
}

pub struct Walk {
    inner: ignore::WalkParallel,
    /// The file extensions to include during the traversal.
    extensions: Extensions,
}

struct WalkBuilder {
    sender: mpsc::Sender<Vec<Arc<OsStr>>>,
    extensions: Extensions,
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for WalkBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(WalkCollector {
            paths: vec![],
            sender: self.sender.clone(),
            extensions: self.extensions.clone(),
        })
    }
}

struct WalkCollector {
    paths: Vec<Arc<OsStr>>,
    sender: mpsc::Sender<Vec<Arc<OsStr>>>,
    extensions: Extensions,
}

impl Drop for WalkCollector {
    fn drop(&mut self) {
        let paths = std::mem::take(&mut self.paths);
        self.sender.send(paths).unwrap();
    }
}

impl ignore::ParallelVisitor for WalkCollector {
    fn visit(&mut self, entry: Result<ignore::DirEntry, ignore::Error>) -> ignore::WalkState {
        match entry {
            Ok(entry) => {
                // Skip VCS metadata directories because they are not special cases for `.hidden(false)`.
                // <https://github.com/BurntSushi/ripgrep/issues/3099#issuecomment-3052460027>
                if entry.file_type().is_some_and(|ty| ty.is_dir())
                    && (entry.file_name() == ".git" || entry.file_name() == ".jj")
                {
                    return ignore::WalkState::Skip;
                }
                if Walk::is_wanted_entry(&entry, &self.extensions) {
                    self.paths.push(entry.path().as_os_str().into());
                }
                ignore::WalkState::Continue
            }
            Err(_err) => ignore::WalkState::Skip,
        }
    }
}
impl Walk {
    /// Will not canonicalize paths.
    /// # Panics
    pub fn new(
        paths: &[PathBuf],
        options: &IgnoreOptions,
        override_builder: Option<Override>,
    ) -> Self {
        assert!(!paths.is_empty(), "At least one path must be provided to Walk::new");

        let mut inner = ignore::WalkBuilder::new(
            paths
                .iter()
                .next()
                .expect("Expected paths parameter to Walk::new() to contain at least one path."),
        );

        if let Some(paths) = paths.get(1..) {
            for path in paths {
                inner.add(path);
            }
        }

        if !options.no_ignore {
            inner.add_custom_ignore_filename(&options.ignore_path);

            if let Some(override_builder) = override_builder {
                inner.overrides(override_builder);
            }
        }

        let require_git = all_paths_have_vcs_boundary(paths);

        let inner = inner
            .ignore(false)
            .git_global(false)
            .git_ignore(true)
            .follow_links(true)
            .hidden(false)
            .require_git(require_git)
            .build_parallel();
        Self { inner, extensions: Extensions::default() }
    }

    pub fn paths(self) -> Vec<Arc<OsStr>> {
        let (sender, receiver) = mpsc::channel::<Vec<Arc<OsStr>>>();
        let mut builder = WalkBuilder { sender, extensions: self.extensions };
        self.inner.visit(&mut builder);
        drop(builder);
        receiver.into_iter().flatten().collect()
    }

    #[cfg_attr(not(test), expect(dead_code))]
    pub fn with_extensions(mut self, extensions: Extensions) -> Self {
        self.extensions = extensions;
        self
    }

    fn is_wanted_entry(dir_entry: &DirEntry, extensions: &Extensions) -> bool {
        let Some(file_type) = dir_entry.file_type() else { return false };
        if file_type.is_dir() {
            return false;
        }
        let Some(file_name) = dir_entry.path().file_name() else { return false };
        let file_name = file_name.to_string_lossy();
        let file_name = file_name.as_ref();
        if [".min.", "-min.", "_min."].iter().any(|e| file_name.contains(e)) {
            return false;
        }
        let Some(extension) = dir_entry.path().extension() else { return false };
        let extension = extension.to_string_lossy();
        extensions.0.contains(&extension.as_ref())
    }
}

fn all_paths_have_vcs_boundary(paths: &[PathBuf]) -> bool {
    let cwd = std::env::current_dir().ok();
    let mut cache = FxHashMap::default();
    paths.iter().all(|path| has_vcs_boundary(path, cwd.as_deref(), &mut cache))
}

fn has_vcs_boundary(path: &Path, cwd: Option<&Path>, cache: &mut FxHashMap<PathBuf, bool>) -> bool {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.map_or_else(|| path.to_path_buf(), |cwd| cwd.join(path))
    };

    let start = if path.is_file() { path.parent().unwrap_or(&path) } else { path.as_path() };

    if let Some(has_boundary) = cache.get(start) {
        return *has_boundary;
    }

    let has_boundary = start.ancestors().any(|dir| {
        cache
            .get(dir)
            .copied()
            .unwrap_or_else(|| dir.join(".git").exists() || dir.join(".jj").exists())
    });
    cache.insert(start.to_path_buf(), has_boundary);
    has_boundary
}

#[cfg(test)]
mod test {
    use std::{env, ffi::OsString, fs, path::Path, slice};

    use ignore::overrides::OverrideBuilder;

    use super::{Extensions, Walk};
    use crate::cli::IgnoreOptions;

    fn collect_js_paths(root: &Path, ignore_options: &IgnoreOptions) -> Vec<String> {
        let override_builder = OverrideBuilder::new(root).build().unwrap();

        let mut paths =
            Walk::new(slice::from_ref(&root.to_path_buf()), ignore_options, Some(override_builder))
                .with_extensions(Extensions(["js"].to_vec()))
                .paths()
                .into_iter()
                .map(|path| {
                    Path::new(&path).strip_prefix(root).unwrap().to_string_lossy().to_string()
                })
                .collect::<Vec<_>>();

        paths.sort();
        paths
    }

    fn empty_ignore_options() -> IgnoreOptions {
        IgnoreOptions {
            no_ignore: false,
            ignore_path: OsString::from(".eslintignore"),
            ignore_pattern: vec![],
        }
    }

    #[test]
    fn test_walk_with_extensions() {
        let fixture = env::current_dir().unwrap().join("fixtures/cli/walk_dir");
        let fixtures = vec![fixture.clone()];
        let ignore_options = IgnoreOptions {
            no_ignore: false,
            ignore_path: OsString::from(".gitignore"),
            ignore_pattern: vec![],
        };

        let override_builder = OverrideBuilder::new("/").build().unwrap();

        let mut paths = Walk::new(&fixtures, &ignore_options, Some(override_builder))
            .with_extensions(Extensions(["js", "vue"].to_vec()))
            .paths()
            .into_iter()
            .map(|path| {
                Path::new(&path).strip_prefix(&fixture).unwrap().to_string_lossy().to_string()
            })
            .collect::<Vec<_>>();
        paths.sort();

        assert_eq!(paths, vec!["bar.vue", "foo.js"]);
    }

    #[test]
    fn test_gitignore_without_git_repo() {
        // Validate that `.gitignore` files are respected even when no `.git` directory is present.

        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        fs::write(temp_path.join("included.js"), "debugger;").unwrap();
        fs::write(temp_path.join("ignored.js"), "debugger;").unwrap();

        // Create .gitignore to ignore one file
        fs::write(temp_path.join(".gitignore"), "ignored.js\n").unwrap();

        // Verify no .git directory exists
        assert!(!temp_path.join(".git").exists());

        // Use the default ignore filename without creating that file, so this relies on
        // `.gitignore` auto-discovery rather than explicit ignore loading.
        let ignore_options = empty_ignore_options();

        // Only included.js should be found; ignored.js should be filtered by auto-discovered .gitignore
        // Without .git_ignore(true) and .require_git(false), both files would be found
        assert_eq!(collect_js_paths(temp_path, &ignore_options), vec!["included.js"]);
    }

    #[test]
    fn test_parent_gitignore_does_not_cross_git_repo_boundary() {
        // A parent `.gitignore` must not apply once the walk enters a nested
        // repository. The nested repo's own `.gitignore` should still apply.
        //
        // File structure:
        // root/
        //   .gitignore          # ignores everything
        //   repo/
        //     .git/
        //     .gitignore        # ignores ignored.js
        //     included.js
        //     ignored.js
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
        let repo_path = temp_path.join("repo");

        fs::create_dir(&repo_path).unwrap();
        fs::create_dir(repo_path.join(".git")).unwrap();
        fs::write(temp_path.join(".gitignore"), "*\n").unwrap();
        fs::write(repo_path.join(".gitignore"), "ignored.js\n").unwrap();
        fs::write(repo_path.join("included.js"), "debugger;").unwrap();
        fs::write(repo_path.join("ignored.js"), "debugger;").unwrap();

        assert_eq!(collect_js_paths(&repo_path, &empty_ignore_options()), vec!["included.js"]);
    }

    #[test]
    fn test_parent_gitignore_does_not_cross_git_worktree_file_boundary() {
        // Git worktrees use a `.git` file instead of a `.git` directory. That
        // file is still a repository boundary for parent `.gitignore` lookup.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
        let repo_path = temp_path.join("repo");

        fs::create_dir(&repo_path).unwrap();
        fs::write(temp_path.join(".gitignore"), "*\n").unwrap();
        fs::write(repo_path.join(".git"), "gitdir: /tmp/worktrees/repo/.git\n").unwrap();
        fs::write(repo_path.join("included.js"), "debugger;").unwrap();

        assert_eq!(collect_js_paths(&repo_path, &empty_ignore_options()), vec!["included.js"]);
    }

    #[test]
    fn test_repo_gitignore_applies_when_walking_from_subdirectory() {
        // Stopping parent lookup at the repository boundary must still keep
        // repo-local parent `.gitignore` files active for subdirectory walks.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();
        let repo_path = temp_path.join("repo");
        let src_path = repo_path.join("src");

        fs::create_dir(&repo_path).unwrap();
        fs::create_dir(&src_path).unwrap();
        fs::create_dir(repo_path.join(".git")).unwrap();
        fs::write(temp_path.join(".gitignore"), "*\n").unwrap();
        fs::write(repo_path.join(".gitignore"), "ignored.js\n").unwrap();
        fs::write(src_path.join("included.js"), "debugger;").unwrap();
        fs::write(src_path.join("ignored.js"), "debugger;").unwrap();

        assert_eq!(collect_js_paths(&src_path, &empty_ignore_options()), vec!["included.js"]);
    }

    #[test]
    fn test_walk_skips_jj_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();

        fs::create_dir(temp_path.join(".jj")).unwrap();
        fs::write(temp_path.join("included.js"), "debugger;").unwrap();
        fs::write(temp_path.join(".jj").join("ignored.js"), "debugger;").unwrap();

        assert_eq!(collect_js_paths(temp_path, &empty_ignore_options()), vec!["included.js"]);
    }
}
