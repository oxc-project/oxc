use std::{
    path::{Path, PathBuf},
    sync::mpsc,
};

use ignore::{overrides::OverrideBuilder, DirEntry};
use oxc_span::VALID_EXTENSIONS;

use crate::IgnoreOptions;

pub struct Walk {
    inner: ignore::WalkParallel,
}

struct WalkBuilder {
    sender: mpsc::Sender<Vec<Box<Path>>>,
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for WalkBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(WalkCollector { paths: vec![], sender: self.sender.clone() })
    }
}

struct WalkCollector {
    paths: Vec<Box<Path>>,
    sender: mpsc::Sender<Vec<Box<Path>>>,
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
                if entry.file_type().is_some_and(|ft| !ft.is_dir()) && Walk::is_wanted_entry(&entry)
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
    /// # Panics
    pub fn new(paths: &[PathBuf], options: &IgnoreOptions) -> Self {
        let paths = paths
            .iter()
            .map(|p| p.canonicalize().unwrap_or_else(|_| p.clone()))
            .collect::<Vec<_>>();
        let mut inner = ignore::WalkBuilder::new(&paths[0]);

        if let Some(paths) = paths.get(1..) {
            for path in paths {
                inner.add(path);
            }
        }

        if !options.no_ignore {
            inner.add_custom_ignore_filename(&options.ignore_path);

            if !options.ignore_pattern.is_empty() {
                let mut override_builder = OverrideBuilder::new(Path::new("/"));
                for pattern in &options.ignore_pattern {
                    // Meaning of ignore pattern is reversed
                    // <https://docs.rs/ignore/latest/ignore/overrides/struct.OverrideBuilder.html#method.add>
                    let pattern = format!("!{pattern}");
                    override_builder.add(&pattern).unwrap();
                }
                let overrides = override_builder.build().unwrap();
                inner.overrides(overrides);
            }
        }
        // Turning off `follow_links` because:
        // * following symlinks is a really slow syscall
        // * it is super rare to have symlinked source code
        let inner = inner.ignore(false).git_global(false).follow_links(false).build_parallel();
        Self { inner }
    }

    pub fn paths(self) -> Vec<Box<Path>> {
        let (sender, receiver) = mpsc::channel::<Vec<Box<Path>>>();
        let mut builder = WalkBuilder { sender };
        self.inner.visit(&mut builder);
        drop(builder);
        receiver.into_iter().flatten().collect()
    }

    fn is_wanted_entry(dir_entry: &DirEntry) -> bool {
        let Some(file_type) = dir_entry.file_type() else { return false };
        if file_type.is_dir() {
            return false;
        }
        let Some(file_name) = dir_entry.path().file_name() else { return false };
        if [".min.", "-min.", "_min."].iter().any(|e| file_name.to_string_lossy().contains(e)) {
            return false;
        }
        let Some(extension) = dir_entry.path().extension() else { return false };
        VALID_EXTENSIONS.contains(&extension.to_string_lossy().as_ref())
    }
}
