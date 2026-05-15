use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
    sync::mpsc,
};

use ignore::{DirEntry, overrides::Override};
use oxc_config::{all_paths_have_vcs_boundary, configure_walk_builder};
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
        cwd: &Path,
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

        let has_vcs_boundary = all_paths_have_vcs_boundary(paths, cwd);
        let inner = configure_walk_builder(&mut inner, has_vcs_boundary)
            .follow_links(true)
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
    use std::{env, ffi::OsString, fs, path::Path, slice};

    use ignore::overrides::OverrideBuilder;

    use super::{Extensions, Walk};
    use crate::cli::IgnoreOptions;

    fn collect_js_paths(root: &Path, ignore_options: &IgnoreOptions) -> Vec<String> {
        let override_builder = OverrideBuilder::new(root).build().unwrap();

        let mut paths = Walk::new(
            slice::from_ref(&root.to_path_buf()),
            root,
            ignore_options,
            Some(override_builder),
        )
        .with_extensions(Extensions(["js"].to_vec()))
        .paths()
        .into_iter()
        .map(|path| Path::new(&path).strip_prefix(root).unwrap().to_string_lossy().to_string())
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

        let mut paths = Walk::new(&fixtures, &fixture, &ignore_options, Some(override_builder))
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
    fn test_walk_skips_jj_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path();

        fs::create_dir(temp_path.join(".jj")).unwrap();
        fs::write(temp_path.join("included.js"), "debugger;").unwrap();
        fs::write(temp_path.join(".jj").join("ignored.js"), "debugger;").unwrap();

        assert_eq!(collect_js_paths(temp_path, &empty_ignore_options()), vec!["included.js"]);
    }
}
