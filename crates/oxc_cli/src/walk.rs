use std::{
    path::{Path, PathBuf},
    sync::mpsc,
};

use ignore::{overrides::OverrideBuilder, DirEntry};
use oxc_span::VALID_EXTENSIONS;
use rustc_hash::FxHashSet;

use crate::IgnoreOptions;

#[derive(Clone)]
pub struct Extensions(pub Vec<&'static str>);

impl Default for Extensions {
    fn default() -> Self {
        Self(VALID_EXTENSIONS.to_vec())
    }
}

pub struct Walk {
    inners: Vec<ignore::WalkParallel>,
    /// The file extensions to include during the traversal.
    extensions: Extensions,
}

struct WalkBuilder {
    sender: mpsc::Sender<Vec<Box<Path>>>,
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
    paths: Vec<Box<Path>>,
    sender: mpsc::Sender<Vec<Box<Path>>>,
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
                if entry.file_type().is_some_and(|ft| !ft.is_dir())
                    && Walk::is_wanted_entry(&entry, &self.extensions)
                {
                    self.paths.push(entry.path().to_path_buf().into_boxed_path());
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
    pub fn new(paths: &[PathBuf], options: &IgnoreOptions) -> Self {
        assert!(!paths.is_empty(), "At least one path must be provided to Walk::new");

        let walks = Self::build_walks(paths, options);

        // Turning off `follow_links` because:
        // * following symlinks is a really slow syscall
        // * it is super rare to have symlinked source code
        let inners = walks
            .into_iter()
            .map(|mut walk| {
                walk.ignore(false).git_global(false).follow_links(false).build_parallel()
            })
            .collect::<Vec<_>>();
        Self { inners, extensions: Extensions::default() }
    }

    pub fn paths(self) -> Vec<Box<Path>> {
        let (sender, receiver) = mpsc::channel::<Vec<Box<Path>>>();
        let mut builder = WalkBuilder { sender, extensions: self.extensions };
        for inner in self.inners {
            inner.visit(&mut builder);
        }

        drop(builder);
        let paths: FxHashSet<_> = receiver.into_iter().flatten().collect();
        paths.into_iter().collect()
    }

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
        if [".min.", "-min.", "_min."].iter().any(|e| file_name.to_string_lossy().contains(e)) {
            return false;
        }
        let Some(extension) = dir_entry.path().extension() else { return false };
        let extension = extension.to_string_lossy();
        extensions.0.contains(&extension.as_ref())
    }

    fn build_walks(paths: &[PathBuf], options: &IgnoreOptions) -> Vec<ignore::WalkBuilder> {
        paths.iter().fold(vec![], |mut walks: Vec<ignore::WalkBuilder>, path| {
            if path.is_dir() {
                let last_walk = walks.last_mut();
                if let Some(last_walk) = last_walk {
                    last_walk.add(path);
                } else {
                    let mut walk_builder = ignore::WalkBuilder::new(path);
                    if !options.no_ignore {
                        walk_builder.add_custom_ignore_filename(&options.ignore_path);
                        if !options.ignore_pattern.is_empty() {
                            let patterns = options
                                .ignore_pattern
                                .iter()
                                .map(|s| format!("!{s}"))
                                .collect::<Vec<String>>();

                            Self::add_overrides_for_walk(&mut walk_builder, &patterns);
                        }
                    }
                    walks.push(walk_builder);
                }
            } else if path.is_file() {
                // If file's path passed to `Walk``, the ignorePattern becomes invalid,
                // so we need add file path's parent dir but ignore everything
                // except the file itself first to escape this situation.
                // https://github.com/oxc-project/oxc/pull/2472#issuecomment-1970377231
                let parent = path.parent().unwrap();
                let mut walk_builder = ignore::WalkBuilder::new(parent);
                let ignore_all_file_of_parent = format!("!{}/*", parent.to_string_lossy());
                let include_only_current_file = format!("{}", path.to_string_lossy());
                let mut patterns: Vec<String> =
                    vec![ignore_all_file_of_parent, include_only_current_file];

                if !options.no_ignore {
                    walk_builder.add_custom_ignore_filename(&options.ignore_path);
                    let ignore_pattern = options
                        .ignore_pattern
                        .iter()
                        .map(|s| format!("!{s}"))
                        .collect::<Vec<String>>();
                    patterns.extend(ignore_pattern);
                }

                Self::add_overrides_for_walk(&mut walk_builder, &patterns);
                walks.push(walk_builder);
            }

            walks
        })
    }

    // Meaning of ignore pattern is reversed
    // <https://docs.rs/ignore/latest/ignore/overrides/struct.OverrideBuilder.html#method.add>
    fn add_overrides_for_walk(walk_builder: &mut ignore::WalkBuilder, patterns: &[String]) {
        let mut override_builder = OverrideBuilder::new(Path::new("/"));
        for pattern in patterns {
            override_builder.add(pattern).unwrap();
        }
        let overrides = override_builder.build().unwrap();
        walk_builder.overrides(overrides);
    }
}

// ignore windows because of the different path separator
#[cfg(all(test, not(target_os = "windows")))]
mod test {
    use std::{
        env,
        ffi::OsString,
        path::{Path, PathBuf},
    };

    use crate::IgnoreOptions;

    use super::{Extensions, Walk};

    fn test(fixtures: &[PathBuf], options: &IgnoreOptions, root: &Path) -> Vec<String> {
        let mut paths = Walk::new(fixtures, options)
            .paths()
            .into_iter()
            .map(|p| p.strip_prefix(root).unwrap().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        paths.sort();

        paths
    }

    #[test]
    fn test_walk_with_extensions() {
        let fixture: &PathBuf = &env::current_dir().unwrap().join("fixtures/walk_dir");
        let fixtures = vec![fixture.clone()];
        let ignore_options = IgnoreOptions {
            no_ignore: false,
            ignore_path: OsString::from(".gitignore"),
            ignore_pattern: vec![],
        };

        let mut paths = Walk::new(&fixtures, &ignore_options)
            .with_extensions(Extensions(["js", "vue"].to_vec()))
            .paths()
            .into_iter()
            .map(|p| p.strip_prefix(fixture).unwrap().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        paths.sort();

        assert_eq!(paths, vec!["bar.vue", "baz/test1.js", "baz/test2.js", "foo.js"]);
    }

    #[test]
    fn test_dir_with_ignore_pattern() {
        let root: &PathBuf = &env::current_dir().unwrap().join("fixtures/walk_dir");
        let ignore_options = IgnoreOptions {
            no_ignore: false,
            ignore_path: OsString::from(".gitignore"),
            ignore_pattern: vec!["foo.js".to_string()],
        };

        let fixtures = vec![root.clone()];
        let paths = test(&fixtures, &ignore_options, root);

        assert_eq!(paths, ["baz/test1.js", "baz/test2.js"].to_vec());
    }

    #[test]
    fn test_single_file_path_with_ignore_pattern() {
        let root: &PathBuf = &env::current_dir().unwrap().join("fixtures/walk_dir");
        let ignore_options = IgnoreOptions {
            no_ignore: false,
            ignore_path: OsString::from(".gitignore"),
            ignore_pattern: vec!["test1.js".to_string()],
        };

        let fixtures = vec![root.join("baz/test1.js")];
        let paths = test(&fixtures, &ignore_options, root);

        assert!(paths.is_empty());
    }

    #[test]
    fn test_multi_file_path_with_ignore_pattern() {
        let root: &PathBuf = &env::current_dir().unwrap().join("fixtures/walk_dir");
        let ignore_options = IgnoreOptions {
            no_ignore: false,
            ignore_path: OsString::from(".gitignore"),
            ignore_pattern: vec!["test1.js".to_string()],
        };

        let fixtures = vec![root.clone(), root.join("baz/test1.js")];
        let paths = test(&fixtures, &ignore_options, root);
        assert_eq!(paths, ["baz/test2.js", "foo.js"].to_vec());

        let fixtures = vec![root.clone(), root.join("baz/test1.js"), root.join("baz/test2.js")];
        let paths = test(&fixtures, &ignore_options, root);
        assert_eq!(paths, ["baz/test2.js", "foo.js"].to_vec());
    }

    #[test]
    fn test_file_path_with_no_ignore() {
        let root: &PathBuf = &env::current_dir().unwrap().join("fixtures/walk_dir");
        let ignore_options = IgnoreOptions {
            no_ignore: true,
            ignore_path: OsString::from(".gitignore"),
            ignore_pattern: vec!["test1.js".to_string()],
        };

        let fixtures = vec![root.join("baz/test1.js")];
        let paths = test(&fixtures, &ignore_options, root);

        assert_eq!(paths, ["baz/test1.js"].to_vec());
    }
}
