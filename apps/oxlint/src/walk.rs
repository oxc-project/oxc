use std::{ffi::OsStr, path::PathBuf, sync::Arc, sync::mpsc};

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
            .require_git(false)
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
        let fixture = env::current_dir().unwrap().join("fixtures/walk_dir");
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

        // Only included.js should be found; ignored.js should be filtered by auto-discovered .gitignore
        // Without .git_ignore(true) and .require_git(false), both files would be found
        assert_eq!(paths, vec!["included.js"]);
    }
}
