use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
    sync::mpsc,
};

use ignore::{DirEntry, overrides::Override};
use oxc_linter::LINTABLE_EXTENSIONS;

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
                // Skip traversing `.git` directories because `.git` is not a special case for `.hidden(false)`.
                // <https://github.com/BurntSushi/ripgrep/issues/3099#issuecomment-3052460027>
                if entry.file_type().is_some_and(|ty| ty.is_dir()) && entry.file_name() == ".git" {
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
fn any_path_in_git_repo(paths: &[PathBuf]) -> bool {
    paths.iter().any(|p| {
        let start: &Path = if p.is_file() { p.parent().unwrap_or(p.as_path()) } else { p };
        let mut current = start.to_path_buf();
        loop {
            if current.join(".git").exists() {
                return true;
            }
            if !current.pop() {
                return false;
            }
        }
    })
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

        let inner = inner
            .ignore(false)
            .git_global(false)
            .git_ignore(true)
            .follow_links(true)
            .hidden(false)
            // Inside a git repo: enforce git-boundary semantics so outer repos'
            // .gitignore rules do not bleed into nested repos.
            // Outside a git repo: disable the requirement so .gitignore files are
            // still respected without a .git ancestor (see #17375).
            .require_git(any_path_in_git_repo(paths))
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

#[cfg(test)]
mod test {
    use std::{env, ffi::OsString, fs, path::Path};

    use ignore::overrides::OverrideBuilder;

    use super::{Extensions, Walk};
    use crate::cli::IgnoreOptions;

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

        // Use empty ignore_path to rely on auto-discovery, not explicit loading
        let ignore_options = IgnoreOptions {
            no_ignore: false,
            ignore_path: OsString::from(""), // Empty = rely on auto-discovery
            ignore_pattern: vec![],
        };

        let override_builder = OverrideBuilder::new(temp_path).build().unwrap();

        let mut paths =
            Walk::new(&[temp_path.to_path_buf()], &ignore_options, Some(override_builder))
                .with_extensions(Extensions(["js"].to_vec()))
                .paths()
                .into_iter()
                .map(|path| {
                    Path::new(&path)
                        .strip_prefix(temp_path)
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                })
                .collect::<Vec<_>>();

        paths.sort();

        // Only included.js should be found; ignored.js should be filtered by auto-discovered .gitignore.
        // Since the temp dir has no .git ancestor, any_path_in_git_repo returns false,
        // keeping require_git(false) so .gitignore rules apply without a git repo.
        assert_eq!(paths, vec!["included.js"]);
    }

    #[test]
    fn test_gitignore_within_git_repo() {
        // Validate that `.gitignore` files are respected inside a git repository.

        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();

        fs::create_dir(temp_path.join(".git")).unwrap();
        fs::write(temp_path.join("included.js"), "debugger;").unwrap();
        fs::write(temp_path.join("ignored.js"), "debugger;").unwrap();
        fs::write(temp_path.join(".gitignore"), "ignored.js\n").unwrap();

        let ignore_options = IgnoreOptions {
            no_ignore: false,
            ignore_path: OsString::from(""),
            ignore_pattern: vec![],
        };

        let override_builder = OverrideBuilder::new(temp_path).build().unwrap();

        let mut paths =
            Walk::new(&[temp_path.to_path_buf()], &ignore_options, Some(override_builder))
                .with_extensions(Extensions(["js"].to_vec()))
                .paths()
                .into_iter()
                .map(|path| {
                    Path::new(&path)
                        .strip_prefix(temp_path)
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                })
                .collect::<Vec<_>>();

        paths.sort();

        // any_path_in_git_repo returns true, so require_git(true) is used and the
        // .gitignore within the repo is still respected.
        assert_eq!(paths, vec!["included.js"]);
    }

    #[test]
    fn test_gitignore_nested_git_repo_isolation() {
        // Validate that a parent repo's .gitignore does not bleed into a nested repo.
        // This is the core bug fixed by using require_git(true) inside git repos.

        let temp_dir = tempfile::tempdir().unwrap();
        let parent_path = temp_dir.path();
        let child_path = parent_path.join("child");

        // Parent repo: .gitignore excludes all .js files
        fs::create_dir(parent_path.join(".git")).unwrap();
        fs::write(parent_path.join(".gitignore"), "*.js\n").unwrap();

        // Child (nested) repo: contains a .js file that must NOT be excluded
        fs::create_dir(&child_path).unwrap();
        fs::create_dir(child_path.join(".git")).unwrap();
        fs::write(child_path.join("foo.js"), "debugger;").unwrap();

        let ignore_options = IgnoreOptions {
            no_ignore: false,
            ignore_path: OsString::from(""),
            ignore_pattern: vec![],
        };

        let override_builder = OverrideBuilder::new(&child_path).build().unwrap();

        let mut paths = Walk::new(std::slice::from_ref(&child_path), &ignore_options, Some(override_builder))
            .with_extensions(Extensions(["js"].to_vec()))
            .paths()
            .into_iter()
            .map(|path| {
                Path::new(&path).strip_prefix(&child_path).unwrap().to_string_lossy().to_string()
            })
            .collect::<Vec<_>>();

        paths.sort();

        // foo.js must be found: require_git(true) stops add_parents at child/.git so the
        // parent's *.js rule is never loaded. With require_git(false) this would return [].
        assert_eq!(paths, vec!["foo.js"]);
    }
}
